# Copilot subagent: build-debugger

- timestamp: 20260911-030928
- mode: read-only
- task: The user explicitly authorized sending Helix code, build, tests and documentation to Copilot and Antigravity for H6 review. READ ONLY bounded release audit at C:/projetos2026/curriculo/Helix-h6. Read docs/reviews/H6_PACKET.md, docs/ARCHITECTURE.md, docs/DEVELOPMENT.md, scripts/verify.sh, .github/workflows/ci.yml, build.rs, cpp/CMakeLists.txt, rust-toolchain.toml, requirements.txt and scripts/agents/*.sh. Check for concrete clean-checkout and exit propagation gaps, stale build-state false PASS, missing profiles/modes. Return severity, file:line, reproduction, smallest fix; under 1000 words. Do not edit or execute tests; no runtime changes or H5 campaign, Python -O out of scope. No milestone verdict.

## Response

● skill(docs-audit)

● update_todo todos: "- [in_progress] Inspect governing docs, H6 packet, and repos…"
  └ TODO list updated

/ Search (glob)
  │ "build.rs"
  └ 1 file found

/ Search (glob)
  │ "docs/{ARCHITECTURE.md,DEVELOPMENT.md}"
  └ 2 files found

/ Search (glob)
  │ "scripts/agents/*.sh"
  └ 6 files found

/ Search (glob)
  │ "rust-toolchain.toml"
  └ 1 file found

/ Search (glob)
  │ "AGENTS.md"
  └ 1 file found

/ Search (glob)
  │ "docs/reviews/H6_PACKET.md"
  └ 1 file found

/ Search (glob)
  │ "cpp/CMakeLists.txt"
  └ 1 file found

/ Search (glob)
  │ ".github/workflows/ci.yml"
  └ 1 file found

/ Search (glob)
  │ "requirements.txt"
  └ 1 file found

/ Search (glob)
  │ "scripts/verify.sh"
  └ 1 file found

● Read rust-toolchain.toml
  └ 4 lines read

● Read build.rs
  └ 36 lines read

● Read requirements.txt
  └ 2 lines read

● Read DEVELOPMENT.md
  │ docs\DEVELOPMENT.md
  └ 61 lines read

● Read ARCHITECTURE.md
  │ docs\ARCHITECTURE.md
  └ 149 lines read

● Read ci.yml
  │ .github\workflows\ci.yml
  └ 19 lines read

● Read H6_PACKET.md
  │ docs\reviews\H6_PACKET.md
  └ 67 lines read

● Read verify.sh
  │ scripts\verify.sh
  └ 39 lines read

● Read AGENTS.md
  └ 60 lines read

● Read CMakeLists.txt
  │ cpp\CMakeLists.txt
  └ 32 lines read

● Read release-review.sh
  │ scripts\agents\release-review.sh
  └ 9 lines read

● Read parallel-review.sh
  │ scripts\agents\parallel-review.sh
  └ 8 lines read

● Read new-worktree.sh
  │ scripts\agents\new-worktree.sh
  └ 5 lines read

● Read copilot-agent.sh
  │ scripts\agents\copilot-agent.sh
  └ 37 lines read

● Read check-tools.sh
  │ scripts\agents\check-tools.sh
  └ 36 lines read

● Read agy-agent.sh
  │ scripts\agents\agy-agent.sh
  └ 38 lines read

/ Search (glob)
  │ "scripts/delegation.md"
  └ No matches found

/ Search (glob)
  │ "docs/ROADMAP.md"
  └ 1 file found

The governing packet explicitly makes clean-checkout reproduction, pinned-toolchain selection, and delegation-profile completeness H6 scope. I’m checking those contracts against the available profiles and repository metadata before classifying findings.

● Read ROADMAP.md
  │ docs\ROADMAP.md
  └ 23 lines read

/ Search (glob)
  │ "agent-prompts/**/*.md"
  └ No matches found

/ Search (glob)
  │ "scripts/**/*.sh"
  └ 8 files found

/ Search (grep)
  │ "H6|clean|profile|mode|verify\.sh|toolchain|CI|delegat" in *.md
  └ 87 lines found

● Inspect status and tracked executable modes (shell)
  │ git status --short --branch; git ls-files --stage -- scripts/verify.sh scripts/agents/*.sh
  │ .github/workflows/ci.yml rust-toolchain.toml
  └ 12 lines…

● Read delegation.md
  └ 221 lines read

/ Search (glob)
  │ "agent-prompts"
  └ No matches found

/ Search (glob)
  │ "agent-prompts/copilot/*.md"
  └ No matches found

/ Search (glob)
  │ "agent-prompts/antigravity/*.md"
  └ No matches found

/ Search (glob)
  │ ".github/agents/*.agent.md"
  └ 8 files found

/ Search (grep)
  │ "release-review\.sh|parallel-review\.sh|agy-agent\.sh|copilot-agent\.sh|check-tools\.sh|new-work
  │ tree\.sh"
  └ 23 lines found

● List repository agent profiles (shell)
  │ Get-ChildItem -Force -Recurse -File .github\agents | Select-Object -ExpandProperty FullName
  └ 9 lines…

● List delegation prompt profiles (shell)
  │ Get-ChildItem -Force -Recurse -File agent-prompts | Select-Object -ExpandProperty FullName
  └ 1 line…

● Number audited build and agent scripts (shell)
  │ $files = @('scripts\verify.sh','.github\workflows\ci.yml','build.rs','cpp\CMakeLists.txt','rust-
  │ toolchain.toml','requirements.txt') + (Get-ChildItem scripts\agents\*.sh | Select-Object
  │ -ExpandProperty FullName); foreach ($f in $files) { Write-Output "--- $f"; $n=0; Get-Content $f
  │ | ForEach-Object { $n++; '{0,4}: {1}' -f $n,$_ } }
  └ 278 lines…

● update_todo todos: "- [done] Inspect governing docs, H6 packet, and repository s…"
  └ TODO list updated

/ Search (grep)
  │ "agent-prompts/|independent-reviewer|benchmark-critic|architecture-critic"
  └ 14 lines found

/ Search (grep)
  │ "set -e|wait "\$P|\| tee|mkdir -p|cmake -S|cargo (build|test|clippy)|rust-toolchain" (scripts,
  │ .github, build.rs, cpp\CMakeLists.txt)
  └ 33 lines found

● Number H6 release requirements (shell)
  │ $f='docs\reviews\H6_PACKET.md'; $n=0; Get-Content $f | ForEach-Object { $n++; '{0,4}: {1}' -f
  │ $n,$_ }
  └ 68 lines…

● Confirm tracked delegation profiles and modes (shell)
  │ git ls-files --stage -- agent-prompts scripts/agents .github/agents; git ls-files agent-prompts
  │ | Measure-Object | Select-Object -ExpandProperty Count
  └ 16 lines…

● update_todo todos: "- [done] Inspect governing docs, H6 packet, and repository s…"
  └ TODO list updated

**Release audit findings**

| Severity | File:line | Reproduction | Smallest fix |
|---|---|---|---|
| **BLOCKER** | `scripts/agents/agy-agent.sh:5-6`; `scripts/agents/release-review.sh:7` | `release-review.sh` always invokes `agy-agent.sh independent-reviewer`, but `agent-prompts/antigravity/` is absent and ignored by `.gitignore`. The wrapper exits 2 with `Unknown profile`, so the required Antigravity H6 review cannot run from a clean checkout. | Track the required Antigravity prompt profiles, or change the wrapper/release workflow to use a versioned in-repository profile location. Add a clean-checkout existence check. |
| **IMPORTANT** | `scripts/agents/*.sh` file modes | `git ls-files --stage` reports all agent wrappers as `100644`. On Linux, invoking the documented command `scripts/agents/release-review.sh ...` returns permission denied before review starts. | Mark all executable agent wrappers `100755`, or document and consistently invoke them through `bash`. Executable mode is preferable. |
| **IMPORTANT** | `scripts/agents/parallel-review.sh:7`; `scripts/agents/release-review.sh:8` | `wait "$P1" "$P2" "$P3"` / `wait "$P1" ... "$P4"` returns the status of the final waited job, not an aggregate failure status. If an earlier reviewer fails and the last reviewer succeeds, the wrapper prints “reviews complete” and exits zero. | Wait for each PID separately while accumulating a nonzero status, then exit nonzero if any child failed. |
| **IMPORTANT** | `scripts/verify.sh:7-8,14-17,25-26` | Existing `build/native`, `build/sanitized`, or `target/` state is reused. A cached CMake configuration can disable tests, and stale binaries can remain available to `validate.py`; incremental Cargo artifacts can likewise be reused. A rerun can therefore pass without proving a clean build. | Require fresh build directories (for example, unique temporary build roots) or explicitly remove/clean them before configuration; use a clean Cargo target directory for the release gate. Preserve artifacts outside those roots. |
| **IMPORTANT** | `.github/workflows/ci.yml:11-13` | CI invokes Cargo through `verify.sh` but never explicitly installs/selects Rust `1.98.1`. It relies on the runner’s ambient Rust/rustup behavior despite H6 requiring explicit pinned-toolchain selection. | Add an explicit Rust setup/install step for `1.98.1` with `rustfmt` and `clippy`, or invoke Cargo with the pinned toolchain explicitly. |
| **MODERATE** | `scripts/agents/check-tools.sh:26-33` | The script prints `[OK] agy` before probing it, and suppresses failures from `agy models` and `agy agents` with `|| true`; a broken installation can therefore appear healthy and exit zero. | Check both subcommands, print a failure state, and return nonzero when required discovery fails. |
| **MODERATE** | `requirements.txt:1-2`; `.github/workflows/ci.yml:11` | Python dependencies are pinned, but clean-checkout setup requirements used by verification (`cmake`, Clang/`clang-format`, sanitizer runtime, `time`, Rust toolchain) are only implicit in CI and not documented in `DEVELOPMENT.md`. | Document the complete Linux dependency set and exact setup/verification commands, including the pinned Rust toolchain. |

`build.rs` and `cpp/CMakeLists.txt` have explicit Debug/Release and sanitizer settings; no additional concrete profile defect was identified there.

