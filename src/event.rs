use crate::bridge::ffi::NativeResult;
use std::time::Instant;

pub struct Event {
    pub event_id: u64,
    pub channel_id: u32,
    pub sequence: u64,
    pub samples: Vec<f64>,
    pub admission_attempt: Instant,
}

impl Event {
    pub fn new(event_id: u64, samples: &[f64]) -> Self {
        // Materialize first, then begin the admission attempt (no queue in H2).
        let samples = samples.to_vec();
        Self {
            event_id,
            channel_id: (event_id % 4) as u32,
            sequence: event_id / 4,
            samples,
            admission_attempt: Instant::now(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProcessedEvent {
    pub event_id: u64,
    pub channel_id: u32,
    pub sequence: u64,
    pub result: NativeResult,
    pub latency_ns: u64,
}
