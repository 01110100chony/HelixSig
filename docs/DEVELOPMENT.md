# AI-driven development governance

## Authority and workflow

The user owns scope, budget, learning verification and publication. The leader
owns contracts, work packets and technical gates. One implementer changes code;
a separate-context reviewer examines a frozen candidate without editing code.
Use Astra when available. Call it AI review, never independent human review.

Work packet -> implementation -> relevant checks -> candidate commit -> review
-> corrections/re-review -> TECH_PASS -> integration -> next milestone.

Each packet defines objective, contracts, allowed edits, exclusions, required
tests and exact acceptance command. Record it in docs/reviews/Hn.md. Code changes
after review invalidate affected evidence. Two inconclusive rounds trigger a
minimal reproducer and leader decision, not an indefinite agent loop.

## Gates

G0: verify WSL2, Windows compatibility, virtualization, at least approximately
10 GiB available, tools and a minimal mixed-language build. Do not silently
substitute a Windows implementation if this fails.

Every technical gate requires mandatory behavior, relevant regression checks,
separate-context review, no blocking findings, accurate docs and evidence tied
to the candidate. Missing evidence is NOT_RUN, not PASS.

Blocking/high: UB, wrong numbers, deadlock, silent loss, false evidence or absent
requirements: no advance. Moderate issues that affect reproducibility/acceptance
also block. Optional style improvements may be deferred explicitly.

Human learning is separate and can remain pending during implementation. No AI
can mark user learning or public-release approval as passed on the user's behalf.

## Change rules

Proceed automatically with in-contract fixes, relevant tests/docs and integration
after technical approval. User decision is needed for mandatory scope cuts,
incompatible frozen-contract changes, budget expansion, purchases and publication.
Do not weaken tests, loosen tolerances or remove mandatory sanitizer checks.
Record contract-change rationale and invalidate affected gates. New dependencies
must have a concrete purpose. Optimization requires before/after measurements.

Scripts define commands; architecture defines contracts; review documents define
evidence. Full logs live in ignored artifacts and durable release evidence later.
Performance evidence requires an identified commit and clean tracked tree.

## Review template

Each H0-H6 review contains Scope and frozen contracts; Implementation handoff
(candidate code commit, changes, limitations); Acceptance matrix (requirement,
evidence, result); Verification (command, environment, exit code, reference);
Findings (ID, severity, location, reproduction, correction, status); Re-review
history; Technical verdict; Learning checkpoint; Handoff.

## CI

Linux formatting/lint/tests + NumPy/Parquet integration; standalone Clang ASan
and UBSan; benchmark smoke only. CI starts at H0. Integrated native sanitizer
smoke is optional with a one-hour investigation limit; TSan is deferred. No
throughput threshold and no full benchmark campaign in routine CI.

## Reproduce a release candidate

Supported validation environment: x86-64 Ubuntu 24.04 (native Linux or WSL2),
Python 3.12 in standard execution mode, GCC/G++ 13 for the default Cargo/CMake
build, Clang/clang-format 18 for standalone ASan/UBSan and formatting, and CMake
3.20 or newer (Ubuntu supplies 3.28). Rust is exactly the channel in
`rust-toolchain.toml`, currently 1.98.1, with rustfmt and Clippy. Cargo.lock and
requirements.txt are committed; NumPy 2.5.3 and PyArrow 25.0.1 are pinned.
The full verification suite needs at least two available CPUs and 5 GiB free
before smoke; allow approximately 10 GiB for a fresh checkout, dependencies and
builds. These are setup requirements, not a performance resource claim.

Use a new checkout on the Linux filesystem, not `/mnt/c` or an existing build
directory. Clone the repository and check out the exact candidate SHA supplied
in the H6 human packet. No pre-existing target/, build/, artifacts/ or .venv/
is needed. The small binary corpus in fixtures/small is tracked. Generation and
validation create their own new output directories under ignored artifacts/.

Install the system dependencies (same list used by CI):

```bash
sudo apt-get update
sudo apt-get install -y build-essential cmake clang-18 clang-format-18 libclang-rt-18-dev python3-venv python3-pip git time util-linux procps curl ca-certificates
```

If rustup is not installed, install it using the [official Rust installer](https://www.rust-lang.org/tools/install)
and open a shell with its Cargo bin directory on PATH. Existing rustup users do
not need a global default-toolchain change. From the checkout root:

```bash
toolchain="$(python3 -c 'import tomllib; print(tomllib.load(open("rust-toolchain.toml", "rb"))["toolchain"]["channel"])')"
rustup toolchain install "$toolchain" --profile minimal --component rustfmt --component clippy
rustup show active-toolchain
python3 -m venv .venv
.venv/bin/python -m pip install -r requirements.txt
set -euo pipefail
mkdir -p artifacts
HELIX_PYTHON=.venv/bin/python scripts/verify.sh H6 2>&1 | tee artifacts/verification.log
```

Without an argument, verify.sh selects H6. Explicit H0-H5 commands remain
available. H6 runs offline delegation-tool regression checks and the complete
H5 suite: native Release CTest/oracle, mandatory standalone Clang ASan/UBSan
CTest/oracle, C++ formatting, Rust formatting/Clippy/tests, H2 sequential,
H3 concurrent and H4 failure readback, H5 runner adversarial checks, and the
200-run Release smoke. Smoke uses 257 events/run and is not performance evidence.
No external AI CLI or credentials are needed by verify.sh or CI.

The script stops nonzero at the first correctness failure. The `pipefail` above
also preserves failure through tee. CMake test/sanitizer options are explicitly
set on each invocation and an empty CTest selection fails. Native builds use
new directories under build/verify-* each time, avoiding CMake cache option loss
when the compiler changes. Routine reruns may reuse the Cargo cache; release acceptance specifically requires a fresh
clone and venv at the proposed SHA. Do not substitute a cached local gate for it.
Use the pinned environment with no custom Cargo/CMake target/profile overrides.

One Linux CI job runs the same command on pushes, PRs and manual dispatch,
including sanitizers; no evidence currently justifies a larger matrix or moving
mandatory checks out of normal CI. The job has a 45-minute bound and retains
artifacts even on failure. The 744-run measurement campaign remains an explicit
manual experiment; it is not part of H6 verification. Hosted CI results must be
reported separately from local reproduction of its commands.

For human inspection, run `cargo run --locked -- --help` and
`cargo run --locked -- run --help`; README.md contains sequential and concurrent
examples. Use a new output directory for each run. Review the H6 study and
Portuguese demonstration guide before granting learning or publication approval.
