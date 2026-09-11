# H6 release-candidate work packet

Base: H5 closeout `6ba29f9`; measured H5 candidate
`1d8724c9521e7d88a43e6f751c4681e1fbbf75c6`. H0-H5 remain TECH_PASS.
The primary orchestrator owns scope, adjudication and the technical verdict.
Human learning, merge, tagging and publication are separate user decisions.

## Inventory before implementation

Already present: pinned Rust 1.98.1 with rustfmt/Clippy, Cargo.lock, exact
NumPy/PyArrow versions, independent CMake C++20 kernel, CXX build integration,
bounded runtime/failure regressions, standalone ASan/UBSan, 533-case oracle,
H5 runner adversarial tests and a 200-run smoke-only campaign. `verify.sh H5`
already composes these checks and propagates failure with `set -euo pipefail`.
No new verification wrapper or performance campaign is needed.

Remaining: reproduce from clean Linux checkouts; document complete dependencies
and exact local/CI commands; explicitly select the pinned Rust toolchain in CI;
reconcile stale README/status/provenance; audit executable modes, generated files
and missing delegation profiles; produce the architecture study, targeted sanity
review, finding adjudications and final human-review packet.

Likely edits: scripts/verify.sh, .github/workflows/ci.yml, README.md,
docs/DEVELOPMENT.md, docs/ROADMAP.md, docs/EXPERIMENTS.md,
docs/STUDY_GUIDE.pt-BR.md, docs/reviews/H6*.md, release study/evidence, and narrowly
justified build/repository hygiene. Existing user edits in the original checkout
are preserved; work takes place in `codex/h6-release-candidate` in another worktree.

## Sequential gates

| Gate | Work | Required evidence before advancement |
|---|---|---|
| A: scope/inventory | Read contracts, prior reviews and tooling; independent read-only inventory | This packet; bounded file map and adjudication |
| B: initial reproduction | New Linux clone of H5 closeout; documented setup and full existing H5 verification | Commit, environment, commands, exit status/log; concrete setup defects recorded |
| C: release preparation | Minimal verification/CI/hygiene fixes; docs reconciliation and technical study | Local H6 gate, failure propagation check, clean diff and H5 preservation audit |
| D: frozen AI review | Read-only build/test/docs and architecture/correctness review in separate contexts | All findings classified VALID/INVALID/DUPLICATE/NEEDS-EXPERIMENT, severity/reason/fix; no unresolved release blockers |
| E: final clean room | Freeze commit; another new Linux clone and venv with no build directories | Exact proposed SHA, versions, setup and H6 commands, zero exit, logs, human packet |

Environment problems may leave B pending while independent audit/documentation
work proceeds; they never count as PASS. Correctness failures stop the dependent
gate. Any candidate edit after E requires another verification of the new commit.
Final attestation is stored outside the frozen source tree to avoid changing the
commit it attests; its SHA and location are delivered in the human-review packet.

## Contracts and exclusions

ARCHITECTURE.md and the H0-H5 reviews remain authoritative. Do not change kernel
algorithms/tolerances, FFI ownership, concurrency, queue/drop/failure semantics,
benchmark workloads, H5 measurements or preserved raw evidence. A concrete defect
requiring such a change must be explained to the user before implementation.
No 744-run campaign, feature development, dependency expansion without need,
Windows-native target, async runtime, networking, GPU or C++ threads.
Standard Python execution is supported; Python `-O` is not an H5 blocker.

Reuse the full H5 suite for H6, including mandatory standalone sanitizers and
smoke. Keep one compact Linux CI job unless measurements justify separation;
normal CI must not run the full performance campaign. AI findings are advisory.
Only validated BLOCKER/IMPORTANT release-correctness defects warrant behavior
fixes. Documentation and build hygiene remain in scope.

## Acceptance and human boundary

H6 TECH_PASS requires clean build/tests, representative CI, documented sufficient
setup, no hidden local artifacts, accurate docs/study, intact H5 provenance,
review adjudication, explicit limitations, final exact-commit revalidation and a
human packet. Otherwise H6 NOT_READY with concrete blockers. No technical gate
grants human learning or publication approval. Do not merge, tag or publish.
