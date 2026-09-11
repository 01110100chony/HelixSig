# Antigravity subagent: independent-reviewer

- timestamp: 20260910-153630
- mode: read-only
- model: gemini-3.8-flash-high
- effort: high
- task: READ ONLY bounded H5 benchmark review, candidate e6fa29e. Work in C:/projetos2026/curriculo/Helix. Read docs/reviews/H5.md, docs/reviews/H5_PACKET.md, docs/EXPERIMENTS.md; inspect examples/ffi_bench.rs, cpp/tools/kernel_bench.cpp, scripts/experiment_support.py and scripts/experiment.py. Only local filesystem inspection, no connectors/network/delegation/edits. Verify per-event result verification outside timing, identical event/batch preparation and native input hash authentication. Report concrete BLOCKER/IMPORTANT defects with file:line and trigger or explicitly none; distinguish in-scope requirements from optional improvements. Do not decide milestone PASS/FAIL.

## Response

# Independent External Review: Milestone H5 (Candidate `e6fa29e`)

- **Reviewer:** Independent External Reviewer (Antigravity / Gemini 3.8 Flash High)
- **Review mode:** STRICTLY READ-ONLY (no files created, edited, deleted, or mutated; read-only local inspection only)
- **Repository:** [`Helix`](file:///C:/projetos2026/curriculo/Helix)
- **Candidate under review:** `e6fa29e` (*"Validate all microbenchmark results and authenticate measured inputs"*)
- **Authoritative contracts:** [`AGENTS.md`](file:///C:/projetos2026/curriculo/Helix/AGENTS.md), [`docs/ARCHITECTURE.md`](file:///C:/projetos2026/curriculo/Helix/docs/ARCHITECTURE.md), [`docs/EXPERIMENTS.md`](file:///C:/projetos2026/curriculo/Helix/docs/EXPERIMENTS.md), [`docs/reviews/H5_PACKET.md`](file:///C:/projetos2026/curriculo/Helix/docs/reviews/H5_PACKET.md), and [`docs/reviews/H5.md`](file:///C:/projetos2026/curriculo/Helix/docs/reviews/H5.md)

---

## 1. Executive Summary & Verification Findings

### BLOCKER Findings
**NONE.** No contract breaches, safety regressions, unbounded memory allocations, or compilation defects were detected in candidate `e6fa29e`.

### IMPORTANT Findings
**NONE.** The prior findings recorded in [`docs/reviews/H5.md`](file:///C:/projetos2026/curriculo/Helix/docs/reviews/H5.md#L24-L29) (specifically H5-R1 on cancellation/reordering hiding and H5-R2 on native payload provenance) have been concretely and correctly resolved at candidate `e6fa29e`.

---

## 2. Verification of Specific In-Scope Requirements

### A. Per-Event Result Verification Outside Timing
- **Rust FFI Driver ([`examples/ffi_bench.rs`](file:///C:/projetos2026/curriculo/Helix/examples/ffi_bench.rs#L57-L137)):**
  - Verification memory is preallocated *before* timing begins ([`ffi_bench.rs:61`](file:///C:/projetos2026/curriculo/Helix/examples/ffi_bench.rs#L61)): `let mut observed = vec![ffi::NativeResult::default(); args.events as usize];`.
  - Inside the measured loop, only batch result copying ([`ffi_bench.rs:105`](file:///C:/projetos2026/curriculo/Helix/examples/ffi_bench.rs#L105)), observable checksum accumulation ([`ffi_bench.rs:96-104`](file:///C:/projetos2026/curriculo/Helix/examples/ffi_bench.rs#L96-L104)), and batch size counting occur.
  - The monotonic timer is stopped *immediately* upon loop termination ([`ffi_bench.rs:109`](file:///C:/projetos2026/curriculo/Helix/examples/ffi_bench.rs#L109)): `let duration = start.elapsed().as_nanos();`.
  - *Outside timing*, every captured result is verified for bit-exact consistency against all previous iterations of the cyclic corpus row ([`ffi_bench.rs:110-114`](file:///C:/projetos2026/curriculo/Helix/examples/ffi_bench.rs#L110-L114)):
    ```rust
    for (index, result) in observed.iter().enumerate() {
        if result != &observed[index % rows] {
            return Err("repeated corpus row produced inconsistent results".into());
        }
    }
    ```
  - The distinct row tuples are emitted in JSON under `verification` ([`ffi_bench.rs:115-136`](file:///C:/projetos2026/curriculo/Helix/examples/ffi_bench.rs#L115-L136)).
- **Native C++ Reference Driver ([`cpp/tools/kernel_bench.cpp`](file:///C:/projetos2026/curriculo/Helix/cpp/tools/kernel_bench.cpp#L61-L127)):**
  - Preallocates `observed` before timing starts ([`kernel_bench.cpp:61`](file:///C:/projetos2026/curriculo/Helix/cpp/tools/kernel_bench.cpp#L61)): `std::vector<helix::EventResult> observed(events);`.
  - Within timing, slices are copied to `observed` ([`kernel_bench.cpp:92`](file:///C:/projetos2026/curriculo/Helix/cpp/tools/kernel_bench.cpp#L92)) alongside checksum accumulation ([`kernel_bench.cpp:84-91`](file:///C:/projetos2026/curriculo/Helix/cpp/tools/kernel_bench.cpp#L84-L91)).
  - Timer stops immediately upon loop exit ([`kernel_bench.cpp:96-98`](file:///C:/projetos2026/curriculo/Helix/cpp/tools/kernel_bench.cpp#L96-L98)).
  - *Outside timing*, checks pairwise equality across all repetitions ([`kernel_bench.cpp:101-109`](file:///C:/projetos2026/curriculo/Helix/cpp/tools/kernel_bench.cpp#L101-L109)) and emits full result tuples for each unique row ([`kernel_bench.cpp:119-126`](file:///C:/projetos2026/curriculo/Helix/cpp/tools/kernel_bench.cpp#L119-L126)).
- **Independent Oracle Readback in Python ([`scripts/experiment_support.py:validate_micro`](file:///C:/projetos2026/curriculo/Helix/scripts/experiment_support.py#L98-L110)):**
  - Executed by the runner after each microbenchmark child process terminates ([`scripts/experiment.py:236`](file:///C:/projetos2026/curriculo/Helix/scripts/experiment.py#L236)).
  - Checks `summary["repeat_consistent"] is True`.
  - Compares every unique row's `status` (exact 0), `peak_index` (exact integer match), and floating-point outputs (`baseline`, `peak_amplitude`, `integral`) against [`oracle.py:features`](file:///C:/projetos2026/curriculo/Helix/scripts/oracle.py#L9-L27) at the frozen numerical tolerance: `abs(actual - expected) <= 1e-10 + 1e-10 * abs(expected)`.
  - Adversarial test [`test_micro_validation_rejects_checksum_preserving_errors_and_reordering`](file:///C:/projetos2026/curriculo/Helix/scripts/validate_experiments.py#L16-L36) confirms that cancelling perturbations, reversed row order, and `repeat_consistent = False` are immediately rejected with assertion errors.

### B. Identical Event/Batch Preparation
- **Microbenchmarks vs. Production Pipeline:**
  - Both [`examples/ffi_bench.rs`](file:///C:/projetos2026/curriculo/Helix/examples/ffi_bench.rs#L69-L95) and [`cpp/tools/kernel_bench.cpp`](file:///C:/projetos2026/curriculo/Helix/cpp/tools/kernel_bench.cpp#L66-L82) support identical modes:
    1. `prepared`: Directly slices/subspans the contiguous flat buffer without allocating or copying.
    2. `pack`: Copies owned rows from `source` into a preallocated contiguous `packed` flat buffer.
  - In both drivers, the resulting slice (`samples`) is identical regardless of whether `event` or `batch` FFI mode is invoked:
    - `event` mode iterates over slices of width $N$ and calls [`process_event`](file:///C:/projetos2026/curriculo/Helix/cpp/src/kernel.cpp#L45-L81).
    - `batch` mode passes the flat buffer to [`process_batch`](file:///C:/projetos2026/curriculo/Helix/cpp/src/kernel.cpp#L83-L97) in one call.
  - This structure precisely models the production worker packing logic in [`ProcessingBuffer::process`](file:///C:/projetos2026/curriculo/Helix/src/processing.rs#L26-L69), where events are flattened into `self.flat` before calling `process_event` or `process_batch`.

### C. Native Input Hash Authentication
- In [`scripts/experiment.py`](file:///C:/projetos2026/curriculo/Helix/scripts/experiment.py#L38-L42), [`authenticate_payload`](file:///C:/projetos2026/curriculo/Helix/scripts/experiment.py#L38-L42) calculates the SHA-256 digest of the payload file using `hashlib.file_digest`.
- In [`scripts/experiment.py:210-213`](file:///C:/projetos2026/curriculo/Helix/scripts/experiment.py#L210-L213):
  1. *Before launch:* The runner authenticates `signals.f64le` against the generated manifest hash and records `record["input_sha256"]`.
  2. *After execution:* The runner immediately re-authenticates `signals.f64le` against `record["input_sha256"]` to ensure the child process did not mutate or substitute the input file during execution.
- The input hash is saved in each run record in `runs.jsonl` ([`scripts/experiment.py:242`](file:///C:/projetos2026/curriculo/Helix/scripts/experiment.py#L242)).
- Adversarial test [`test_payload_reordering_is_rejected_even_with_same_values`](file:///C:/projetos2026/curriculo/Helix/scripts/validate_experiments.py#L37-L46) verifies that payload byte modifications or reorderings are caught by the authentication check.

---

## 3. In-Scope Requirements vs. Optional Improvements

| Item | Classification | Rationale & Status |
|---|---|---|
| **Per-event validation outside timing** | **In-Scope Requirement** | Satisfied in `e6fa29e`. All timed outputs are copied to preallocated buffers; consistency across all repetitions and exact comparison against NumPy oracle occur outside timing. |
| **Identical event/batch preparation** | **In-Scope Requirement** | Satisfied in `e6fa29e`. Both Rust and C++ drivers use identical row slices and packing routines across both call granularities. |
| **Native input SHA-256 authentication** | **In-Scope Requirement** | Satisfied in `e6fa29e`. The runner authenticates `signals.f64le` before and after every execution, saving `input_sha256` to the run record. |
| **C++ binary internal SHA-256 computation** | **Optional Improvement** | Unnecessary and excluded. Adding cryptographic hashing directly to `kernel_bench.cpp` would introduce third-party C++ dependencies (e.g., OpenSSL), violating the strict contract ("no additional dependencies" in [`ARCHITECTURE.md:148`](file:///C:/projetos2026/curriculo/Helix/docs/ARCHITECTURE.md#L148)). External authentication by the runner before/after child launch is appropriate. |
| **Threshold-based load/swap rejection (H5-R3)** | **Optional / Out-of-Scope** | Correctly adjudicated as INVALID for mandatory gate in [`docs/reviews/H5.md:28`](file:///C:/projetos2026/curriculo/Helix/docs/reviews/H5.md#L28). The preregistered protocol ([`docs/EXPERIMENTS.md:74-76`](file:///C:/projetos2026/curriculo/Helix/docs/EXPERIMENTS.md#L74-L76)) requires reporting all scheduled runs and disclosing load/swap metrics, avoiding post-hoc selective rerun bias. |

---

## 4. Unresolved Questions Needing Experiment

1. **Full Campaign Execution on `e6fa29e`:**
   - Candidate `32c2730` passed the complete automated gate (`scripts/verify.sh H5`) in an initial Linux checkout, but the full 744-run campaign (96 micro configurations + 28 pipeline configurations $\times$ 6 repetitions) has not yet been executed on the updated candidate `e6fa29e`.
   - *Experiment needed:* Execute `scripts/verify.sh H5` followed by `scripts/experiment.py --out artifacts/h5-campaign` and `scripts/experiment_analysis.py artifacts/h5-campaign` in a clean Linux filesystem environment to populate the campaign evidence and verify that `runs.jsonl`, `analysis.json`, and `analysis.md` complete within the 4 GiB budget.
2. **Commit Hash Record in H5 Documentation:**
   - In [`docs/reviews/H5.md:8`](file:///C:/projetos2026/curriculo/Helix/docs/reviews/H5.md#L8), `- Candidate code commit:` is currently `NOT_CREATED` and the acceptance matrix is `NOT_RUN`.
   - *Follow-up needed:* Once the full Linux gate and campaign run on `e6fa29e`, the commit hash (`e6fa29e...`) and run results should be recorded in the H5 closeout table.

---

## 5. Technical Review Recommendation

**Recommendation on Technical Implementation:** **PASS-READY FOR CAMPAIGN EXECUTION**  
The implementation in candidate `e6fa29e` successfully resolves findings H5-R1 and H5-R2, satisfies all frozen contract requirements for microbenchmarks and runner authentication, and introduces no blockers or regressions.

**Recommendation on Milestone Acceptance Evidence:** **INSUFFICIENT EVIDENCE FOR CLOSEOUT**  
Per [`docs/reviews/H5_PACKET.md:51-54`](file:///C:/projetos2026/curriculo/Helix/docs/reviews/H5_PACKET.md#L51-L54) and [`docs/EXPERIMENTS.md:39-40`](file:///C:/projetos2026/curriculo/Helix/docs/EXPERIMENTS.md#L39-L40), technical evidence is incomplete until the full Linux-filesystem campaign is executed, validated, and archived.

*(Per [`AGENTS.md:49-55`](file:///C:/projetos2026/curriculo/Helix/AGENTS.md#L49-L55) and review instructions, this external review does not decide milestone PASS/FAIL, which remains the exclusive authority of the human user and lead technical orchestrator).*
