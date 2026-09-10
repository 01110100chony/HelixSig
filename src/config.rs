use clap::{Args, ValueEnum};
use serde::Serialize;
use std::path::PathBuf;

pub const MAX_CORPUS_BYTES: u64 = 64 * 1024 * 1024;
pub const MAX_MANIFEST_BYTES: u64 = 64 * 1024;
pub const MAX_DATA_BYTES: u64 = 256 * 1024 * 1024;

#[derive(Debug, Clone, Copy, Serialize, ValueEnum, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Execution {
    Sequential,
    Concurrent,
}

#[derive(Debug, Clone, Copy, Serialize, ValueEnum, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum FfiMode {
    Event,
    Batch,
}

#[derive(Debug, Clone, Copy, Serialize, ValueEnum, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Policy {
    Block,
    DropNew,
}

#[derive(Debug, Clone, Args, Serialize)]
pub struct RunConfig {
    #[arg(long)]
    pub corpus: PathBuf,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(long, default_value_t = 100_000)]
    pub events: u64,
    #[arg(long, value_enum, default_value_t = Execution::Sequential)]
    pub execution: Execution,
    #[arg(long, default_value_t = 2)]
    pub workers: usize,
    #[arg(long, default_value_t = 256)]
    pub queue_capacity: usize,
    #[arg(long, default_value_t = 16)]
    pub batch_size: usize,
    #[arg(long, value_enum, default_value_t = FfiMode::Batch)]
    pub ffi: FfiMode,
    #[arg(long, value_enum, default_value_t = Policy::Block)]
    pub policy: Policy,
    /// Defaults to the corpus manifest's baseline width.
    #[arg(long)]
    pub baseline_samples: Option<u32>,
}

impl RunConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.execution != Execution::Sequential {
            return Err("concurrent execution is unavailable until H3".into());
        }
        if !(1..=1_000_000).contains(&self.events) {
            return Err("events must be in 1..=1000000".into());
        }
        if !(1..=64).contains(&self.batch_size) || !(1..=4096).contains(&self.queue_capacity) {
            return Err("batch-size must be in 1..=64 and queue-capacity in 1..=4096".into());
        }
        let processors = std::thread::available_parallelism()
            .map_err(|e| e.to_string())?
            .get();
        if self.workers == 0 || self.workers > processors {
            return Err(format!("workers must be in 1..={processors}"));
        }
        if self.baseline_samples.is_some_and(|k| k == 0 || k >= 4096) {
            return Err("baseline-samples must be in 1..4096".into());
        }
        match std::fs::symlink_metadata(&self.out) {
            Ok(_) => return Err("output path already exists".into()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(format!("cannot inspect output path: {e}")),
        }
        Ok(())
    }

    pub fn memory_estimate(&self, corpus_bytes: u64, width: u64) -> Result<u64, String> {
        // Conservative even in sequential mode: future queue/event/flat buffers,
        // twice the corpus for loading, and 32 MiB for writer, metrics and metadata.
        let estimate = (|| {
            let slots = (self.workers as u64)
                .checked_mul(self.batch_size as u64)?
                .checked_mul(2)?
                .checked_add(self.queue_capacity as u64)?
                .checked_add(1)?;
            corpus_bytes
                .checked_mul(2)?
                .checked_add(width.checked_mul(8)?.checked_mul(slots)?)?
                .checked_add(32 * 1024 * 1024)
        })()
        .ok_or("memory estimate overflow")?;
        if estimate > MAX_DATA_BYTES {
            return Err(format!(
                "estimated data buffers {estimate} exceed {MAX_DATA_BYTES} bytes"
            ));
        }
        Ok(estimate)
    }
}
