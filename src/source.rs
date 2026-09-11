use crate::config::{RunConfig, MAX_CORPUS_BYTES, MAX_MANIFEST_BYTES};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Manifest {
    pub schema_version: u32,
    pub file: String,
    pub rows: u64,
    pub samples: u64,
    pub baseline_samples: u32,
    pub dtype: String,
    pub sha256: String,
}

pub struct Corpus {
    pub manifest: Manifest,
    pub estimated_data_bytes: u64,
    data: Vec<f64>,
}

fn regular_file(path: &Path) -> Result<File, String> {
    // Reject FIFOs/devices before opening; corpus paths are trusted local inputs.
    let metadata = std::fs::metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if !metadata.is_file() {
        return Err(format!("{} must be a regular file", path.display()));
    }
    File::open(path).map_err(|e| e.to_string())
}

impl Corpus {
    pub fn load(config: &mut RunConfig) -> Result<Self, String> {
        let file = regular_file(&config.corpus.join("manifest.json"))?;
        if file.metadata().map_err(|e| e.to_string())?.len() > MAX_MANIFEST_BYTES {
            return Err("manifest exceeds 64 KiB".into());
        }
        let mut bytes = Vec::new();
        file.take(MAX_MANIFEST_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() as u64 > MAX_MANIFEST_BYTES {
            return Err("manifest exceeds 64 KiB".into());
        }
        let mut manifest: Manifest = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        if manifest.schema_version != 1
            || manifest.file != "signals.f64le"
            || manifest.dtype != "float64-le"
        {
            return Err("require schema 1, signals.f64le and float64-le".into());
        }
        if manifest.rows == 0 || !(2..=4096).contains(&manifest.samples) {
            return Err("invalid corpus dimensions".into());
        }
        if manifest.baseline_samples == 0
            || u64::from(manifest.baseline_samples) >= manifest.samples
        {
            return Err("invalid manifest baseline_samples".into());
        }
        let baseline = config.baseline_samples.unwrap_or(manifest.baseline_samples);
        if baseline == 0 || u64::from(baseline) >= manifest.samples {
            return Err("require 1 <= baseline-samples < sample width".into());
        }
        config.baseline_samples = Some(baseline);
        let size = manifest
            .rows
            .checked_mul(manifest.samples)
            .and_then(|n| n.checked_mul(8))
            .ok_or("corpus dimensions overflow")?;
        if size > MAX_CORPUS_BYTES {
            return Err("corpus exceeds 64 MiB".into());
        }
        let estimated_data_bytes = config.memory_estimate(size, manifest.samples)?;
        if manifest.sha256.len() != 64 || !manifest.sha256.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("invalid SHA-256 encoding".into());
        }
        manifest.sha256.make_ascii_lowercase();
        let mut file = regular_file(&config.corpus.join("signals.f64le"))?;
        if file.metadata().map_err(|e| e.to_string())?.len() != size {
            return Err("corpus length does not match dimensions".into());
        }
        // Validate the hash before allocating waveform storage. Recheck while
        // loading through the same handle so a changing payload is rejected.
        let mut hasher = Sha256::new();
        let mut block = [0u8; 8192];
        let mut remaining = size;
        while remaining > 0 {
            let count = remaining.min(block.len() as u64) as usize;
            file.read_exact(&mut block[..count])
                .map_err(|e| e.to_string())?;
            hasher.update(&block[..count]);
            remaining -= count as u64;
        }
        if format!("{:x}", hasher.finalize()) != manifest.sha256 {
            return Err("corpus SHA-256 mismatch".into());
        }
        file.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
        let mut data = Vec::with_capacity((size / 8) as usize);
        let mut hasher = Sha256::new();
        remaining = size;
        while remaining > 0 {
            let count = remaining.min(block.len() as u64) as usize;
            file.read_exact(&mut block[..count])
                .map_err(|e| e.to_string())?;
            hasher.update(&block[..count]);
            for value in block[..count].as_chunks::<8>().0 {
                data.push(f64::from_le_bytes(*value));
            }
            remaining -= count as u64;
        }
        if file.read(&mut block[..1]).map_err(|e| e.to_string())? != 0
            || format!("{:x}", hasher.finalize()) != manifest.sha256
        {
            return Err("corpus changed during loading".into());
        }
        Ok(Self {
            manifest,
            estimated_data_bytes,
            data,
        })
    }

    pub fn row(&self, event_id: u64) -> &[f64] {
        let width = self.manifest.samples as usize;
        let offset = (event_id % self.manifest.rows) as usize * width;
        &self.data[offset..offset + width]
    }
}
