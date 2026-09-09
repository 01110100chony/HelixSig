# Architecture and frozen contracts

## Purpose

An academic portfolio experiment complementing M2C with substantive C++20,
Rust/C++ ownership, bounded concurrency, explicit overload and reproducible
performance evidence. No minimum throughput or promised internship outcome.

```
Python corpus + independent oracle
               |
Rust load/validate -> producer -> bounded event queue -> Rust workers
                                                           |
                                                  CXX adapter -> C++ kernel
                                                           |
Rust main collector/writer <- bounded result queue <--------+
            |
      Parquet + JSON -> Python validation and analysis
```

Sequential mode reuses source, processing, adapter and output without channels
or threads. C++ owns no threads, mutable globals, files or runtime timestamps.

## Corpus and events (freeze H0/H2)

The corpus is a row-major little-endian float64 matrix plus a versioned JSON
manifest with dimensions, signal parameters, seed, versions and SHA-256. Validate
checked dimensions, exact file length and hash before allocation/production.
Corpus payload is limited to 64 MiB. A run has a fixed sample width, 2..4096.

Python alone generates baseline + Gaussian noise + Gaussian pulse. The pulse is
zero in the first K baseline samples. Reproduction uses the corpus hash, not a
promise of identical random streams across library versions.

Event fields: u64 event_id, u32 channel_id=id%4, u64 sequence=id/4, owned Vec<f64>
samples and internal monotonic admission-attempt time. Corpus row=id%rows.
IDs start at zero for each finite run. Parallel output order is unspecified.

Defaults: 256 samples, K=32, 256 corpus rows, 100000 events, W=2, Q=256, B=16.
Bounds: N<=4096, B<=64, Q<=4096, workers<=available processors. The result queue
holds 256 individual results. Writer batch and row group size are 4096.

## Numerical contract (freeze H0)

For 1<=K<N: baseline=mean(x[:K]); corrected=x[K:]-baseline;
peak_amplitude=max(corrected); peak_index=K+first_argmax(corrected);
integral=sum(corrected), signed, in amplitude-times-sample units.

Reject invalid sizes/configuration, nonfinite samples and nonfinite numerical
results. No clipping, filters, fitting, fast-math or physical calibration.
NumPy is independent of C++. Float acceptance is abs(a-r)<=1e-10+1e-10*abs(r).
Indices and statuses must match exactly. Tolerance changes require justification
and review, never merely a failing test.

## Native API and FFI (freeze H1)

One public C++ header exposes Config, EventResult, statuses, process_event(span
of const double, config), and process_batch(flat samples, width, config, output
span). Kernel processing allocates nothing and retains no pointers. A structurally
invalid batch leaves output untouched. A valid batch writes one status/result
per event; one invalid signal does not poison other signals.

The independent kernel includes no Rust headers. CXX adapter converts shared
scalars/slices to native types. Rust owns both buffers, borrowing them for the
synchronous call only. No callbacks or unsafe Send/Sync implementations. Bridge
functions return Result for adapter exceptions; ordinary numerical errors use
per-event status. Native fatal faults are not recoverable exceptions.

Producer copies a corpus row into an event. Workers copy events into a reusable
flat buffer. Both FFI modes use that identical preparation; event mode calls
on each slice, batch mode calls once. No additional sample copy in the bridge.
Do not describe the whole pipeline as zero-copy.

## Runtime and shutdown (freeze H3/H4)

One producer, W Rust threads and the main collector/writer use crossbeam bounded
channels. Workers wait for one event, then try receiving up to B without waiting
to fill a batch. Record actual sizes. No async runtime or independent C++ pool.

block waits interruptibly for input capacity; drop-new tries once and records
rejection. Only the input queue drops. The result queue applies backpressure.

Normal end closes the input, drains workers/results, joins all threads and closes
the writer. Writer failure stops admission but accepted work drains; collector
keeps receiving without further disk writes. Ctrl+C also stops admission and
drains. Worker panic requests fatal stop; owned in-flight and queued work is
explicitly classified as aborted. Supervisor retains a receiver for final drain.
No promise for SIGKILL, OOM kill, native fatal faults or stuck kernel I/O.

Counters: produced (materialized admission attempts), accepted, dropped (full
input queue), not_admitted (stopped), processed (valid numerical result), failed
(invalid processing), aborted (accepted without result on fatal stop), written
(rows in finalized valid output), unwritten (processed without valid output).

```
produced = accepted + dropped + not_admitted
accepted = processed + failed + aborted
processed = written + unwritten
```

Normal runs require not_admitted=failed=aborted=unwritten=0; block also dropped=0.
Update counters at transitions; never repair them to force reconciliation.
Small tests additionally check ID sets and disjoint classifications.

Bound all data buffers. Sample estimate is 8*N*(Q+2*W*B+1); add corpus, results,
writer, diagnostics and histograms. Reject estimated data buffers above 256 MiB.
This is not an RSS ceiling. Keep at most 16 diagnostic examples and no unbounded
per-event map, sorting buffer or error list.

## Output and metrics (freeze H2/H4)

Parquet columns: event_id:uint64, channel_id:uint32, sequence:uint64,
baseline:float64, peak_amplitude:float64, peak_index:uint32, integral:float64,
latency_ns:uint64. Only valid results; uncompressed, dictionary disabled.

Create a new run directory. Work in a .incomplete file. Only finalize after
successful footer close; valid failed/interrupted runs use a partial filename.
Writer failure leaves an untrusted incomplete file and written=0. No durability
claim against power loss. Initial running marker and final JSON summary; also
emit final JSON to stdout. Version schemas and include effective config, corpus
hash, outcome/reasons, counters, metrics and output state.

Exit codes: completed=0, completed_with_drops=2, failed=1, interrupted=130,
invalid configuration=64. The experiment runner explicitly accepts code 2 as
completed WITH losses, never relabeling it lossless.

Pipeline duration begins after corpus load/output open, before thread startup,
and ends after joins and writer finalization. Throughputs=processed/duration
and written/duration. Latency is admission attempt to numerical result available:
includes block wait, queue and preparation; excludes corpus load and writing.
Batch completion timestamps follow call return; event mode timestamps each call.
Keep bounded per-worker histograms, null empty percentiles, explicit overflow
counts, actual batch sizes and raw successful-event latency in Parquet.

## Build and exclusions

Single Rust crate. CMake builds the independent static kernel and CTest.
Cargo build.rs invokes cmake; cxx-build compiles only adapter/generated bridge.
Use the same C++20 compiler/profile without duplicate kernel compilation.

Excluded: hardware, network, ROOT, GPU, frontend, HTTP, distributed services,
crash recovery, custom queues/allocators, Windows native support, custom SIMD.
