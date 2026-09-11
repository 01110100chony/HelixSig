# H6 human-review packet

## Candidate and verdict

The H6 release candidate is the immutable commit containing this packet. Its
exact 40-character SHA is recorded in `H6_FINAL_ATTESTATION.md`, distributed next
to the checkout after the post-commit clean-room run. The SHA in that attestation
must equal `git rev-parse HEAD`; otherwise this packet says H6 NOT_READY.

Technical verdict with a matching PASS attestation: **H6 TECH_PASS**.
This does not approve user learning, merge, tagging, publication or release.

## Changes from H5 closeout

- `scripts/verify.sh H6` is the authoritative release gate and defaults when no
  gate is supplied. It composes all existing H5 correctness checks with offline
  release-tool regression, fresh native build directories and explicit CMake
  Release/ASan/UBSan profiles.
- CI installs the documented Ubuntu dependencies, selects Rust from
  rust-toolchain.toml, creates the pinned Python venv and runs the same command.
- Review wrappers propagate every child failure, are executable in Git, and have
  tracked read-only Antigravity profiles.
- README, development setup, H5 provenance, roadmap, Portuguese study guide,
  H6 technical study, review/adjudication and reproducibility evidence agree.
- Production Rust/C++, FFI contracts, Cargo/dependency locks, fixtures, H5
  workloads, experiment scripts, measured values and committed H5 evidence are
  unchanged from `6ba29f9`.

## Architecture summary

Python produces and independently checks a versioned waveform corpus. Rust owns
corpus/event/result memory, validates inputs, drives one producer and bounded
Rust worker/result queues, collects accounting and writes Parquet/JSON. CXX lends
Rust-owned slices synchronously to a reentrant allocation-free C++20 numerical
kernel; C++ retains no pointers and owns no threads or files. Block admission
applies backpressure; drop-new records one-shot rejections. The main collector
drains before joins/finalization. Failures distinguish dropped, not-admitted,
failed, aborted, written and unwritten work through three conservation identities.

## Exact clean-room verification

From a new Ubuntu 24.04 Linux-filesystem clone at the candidate SHA, with no
build/, target/, artifacts/, .venv/ or reused Cargo home, follow
docs/DEVELOPMENT.md and run:

```bash
python3 -m venv .venv
PIP_NO_CACHE_DIR=1 .venv/bin/python -m pip install -r requirements.txt
export CARGO_HOME="$PWD/artifacts/cargo-home"
mkdir -p artifacts
set -euo pipefail
HELIX_PYTHON=.venv/bin/python scripts/verify.sh H6 2>&1 | tee artifacts/verification.log
```

The external attestation records the setup commands, versions, clean state,
final SHA, exit status and log location. Acceptance requires 24 Rust tests,
native Release and standalone ASan/UBSan CTest plus 533 NumPy oracle cases per
profile, 42 H2 cases, 24 H3 runs, seven H4 cases, eight H5 runner tests,
format/Clippy, 200 smoke runs and offline reviewer-exit checks. Smoke is not
performance evidence. The 744-run campaign is not rerun for H6.

## Gate and review result

The H5 closeout baseline passed in a first clean clone. A stale-CMake experiment
found and corrected option loss when changing compilers; the invalidated run is
not acceptance evidence. Corrected local and pre-closeout clean-room H6 gates
passed with real Release and sanitizer flags inspected. Copilot auto reviewed
build/test/docs, and Antigravity gemini-3.8-flash-high reviewed architecture,
FFI, concurrency, failure semantics and evidence. The primary orchestrator
reconstructed and adjudicated every finding in H6.md. No unresolved BLOCKER or
IMPORTANT implementation finding remains. These are AI reviews.

## H5 provenance

- Measured candidate: `1d8724c9521e7d88a43e6f751c4681e1fbbf75c6`.
- H5 closeout: `6ba29f9`.
- Preserved campaign: 744 validated runs, 124 warmups and 620 measurements.
- Local raw archive SHA-256:
  `6fb20bc27a2a6db76fb30afae75691e96dc7aa638b0a34192fd7e812ab55990e`.
- H6 neither remeasured nor changed H5 data. The ignored raw archive is not in a
  clean clone and has no public download claim; obtain it from the project owner.

## Known limitations

- Evidence is one WSL2/Ubuntu 24.04 environment; hosted GitHub Actions was not
  executed during local closeout, though its exact commands passed locally.
- H5 conclusions characterize one host and single-axis design. They do not prove
  general scaling, exact FFI overhead, hard-real-time behavior or causal bottlenecks.
- No guarantee covers hostile same-user file replacement, SIGKILL/OOM, native
  fatal faults, stuck kernel I/O, crash recovery or power-loss durability.
- `check-tools.sh` is best-effort diagnostics and can underreport probe failure;
  actual reviewer commands and their saved outputs are the review evidence.
- Human learning demonstration remains pending.

## Human next step

Confirm the external attestation SHA equals this checkout, inspect H6.md and the
linked evidence, run or sample the clean-room command, and complete the Portuguese
H6 demonstration. If satisfied, approve merge of `codex/h6-release-candidate`.
After the merge commit is separately checked as desired, create the release-
candidate tag `v0.1.0-rc.1`. Do not publish a final release merely from TECH_PASS.
