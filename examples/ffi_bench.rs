//! H5 measurement harness; never part of the production replay path.
use clap::{Parser, ValueEnum};
use helix::bridge::ffi;
use helix::config::{Execution, FfiMode, Policy, RunConfig};
use helix::source::Corpus;
use std::hint::black_box;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Preparation {
    Prepared,
    Pack,
}

#[derive(Parser)]
struct Args {
    #[arg(long)]
    corpus: PathBuf,
    #[arg(long)]
    batch_size: usize,
    #[arg(long)]
    events: u64,
    #[arg(long, value_enum)]
    ffi: FfiMode,
    #[arg(long, value_enum)]
    preparation: Preparation,
}

fn measure(args: Args) -> Result<serde_json::Value, String> {
    if !(1..=64).contains(&args.batch_size) || !(1..=1_000_000).contains(&args.events) {
        return Err("require B in 1..=64 and events in 1..=1000000".into());
    }
    let mut config = RunConfig {
        corpus: args.corpus,
        out: PathBuf::new(),
        events: args.events,
        execution: Execution::Sequential,
        workers: 1,
        queue_capacity: 1,
        batch_size: args.batch_size,
        ffi: args.ffi,
        policy: Policy::Block,
        baseline_samples: None,
    };
    let corpus = Corpus::load(&mut config)?;
    let width = corpus.manifest.samples as usize;
    let rows = corpus.manifest.rows as usize;
    if rows != 256 {
        return Err("the benchmark requires the preregistered 256-row corpus".into());
    }
    // Identical source rows and preallocated buffers for both FFI granularities.
    let source: Vec<Vec<f64>> = (0..rows).map(|i| corpus.row(i as u64).to_vec()).collect();
    let prepared: Vec<f64> = source.iter().flatten().copied().collect();
    let mut packed = vec![0.0; args.batch_size * width];
    let mut results = vec![ffi::NativeResult::default(); args.batch_size];
    let mut count = 0;
    let mut checksum = 0.0;
    let mut batches = vec![0u64; 65];
    // Bounded by the CLI event limit; capture is timed, verification is not.
    let mut observed = vec![ffi::NativeResult::default(); args.events as usize];
    let start = Instant::now();
    while count < args.events {
        let row = count as usize % rows;
        let batch = args
            .batch_size
            .min(rows - row)
            .min((args.events - count) as usize);
        let samples = match args.preparation {
            Preparation::Prepared => &prepared[row * width..(row + batch) * width],
            Preparation::Pack => {
                for (out, input) in packed[..batch * width]
                    .chunks_exact_mut(width)
                    .zip(&source[row..row + batch])
                {
                    out.copy_from_slice(black_box(input));
                }
                &packed[..batch * width]
            }
        };
        match args.ffi {
            FfiMode::Event => {
                for (out, input) in results[..batch].iter_mut().zip(samples.chunks_exact(width)) {
                    *out = ffi::process_event(black_box(input), corpus.manifest.baseline_samples)
                        .map_err(|e| e.to_string())?;
                }
            }
            FfiMode::Batch => ffi::process_batch(
                black_box(samples),
                width,
                corpus.manifest.baseline_samples,
                &mut results[..batch],
            )
            .map_err(|e| e.to_string())?,
        }
        for result in black_box(&results[..batch]) {
            if result.status != 0 {
                return Err("invalid numerical benchmark result".into());
            }
            checksum += result.baseline
                + result.peak_amplitude
                + result.integral
                + f64::from(result.peak_index);
        }
        observed[count as usize..count as usize + batch].copy_from_slice(&results[..batch]);
        count += batch as u64;
        batches[batch] += 1;
    }
    let duration = start.elapsed().as_nanos();
    for (index, result) in observed.iter().enumerate() {
        if result != &observed[index % rows] {
            return Err("repeated corpus row produced inconsistent results".into());
        }
    }
    let verification: Vec<_> = observed
        .iter()
        .take(rows)
        .map(|r| {
            (
                r.status,
                r.baseline,
                r.peak_amplitude,
                r.peak_index,
                r.integral,
            )
        })
        .collect();
    Ok(
        serde_json::json!({"schema_version": 1, "engine": "rust_ffi",
        "events": count, "duration_ns": duration as u64, "checksum": checksum,
        "samples": width, "batch_size": args.batch_size,
        "ffi": args.ffi, "preparation": match args.preparation {
            Preparation::Prepared => "prepared", Preparation::Pack => "pack" },
        "corpus_sha256": corpus.manifest.sha256, "actual_batch_sizes": batches,
        "repeat_consistent": true, "verification": verification}),
    )
}

fn main() {
    match measure(Args::parse()) {
        Ok(result) => println!("{result}"),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
