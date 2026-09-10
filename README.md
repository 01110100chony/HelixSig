# Helix

A Linux-first laboratory for bounded concurrent signal processing in Rust with an
independent C++20 numerical library and a Python/NumPy oracle.

**Status: G0-H2 passed; H3 bounded runtime is under gate validation.
H4-H6 are not implemented. No performance claim is approved.**

The experiment studies worker count, queue capacity, processing batch size and
per-event versus batched FFI. It is finite synthetic replay, not a real-time DAQ
system or physical simulation. There is no required speedup.

See [architecture](docs/ARCHITECTURE.md), [development governance](docs/DEVELOPMENT.md),
[milestones](docs/ROADMAP.md), [experiments](docs/EXPERIMENTS.md), and the
[Portuguese study guide](docs/STUDY_GUIDE.pt-BR.md).

Primary environment: Ubuntu 24.04 in WSL2. Source, builds and measured data belong
on the Linux filesystem. Windows is an editing interface, not a supported target.

## Sequential replay (H2)

In Ubuntu/WSL2 with the G0 toolchain installed:

```bash
python3 -m venv .venv
.venv/bin/python -m pip install -r requirements.txt
.venv/bin/python scripts/generate_data.py --out artifacts/example-corpus
cargo run --locked -- run --corpus artifacts/example-corpus --out artifacts/example-run \
  --events 4101 --execution sequential --workers 1 --batch-size 16 --ffi batch
HELIX_PYTHON=.venv/bin/python scripts/verify.sh H2
```

The corpus and run directories must be new. The CLI defaults to 100000 events,
sequential execution, workers=2, queue capacity=256, batch size=16, batch FFI and
block policy. Baseline width defaults to the corpus manifest (32 for the default
generator). `--ffi event` uses the same sample preparation and calls C++ once per
event. Both modes return identical numerical features. `--baseline-samples K`
overrides processing configuration after validation; it does not regenerate data.

Sequential execution creates no runtime threads or channels and has no queue
drops. H3 adds `--execution concurrent --policy block`: one producer, bounded
input queue, W Rust workers and the main collector/writer. Concurrent drop-new
is rejected until H4. Worker count must be between one and the available
processor count, even in sequential mode.

Workers wait for one event and then try to collect up to the configured batch
size without waiting to fill it. Batch sizes and output order depend on scheduling;
compare parallel results by event ID. The result queue holds 256 individual
outcomes and applies backpressure. Workers drain accepted work after admission
stops, and all threads join before output finalization. The final summary contains
`worker_metrics` in worker-index order; merged percentiles come from merging the
bounded histograms, never averaging per-worker percentiles. In sequential mode
`worker_metrics` is empty. Validate with `scripts/verify.sh H3` and the same Python
environment used by H2.

Each run starts with `running.json`. A successfully closed and renamed output is
`events.parquet`, or `events.partial.parquet` when numerical failures occurred.
Only valid results appear in Parquet. A writer error stops new admission, drains
the already accepted batch, and leaves `events.parquet.incomplete` untrusted.
Buffered rows never count as written. A historical `running.json` remains; the
final `summary.json` supersedes it. `summary.json.incomplete` is not a final summary.
Final JSON is also emitted to stdout, including failures before output creation.
Malformed CLI syntax is reported to stderr. H2 exit codes are 0 (completed),
1 (failed), and 64 (invalid configuration/input). Interrupt handling belongs to H4.

The version-1 summary flattens all counters and records effective configuration,
corpus hash, output state, failure reason, at most 16 diagnostics, estimated data
buffers, duration and processed/written throughput. Latency measures admission
attempt to native result availability, excluding writing. The bounded HDR
histogram uses three significant digits and covers 0..60000000000 ns; overflows
are counted separately, and percentiles describe only recorded observations
(null when empty). Parquet preserves raw successful-event latencies. Actual
processing batch counts are indexed by size in `metrics.actual_batch_sizes`.

Payloads are limited to 64 MiB, manifests to 64 KiB and runs to 1000000 events.
Memory preflight conservatively includes twice the corpus, bounded sample
buffers, 32 MiB for Parquet buffers/metadata and diagnostics, and 1 MiB per worker
plus one merged histogram/report allowance. The
256 MiB limit is an estimate of data buffers, not a process RSS ceiling. New Rust
dependencies serve CLI parsing (clap), JSON (serde), integrity checking (sha2),
Parquet serialization (parquet without Arrow/compression features), and bounded
latency measurement (hdrhistogram). H3 adds crossbeam-channel for bounded queues;
tempfile is test-only.
