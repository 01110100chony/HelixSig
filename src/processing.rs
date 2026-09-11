use crate::bridge::ffi::{self, NativeResult};
use crate::config::FfiMode;
use crate::event::{Event, ProcessedEvent};
use std::time::Instant;

pub struct ProcessingBuffer {
    width: usize,
    capacity: usize,
    flat: Vec<f64>,
    native: Vec<NativeResult>,
    completed: Vec<Result<ProcessedEvent, String>>,
}

impl ProcessingBuffer {
    pub fn new(width: usize, capacity: usize) -> Self {
        assert!((2..=4096).contains(&width) && (1..=64).contains(&capacity));
        Self {
            width,
            capacity,
            flat: Vec::with_capacity(width * capacity),
            native: vec![NativeResult::default(); capacity],
            completed: Vec::with_capacity(capacity),
        }
    }

    pub fn process(
        &mut self,
        events: &[Event],
        baseline: u32,
        mode: FfiMode,
    ) -> &[Result<ProcessedEvent, String>] {
        assert!(events.len() <= self.capacity);
        self.flat.clear();
        self.completed.clear();
        for event in events {
            assert_eq!(event.samples.len(), self.width);
            self.flat.extend_from_slice(&event.samples);
        }
        // Both modes use the identical packed buffer. Only FFI call granularity
        // and the immediately following result-available timestamp differ.
        match mode {
            FfiMode::Event => {
                for (event, samples) in events.iter().zip(self.flat.chunks_exact(self.width)) {
                    let result = ffi::process_event(samples, baseline);
                    let completed = Instant::now();
                    self.completed.push(convert(
                        event,
                        result.map_err(|e| e.to_string()),
                        completed,
                    ));
                }
            }
            FfiMode::Batch => {
                let result = ffi::process_batch(
                    &self.flat,
                    self.width,
                    baseline,
                    &mut self.native[..events.len()],
                );
                let completed = Instant::now();
                for (event, native) in events.iter().zip(&self.native) {
                    let result = result.as_ref().map(|_| *native).map_err(|e| e.to_string());
                    self.completed.push(convert(event, result, completed));
                }
            }
        }
        &self.completed
    }
}

fn convert(
    event: &Event,
    result: Result<NativeResult, String>,
    completed: Instant,
) -> Result<ProcessedEvent, String> {
    let result = result?;
    if result.status != 0 {
        return Err(format!(
            "event {}: native status {}",
            event.event_id, result.status
        ));
    }
    Ok(ProcessedEvent {
        event_id: event.event_id,
        channel_id: event.channel_id,
        sequence: event.sequence,
        result,
        latency_ns: completed
            .duration_since(event.admission_attempt)
            .as_nanos()
            .min(u64::MAX as u128) as u64,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn latency_uses_admission_clock_and_batch_results_share_completion() {
        for mode in [FfiMode::Event, FfiMode::Batch] {
            let mut buffer = ProcessingBuffer::new(6, 2);
            let before = Instant::now();
            let events = [20, 10].map(|age_ms| Event {
                event_id: age_ms,
                channel_id: (age_ms % 4) as u32,
                sequence: age_ms / 4,
                samples: vec![1.0, 1.0, 4.0, 3.0, 2.0, 0.0],
                admission_attempt: before - Duration::from_millis(age_ms),
            });
            let results = buffer.process(&events, 2, mode);
            let after = Instant::now();
            let available: Vec<_> = events
                .iter()
                .zip(results)
                .map(|(event, result)| {
                    let result = result.as_ref().unwrap();
                    let available =
                        event.admission_attempt + Duration::from_nanos(result.latency_ns);
                    assert!(available >= before && available <= after);
                    available
                })
                .collect();
            if mode == FfiMode::Batch {
                assert_eq!(available[0], available[1]);
            } else {
                assert!(available[0] <= available[1]);
            }
        }
    }
}
