//! Bounded Rust runtime. The main thread always drains the result channel.
use crate::config::{Policy, RunConfig};
use crate::control::Control;
use crate::event::{Event, ProcessedEvent};
use crate::metrics::Metrics;
use crate::output::ResultSink;
use crate::processing::ProcessingBuffer;
use crate::runtime::{collect_result, finish_execution, RunSummary};
use crate::source::Corpus;
use crossbeam_channel::{
    bounded, Receiver, RecvTimeoutError, SendTimeoutError, Sender, TrySendError,
};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::thread;
use std::time::{Duration, Instant};

const RESULT_CAPACITY: usize = 256;
const STOP_POLL: Duration = Duration::from_millis(10);

#[cfg(not(test))]
#[derive(Default)]
struct Hooks {}
#[cfg(test)]
struct Hooks(std::sync::Arc<dyn Fn(Point, u64) + Send + Sync>);
#[cfg(test)]
impl Default for Hooks {
    fn default() -> Self {
        Self(std::sync::Arc::new(|_, _| {}))
    }
}
#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Point {
    Produced,
    Accepted,
    Dropped,
    NotAdmitted,
    ProducerDone,
    BeforeReceive(usize),
    BeforeProcess(usize),
    Processed(usize),
    Failed(usize),
    Panicked(usize),
    Aborted,
}
// Observer expressions are entirely absent from production builds.
macro_rules! observe {
    ($hooks:ident, $point:expr, $id:expr) => {
        #[cfg(test)]
        ($hooks.0)($point, $id);
    };
}

#[derive(Default)]
struct ProducerReport {
    produced: u64,
    accepted: u64,
    dropped: u64,
    not_admitted: u64,
    panicked: bool,
}
#[derive(Default)]
struct WorkerReport {
    metrics: Metrics,
    aborted: u64,
    panicked: bool,
}
type Outcome = Result<ProcessedEvent, String>;

pub(crate) fn execute(
    corpus: &Corpus,
    writer: &mut impl ResultSink,
    summary: &mut RunSummary,
    stop: &Control,
) {
    execute_with_hooks(corpus, writer, summary, stop, &Hooks::default());
}

fn execute_with_hooks(
    corpus: &Corpus,
    writer: &mut impl ResultSink,
    summary: &mut RunSummary,
    stop: &Control,
    hooks: &Hooks,
) {
    let start = Instant::now();
    let config = summary.config.clone();
    let (input_tx, supervisor_rx) = bounded(config.queue_capacity);
    let (result_tx, result_rx) = bounded(RESULT_CAPACITY);
    let mut metrics = Metrics::default();
    let mut writer_failed = false;

    thread::scope(|scope| {
        let mut workers = Vec::with_capacity(config.workers);
        // No producer is started if any worker creation fails.
        for index in 0..config.workers {
            let input = supervisor_rx.clone();
            let results = result_tx.clone();
            let config = &config;
            match thread::Builder::new()
                .name(format!("helix-worker-{index}"))
                .spawn_scoped(scope, move || {
                    worker(
                        input,
                        results,
                        config,
                        corpus.manifest.samples as usize,
                        stop,
                        index,
                        hooks,
                    )
                }) {
                Ok(handle) => workers.push(handle),
                Err(error) => {
                    stop.abort();
                    summary.fail(format!("worker {index} startup: {error}"));
                    break;
                }
            }
        }
        drop(result_tx);
        let producer = if !stop.fatal() {
            let config = &config;
            match thread::Builder::new()
                .name("helix-producer".into())
                .spawn_scoped(scope, move || {
                    produce(corpus, input_tx, config, stop, hooks)
                }) {
                Ok(handle) => Some(handle),
                Err(error) => {
                    stop.abort();
                    summary.fail(format!("producer startup: {error}"));
                    None
                }
            }
        } else {
            drop(input_tx);
            None
        };

        // A writer failure must never disconnect the collector: draining releases
        // blocked sends and lets all accepted work reach a terminal state.
        for outcome in result_rx {
            collect_result(&outcome, writer, summary, &mut writer_failed);
            if writer_failed {
                stop.graceful();
            }
        }
        if let Some(producer) = producer {
            let report = producer.join().expect("producer panic is contained");
            summary.counters.produced += report.produced;
            summary.counters.accepted += report.accepted;
            summary.counters.dropped += report.dropped;
            summary.counters.not_admitted += report.not_admitted;
            if report.panicked {
                summary.fail("producer panicked".into());
            }
        }
        for (index, worker) in workers.into_iter().enumerate() {
            let report = worker.join().expect("worker panic is contained");
            summary.counters.aborted += report.aborted;
            metrics.merge(&report.metrics);
            summary.worker_metrics.push(report.metrics.summary());
            if report.panicked {
                summary.fail(format!("worker {index} panicked"));
            }
        }
        // Producer and workers have terminated. Explicitly classify the accepted
        // queue entries still owned by the retained supervisor receiver.
        for _event in supervisor_rx.try_iter() {
            summary.counters.aborted += 1;
            observe!(hooks, Point::Aborted, _event.event_id);
        }
    });
    if stop.fatal() && summary.reason.is_none() {
        summary.fail("fatal runtime stop".into());
    }
    finish_execution(writer, summary, &metrics, writer_failed, start, stop);
}

fn produce(
    corpus: &Corpus,
    input: Sender<Event>,
    config: &RunConfig,
    stop: &Control,
    _hooks: &Hooks,
) -> ProducerReport {
    let mut report = ProducerReport::default();
    let mut pending = None;
    let result = catch_unwind(AssertUnwindSafe(|| {
        for id in 0..config.events {
            if stop.requested() {
                break;
            }
            let mut event = Event::new(id, corpus.row(id));
            report.produced += 1;
            pending = Some(id);
            observe!(_hooks, Point::Produced, id);
            loop {
                if stop.requested() {
                    report.not_admitted += 1;
                    pending = None;
                    observe!(_hooks, Point::NotAdmitted, id);
                    return;
                }
                if config.policy == Policy::DropNew {
                    match input.try_send(event) {
                        Ok(()) => {
                            report.accepted += 1;
                            pending = None;
                            observe!(_hooks, Point::Accepted, id);
                        }
                        Err(TrySendError::Full(_)) => {
                            report.dropped += 1;
                            pending = None;
                            observe!(_hooks, Point::Dropped, id);
                        }
                        Err(TrySendError::Disconnected(_)) => {
                            report.not_admitted += 1;
                            pending = None;
                            observe!(_hooks, Point::NotAdmitted, id);
                            stop.abort();
                        }
                    }
                    break;
                }
                match input.send_timeout(event, STOP_POLL) {
                    Ok(()) => {
                        report.accepted += 1;
                        pending = None;
                        observe!(_hooks, Point::Accepted, id);
                        break;
                    }
                    Err(SendTimeoutError::Timeout(returned)) => event = returned,
                    Err(SendTimeoutError::Disconnected(_)) => {
                        report.not_admitted += 1;
                        pending = None;
                        observe!(_hooks, Point::NotAdmitted, id);
                        stop.abort();
                        return;
                    }
                }
            }
        }
    }));
    if result.is_err() {
        stop.abort();
        report.panicked = true;
        if let Some(_id) = pending {
            report.not_admitted += 1;
            observe!(_hooks, Point::NotAdmitted, _id);
        }
    }
    observe!(_hooks, Point::ProducerDone, report.produced);
    report
}

fn worker(
    input: Receiver<Event>,
    results: Sender<Outcome>,
    config: &RunConfig,
    width: usize,
    stop: &Control,
    _index: usize,
    _hooks: &Hooks,
) -> WorkerReport {
    let mut report = WorkerReport::default();
    let mut processing = ProcessingBuffer::new(width, config.batch_size);
    let mut events = Vec::with_capacity(config.batch_size);
    let mut delivered = 0;
    let result = catch_unwind(AssertUnwindSafe(|| loop {
        events.clear();
        delivered = 0;
        if stop.fatal() {
            return;
        }
        observe!(_hooks, Point::BeforeReceive(_index), 0);
        match input.recv_timeout(STOP_POLL) {
            Ok(event) => events.push(event),
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => return,
        }
        while events.len() < config.batch_size {
            match input.try_recv() {
                Ok(event) => events.push(event),
                Err(_) => break,
            }
        }
        if stop.fatal() {
            return;
        }
        report.metrics.record_batch(events.len());
        observe!(_hooks, Point::BeforeProcess(_index), events[0].event_id);
        for outcome in processing.process(
            &events,
            config.baseline_samples.expect("validated baseline"),
            config.ffi,
        ) {
            if results.send(outcome.clone()).is_err() {
                stop.abort();
                return;
            }
            delivered += 1;
            if let Ok(event) = outcome {
                report.metrics.record_latency(event.latency_ns);
            }
            observe!(
                _hooks,
                if outcome.is_ok() {
                    Point::Processed(_index)
                } else {
                    Point::Failed(_index)
                },
                events[delivered - 1].event_id
            );
        }
    }));
    if result.is_err() {
        stop.abort();
        report.panicked = true;
    }
    if report.panicked {
        observe!(_hooks, Point::Panicked(_index), 0);
    }
    // The batch survives catch_unwind. Already delivered outcomes belong to the
    // result queue/collector; only unreported owned entries become aborted.
    report.aborted += (events.len() - delivered) as u64;
    #[cfg(test)]
    for event in events.iter().skip(delivered) {
        observe!(_hooks, Point::Aborted, event.event_id);
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Execution, FfiMode};
    use crate::output::{OutputState, OutputSummary, OutputWriter};
    use crate::runtime::RunStatus;
    use std::collections::BTreeSet;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};

    const WAIT: Duration = Duration::from_secs(5);

    #[derive(Clone, Default)]
    struct Audit(Arc<Mutex<Vec<(Point, u64)>>>);
    impl Audit {
        fn record(&self, point: Point, id: u64) {
            if matches!(
                point,
                Point::Produced
                    | Point::Accepted
                    | Point::Dropped
                    | Point::NotAdmitted
                    | Point::Processed(_)
                    | Point::Failed(_)
                    | Point::Aborted
            ) {
                let mut entries = self.0.lock().unwrap();
                assert!(id < 1024 && entries.len() < 4096, "bounded test audit");
                entries.push((point, id));
            }
        }
        fn verify(&self, summary: &RunSummary) {
            let entries = self.0.lock().unwrap();
            let ids = |predicate: fn(Point) -> bool, count: u64| {
                let all: Vec<_> = entries
                    .iter()
                    .filter(|(point, _)| predicate(*point))
                    .map(|(_, id)| *id)
                    .collect();
                let unique: BTreeSet<_> = all.iter().copied().collect();
                assert_eq!(all.len(), unique.len(), "duplicate classification");
                assert_eq!(unique.len() as u64, count);
                unique
            };
            let c = &summary.counters;
            let produced = ids(|p| p == Point::Produced, c.produced);
            let accepted = ids(|p| p == Point::Accepted, c.accepted);
            let dropped = ids(|p| p == Point::Dropped, c.dropped);
            let stopped = ids(|p| p == Point::NotAdmitted, c.not_admitted);
            let processed = ids(|p| matches!(p, Point::Processed(_)), c.processed);
            let failed = ids(|p| matches!(p, Point::Failed(_)), c.failed);
            let aborted = ids(|p| p == Point::Aborted, c.aborted);
            for group in [
                [&accepted, &dropped, &stopped],
                [&processed, &failed, &aborted],
            ] {
                assert!(group[0].is_disjoint(group[1]));
                assert!(group[0].is_disjoint(group[2]));
                assert!(group[1].is_disjoint(group[2]));
            }
            assert_eq!(
                produced,
                accepted
                    .union(&dropped)
                    .copied()
                    .collect::<BTreeSet<_>>()
                    .union(&stopped)
                    .copied()
                    .collect()
            );
            assert_eq!(
                accepted,
                processed
                    .union(&failed)
                    .copied()
                    .collect::<BTreeSet<_>>()
                    .union(&aborted)
                    .copied()
                    .collect()
            );
            assert!(c.reconciles());
            assert_eq!(summary.metrics.latency_observations, c.processed);
        }
    }

    #[derive(Default)]
    struct Sink {
        rows: Vec<ProcessedEvent>,
        first: Option<Box<dyn FnOnce() -> Result<(), String>>>,
        fail: bool,
    }
    impl ResultSink for Sink {
        fn push(&mut self, row: &ProcessedEvent) -> Result<(), String> {
            if let Some(first) = self.first.take() {
                first()?;
            }
            if self.fail {
                return Err("injected writer failure".into());
            }
            assert!(self.rows.len() < 1024);
            self.rows.push(row.clone());
            Ok(())
        }
        fn finish(&mut self, _: bool) -> Result<u64, String> {
            Ok(self.rows.len() as u64)
        }
        fn summary(&self) -> OutputSummary {
            OutputSummary::default()
        }
    }

    fn prepared(events: u64, batch: usize, queue: usize, policy: Policy) -> (Corpus, RunSummary) {
        let mut config = RunConfig {
            corpus: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/small"),
            out: PathBuf::new(),
            events,
            execution: Execution::Concurrent,
            workers: 1,
            queue_capacity: queue,
            batch_size: batch,
            ffi: FfiMode::Batch,
            policy,
            baseline_samples: None,
        };
        let corpus = Corpus::load(&mut config).unwrap();
        (corpus, RunSummary::new(config))
    }

    #[test]
    fn drop_new_full_queue_attempts_once_and_partitions_ids() {
        for ffi in [FfiMode::Event, FfiMode::Batch] {
            let (corpus, mut summary) = prepared(64, 16, 1, Policy::DropNew);
            summary.config.ffi = ffi;
            let (release, wait) = bounded(1);
            let blocked = AtomicBool::new(false);
            let audit = Audit::default();
            let trace = audit.clone();
            let hooks = Hooks(Arc::new(move |point, id| {
                trace.record(point, id);
                if point == Point::BeforeReceive(0) && !blocked.swap(true, Ordering::SeqCst) {
                    wait.recv_timeout(WAIT)
                        .expect("producer completes while worker is held");
                }
                if point == Point::ProducerDone {
                    release.send(()).unwrap();
                }
            }));
            let mut sink = Sink::default();
            execute_with_hooks(
                &corpus,
                &mut sink,
                &mut summary,
                &Control::default(),
                &hooks,
            );
            assert_eq!(summary.exit_code(), 2);
            assert_eq!(summary.status, RunStatus::CompletedWithDrops);
            assert_eq!(summary.counters.accepted, 1);
            assert_eq!(summary.counters.dropped, 63);
            assert_eq!(sink.rows[0].event_id, 0);
            audit.verify(&summary);
        }
    }

    #[test]
    fn slow_collector_pending_admission_interrupt_and_writer_error_drain() {
        for writer_error in [false, true] {
            let (corpus, mut summary) = prepared(1000, 1, 1, Policy::Block);
            let control = Arc::new(Control::default());
            let (ready, ready_rx) = bounded(1);
            let (release, release_rx) = bounded(1);
            let audit = Audit::default();
            let trace = audit.clone();
            let hooks = Hooks(Arc::new(move |point, id| {
                trace.record(point, id);
                // One held by collector + 256 results + one worker + one input,
                // followed by the producer's pending admission.
                if point == Point::Produced && id == 259 {
                    ready.send(()).unwrap();
                    release_rx
                        .recv_timeout(WAIT)
                        .expect("collector requests stop");
                }
            }));
            let stop = control.clone();
            let mut sink = Sink {
                fail: writer_error,
                first: Some(Box::new(move || {
                    ready_rx
                        .recv_timeout(WAIT)
                        .map_err(|e| format!("saturation gate: {e}"))?;
                    stop.interrupt();
                    if writer_error {
                        stop.graceful();
                    }
                    release.send(()).map_err(|e| e.to_string())?;
                    Ok(())
                })),
                ..Sink::default()
            };
            execute_with_hooks(&corpus, &mut sink, &mut summary, &control, &hooks);
            assert_eq!(summary.exit_code(), if writer_error { 1 } else { 130 });
            assert_eq!(summary.counters.produced, 260);
            assert_eq!(summary.counters.accepted, 259);
            assert_eq!(summary.counters.not_admitted, 1);
            assert_eq!(summary.counters.processed, 259);
            assert_eq!(summary.counters.aborted, 0);
            assert_eq!(summary.counters.written, if writer_error { 0 } else { 259 });
            assert_eq!(
                summary.counters.unwritten,
                if writer_error { 259 } else { 0 }
            );
            if writer_error {
                assert!(summary.reason.as_ref().unwrap().contains("writer"));
            }
            audit.verify(&summary);
        }
    }

    #[test]
    fn fatal_stop_reaches_other_workers_without_losing_ids() {
        let (corpus, mut summary) = prepared(32, 4, 8, Policy::Block);
        summary.config.workers = 2;
        let (release, release_rx) = bounded(2);
        let (fatal_ready, fatal_rx) = bounded(1);
        let started = [AtomicBool::new(false), AtomicBool::new(false)];
        let audit = Audit::default();
        let trace = audit.clone();
        let hooks = Hooks(Arc::new(move |point, id| {
            trace.record(point, id);
            if point == Point::Accepted && id == 7 {
                release.send(()).unwrap();
                release.send(()).unwrap();
            }
            if let Point::BeforeReceive(index) = point {
                if !started[index].swap(true, Ordering::SeqCst) {
                    release_rx.recv_timeout(WAIT).expect("input prefill");
                }
            }
            if point == Point::BeforeProcess(0) {
                panic!("injected worker-zero panic with another worker active");
            }
            if point == Point::Panicked(0) {
                fatal_ready.send(()).unwrap();
            }
            if point == Point::BeforeProcess(1) {
                fatal_rx
                    .recv_timeout(WAIT)
                    .expect("worker-zero reaches panic");
            }
        }));
        let mut sink = Sink::default();
        execute_with_hooks(
            &corpus,
            &mut sink,
            &mut summary,
            &Control::default(),
            &hooks,
        );
        assert_eq!(summary.exit_code(), 1);
        assert!(summary.counters.aborted > 0);
        assert_eq!(summary.counters.unwritten, 0);
        assert_eq!(summary.diagnostics, vec!["worker 0 panicked"]);
        audit.verify(&summary);
    }

    #[test]
    fn worker_panic_preserves_delivered_results_and_aborts_owned_and_queued_ids() {
        use parquet::file::reader::{FileReader, SerializedFileReader};
        for after_delivery in [false, true] {
            for ffi in [FfiMode::Event, FfiMode::Batch] {
                let (corpus, mut summary) = prepared(12, 4, 8, Policy::Block);
                summary.config.ffi = ffi;
                let directory = tempfile::tempdir().unwrap();
                summary.config.out = directory.path().to_owned();
                let (filled, filled_rx) = bounded(1);
                let (done, done_rx) = bounded(1);
                let first_receive = AtomicBool::new(true);
                let audit = Audit::default();
                let trace = audit.clone();
                let hooks = Hooks(Arc::new(move |point, id| {
                    trace.record(point, id);
                    if point == Point::Accepted && id == 7 {
                        filled.send(()).unwrap();
                    }
                    if point == Point::ProducerDone {
                        done.send(()).unwrap();
                    }
                    if point == Point::BeforeReceive(0)
                        && first_receive.swap(false, Ordering::SeqCst)
                    {
                        filled_rx.recv_timeout(WAIT).expect("prefilled input queue");
                    }
                    if point == Point::BeforeProcess(0) {
                        done_rx.recv_timeout(WAIT).expect("all 12 events admitted");
                        assert!(after_delivery, "injected panic before native processing");
                    }
                    assert!(
                        !(after_delivery && point == Point::Processed(0) && id == 0),
                        "injected panic after one delivered result"
                    );
                }));
                let mut writer = OutputWriter::open(directory.path()).unwrap();
                execute_with_hooks(
                    &corpus,
                    &mut writer,
                    &mut summary,
                    &Control::default(),
                    &hooks,
                );
                assert_eq!(summary.exit_code(), 1);
                assert_eq!(summary.counters.accepted, 12);
                let processed = u64::from(after_delivery);
                assert_eq!(summary.counters.processed, processed);
                assert_eq!(summary.counters.aborted, 12 - processed);
                assert_eq!(summary.counters.written, processed);
                assert_eq!(summary.output.state, OutputState::Partial);
                let reader = SerializedFileReader::new(
                    std::fs::File::open(summary.output.file.as_ref().unwrap()).unwrap(),
                )
                .unwrap();
                assert_eq!(
                    reader.metadata().file_metadata().num_rows(),
                    processed as i64
                );
                audit.verify(&summary);
            }
        }
    }
}
