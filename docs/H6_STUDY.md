# H6 technical study

This describes the implemented Helix system at H5 closeout and H6 release
preparation. H6 changes verification, CI and documentation, not the numerical
kernel, runtime or experimental design. [ARCHITECTURE.md](ARCHITECTURE.md) is the
frozen contract; [the release review](reviews/H6.md) records acceptance evidence.
This is finite synthetic replay, not acquisition hardware or hard-real-time DAQ.

## Data and control path

```text
Python generator -> versioned manifest + little-endian float64 corpus
  -> Rust validation/loading -> producer -> bounded input queue
  -> W Rust workers -> reusable flat buffer -> synchronous CXX adapter
  -> independent C++20 numerical kernel -> Rust outcomes
  -> bounded result queue -> main collector/writer -> Parquet + JSON
  -> independent Python numerical/accounting readback and experiment analysis
```

Python creates baseline, Gaussian noise and a Gaussian pulse (zero during the
first K baseline samples), plus an independent NumPy reference. Rust checks
manifest size/schema, dimensions, file length and SHA-256 before allocating
waveform storage, then rechecks the payload while loading it. See
[source.rs](../src/source.rs) and [generate_data.py](../scripts/generate_data.py).
Event id starts at zero; row = id % corpus_rows, channel = id % 4 and sequence =
id / 4. The producer materializes owned events and timestamps admission attempts.

The kernel computes baseline mean, first maximum of the corrected post-baseline
samples and signed integral. It rejects invalid dimensions/configuration,
nonfinite samples and nonfinite results. Pairwise reduction and float64
tolerance `abs(a-r) <= 1e-10 + 1e-10*abs(r)` are unchanged. Indices/statuses match
exactly. See [kernel.cpp](../cpp/src/kernel.cpp), [oracle.py](../scripts/oracle.py)
and the H0 review for the demonstrated cancellation correction.

## Ownership and FFI

Rust owns corpus storage, events, flat sample buffers, bridge result buffers,
threads, clocks, channels, histograms and files. There are two sample copies:
corpus row into the event, then event samples into a reusable worker buffer.
Both FFI modes perform identical preparation. Event mode calls once per event;
batch mode calls once per collected batch. The entire pipeline is not zero-copy.

CXX borrows immutable samples and exclusive output only for a synchronous call.
The adapter uses a fixed array of up to 64 native results and explicitly maps
fields; it neither reinterprets Rust layout nor copies sample payloads. C++ owns
only bounded temporary computation state, retains no pointers, allocates nothing
in kernel processing, and owns no runtime threads, files or mutable globals.
Structural batch errors leave Rust output untouched and translate to Result
errors; ordinary numerical failure is a per-event status. There are no Rust
callbacks across the boundary, so worker-panic unwinding is handled on the Rust
side. Native fatal faults are not recoverable C++ exceptions. See
[bridge.rs](../src/bridge.rs), [adapter.cpp](../cpp/ffi/adapter.cpp) and
[processing.rs](../src/processing.rs).

## Concurrency, backpressure and shutdown

Sequential mode uses the same source, processing and output components with no
channels or runtime threads. Concurrent mode has one producer, W scoped Rust
worker threads and the main thread as collector. Crossbeam bounds the input by
Q and the result queue by 256 individual outcomes. A worker waits for one event,
then tries to receive up to B immediately; B is a maximum, not a promise to fill.
Actual batch sizes and parallel output order depend on scheduling.

Block admission retries with a 10 ms timeout so it can observe stop requests.
Drop-new attempts once and rejects a full input queue. Only this input queue
drops; the result queue blocks workers when the collector/writer is slower.
The collector keeps receiving after writer failure, discarding further disk
writes while accepted work drains. It receives until sender closure before
joining producer/workers, avoiding a join while results are blocked on delivery.
Normal end closes input, drains, joins and finalizes output. Fatal worker stop
classifies unreported owned events and remaining supervisor-held queue entries
as aborted; already delivered results stay owned by the collector. See
[concurrent.rs](../src/concurrent.rs) and [control.rs](../src/control.rs).

Data buffers are bounded: N <= 4096, B <= 64, Q <= 4096, W <= available CPUs,
corpus <= 64 MiB, output groups <= 4096 rows, diagnostics <= 16 examples and
bounded histograms. Run configuration permits at most 1,000,000 attempts, keeping
event counters well below u64 overflow. Checked memory preflight includes corpus,
sample buffers, writer and metrics and rejects estimates over 256 MiB. This
estimate is not an RSS ceiling. See [config.rs](../src/config.rs).

## Failure and output model

| Condition | Outcome and accounting | Output / exit |
|---|---|---|
| Normal block replay | All produced work accepted and processed | Finalized Parquet / 0 |
| Drop-new overload | Rejected attempts retained as dropped | Finalized Parquet with explicit losses / 2 |
| Invalid CLI/config/corpus | Rejected before event production | Diagnostic summary where applicable / 64 |
| Invalid numerical event | Event failed; valid events remain collectable | Valid partial Parquet if writer succeeds / 1 |
| Writer failure | Stop admission, drain accepted work; successful results remain unwritten | Untrusted incomplete Parquet, written=0 / 1 |
| Contained worker panic | Fatal stop; delivered results retained, unreported accepted events aborted | Partial Parquet if writer succeeds / 1 |
| SIGINT | Stop admission and drain; failures take precedence | Partial Parquet if writer succeeds / 130 absent failure |
| Summary persistence failure | Failure reported to stdout; valid rows remain accounted | Final summary absent; output marked partial if rename succeeds / 1 |

The accounting identities are updated at transitions, never repaired afterward:

```text
produced = accepted + dropped + not_admitted
accepted = processed + failed + aborted
processed = written + unwritten
```

Only successful numerical results become rows. They count as written only after
footer close and output rename succeed. A historical running.json is not proof
of completion; summary.json supersedes it, and `.incomplete` files are untrusted.
Rust checks writes, close and rename results; stdout emission failure also exits
nonzero. This is not crash recovery or power-loss durability. SIGKILL, OOM kill,
native fatal faults and stuck kernel I/O are outside graceful-shutdown guarantees.
See [runtime.rs](../src/runtime.rs), [output.rs](../src/output.rs),
[main.rs](../src/main.rs) and [failure validation](../scripts/validate_failures.py).

## Measurement, observation and hypothesis

**Measurement provenance.** H5 measured candidate
`1d8724c9521e7d88a43e6f751c4681e1fbbf75c6`; closeout `6ba29f9`.
The preserved campaign contains 744 validated runs (124 warmups, 620 measured),
96 micro configurations and 28 pipeline configurations on one Ryzen 5 3400G,
Ubuntu 24.04/WSL2 environment with eight logical CPUs, about 7.7 GiB RAM and
2 GiB swap. Actual limits supersede the planned four-CPU setting. No H6
performance campaign or new measurement is claimed.

**Observed scaling.** At N=256/Q=256/B=16, batch FFI and block policy, median
written throughput for W=1,2,4 was 368418, 507085 and 509060 events/s. W=2 and
W=4 were close relative to between-run spread: a plateau in these configurations,
not a general scaling law or a result for every FFI/policy combination.

**Observed overload.** At the W=2 center, batch/drop-new median loss fraction was
0.89246 and median written throughput 402177 events/s. Accepted-result speed
must not hide the approximately 89.246% median loss of attempted events.

**Observed FFI costs.** N=256/B=16 prepared Rust event/batch medians were
423.4/414.1 ns per event with overlapping interquartile intervals. Native
event/batch medians were 437.6/405.4 ns. Common result-capture/checksum
instrumentation is inside timing; allocation/loading/output are outside it.
These executables do not isolate the exact cost of crossing FFI.

**Hypotheses, not demonstrated causes.** Writer/SSD cost, scheduling or copying
could contribute to the observed plateau. The single-axis design and uncontrolled
host load do not identify a causal bottleneck or interactions. Additional
profiling/controlled experiments would be needed; they are outside H6.

The [protocol and full analysis](EXPERIMENTS.md) retain dispersion and per-run
percentiles, not averages of percentiles. Pipeline duration includes joins and
writer finalization; latency ends when the numerical result is available and
excludes writing. Peak observed child RSS (35120 KiB) and absence of histogram
overflow are observations, not future resource/latency guarantees. H5 raw data is
retained locally with a committed SHA-256 index, not distributed by a Git clone.

## Release and human study

[DEVELOPMENT.md](DEVELOPMENT.md#reproduce-a-release-candidate) defines the exact
setup and authoritative H6 command used by CI. [H6.md](reviews/H6.md) connects
each requirement to executable and AI review evidence and records limitations.
The [Portuguese study guide](STUDY_GUIDE.pt-BR.md) asks the user to reproduce,
trace ownership/accounting, defend a qualified observation and explain a
limitation. Technical acceptance cannot certify that learning or authorize a
merge, public release or tag on the user's behalf.
