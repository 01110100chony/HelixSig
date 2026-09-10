# H5 implementation packet

Prerequisite: H4 TECH_PASS, candidate 4211e93, closeout a13eca1. The user
authorizes H5 after complete H4 verification. ARCHITECTURE.md and EXPERIMENTS.md
remain authoritative; no runtime, numerical, FFI or output contract changes.

## Scope

- Add a Rust FFI microbenchmark executable and a standalone native C++ reference.
  Replay the same generated 256-row corpus for N={64,256,4096}, B={1,4,16,64},
  event/batch calls and prepared/pack+call timing. Allocation, corpus loading and
  output are outside timing; copying for pack+call and observable result checksum
  consumption are inside. Use 100000 events, one warmup and five measurements.
- Add a Linux campaign runner that builds Release with the same C++ compiler,
  no LTO or fast-math, and preserves exact commands, build logs, binary hashes,
  commit/clean tree, corpus hashes, effective resources, versions, load/swap,
  exit codes and per-process peak Linux RSS. Refuse performance measurements on
  mounted Windows paths or dirty tracked source; smoke is explicitly non-evidence.
- Pipeline: deduplicated single-axis union around N=256,W=2,Q=256,B=16 with
  W={1,2,4}, Q={64,256,1024}, B={1,16,64}, both policies and FFI modes. This is
  28 configurations, not a full factorial. One warmup and five measurements per
  configuration, 100000 events each, seeded randomized order per round.
- Preserve Parquet, summaries, actual batch sizes, successful-event raw latencies
  and failure records. Accept exit 2 only with reconciled completed_with_drops.
  Verify numerical output against the independent Python oracle outside timing.
- Enforce a conservative preflight and incremental 4 GiB artifact budget; retain
  incomplete evidence on errors. Bound subprocess timeouts and analysis memory.
- Analyze measured repetitions only: medians, quartiles/range, individual-run
  percentiles and population/overflow flags. Never average percentiles, silently
  omit failed runs or infer exact FFI overhead from different executables.
- Record actual WSL resources. Do not change global WSL settings. Stable actual
  resources may differ from the preregistered planned 4 CPUs/8 GiB/2 GiB.
  Optional N=4096 pipeline/plot polish remain deferred; native N=4096 is required.

## Allowed edits and exclusions

Benchmark-only Rust example, cpp/tools reference, CMake benchmark target,
scripts/experiment*.py, scripts/validate_experiments.py, verify/CI and relevant
documentation/evidence. No production kernel, runtime or bridge behavior changes;
no additional dependencies, performance targets, optimizations or publication.
One implementation writer; freeze candidate during separate-context AI review.

## Acceptance

`HELIX_PYTHON=.venv/bin/python scripts/verify.sh H5` includes the complete H4 gate,
runner/analysis adversarial checks and a small Release smoke campaign. Smoke must
exercise all microbenchmark dimensions and both overload policies but is never
reported as performance evidence. Mandatory standalone ASan/UBSan stay enabled.

A clean Linux-filesystem candidate must complete the full preregistered campaign
within budget. All runs have validated accounting, numerical readback, preserved
raw evidence and metadata; analysis reports dispersion and limitations. Archive
compact durable metadata/results and identify the local raw-data location.
Separate-context benchmark AI review and leader adjudication must close all
blocking findings before TECH_PASS. Learning and publication remain user-owned.
