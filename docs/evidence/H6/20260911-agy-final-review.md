# Antigravity subagent: independent-reviewer

- timestamp: 20260911-032942
- mode: read-only
- model: gemini-3.8-flash-high
- effort: high
- task: The user explicitly authorizes sending relevant Helix files to this CLI for H6 review. Frozen candidate 7915688502af9751119fb59f51459478a794cf69 in C:/projetos2026/curriculo/Helix-h6. READ ONLY; do not run gates, edit, install, commit or delegate. Read AGENTS.md, docs/reviews/H6_PACKET.md, docs/reviews/H6.md, docs/ARCHITECTURE.md and docs/evidence/H6/hygiene.md. Implementation efed5d0 passed full H6 suite; current commit adds only evidence. Final clean-room gate intentionally pending. H5 remains TECH_PASS: no full campaign or Python -O redesign. Bounded objective: independent architecture and correctness sanity review. Read docs/H6_STUDY.md, docs/EXPERIMENTS.md, relevant src/config.rs source.rs processing.rs bridge.rs concurrent.rs control.rs runtime.rs output.rs main.rs, cpp/ffi/adapter.cpp and cpp/src/kernel.cpp, plus release verification/CI diff since 6ba29f9. Find supported-contract high-impact output/exit, ownership/FFI, counters, shutdown, stale-state or scope defects. Check the study faithfully describes actual code and H5 observations without invented causes. No architectural redesign or speculative edge cases. Return concrete BLOCKER/IMPORTANT findings with file:line, evidence and minimal remedy; otherwise state coverage and real limitations. Under 1400 words. Do not decide milestone PASS/FAIL.

## Response

# H6 Independent Architecture & Correctness Sanity Review

**Candidate Commit:** `7915688502af9751119fb59f51459478a794cf69`  
**Verified Source Base:** `efed5d040f04c886d436e2b7fcd424930f597d49` (H5 closeout base: `6ba29f9`)  
**Context:** Independent external AI review (Gemini model family); read-only inspection. Milestone verdict is strictly reserved for the primary orchestrator.

---

## 1. Review Scope & Methodology

This evaluation inspected the codebase at candidate `7915688`, covering:
- Governing contracts and packets: [AGENTS.md](file:///C:/projetos2026/curriculo/Helix-h6/AGENTS.md), [H6_PACKET.md](file:///C:/projetos2026/curriculo/Helix-h6/docs/reviews/H6_PACKET.md), [H6.md](file:///C:/projetos2026/curriculo/Helix-h6/docs/reviews/H6.md), [ARCHITECTURE.md](file:///C:/projetos2026/curriculo/Helix-h6/docs/ARCHITECTURE.md), and [hygiene.md](file:///C:/projetos2026/curriculo/Helix-h6/docs/evidence/H6/hygiene.md).
- Preserved implementation and contracts: [`src/config.rs`](file:///C:/projetos2026/curriculo/Helix-h6/src/config.rs), [`src/source.rs`](file:///C:/projetos2026/curriculo/Helix-h6/src/source.rs), [`src/processing.rs`](file:///C:/projetos2026/curriculo/Helix-h6/src/processing.rs), [`src/bridge.rs`](file:///C:/projetos2026/curriculo/Helix-h6/src/bridge.rs), [`src/concurrent.rs`](file:///C:/projetos2026/curriculo/Helix-h6/src/concurrent.rs), [`src/control.rs`](file:///C:/projetos2026/curriculo/Helix-h6/src/control.rs), [`src/runtime.rs`](file:///C:/projetos2026/curriculo/Helix-h6/src/runtime.rs), [`src/output.rs`](file:///C:/projetos2026/curriculo/Helix-h6/src/output.rs), [`src/main.rs`](file:///C:/projetos2026/curriculo/Helix-h6/src/main.rs), [`cpp/ffi/adapter.cpp`](file:///C:/projetos2026/curriculo/Helix-h6/cpp/ffi/adapter.cpp), and [`cpp/src/kernel.cpp`](file:///C:/projetos2026/curriculo/Helix-h6/cpp/src/kernel.cpp).
- Release diffs, CI, and tools since `6ba29f9`: [`scripts/verify.sh`](file:///C:/projetos2026/curriculo/Helix-h6/scripts/verify.sh), [`scripts/validate_release.sh`](file:///C:/projetos2026/curriculo/Helix-h6/scripts/validate_release.sh), [`.github/workflows/ci.yml`](file:///C:/projetos2026/curriculo/Helix-h6/.github/workflows/ci.yml), and reviewer wrappers in `scripts/agents/`.
- Technical study and provenance: [H6_STUDY.md](file:///C:/projetos2026/curriculo/Helix-h6/docs/H6_STUDY.md) and [EXPERIMENTS.md](file:///C:/projetos2026/curriculo/Helix-h6/docs/EXPERIMENTS.md).

---

## 2. Architecture, Ownership & FFI Sanity

1. **Memory Ownership & Buffer Lifetimes:**
   - As documented in [ARCHITECTURE.md](file:///C:/projetos2026/curriculo/Helix-h6/docs/ARCHITECTURE.md#L63-L80), Rust maintains complete ownership of waveform storage, event queues, flat sample buffers, and result arrays.
   - In [`ProcessingBuffer::process`](file:///C:/projetos2026/curriculo/Helix-h6/src/processing.rs#L26-L69), both `FfiMode::Event` and `FfiMode::Batch` use an identical reusable buffer (`self.flat`). Samples are copied once into `Event` and once into `flat`, confirming the contract statement that the pipeline is **not** zero-copy.
   - In [`adapter.cpp`](file:///C:/projetos2026/curriculo/Helix-h6/cpp/ffi/adapter.cpp#L19-L35), `rust::Slice<const double>` and `rust::Slice<NativeResult>` are borrowed strictly for the synchronous duration of the call. C++ allocates zero dynamic heap memory, retains zero pointers, creates no background threads, and accesses no mutable globals.
2. **Error Translation & Kernel Boundary Isolation:**
   - In [`kernel.cpp`](file:///C:/projetos2026/curriculo/Helix-h6/cpp/src/kernel.cpp#L83-L97), structural batch anomalies (`samples.size() / width != output.size()`, invalid width, batch > 64) return `BatchStatus::invalid_shape` without writing to output. [`adapter.cpp:27-31`](file:///C:/projetos2026/curriculo/Helix-h6/cpp/ffi/adapter.cpp#L27-L31) maps these to `std::invalid_argument`, which the CXX boundary catches and converts into Rust `Result::Err`.
   - Numerical errors (non-finite samples, division by zero, non-finite results) are reported as per-event enum values in [`EventResult::status`](file:///C:/projetos2026/curriculo/Helix-h6/cpp/include/helix/kernel.hpp#L23-L29), preventing a single malformed event from poisoning neighboring batch results.
   - Worker panic containment is isolated on the Rust side via [`catch_unwind(AssertUnwindSafe(...))`](file:///C:/projetos2026/curriculo/Helix-h6/src/concurrent.rs#L271-L317); unwinding does not cross the native FFI boundary.

---

## 3. Concurrency, Shutdown & Accounting Rigor

1. **Conservation Identities:**
   Helix tracks three conservation identities verified in [`Counters::reconciles`](file:///C:/projetos2026/curriculo/Helix-h6/src/runtime.rs#L26-L31) and asserted at runtime finish:
   $$\text{produced} = \text{accepted} + \text{dropped} + \text{not\_admitted}$$
   $$\text{accepted} = \text{processed} + \text{failed} + \text{aborted}$$
   $$\text{processed} = \text{written} + \text{unwritten}$$
2. **Transition Exclusivity:**
   - **Admission:** In [`produce`](file:///C:/projetos2026/curriculo/Helix-h6/src/concurrent.rs#L181-L256), every materialized attempt incrementing `produced` transitions to either `accepted` (via channel send), `dropped` (under `Policy::DropNew` when the queue is saturated), or `not_admitted` (upon channel closure or observed stop request).
   - **Processing & Abort Accounting:** In [`worker`](file:///C:/projetos2026/curriculo/Helix-h6/src/concurrent.rs#L258-L333), accepted events pulled from `input` either deliver results to the collector (`delivered += 1`, transitioning to `processed` or `failed`) or, upon early exit/panic, undelivered entries are accounted in [`report.aborted += (events.len() - delivered)`](file:///C:/projetos2026/curriculo/Helix-h6/src/concurrent.rs#L327).
   - **Queue Remnants:** After worker join, remaining queued events in the supervisor channel are extracted and added to `aborted` ([`concurrent.rs:170-173`](file:///C:/projetos2026/curriculo/Helix-h6/src/concurrent.rs#L170-L173)).
   - **Disk Finalization:** Valid numerical results initialize as `unwritten`. They transition to `written` in [`finish_execution`](file:///C:/projetos2026/curriculo/Helix-h6/src/runtime.rs#L291-L295) only upon successful return of [`writer.finish`](file:///C:/projetos2026/curriculo/Helix-h6/src/output.rs#L201-L217), which guarantees full Parquet row-group flush, footer closure, and atomic file rename.
3. **Deadlock-Free Backpressure & Drain:**
   - The result queue is bounded at 256 items ([`concurrent.rs:17`](file:///C:/projetos2026/curriculo/Helix-h6/src/concurrent.rs#L17)). In [`execute_with_hooks`](file:///C:/projetos2026/curriculo/Helix-h6/src/concurrent.rs#L143-L148), if a disk write failure occurs (`writer_failed = true`), the collector initiates a graceful stop to cease new event production, but continues draining `result_rx`. This prevents worker threads from deadlocking on `results.send()`.
   - Threads are joined only after `result_rx` drains completely, preventing sender-deadlock prior to thread termination.

---

## 4. Exit Codes & Output Contracts

1. **CLI & Process Exit Consistency:**
   - [`main.rs:19-23`](file:///C:/projetos2026/curriculo/Helix-h6/src/main.rs#L19-L23): CLI syntax errors exit `64` (`EX_USAGE`), while `--help`/`--version` exit `0`.
   - [`RunSummary::exit_code`](file:///C:/projetos2026/curriculo/Helix-h6/src/runtime.rs#L92-L100): Maps status faithfully to contract:
     - `Completed` $\rightarrow 0$
     - `CompletedWithDrops` $\rightarrow 2$
     - `Interrupted` $\rightarrow 130$
     - `Failed` $\rightarrow 1$
     - `InvalidConfiguration` $\rightarrow 64$
   - Stdout serialization failures in `main.rs:35` trigger an explicit fallback to exit `1`.
2. **File Artifact Integrity:**
   - Incomplete runs leave `events.parquet.incomplete` (state: `Incomplete`).
   - Partial runs due to numerical failure or SIGINT finalize to `events.partial.parquet` (state: `Partial`).
   - Successful runs finalize to `events.parquet` (state: `Finalized`), even when event loss occurs under `drop-new` (exit `2`).
   - In [`finalize_summary`](file:///C:/projetos2026/curriculo/Helix-h6/src/runtime.rs#L168-L178), failures during `summary.json` emission mark the output state as `Partial` and rename the Parquet file to avoid ambiguous clean-completion claims.

---

## 5. Release Verification, Hygiene & CI Diff Audit (since `6ba29f9`)

1. **Source & Preserve Integrity:**
   - A diff against `6ba29f9` across `src/`, `cpp/`, `examples/`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `requirements.txt`, `fixtures/`, `docs/evidence/H5/`, and experiment scripts is **completely empty** ($0$ modifications). Core implementation and H5 evidence are byte-for-byte preserved.
2. **CMake Cache Invalidation Fix ([`verify.sh:16-28`](file:///C:/projetos2026/curriculo/Helix-h6/scripts/verify.sh#L16-L28)):**
   - Candidate `efed5d0` replaced static `build/native` and `build/sanitized` directories with dynamic, unique per-run paths (`mktemp -d build/verify-XXXXXX`).
   - This eliminates CMake option cache clobbering (H6-R4) where compiler switches discarded sanitizer or release flags.
3. **Agent Review Tooling Fixes ([`validate_release.sh`](file:///C:/projetos2026/curriculo/Helix-h6/scripts/validate_release.sh)):**
   - The multi-PID wait loop flaw (where `wait $P1 $P2 ...` masked earlier child failure) was fixed in [`parallel-review.sh`](file:///C:/projetos2026/curriculo/Helix-h6/scripts/agents/parallel-review.sh#L7-L14) and [`release-review.sh`](file:///C:/projetos2026/curriculo/Helix-h6/scripts/agents/release-review.sh#L8-L15) by accumulating exit statuses across all PIDs.
   - Offline verification in `validate_release.sh` confirms that seven individual reviewer failure injections fail the group wrapper as expected.
   - All shell scripts in `scripts/` and `scripts/agents/` are tracked with mode `100755`.
   - Antigravity prompt profiles in [`agent-prompts/antigravity/`](file:///C:/projetos2026/curriculo/Helix-h6/agent-prompts/antigravity/) are tracked and removed from `.gitignore`.
4. **CI Pipeline Alignment ([`.github/workflows/ci.yml`](file:///C:/projetos2026/curriculo/Helix-h6/.github/workflows/ci.yml)):**
   - CI explicitly reads `rust-toolchain.toml` to install Rust `1.98.1` with `rustfmt` and `clippy`.
   - Explicitly installs system packages matching [`docs/DEVELOPMENT.md`](file:///C:/projetos2026/curriculo/Helix-h6/docs/DEVELOPMENT.md#L83-L86) and executes `verify.sh H6` with `pipefail`.

---

## 6. Fidelity of H6 Technical Study (`docs/H6_STUDY.md`)

- **Code Alignment:** The descriptions of buffer ownership, the two-copy model, pairwise summation, and the failure/status matrix match the exact implementation in `src/` and `cpp/`.
- **H5 Observations:** The reported metrics—including W=1,2,4 median throughputs (368,418; 507,085; and 509,060 events/s), drop-new median loss fraction (0.89246), and nanosecond FFI benchmark timings—match the validated records in [`EXPERIMENTS.md`](file:///C:/projetos2026/curriculo/Helix-h6/docs/EXPERIMENTS.md#L50-L63) and [`docs/evidence/H5/summary.json`](file:///C:/projetos2026/curriculo/Helix-h6/docs/evidence/H5/summary.json).
- **Hypotheses vs. Causes:** [H6_STUDY.md:137-141](file:///C:/projetos2026/curriculo/Helix-h6/docs/H6_STUDY.md#L137-L141) explicitly designates potential bottleneck explanations (copying costs, scheduling overhead, disk persistence limits) as *unproven hypotheses*, acknowledging that the single-axis protocol cannot establish causality without dedicated profiling. No invented causes are asserted.

---

## 7. Findings

### BLOCKER Findings
*None.* No supported-contract violations, memory safety issues, counter leaks, or false-pass conditions were identified.

### IMPORTANT Findings
*None.* Previous findings (H6-R1 through H6-R5, H6-R7, H6-R8, H6-R10) were correctly remediated and confirmed in commits `5d83ce2` through `efed5d0`.

### MINOR / Observational Notes
1. **Deferred Discovery Utility Limitation (H6-R6):**  
   - **File & Line:** [`scripts/agents/check-tools.sh:26-33`](file:///C:/projetos2026/curriculo/Helix-h6/scripts/agents/check-tools.sh)  
   - **Evidence:** `check-tools.sh` is an optional developer discovery script, not part of the `verify.sh` or CI gate. It prints `[OK]` before subcommands finish.  
   - **Impact / Remedy:** Already tracked and adjudicated in `H6.md` as MINOR; no action required for release correctness since actual subagent wrappers enforce strict exit codes.

---

## 8. Review Coverage & Real Limitations

1. **Covered Areas:**
   - Static analysis of Rust and C++ source code, FFI boundary safety, crossbeam concurrency, counter invariants, shutdown edge cases, exit code paths, release tooling, CMake scripts, CI configuration, and H6 study accuracy.
2. **Real Limitations & Bounded Scope:**
   - **Execution Environment:** This review was conducted in a read-only environment without executing tests or gates.
   - **Gate E Pending:** Final clean-room reproduction (Gate E) in an isolated, fresh clone without cached Cargo or venv directories remains pending as required by [H6_PACKET.md](file:///C:/projetos2026/curriculo/Helix-h6/docs/reviews/H6_PACKET.md#L37).
   - **Supported Scope Exclusions:** Python `-O` optimization flag, hostile same-user file replacement, native fatal signals (`SIGKILL`), and power-loss disk corruption remain outside the supported contract.

---

## 9. Conclusion

The implementation and release candidate at `7915688` (base `efed5d0`) adhere strictly to the frozen contracts in `ARCHITECTURE.md`. Concurrency, FFI boundary ownership, accounting identities, and shutdown mechanics are sound. The technical study accurately reflects the codebase and experimental data without unjustified claims. Advancement to Gate E (clean-room revalidation) is technically justified.
