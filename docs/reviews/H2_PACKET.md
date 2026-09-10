# H2 implementation packet

Prerequisite: H1 TECH_PASS. Leader implements; no worker delegation.

## Interfaces and scope
- Add config/source/event/processing/metrics/output/runtime modules and main CLI.
- `RunConfig` is the effective serializable configuration. `run(config)` validates
  inputs before production and returns a machine-readable `RunSummary`.
- CLI: `helix run --corpus DIR --out NEW_DIR`, with --events, --execution,
  --workers, --queue-capacity, --batch-size, --ffi and --policy, --baseline-samples.
- H2 runs sequentially without threads/channels; concurrent mode is rejected until H3.
- One reusable ProcessingBuffer packs all sample data identically for both FFI modes.
- Each Event owns samples and admission-attempt Instant. ProcessedEvent owns only
  identifiers, numeric features and measured latency; no waveform in the output.
- Source requires manifest schema 1, file signals.f64le, float64-le, checked shape,
  exact length and SHA-256. Only allow the fixed filename, not manifest path traversal.
- Bound v0.1 runs to 1..1000000 events to also bound Parquet row-group metadata.
  Corpus <=64 MiB, manifest <=64 KiB. Memory preflight includes corpus loading
  transient storage, sample buffers and conservative fixed metrics/writer allowance.
- Fixed schema/output settings follow ARCHITECTURE.md. Existing run directories
  are rejected without modifying their files. Only successful footer close and
  rename allow written>0. Invalid events permit valid partial output, status failed.
- Flatten counters in JSON; include effective configuration, corpus SHA, status,
  reason, duration, throughputs, latency histogram coverage and output state.
- Empty latency observations use null; histogram overflow is counted explicitly.
- No retries, recovery, tuning, extra input/output formats or CLI fault flags.

## Acceptance
- 4101 events exercise a full writer group and final tail; Python readback validates
  all rows/features/IDs against NumPy and summary accounting.
- Both FFI modes produce equal numerical results by ID (latency need not match).
- Invalid configuration/hash/shape and existing output dir do not start production.
- NaN event produces failure accounting and valid partial file, not success.
- Output startup failure reports failed; buffered/unfinalized rows never count written.
- Throughput/latency definitions follow frozen architecture, not writer timestamps.
- fmt, clippy, Rust tests, native/oracle regressions and independent gate review.
