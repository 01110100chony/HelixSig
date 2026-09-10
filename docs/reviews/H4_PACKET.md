# H4 implementation packet

Prerequisite: H3 TECH_PASS, integrated at 4000d56. User authorizes autonomous
H4 followed by H5, without scope expansion. One production writer, isolated
external test authoring, frozen candidate review and leader adjudication.

## Scope and contracts

- Enable concurrent drop-new with one try_send per materialized event. Only a
  full input queue drops. A stop/disconnection classifies a pending event as
  not_admitted; accepted work never silently disappears.
- Add completed_with_drops (exit 2) and interrupted (exit 130). Completed-with-
  drops uses valid finalized output; failed/interrupted valid output is partial.
  A processing/writer/fatal failure takes precedence over an interrupt; losses
  never relabel a run as lossless. Preserve the three accounting equations.
- Register a SIGINT flag with signal-hook, without a helper runtime thread.
  Register before corpus loading/output startup; unregister after run completion.
  Both sequential and concurrent modes stop admission and drain accepted work.
- Preserve H3 writer-failure behavior: stop admission, keep collecting, join,
  leave untrusted .incomplete output and written=0. Summary publication failure
  retains already finalized valid rows and reports the actual state on stdout.
- Contain worker panic, request fatal stop, classify unreported owned events and
  queued accepted events as aborted, and drain already emitted outcomes. No
  recovery, retry, stuck-native-call guarantee, async runtime or C++ threads.
- Test-only hooks may synchronize producer/worker stages and inject panic. They
  must compile away in production, expose no CLI fault flags, and never change
  FFI ownership, numerical behavior or runtime scheduling in normal builds.
- Test-only ID audits remain bounded by small fixtures. Production retains only
  counters, bounded histograms, worker reports and at most 16 diagnostic examples.

Allowed changes: Rust control/concurrent/runtime/config modules and their tests,
signal-hook dependency, Python failure acceptance, verify/CI and relevant docs.
Native kernel, FFI, corpus layout, numerical tolerance and Parquet columns freeze.

## Acceptance

- Deterministic full-input-queue drop-new with accepted/dropped disjoint ID sets,
  exact accounting and exit-2 semantics; normal block remains lossless.
- Deterministic slow-collector/backpressure, graceful stop while a producer has a
  pending admission, and writer failure while accepted work is outstanding.
- Worker panic before processing and after a delivered result; accepted IDs are
  partitioned into processed/failed/aborted exactly once, with no deadlock.
- Real SIGINT to both CLI execution modes after output begins, exit 130, valid
  partial Parquet and reconciled IDs/counters. Registered handler is removed.
- CLI drop-new status/exit match actual losses (do not require a scheduling-
  dependent loss count), concurrent real write failure, NaN result isolation and
  all H0-H3 regressions. Every subprocess has a timeout; test synchronization
  waits also time out instead of hanging the gate.
- `HELIX_PYTHON=.venv/bin/python scripts/verify.sh H4`, mandatory standalone
  ASan/UBSan, separate-context external AI review and adjudication.

H5 experiments remain blocked on H4 TECH_PASS. No benchmark result is inferred
from these correctness/failure tests and no performance target is introduced.
