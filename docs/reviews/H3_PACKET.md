# H3 implementation packet

Prerequisite: H2 TECH_PASS, integrated locally at a252293. User authorizes H3
followed by H4, with a separate gate before advancing. One production writer;
read-only external workers use explicit file/tool scope. Review freezes code.

## Scope and interfaces

- Enable concurrent execution with block policy. Concurrent drop-new is rejected
  until H4; sequential behavior remains available and lossless for either policy.
- One producer, W Rust workers and the main collector use crossbeam-channel:
  input capacity Q and result capacity 256 individual outcomes. No async runtime,
  C++ threads, callbacks, unsafe Send/Sync, sample-copy or numerical changes.
- Borrow the validated immutable corpus through scoped Rust threads. Each event
  owns its samples; each worker owns its reusable ProcessingBuffer and histogram.
- Workers wait for one event, then try_recv up to B, never waiting to fill.
  Result sends apply backpressure and never drop valid/failed outcomes.
- Admission accounting belongs to the producer; processing/output accounting
  belongs to the collector. Join reports aggregate transition counters, never
  infer or repair them from other counters. A retained input receiver permits
  explicit final drain after fatal internal errors.
- Normal closure: producer drops sender; workers drain and close results;
  collector drains results, joins every thread and finalizes output. Writer
  error stops admission, but collector keeps receiving accepted work.
- Add minimal internal fatal-stop/panic containment so thread creation failure
  or unexpected worker termination cannot strand bounded sends. H4 separately
  exercises injected panic, overload, interrupt and failure semantics.
- Block admission checks stop requests with bounded send_timeout waits. Pending
  materialized events become not_admitted when stopped. Queued/unclassified
  accepted events on fatal stop become aborted, with no counter reconciliation fixups.
- Summary keeps merged HDR percentiles and bounded per-worker metrics. Merge
  histograms, never average percentiles. Count successful-event latencies only.
- Memory preflight adds a conservative 1 MiB per worker plus one merged histogram
  to H2's corpus/sample/writer allowance. This accounts for newly live buffers;
  the 256 MiB estimated-data bound remains unchanged.

Allowed edits: Rust runtime/config/metrics modules and internal outcome helpers,
Cargo.toml/lock (crossbeam-channel only), scripts/tests/CI and relevant docs.
No corpus, native/FFI, numerical tolerance, Parquet schema or ownership changes.

## Acceptance

- W=1/2/4 when available, Q=1 and normal capacity, B=1/16/64, event/batch FFI;
  exact numerical and ID equivalence with sequential runs regardless of order.
- Minimal runs, fewer events than workers, batch tails and 4101-row output;
  all counters and actual batch sizes reconcile, with no losses in block mode.
- Histograms merge with bounded storage, empty workers remain null, latency
  coverage equals processed, and estimates reject excessive worker buffers.
- Graceful channel closure/drain/join is bounded by subprocess test timeouts.
- H2 malformed-corpus/output/numerical/write-error regressions remain passing.
- Full gate: `HELIX_PYTHON=.venv/bin/python scripts/verify.sh H3` (fmt/clippy,
  Rust tests, H0/H1/H2 regressions, mandatory standalone ASan/UBSan, Python
  concurrent readback). Separate-context AI review and adjudication required.

H4 adds drop-new/exit 2, Ctrl+C/exit 130 and deterministic saturation, slow
consumer, writer-failure and worker-panic acceptance. Do not claim H4 in this gate.
