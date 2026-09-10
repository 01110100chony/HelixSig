# Helix

A Linux-first laboratory for bounded concurrent signal processing in Rust with an
independent C++20 numerical library and a Python/NumPy oracle.

**Status: G0, H0 and H1 passed; H2 sequential pipeline is under gate validation.
H3-H6 are not implemented. No performance claim is approved.**

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

H2 creates no runtime threads or channels. Worker count, queue capacity and policy
are validated and recorded for the later runtime; sequential execution has no
queue drops. `--execution concurrent` is rejected until H3. Worker count must be
between one and the available processor count, even in sequential mode.

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
buffers, and 32 MiB for Parquet buffers/metadata, histograms and diagnostics. The
256 MiB limit is an estimate of data buffers, not a process RSS ceiling. New Rust
dependencies serve CLI parsing (clap), JSON (serde), integrity checking (sha2),
Parquet serialization (parquet without Arrow/compression features), and bounded
latency measurement (hdrhistogram); tempfile is test-only.
