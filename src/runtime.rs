use crate::config::{Execution, RunConfig};
use crate::control::Control;
use crate::event::{Event, ProcessedEvent};
use crate::metrics::{Metrics, MetricsSummary};
use crate::output::{write_json_new, OutputState, OutputSummary, OutputWriter, ResultSink};
use crate::processing::ProcessingBuffer;
use crate::source::Corpus;
use serde::Serialize;
use std::fs;
use std::time::Instant;

#[derive(Debug, Default, Serialize)]
pub struct Counters {
    pub produced: u64,
    pub accepted: u64,
    pub dropped: u64,
    pub not_admitted: u64,
    pub processed: u64,
    pub failed: u64,
    pub aborted: u64,
    pub written: u64,
    pub unwritten: u64,
}

impl Counters {
    pub fn reconciles(&self) -> bool {
        self.produced == self.accepted + self.dropped + self.not_admitted
            && self.accepted == self.processed + self.failed + self.aborted
            && self.processed == self.written + self.unwritten
    }
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Completed,
    CompletedWithDrops,
    Interrupted,
    Failed,
    InvalidConfiguration,
}

#[derive(Debug, Serialize)]
pub struct RunSummary {
    pub schema_version: u32,
    pub config: RunConfig,
    pub corpus_sha256: Option<String>,
    pub estimated_data_bytes: Option<u64>,
    pub status: RunStatus,
    pub reason: Option<String>,
    pub diagnostics: Vec<String>,
    #[serde(flatten)]
    pub counters: Counters,
    pub duration_ns: u64,
    pub throughput_processed_per_second: Option<f64>,
    pub throughput_written_per_second: Option<f64>,
    pub metrics: MetricsSummary,
    /// Worker-index order; empty in sequential execution.
    pub worker_metrics: Vec<MetricsSummary>,
    pub output: OutputSummary,
}

impl RunSummary {
    pub(crate) fn new(config: RunConfig) -> Self {
        Self {
            schema_version: 1,
            config,
            corpus_sha256: None,
            estimated_data_bytes: None,
            status: RunStatus::Completed,
            reason: None,
            diagnostics: Vec::with_capacity(16),
            counters: Counters::default(),
            duration_ns: 0,
            throughput_processed_per_second: None,
            throughput_written_per_second: None,
            metrics: Metrics::default().summary(),
            worker_metrics: Vec::new(),
            output: OutputSummary::default(),
        }
    }
    pub(crate) fn fail(&mut self, reason: String) {
        let first_failure = self.status != RunStatus::Failed;
        self.status = RunStatus::Failed;
        if first_failure || self.reason.is_none() {
            self.reason = Some(reason.clone());
        }
        if self.diagnostics.len() < 16 {
            self.diagnostics.push(reason);
        }
    }
    pub fn exit_code(&self) -> i32 {
        match self.status {
            RunStatus::Completed => 0,
            RunStatus::CompletedWithDrops => 2,
            RunStatus::Interrupted => 130,
            RunStatus::Failed => 1,
            RunStatus::InvalidConfiguration => 64,
        }
    }
}

pub fn run(config: RunConfig) -> RunSummary {
    let control = Control::default();
    let _registration = match control.register_sigint() {
        Ok(registration) => registration,
        Err(error) => {
            let mut summary = RunSummary::new(config);
            summary.fail(format!("SIGINT handler setup: {error}"));
            return summary;
        }
    };
    run_controlled(config, &control)
}

fn run_controlled(config: RunConfig, control: &Control) -> RunSummary {
    let mut summary = RunSummary::new(config);
    let corpus = match summary
        .config
        .validate()
        .and_then(|()| Corpus::load(&mut summary.config))
    {
        Ok(corpus) => corpus,
        Err(reason) => {
            summary.status = RunStatus::InvalidConfiguration;
            summary.reason = Some(reason);
            return summary;
        }
    };
    summary.corpus_sha256 = Some(corpus.manifest.sha256.clone());
    summary.estimated_data_bytes = Some(corpus.estimated_data_bytes);
    if let Err(error) = fs::create_dir(&summary.config.out) {
        if error.kind() == std::io::ErrorKind::AlreadyExists {
            summary.status = RunStatus::InvalidConfiguration;
            summary.reason = Some("output path already exists".into());
        } else {
            summary.fail(format!("output startup: {error}"));
        }
        return summary;
    }
    let marker = serde_json::json!({"schema_version": 1, "status": "running",
        "config": summary.config, "corpus_sha256": summary.corpus_sha256});
    match write_json_new(&summary.config.out.join("running.json"), &marker)
        .and_then(|()| OutputWriter::open(&summary.config.out))
    {
        Ok(mut writer) => match summary.config.execution {
            Execution::Sequential => execute(&corpus, &mut writer, &mut summary, control),
            Execution::Concurrent => {
                crate::concurrent::execute(&corpus, &mut writer, &mut summary, control)
            }
        },
        Err(reason) => {
            summary.fail(format!("output startup: {reason}"));
            let incomplete = summary.config.out.join("events.parquet.incomplete");
            if incomplete.exists() {
                summary.output = OutputSummary {
                    state: OutputState::Incomplete,
                    file: Some(incomplete),
                    schema_version: 1,
                };
            }
        }
    }
    finalize_summary(&mut summary);
    summary
}

fn finalize_summary(summary: &mut RunSummary) {
    // The running marker is historical. A finalized summary supersedes it;
    // retaining it avoids a gap with neither marker nor final JSON on failure.
    let temporary = summary.config.out.join("summary.json.incomplete");
    if let Err(reason) = write_json_new(&temporary, &summary).and_then(|()| {
        fs::rename(&temporary, summary.config.out.join("summary.json")).map_err(|e| e.to_string())
    }) {
        summary.fail(format!("summary finalization: {reason}"));
        mark_output_partial(summary);
    }
}

fn mark_output_partial(summary: &mut RunSummary) {
    if summary.output.state == OutputState::Finalized {
        let partial = summary.config.out.join("events.partial.parquet");
        match fs::rename(
            summary.output.file.as_ref().expect("finalized output"),
            &partial,
        ) {
            Ok(()) => {
                summary.output.state = OutputState::Partial;
                summary.output.file = Some(partial);
            }
            Err(error) => summary.fail(format!("partial output rename: {error}")),
        }
    }
}

fn execute(
    corpus: &Corpus,
    writer: &mut impl ResultSink,
    summary: &mut RunSummary,
    control: &Control,
) {
    let start = Instant::now();
    let mut processing =
        ProcessingBuffer::new(corpus.manifest.samples as usize, summary.config.batch_size);
    let mut events = Vec::with_capacity(summary.config.batch_size);
    let mut metrics = Metrics::default();
    let mut writer_failed = false;
    while summary.counters.produced < summary.config.events
        && !writer_failed
        && !control.requested()
    {
        events.clear();
        while events.len() < summary.config.batch_size
            && summary.counters.produced < summary.config.events
        {
            let id = summary.counters.produced;
            let event = Event::new(id, corpus.row(id));
            summary.counters.produced += 1;
            if control.requested() {
                summary.counters.not_admitted += 1;
                break;
            }
            events.push(event);
            summary.counters.accepted += 1;
        }
        if events.is_empty() {
            break;
        }
        metrics.record_batch(events.len());
        for result in processing.process(
            &events,
            summary.config.baseline_samples.expect("validated baseline"),
            summary.config.ffi,
        ) {
            if let Ok(event) = result {
                metrics.record_latency(event.latency_ns);
            }
            collect_result(result, writer, summary, &mut writer_failed);
        }
    }
    finish_execution(writer, summary, &metrics, writer_failed, start, control);
}

pub(crate) fn collect_result(
    result: &Result<ProcessedEvent, String>,
    writer: &mut impl ResultSink,
    summary: &mut RunSummary,
    writer_failed: &mut bool,
) {
    match result {
        Ok(event) => {
            summary.counters.processed += 1;
            // Valid results remain unwritten until the entire file is finalized.
            summary.counters.unwritten += 1;
            if !*writer_failed {
                if let Err(reason) = writer.push(event) {
                    *writer_failed = true;
                    summary.fail(format!("writer: {reason}"));
                }
            }
        }
        Err(reason) => {
            summary.counters.failed += 1;
            summary.fail(reason.clone());
        }
    }
}

pub(crate) fn finish_execution(
    writer: &mut impl ResultSink,
    summary: &mut RunSummary,
    metrics: &Metrics,
    writer_failed: bool,
    start: Instant,
    control: &Control,
) {
    if summary.status == RunStatus::Completed {
        if control.interrupted() {
            summary.status = RunStatus::Interrupted;
            summary.reason = Some("SIGINT requested; accepted work drained".into());
        } else if summary.counters.dropped > 0 {
            summary.status = RunStatus::CompletedWithDrops;
            summary.reason = Some("drop-new rejected events from the full input queue".into());
        }
    }
    if !writer_failed {
        match writer.finish(matches!(
            summary.status,
            RunStatus::Failed | RunStatus::Interrupted
        )) {
            Ok(written) => {
                // Transition only the rows acknowledged by successful finalization.
                summary.counters.written += written;
                summary.counters.unwritten -= written;
            }
            Err(reason) => summary.fail(format!("writer finalization: {reason}")),
        }
    }
    summary.output = writer.summary();
    // SIGINT can arrive while the footer is closing. Keep already finalized
    // valid rows, but label the interrupted run with its actual partial path.
    if control.interrupted()
        && matches!(
            summary.status,
            RunStatus::Completed | RunStatus::CompletedWithDrops
        )
    {
        summary.status = RunStatus::Interrupted;
        summary.reason = Some("SIGINT requested during output finalization".into());
        mark_output_partial(summary);
    }
    summary.duration_ns = start.elapsed().as_nanos().min(u64::MAX as u128) as u64;
    if summary.duration_ns > 0 {
        let seconds = summary.duration_ns as f64 / 1e9;
        summary.throughput_processed_per_second = Some(summary.counters.processed as f64 / seconds);
        summary.throughput_written_per_second = Some(summary.counters.written as f64 / seconds);
    }
    summary.metrics = metrics.summary();
    assert!(
        summary.counters.reconciles(),
        "accounting transition defect"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Execution, FfiMode, Policy};
    use crate::event::ProcessedEvent;
    use std::path::PathBuf;
    use std::time::Duration;

    struct TestSink {
        fail_at: usize,
        calls: usize,
        finalize_failed: bool,
        delay: Duration,
    }
    impl ResultSink for TestSink {
        fn push(&mut self, _: &ProcessedEvent) -> Result<(), String> {
            self.calls += 1;
            std::thread::sleep(self.delay);
            if self.calls == self.fail_at {
                Err("injected write failure".into())
            } else {
                Ok(())
            }
        }
        fn finish(&mut self, _: bool) -> Result<u64, String> {
            if self.finalize_failed {
                Err("injected close failure".into())
            } else {
                Ok(self.calls as u64)
            }
        }
        fn summary(&self) -> OutputSummary {
            OutputSummary::default()
        }
    }
    fn prepared() -> (Corpus, RunSummary) {
        let mut config = RunConfig {
            corpus: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/small"),
            out: PathBuf::new(),
            events: 33,
            execution: Execution::Sequential,
            workers: 1,
            queue_capacity: 1,
            batch_size: 16,
            ffi: FfiMode::Batch,
            policy: Policy::Block,
            baseline_samples: None,
        };
        let corpus = Corpus::load(&mut config).unwrap();
        (corpus, RunSummary::new(config))
    }
    #[test]
    fn write_failure_drains_admitted_batch_without_fabricating_attempts() {
        let (corpus, mut summary) = prepared();
        let mut sink = TestSink {
            fail_at: 3,
            calls: 0,
            finalize_failed: false,
            delay: Duration::ZERO,
        };
        execute(&corpus, &mut sink, &mut summary, &Control::default());
        assert_eq!(summary.status, RunStatus::Failed);
        assert_eq!(summary.counters.produced, 16);
        assert_eq!(summary.counters.accepted, 16);
        assert_eq!(summary.counters.processed, 16);
        assert_eq!(summary.counters.unwritten, 16);
        assert_eq!(summary.counters.written, 0);
        assert_eq!(sink.calls, 3);
        assert!(summary.counters.reconciles());
    }
    #[test]
    fn footer_failure_never_counts_buffered_rows_as_written() {
        let (corpus, mut summary) = prepared();
        let mut sink = TestSink {
            fail_at: usize::MAX,
            calls: 0,
            finalize_failed: true,
            delay: Duration::ZERO,
        };
        execute(&corpus, &mut sink, &mut summary, &Control::default());
        assert_eq!(summary.status, RunStatus::Failed);
        assert_eq!(summary.counters.unwritten, 33);
        assert_eq!(summary.counters.written, 0);
        assert!(summary.counters.reconciles());
    }
    #[test]
    fn summary_failure_keeps_valid_rows_and_marks_output_partial() {
        let directory = tempfile::tempdir().unwrap();
        let (corpus, mut summary) = prepared();
        summary.config.out = directory.path().to_owned();
        let mut writer = OutputWriter::open(directory.path()).unwrap();
        execute(&corpus, &mut writer, &mut summary, &Control::default());
        fs::create_dir(directory.path().join("summary.json.incomplete")).unwrap();
        finalize_summary(&mut summary);
        assert_eq!(summary.status, RunStatus::Failed);
        assert_eq!(summary.output.state, OutputState::Partial);
        assert_eq!(summary.counters.written, 33);
        assert!(summary.counters.reconciles());
        assert!(directory.path().join("events.partial.parquet").is_file());
        assert!(!directory.path().join("events.parquet").exists());
        assert!(!directory.path().join("summary.json").exists());
    }
    #[test]
    fn writer_delay_is_in_duration_but_not_successful_event_latency() {
        for ffi in [FfiMode::Event, FfiMode::Batch] {
            let (corpus, mut summary) = prepared();
            summary.config.events = 1;
            summary.config.ffi = ffi;
            let mut sink = TestSink {
                fail_at: usize::MAX,
                calls: 0,
                finalize_failed: false,
                delay: Duration::from_millis(30),
            };
            execute(&corpus, &mut sink, &mut summary, &Control::default());
            assert!(summary.duration_ns >= 30_000_000);
            assert!(summary.metrics.latency_p99_ns.unwrap() < summary.duration_ns - 20_000_000);
        }
    }

    #[test]
    fn interrupt_during_footer_finalization_keeps_valid_partial_output() {
        struct InterruptOnClose<'a> {
            writer: OutputWriter,
            control: &'a Control,
        }
        impl ResultSink for InterruptOnClose<'_> {
            fn push(&mut self, event: &ProcessedEvent) -> Result<(), String> {
                self.writer.push(event)
            }
            fn finish(&mut self, partial: bool) -> Result<u64, String> {
                let result = self.writer.finish(partial);
                self.control.interrupt();
                result
            }
            fn summary(&self) -> OutputSummary {
                self.writer.summary()
            }
        }
        let (corpus, mut summary) = prepared();
        let directory = tempfile::tempdir().unwrap();
        summary.config.out = directory.path().to_owned();
        let control = Control::default();
        let mut writer = InterruptOnClose {
            writer: OutputWriter::open(directory.path()).unwrap(),
            control: &control,
        };
        execute(&corpus, &mut writer, &mut summary, &control);
        assert_eq!(summary.exit_code(), 130);
        assert_eq!(summary.output.state, OutputState::Partial);
        assert_eq!(summary.counters.written, 33);
        assert!(directory.path().join("events.partial.parquet").is_file());
        assert!(!directory.path().join("events.parquet").exists());
        assert!(summary.counters.reconciles());
    }
}
