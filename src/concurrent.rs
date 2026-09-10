//! Bounded Rust runtime. The main thread always drains the result channel.
use crate::config::RunConfig;
use crate::event::{Event, ProcessedEvent};
use crate::metrics::Metrics;
use crate::output::ResultSink;
use crate::processing::ProcessingBuffer;
use crate::runtime::{collect_result, finish_execution, RunSummary};
use crate::source::Corpus;
use crossbeam_channel::{bounded, Receiver, RecvTimeoutError, SendTimeoutError, Sender};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

const RESULT_CAPACITY: usize = 256;
const STOP_POLL: Duration = Duration::from_millis(10);

#[derive(Default)]
struct Stop {
    admission: AtomicBool,
    fatal: AtomicBool,
}

impl Stop {
    fn requested(&self) -> bool {
        self.admission.load(Ordering::Acquire)
    }
    fn fatal(&self) -> bool {
        self.fatal.load(Ordering::Acquire)
    }
    fn graceful(&self) {
        self.admission.store(true, Ordering::Release);
    }
    fn abort(&self) {
        self.fatal.store(true, Ordering::Release);
        self.graceful();
    }
}

#[derive(Default)]
struct ProducerReport {
    produced: u64,
    accepted: u64,
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

pub(crate) fn execute(corpus: &Corpus, writer: &mut impl ResultSink, summary: &mut RunSummary) {
    let start = Instant::now();
    let config = summary.config.clone();
    let stop = Stop::default();
    let (input_tx, supervisor_rx) = bounded(config.queue_capacity);
    let (result_tx, result_rx) = bounded(RESULT_CAPACITY);
    let mut metrics = Metrics::default();
    let mut writer_failed = false;

    thread::scope(|scope| {
        let mut workers = Vec::with_capacity(config.workers);
        // Spawn workers first. If any spawn fails, no producer is started.
        for index in 0..config.workers {
            let input = supervisor_rx.clone();
            let results = result_tx.clone();
            let config = &config;
            let stop = &stop;
            match thread::Builder::new()
                .name(format!("helix-worker-{index}"))
                .spawn_scoped(scope, move || {
                    worker(
                        input,
                        results,
                        config,
                        corpus.manifest.samples as usize,
                        stop,
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
            let stop = &stop;
            match thread::Builder::new()
                .name("helix-producer".into())
                .spawn_scoped(scope, move || produce(corpus, input_tx, config, stop))
            {
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

        // A writer error never disconnects the collector. This releases blocked
        // worker sends while already accepted events continue to a terminal state.
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
        // Every producer/worker has terminated. The retained receiver owns the
        // remaining accepted queue entries on fatal stop; classify each explicitly.
        for _event in supervisor_rx.try_iter() {
            summary.counters.aborted += 1;
        }
    });
    if stop.fatal() && summary.reason.is_none() {
        summary.fail("fatal runtime stop".into());
    }
    finish_execution(writer, summary, &metrics, writer_failed, start);
}

fn produce(
    corpus: &Corpus,
    input: Sender<Event>,
    config: &RunConfig,
    stop: &Stop,
) -> ProducerReport {
    let mut report = ProducerReport::default();
    let mut pending = false;
    let result = catch_unwind(AssertUnwindSafe(|| {
        for id in 0..config.events {
            if stop.requested() {
                break;
            }
            let mut event = Event::new(id, corpus.row(id));
            report.produced += 1;
            pending = true;
            loop {
                if stop.requested() {
                    report.not_admitted += 1;
                    pending = false;
                    return;
                }
                match input.send_timeout(event, STOP_POLL) {
                    Ok(()) => {
                        report.accepted += 1;
                        pending = false;
                        break;
                    }
                    Err(SendTimeoutError::Timeout(returned)) => event = returned,
                    Err(SendTimeoutError::Disconnected(_)) => {
                        report.not_admitted += 1;
                        pending = false;
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
        if pending {
            report.not_admitted += 1;
        }
    }
    report
}

fn worker(
    input: Receiver<Event>,
    results: Sender<Outcome>,
    config: &RunConfig,
    width: usize,
    stop: &Stop,
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
        }
    }));
    if result.is_err() {
        stop.abort();
        report.panicked = true;
    }
    // The vector is retained outside catch_unwind. Delivered outcomes are
    // already owned by the result queue/collector; only the remainder is aborted.
    report.aborted += (events.len() - delivered) as u64;
    report
}
