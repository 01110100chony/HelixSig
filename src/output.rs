use crate::event::ProcessedEvent;
use parquet::basic::{Compression, Encoding};
use parquet::data_type::{DoubleType, Int32Type, Int64Type};
use parquet::file::properties::WriterProperties;
use parquet::file::writer::SerializedFileWriter;
use parquet::schema::parser::parse_message_type;
use serde::Serialize;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub const ROW_GROUP_SIZE: usize = 4096;
const SCHEMA: &str = "message helix {
    REQUIRED INT64 event_id (INTEGER(64, false));
    REQUIRED INT32 channel_id (INTEGER(32, false));
    REQUIRED INT64 sequence (INTEGER(64, false));
    REQUIRED DOUBLE baseline;
    REQUIRED DOUBLE peak_amplitude;
    REQUIRED INT32 peak_index (INTEGER(32, false));
    REQUIRED DOUBLE integral;
    REQUIRED INT64 latency_ns (INTEGER(64, false));
}";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OutputState {
    NotCreated,
    Incomplete,
    Finalized,
    Partial,
}

#[derive(Debug, Clone, Serialize)]
pub struct OutputSummary {
    pub state: OutputState,
    pub file: Option<PathBuf>,
    pub schema_version: u32,
}

impl Default for OutputSummary {
    fn default() -> Self {
        Self {
            state: OutputState::NotCreated,
            file: None,
            schema_version: 1,
        }
    }
}

/// Runtime boundary also used to exercise deterministic write/close failures.
pub(crate) trait ResultSink {
    fn push(&mut self, event: &ProcessedEvent) -> Result<(), String>;
    fn finish(&mut self, partial: bool) -> Result<u64, String>;
    fn summary(&self) -> OutputSummary;
}

struct ParquetSink<W: Write + Send> {
    writer: Option<SerializedFileWriter<W>>,
    rows: Vec<ProcessedEvent>,
    submitted: u64,
}

impl<W: Write + Send> ParquetSink<W> {
    fn new(output: W) -> Result<Self, String> {
        let schema = Arc::new(parse_message_type(SCHEMA).map_err(|e| e.to_string())?);
        let properties = Arc::new(
            WriterProperties::builder()
                .set_compression(Compression::UNCOMPRESSED)
                .set_dictionary_enabled(false)
                .set_encoding(Encoding::PLAIN)
                .set_max_row_group_size(ROW_GROUP_SIZE)
                .build(),
        );
        let writer =
            SerializedFileWriter::new(output, schema, properties).map_err(|e| e.to_string())?;
        Ok(Self {
            writer: Some(writer),
            rows: Vec::with_capacity(ROW_GROUP_SIZE),
            submitted: 0,
        })
    }

    fn push(&mut self, event: &ProcessedEvent) -> Result<(), String> {
        self.rows.push(event.clone());
        if self.rows.len() == ROW_GROUP_SIZE {
            self.flush_group()?;
        }
        Ok(())
    }

    fn flush_group(&mut self) -> Result<(), String> {
        if self.rows.is_empty() {
            return Ok(());
        }
        let writer = self.writer.as_mut().ok_or("writer already closed")?;
        let mut group = writer.next_row_group().map_err(|e| e.to_string())?;
        for index in 0..8 {
            let mut column = group
                .next_column()
                .map_err(|e| e.to_string())?
                .ok_or("missing column")?;
            match index {
                0 | 2 | 7 => {
                    let values: Vec<i64> = self
                        .rows
                        .iter()
                        .map(|r| match index {
                            0 => r.event_id as i64,
                            2 => r.sequence as i64,
                            _ => r.latency_ns as i64,
                        })
                        .collect();
                    column
                        .typed::<Int64Type>()
                        .write_batch(&values, None, None)
                        .map_err(|e| e.to_string())?;
                }
                1 | 5 => {
                    let values: Vec<i32> = self
                        .rows
                        .iter()
                        .map(|r| {
                            if index == 1 {
                                r.channel_id as i32
                            } else {
                                r.result.peak_index as i32
                            }
                        })
                        .collect();
                    column
                        .typed::<Int32Type>()
                        .write_batch(&values, None, None)
                        .map_err(|e| e.to_string())?;
                }
                _ => {
                    let values: Vec<f64> = self
                        .rows
                        .iter()
                        .map(|r| match index {
                            3 => r.result.baseline,
                            4 => r.result.peak_amplitude,
                            _ => r.result.integral,
                        })
                        .collect();
                    column
                        .typed::<DoubleType>()
                        .write_batch(&values, None, None)
                        .map_err(|e| e.to_string())?;
                }
            }
            column.close().map_err(|e| e.to_string())?;
        }
        group.close().map_err(|e| e.to_string())?;
        self.submitted += self.rows.len() as u64;
        self.rows.clear();
        Ok(())
    }

    fn close(&mut self) -> Result<u64, String> {
        self.flush_group()?;
        self.writer
            .take()
            .ok_or("writer already closed")?
            .close()
            .map_err(|e| e.to_string())?;
        Ok(self.submitted)
    }
}

pub(crate) struct OutputWriter {
    sink: ParquetSink<File>,
    directory: PathBuf,
    output: OutputSummary,
}

impl OutputWriter {
    pub fn open(directory: &Path) -> Result<Self, String> {
        let path = directory.join("events.parquet.incomplete");
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|e| e.to_string())?;
        Ok(Self {
            sink: ParquetSink::new(file)?,
            directory: directory.to_owned(),
            output: OutputSummary {
                state: OutputState::Incomplete,
                file: Some(path),
                schema_version: 1,
            },
        })
    }
}

impl ResultSink for OutputWriter {
    fn push(&mut self, event: &ProcessedEvent) -> Result<(), String> {
        self.sink.push(event)
    }
    fn finish(&mut self, partial: bool) -> Result<u64, String> {
        let count = self.sink.close()?;
        let target = self.directory.join(if partial {
            "events.partial.parquet"
        } else {
            "events.parquet"
        });
        fs::rename(self.output.file.as_ref().expect("opened output"), &target)
            .map_err(|e| e.to_string())?;
        self.output.file = Some(target);
        self.output.state = if partial {
            OutputState::Partial
        } else {
            OutputState::Finalized
        };
        Ok(count)
    }
    fn summary(&self) -> OutputSummary {
        self.output.clone()
    }
}

pub(crate) fn write_json_new(path: &Path, value: &impl Serialize) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    serde_json::to_writer_pretty(&mut file, value).map_err(|e| e.to_string())?;
    file.write_all(b"\n").map_err(|e| e.to_string())?;
    file.flush().map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::ffi::NativeResult;
    use std::io;
    use std::sync::atomic::{AtomicBool, Ordering};

    struct FailingWriter(Arc<AtomicBool>);
    impl Write for FailingWriter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if self.0.load(Ordering::Relaxed) {
                Err(io::Error::other("injected write failure"))
            } else {
                Ok(bytes.len())
            }
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    fn row() -> ProcessedEvent {
        ProcessedEvent {
            event_id: 0,
            channel_id: 0,
            sequence: 0,
            result: NativeResult::default(),
            latency_ns: 1,
        }
    }
    #[test]
    fn output_open_failure_preserves_existing_file() {
        let directory = tempfile::tempdir().unwrap();
        let incomplete = directory.path().join("events.parquet.incomplete");
        fs::write(&incomplete, b"existing data").unwrap();
        assert!(OutputWriter::open(directory.path()).is_err());
        assert_eq!(fs::read(incomplete).unwrap(), b"existing data");
    }
    #[test]
    fn real_parquet_writer_propagates_group_and_footer_errors() {
        for footer in [false, true] {
            let fail = Arc::new(AtomicBool::new(false));
            let mut sink = ParquetSink::new(FailingWriter(fail.clone())).unwrap();
            sink.push(&row()).unwrap();
            if footer {
                sink.flush_group().unwrap();
            }
            fail.store(true, Ordering::Relaxed);
            assert!(sink.close().is_err());
        }
    }
    #[test]
    fn rename_failure_preserves_incomplete_state() {
        let directory = tempfile::tempdir().unwrap();
        let mut writer = OutputWriter::open(directory.path()).unwrap();
        writer.push(&row()).unwrap();
        fs::create_dir(directory.path().join("events.parquet")).unwrap();
        assert!(writer.finish(false).is_err());
        assert_eq!(writer.summary().state, OutputState::Incomplete);
        assert!(directory.path().join("events.parquet.incomplete").is_file());
    }
}
