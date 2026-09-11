# H6 hygiene and bounded sanity audit

Primary-orchestrator audit of source candidate `efed5d040f04c886d436e2b7fcd424930f597d49`
on 2026-09-11. This document reports inspection and bounded experiments; full
release acceptance is recorded in the H6 review and final exact-commit packet.

## Repository and provenance

- `git diff 6ba29f9 -- src cpp examples Cargo.toml Cargo.lock build.rs
  rust-toolchain.toml requirements.txt fixtures docs/evidence/H5
  scripts/experiment.py scripts/experiment_analysis.py
  scripts/experiment_support.py scripts/validate_experiments.py` is empty.
  Production behavior, workload, pins, fixtures and all committed H5 evidence
  are byte-for-byte preserved.
- All tracked shell entry points have mode 100755. Small binary corpus and
  reference/manifest are tracked; no generated runtime data or build binary is
  needed by a clean checkout. `.gitattributes` enforces LF and treats `.f64le`
  as binary. New read-only Antigravity profiles are tracked.
- Local Markdown file links in README/docs resolve. No tracked file exceeds
  1 MiB. The largest preserved file is H5/raw-index.json (545305 bytes), an
  intentional evidence index. No history or preserved evidence was removed.
- target/, build/, artifacts/, .venv/, Python caches and .agent-runs/ remain
  ignored. Normal verification generates outputs in those ignored directories.
- `Get-FileHash -Algorithm SHA256` on the retained original editing-checkout
  `artifacts/h5-1d8724c-raw.tar.gz` returned
  `6fb20bc27a2a6db76fb30afae75691e96dc7aa638b0a34192fd7e812ab55990e`, exactly the
  committed H5 index value. The archive is 106993609 bytes. This confirms the
  archive container is unchanged; H6 did not remeasure or regenerate H5 data.
- Cargo and native project remain version 0.1.0, Cargo publish=false. H6 status
  is a technical gate, not a released version. No branch deletion, push, merge,
  tag, public release or marketing rewrite was performed.
- Original editing checkout stays on codex/h5-reproducible-experiments with its
  pre-existing README change and untracked review_closeoutt.txt preserved.

## Reproduced release-tool defects

- Running the new offline regression against copies of the old group wrappers
  fails with `FAIL: parallel masked cpp-reviewer's nonzero exit`. Against the
  corrected wrappers it passes two successful groups and seven individual
  reviewer-failure positions. No external CLI is called by this test.
- With a temporary PATH entry containing a simulated CMake that exits 29,
  `scripts/verify.sh H6` returns 29 and emits no H6 automated PASS marker.
- Changing the compiler in a reused CMake directory discarded other options
  from that invocation: CMAKE_BUILD_TYPE became empty and HELIX_SANITIZERS OFF.
  The first local H6 attempt at a2414a3 is invalidated and was stopped during
  smoke build. Its partial log is retained only as defect evidence.
  efed5d0 uses new native directories for every gate. Generated flags were
  inspected: native Release has -O3; sanitized Debug has
  -fsanitize=address,undefined and -fno-omit-frame-pointer. No tolerance or
  numerical implementation change was required.

## Targeted correctness inspection

The orchestrator traced config/source, processing/bridge/native adapter,
concurrent/control and runtime/output/main against ARCHITECTURE.md and existing
H4 tests. No new supported-contract BLOCKER/IMPORTANT runtime defect was found.
This is a targeted sanity audit, not a proof of every possible execution.

- Config validates finite run bounds and checked memory arithmetic; source
  checks dimensions, byte limits, exact lengths and hashes before event work.
  Existing u64 counters are bounded by at most one million event attempts.
- Rust owns input/output buffers. FFI borrows synchronously, maps native fields
  explicitly and exposes no Rust callbacks. C++ keeps no pointers, threads or
  mutable globals; structural batch errors preserve output and CXX translates
  adapter exceptions. Worker panic handling stays on the Rust side.
- Producer/workers own their reports. Main drains results before joining; writer
  error stops admission while draining. Fatal stop preserves delivered results
  and classifies unreported owned/queued accepted events as aborted. Existing
  deterministic tests inspect disjoint event-ID classifications.
- Output opens a new run directory/file, checks writes/footer close/rename and
  only then counts rows as written. Final JSON failures are reflected in exit
  status/stdout; `.incomplete` files do not become success evidence.
- Experiment subprocesses check exit status, have bounded process-group timeouts
  and preserve failed records. Exit 2 is explicitly validated as completed with
  losses. H6 does not alter this H5 protocol or reinterpret smoke as measurement.

Supported exclusions remain hostile same-user replacement, native fatal faults,
SIGKILL/OOM kill, stuck kernel I/O and power-loss recovery. The diagnostic
check-tools.sh is best-effort discovery; successful reviewer completion and
adjudication, not its status, are required for review acceptance.
