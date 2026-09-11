# Copilot subagent: docs-auditor

- timestamp: 20260911-032943
- mode: read-only
- task: The user explicitly authorizes sending relevant Helix files to this CLI for H6 review. Frozen candidate 7915688502af9751119fb59f51459478a794cf69 in C:/projetos2026/curriculo/Helix-h6. READ ONLY; do not run gates, edit, install, commit or delegate. Read AGENTS.md, docs/reviews/H6_PACKET.md, docs/reviews/H6.md, docs/ARCHITECTURE.md and docs/evidence/H6/hygiene.md. Implementation efed5d0 passed full H6 suite; current commit adds only evidence. Final clean-room gate intentionally pending. H5 remains TECH_PASS: no full campaign or Python -O redesign. Bounded objective: audit build/test/docs changes since 6ba29f9. Inspect scripts/verify.sh, scripts/validate_release.sh, scripts/agents group helpers, CI, setup docs, README, H6 study, H5 provenance. Verify exit propagation, actual sanitizer/profile selection, clean-checkout sufficiency, current claims, smoke vs measurements and evidence traceability. Existing H6-R1..R10 are adjudicated; test whether fixes address them. Return BLOCKER/IMPORTANT/MINOR findings with file:line, concrete reproduction/impact, smallest fix; list verified corrections and limitations. Under 1400 words. Primary orchestrator owns final verdict.

## Response

● skill(docs-audit)

I’ll perform the requested clean-room audit using only read-only inspection, covering the frozen candidate’s governing docs, implementation scripts, CI, and evidence surfaces.

/ Search (glob)
  │ "AGENTS.md"
  └ 1 file found

/ Search (glob)
  │ "scripts/verify.sh"
  └ 1 file found

/ Search (glob)
  │ "docs/ARCHITECTURE.md"
  └ 1 file found

/ Search (glob)
  │ "docs/reviews/H6_PACKET.md"
  └ 1 file found

/ Search (glob)
  │ "docs/reviews/H6.md"
  └ 1 file found

/ Search (glob)
  │ "docs/**/*h5*"
  └ No matches found

/ Search (glob)
  │ "docs/evidence/H6/hygiene.md"
  └ 1 file found

/ Search (glob)
  │ "scripts/validate_release.sh"
  └ 1 file found

/ Search (glob)
  │ "scripts/agents/**"
  └ 6 files found

/ Search (glob)
  │ "docs/**/*H5*"
  └ 2 files found

/ Search (glob)
  │ "README*"
  └ 1 file found

/ Search (glob)
  │ ".github/workflows/**"
  └ 1 file found

● Read copilot-agent.sh
  │ scripts\agents\copilot-agent.sh
  └ 37 lines read

● Read check-tools.sh
  │ scripts\agents\check-tools.sh
  └ 36 lines read

● Read ci.yml
  │ .github\workflows\ci.yml
  └ 38 lines read

● Read H5_PACKET.md
  │ docs\reviews\H5_PACKET.md
  └ 56 lines read

● Read release-review.sh
  │ scripts\agents\release-review.sh
  └ 16 lines read

● Read agy-agent.sh
  │ scripts\agents\agy-agent.sh
  └ 38 lines read

● Read new-worktree.sh
  │ scripts\agents\new-worktree.sh
  └ 5 lines read

● Read parallel-review.sh
  │ scripts\agents\parallel-review.sh
  └ 15 lines read

● Read H6_PACKET.md
  │ docs\reviews\H6_PACKET.md
  └ 67 lines read

● Read ARCHITECTURE.md
  │ docs\ARCHITECTURE.md
  └ 149 lines read

● Read README.md
  └ 146 lines read

● Read hygiene.md
  │ docs\evidence\H6\hygiene.md
  └ 80 lines read

● Read validate_release.sh
  │ scripts\validate_release.sh
  └ 46 lines read

● Read H6.md
  │ docs\reviews\H6.md
  └ 99 lines read

● Read AGENTS.md
  └ 60 lines read

● Read verify.sh
  │ scripts\verify.sh
  └ 51 lines read

● Read H5.md
  │ docs\reviews\H5.md
  └ 96 lines read

● Read DEVELOPMENT.md
  │ docs\DEVELOPMENT.md
  └ 129 lines read

● Read H6_STUDY.md
  │ docs\H6_STUDY.md
  └ 157 lines read

● Read EXPERIMENTS.md
  │ docs\EXPERIMENTS.md
  └ 149 lines read

● Read independent-reviewer.md
  │ agent-prompts\antigravity\independent-reviewer.md
  └ 7 lines read

● Read rust-toolchain.toml
  └ 4 lines read

● Read architecture-critic.md
  │ agent-prompts\antigravity\architecture-critic.md
  └ 6 lines read

● Read .gitignore
  └ 7 lines read

● Read benchmark-critic.md
  │ agent-prompts\antigravity\benchmark-critic.md
  └ 7 lines read

● Read ROADMAP.md
  │ docs\ROADMAP.md
  └ 23 lines read

● Read STUDY_GUIDE.pt-BR.md
  │ docs\STUDY_GUIDE.pt-BR.md
  └ 87 lines read

● Read CMakeLists.txt
  │ cpp\CMakeLists.txt
  └ 32 lines read

/ Search (grep)
  │ "wait |set -e|set -u|set -o|\| true|\| tee|cmake -S|ctest|cargo test|experiment.py" in *.sh
  │ (scripts)
  └ 34 lines found

/ Search (grep)
  │ "verify\.sh|validate_release|H6|ASAN|UBSAN|CMAKE_BUILD_TYPE|HELIX_SANITIZERS|smoke|measurement|f
  │ ull campaign|TECH_PASS|NOT_READY" in *.md
  └ 163 lines found

● Inspect H6 diff summary (shell)
  │ git --no-pager diff --stat 6ba29f9 7915688502af9751119fb59f51459478a794cf69; git --no-pager diff
  │ --name-status 6ba29f9 7915688502af9751119fb59f51459478a794cf69
  └ 60 lines…

● Read local-verification.log
  │ docs\evidence\H6\local-verification.log
  └ L1:160 (134 lines read)

● Read baseline.log
  │ docs\evidence\H6\baseline.log
  └ L1:80 (80 lines read)

● Read 20260911-copilot-build-audit.md
  │ docs\evidence\H6\20260911-copilot-build-audit.md
  └ L190:230 (32 lines read)

/ Search (grep)
  │ "g0\.sh|/root|H6_STUDY|local-verification|raw archive|clean checkout|fresh clone|standalone" in
  │ *.md
  └ 51 lines found

/ Search (grep)
  │ "subprocess|returncode|exit code|smoke_only|--smoke|validate_release|verify.sh" in *.py
  │ (scripts)
  └ 37 lines found

● Inspect frozen candidate history (shell)
  │ git --no-pager log --oneline --decorate -8; git --no-pager show --stat --oneline --summary
  │ 7915688502af9751119fb59f51459478a794cf69; git --no-pager diff --stat efed5d0
  │ 7915688502af9751119fb59f51459478a794cf69
  └ 29 lines…

## H6 clean-room audit

**Verdict:** No new BLOCKER was found in the reviewed build, test, CI, documentation, or delegation changes. The frozen candidate remains **NOT_READY**, because the required final clean-checkout gate has not been executed at `7915688502af9751119fb59f51459478a794cf69`. The recorded verification is for implementation commit `efed5d0`, while the current commit adds evidence and review-document updates only.

| Severity | File:line | Finding | Reproduction / impact | Smallest fix |
|---|---|---|---|---|
| **IMPORTANT** | `docs/reviews/H6.md:10-15,27,48-57` | Evidence is tied to `efed5d0`, not explicitly to frozen candidate `7915688502af9751119fb59f51459478a794cf69`. | The local H6 suite and sanitizer/profile evidence are recorded at `efed5d0`; the current candidate is an evidence-only descendant. Without final exact-SHA verification, the release packet does not yet prove the frozen commit was checked. | Record `7915688502af9751119fb59f51459478a794cf69` as the candidate, document the exact `efed5d0..7915688` source/code diff as evidence-only, then run the mandated fresh Linux clone/venv gate at `7915688`. |
| **MINOR** | `scripts/validate_release.sh:8-46` | The offline release-tool regression validates the group wrappers using fake nested helpers, but does not directly exercise `scripts/agents/agy-agent.sh` or its profile-loading path. | A regression in `agy-agent.sh` or profile resolution could leave `validate_release.sh` green because the test replaces it with a stub. The tracked profiles and wrapper modes are separately present, so this is test-coverage weakness rather than a demonstrated release defect. | Add a read-only offline probe for `agy-agent.sh` that verifies a tracked profile resolves and a missing profile fails. |
| **MINOR** | `scripts/agents/check-tools.sh:20-33`; `docs/evidence/H6/hygiene.md:77-80` | Tool discovery can report `[OK] agy` even when `agy models` or `agy agents` fails, because both probes use `|| true`. | This can mislead an operator about optional reviewer availability, although it cannot make the automated H6 suite pass and the limitation is documented. | Report probe failures explicitly, or return nonzero when reviewer discovery is required. |

### Verified corrections

- `scripts/verify.sh:2,14-29` uses `set -euo pipefail`, creates fresh native build directories, explicitly selects Release versus Debug, enables/disables `HELIX_SANITIZERS`, requires CTest tests with `--no-tests=error`, and propagates CMake, CTest, oracle, Rust, and smoke failures.
- `scripts/verify.sh:25-31` selects `clang++-18` with `CMAKE_BUILD_TYPE=Debug` and `HELIX_SANITIZERS=ON`; `cpp/CMakeLists.txt:10-17` applies AddressSanitizer and UndefinedBehaviorSanitizer compile/link flags. The recorded local log shows `HELIX_SANITIZERS:BOOL=ON`, Debug, and Clang 18, followed by sanitizer CTest and oracle execution.
- `scripts/agents/parallel-review.sh:2-12` and `release-review.sh:2-13` wait for every child and accumulate failures; they no longer report successful completion after an earlier reviewer fails.
- Agent wrappers are tracked executable files, and all three Antigravity profiles are tracked. The prior H6-R1/R2/R3 corrections are present.
- CI installs the documented native dependencies, creates the pinned Python environment, installs the Rust channel from `rust-toolchain.toml`, and invokes the same H6 verification command (`.github/workflows/ci.yml:14-27`).
- README, development setup, H6 study, Portuguese guide, H5 provenance, smoke-versus-measurement language, and limitations are materially reconciled. Claims remain qualified: no full H5 campaign was rerun, smoke is non-evidence, Python `-O` is out of scope, and H5 observations are not general performance laws.
- `scripts/verify.sh` does not reuse stale native CMake trees. Cargo artifacts may be reused during routine reruns, but `docs/DEVELOPMENT.md:77-118` correctly requires a fresh clone and venv for release acceptance.

### Limitations

The review remains an AI review, not human approval. The current evidence demonstrates the implementation commit’s local gate and preserves H5 provenance, but does not substitute for the required final clean-room execution at the exact frozen SHA. The documented exclusions remain appropriate: no full campaign rerun, no Python `-O` redesign, no hostile same-user filesystem replacement guarantee, and no guarantees for SIGKILL, OOM kill, native fatal faults, stuck I/O, or power loss.

