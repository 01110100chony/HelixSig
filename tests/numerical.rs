use helix::bridge::ffi::{process_batch, process_event, NativeResult};

fn fixture() -> Vec<f64> {
    include_bytes!("../fixtures/small/signals.f64le")
        .chunks_exact(8)
        .map(|chunk| f64::from_le_bytes(chunk.try_into().unwrap()))
        .collect()
}

#[test]
fn fixture_matches_independent_numpy_reference_and_batch() {
    let samples = fixture();
    let before = samples.clone();
    let reference: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/small/reference.json")).unwrap();
    let mut output = vec![NativeResult::default(); 8];
    process_batch(&samples, 64, 32, &mut output).unwrap();
    for (index, result) in output.iter().enumerate() {
        assert_eq!(
            *result,
            process_event(&samples[index * 64..(index + 1) * 64], 32).unwrap()
        );
        assert_eq!(result.status, 0);
        assert_eq!(
            u64::from(result.peak_index),
            reference[index]["peak_index"].as_u64().unwrap()
        );
        for (field, actual) in [
            ("baseline", result.baseline),
            ("peak_amplitude", result.peak_amplitude),
            ("integral", result.integral),
        ] {
            let expected = reference[index][field].as_f64().unwrap();
            assert!(
                (actual - expected).abs() <= 1e-10 + 1e-10 * expected.abs(),
                "{field}: {actual} vs {expected}"
            );
        }
    }
    assert_eq!(samples, before);
}

#[test]
fn empty_partial_single_and_maximum_batches() {
    process_batch(&[], 64, 32, &mut []).unwrap();
    for count in [1, 3, 64] {
        let mut output = vec![NativeResult::default(); count];
        process_batch(&vec![2.0; count * 64], 64, 32, &mut output).unwrap();
        for result in output {
            assert_eq!(result.status, 0);
            assert_eq!(result.baseline, 2.0);
            assert_eq!(result.integral, 0.0);
            assert_eq!(result.peak_index, 32);
        }
    }
}

#[test]
fn structural_errors_preserve_every_output_field() {
    let sentinel = NativeResult {
        status: 91,
        baseline: 8.0,
        peak_amplitude: 9.0,
        peak_index: 7,
        integral: 6.0,
    };
    for (length, width, baseline, count) in [
        (12, 0, 2, 3),
        (12, 5, 2, 3),
        (12, 4, 0, 3),
        (12, 4, 4, 3),
        (260, 4, 2, 65),
        (0, usize::MAX, 1, 0),
    ] {
        let mut output = vec![sentinel; count];
        assert!(process_batch(&vec![1.0; length], width, baseline, &mut output).is_err());
        assert_eq!(output, vec![sentinel; count]);
    }
}

#[test]
fn invalid_events_are_isolated() {
    let mut samples = vec![1.0; 3 * 64];
    samples[64] = f64::NAN;
    samples[128] = f64::INFINITY;
    let bits: Vec<_> = samples.iter().map(|x| x.to_bits()).collect();
    let mut output = vec![NativeResult::default(); 3];
    process_batch(&samples, 64, 32, &mut output).unwrap();
    assert_eq!(
        output.iter().map(|x| x.status).collect::<Vec<_>>(),
        [0, 3, 3]
    );
    assert_eq!(
        samples.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
        bits
    );
    assert_eq!(process_event(&[], 1).unwrap().status, 1);
    assert_eq!(process_event(&[1.0, 2.0], 0).unwrap().status, 2);
    assert_eq!(process_event(&[-f64::MAX, f64::MAX], 1).unwrap().status, 4);
}

#[test]
fn concurrent_calls_have_disjoint_lifetimes_and_no_shared_state() {
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..4)
            .map(|worker| {
                scope.spawn(move || {
                    let samples = vec![f64::from(worker); 3 * 64];
                    let mut output = vec![NativeResult::default(); 3];
                    for _ in 0..1000 {
                        process_batch(&samples, 64, 32, &mut output).unwrap();
                        assert!(output.iter().all(|x| x.status == 0
                            && x.baseline == f64::from(worker)
                            && x.integral == 0.0));
                    }
                })
            })
            .collect();
        for handle in handles {
            handle.join().unwrap();
        }
    });
}

#[test]
fn batch_nonfinite_result_is_isolated_and_matches_event() {
    let width: usize = 64;
    let baseline_samples: u32 = 32;
    let k = baseline_samples as usize;
    let mut samples = vec![1.0; 3 * width];

    for x in &mut samples[width..width + k] {
        *x = -f64::MAX;
    }
    samples[width + k] = f64::MAX;

    for x in &mut samples[2 * width..3 * width] {
        *x = 2.5;
    }

    let bits: Vec<_> = samples.iter().map(|x| x.to_bits()).collect();
    let mut output = vec![NativeResult::default(); 3];

    process_batch(&samples, width, baseline_samples, &mut output).unwrap();

    assert_eq!(output[0].status, 0);
    assert_eq!(output[0].baseline, 1.0);
    assert_eq!(output[1].status, 4);
    assert_eq!(output[2].status, 0);
    assert_eq!(output[2].baseline, 2.5);

    for index in 0..3 {
        let single = process_event(
            &samples[index * width..(index + 1) * width],
            baseline_samples,
        )
        .unwrap();
        assert_eq!(output[index], single);
    }

    assert_eq!(
        samples.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
        bits
    );
}

#[test]
fn concurrent_process_event_calls_are_deterministic_and_isolated() {
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..4)
            .map(|worker| {
                scope.spawn(move || {
                    let val = f64::from(worker) + 1.0;
                    let samples = vec![val; 64];
                    for _ in 0..1000 {
                        let res = process_event(&samples, 32).unwrap();
                        assert_eq!(res.status, 0);
                        assert_eq!(res.baseline, val);
                        assert_eq!(res.peak_amplitude, 0.0);
                        assert_eq!(res.peak_index, 32);
                        assert_eq!(res.integral, 0.0);
                    }
                })
            })
            .collect();
        for handle in handles {
            handle.join().unwrap();
        }
    });
}
