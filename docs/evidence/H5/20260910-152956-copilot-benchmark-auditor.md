# Copilot subagent: benchmark-auditor

- timestamp: 20260910-152956
- mode: read-only
- task: Review frozen H5 candidate 32c2730. READ ONLY. Read docs/ARCHITECTURE.md, docs/EXPERIMENTS.md, docs/reviews/H5_PACKET.md; inspect examples/ffi_bench.rs, cpp/tools/kernel_bench.cpp, scripts/experiment*.py, scripts/validate_experiments.py, cpp/CMakeLists.txt and build.rs. Find concrete measurement, resource/budget, evidence validation, missing scope or reproducibility defects. Local read tools only, no delegation or edits. Return only BLOCKER/IMPORTANT findings with file:line and trigger, or explicitly none, plus coverage/limitations. Do not decide milestone PASS/FAIL.

## Response

● skill(benchmark-audit)

/ Search (glob)
  │ "docs/EXPERIMENTS.md"
  └ 1 file found

/ Search (glob)
  │ "cpp/tools/kernel_bench.cpp"
  └ 1 file found

/ Search (glob)
  │ ".agents/skills/benchmark-audit/SKILL.md"
  └ 1 file found

/ Search (glob)
  │ "examples/ffi_bench.rs"
  └ 1 file found

/ Search (glob)
  │ "docs/ARCHITECTURE.md"
  └ 1 file found

/ Search (glob)
  │ "docs/reviews/H5_PACKET.md"
  └ 1 file found

/ Search (glob)
  │ "scripts/experiment*.py"
  └ 3 files found

/ Search (glob)
  │ "scripts/validate_experiments.py"
  └ 1 file found

/ Search (glob)
  │ "AGENTS.md"
  └ 1 file found

/ Search (glob)
  │ "cpp/CMakeLists.txt"
  └ 1 file found

/ Search (glob)
  │ "build.rs"
  └ 1 file found

● Read AGENTS.md
  └ 60 lines read

● Read SKILL.md
  │ .agents\skills\benchmark-audit\SKILL.md
  └ 9 lines read

● Read EXPERIMENTS.md
  │ docs\EXPERIMENTS.md
  └ 92 lines read

● Read ffi_bench.rs
  │ examples\ffi_bench.rs
  └ 122 lines read

● Read kernel_bench.cpp
  │ cpp\tools\kernel_bench.cpp
  └ 109 lines read

● Read experiment.py
  │ scripts\experiment.py
  └ 261 lines read

● Read experiment_support.py
  │ scripts\experiment_support.py
  └ 95 lines read

● Read validate_experiments.py
  │ scripts\validate_experiments.py
  └ 108 lines read

● Read experiment_analysis.py
  │ scripts\experiment_analysis.py
  └ 97 lines read

● Read CMakeLists.txt
  │ cpp\CMakeLists.txt
  └ 32 lines read

● Read build.rs
  └ 36 lines read

● Read ARCHITECTURE.md
  │ docs\ARCHITECTURE.md
  └ 149 lines read

● Read H5_PACKET.md
  │ docs\reviews\H5_PACKET.md
  └ 55 lines read

/ Search (grep)
  │ "H5|benchmark|experiment|campaign|reproduc|RSS|swap|load|compiler|flags"
  │ (docs\reviews\H5_PACKET.md, docs\EXPERIMENTS.md, docs\ARCHITECTURE.md)
  └ 56 lines found

/ Search (grep)
  │ "def resources|def stable_resources|check_budget|events
  │ =|schedule\(|initial_resources|competing_processes|assert stable_resources|platform=|versions=|k
  │ ernel_bench|corpus_sha256|latency_max_trackable|duration_ns" (scripts, examples, cpp)
  └ 77 lines found

● Inspect candidate commit and worktree (shell)
  │ git --no-pager status --short; git --no-pager rev-parse HEAD; git --no-pager show --stat
  │ --oneline 32c2730
  └ 5 lines…

● Number native benchmark lines (shell)
  │ $p='cpp/tools/kernel_bench.cpp'; $i=0; Get-Content $p | ForEach-Object { $i++; '{0}:{1}' -f
  │ $i,$_ }
  └ 110 lines…

● Number analysis lines (shell)
  │ $p='scripts/experiment_analysis.py'; $i=0; Get-Content $p | ForEach-Object { $i++; if($i -ge 35
  │ -and $i -le 75){ '{0}:{1}' -f $i,$_ } }
  └ 42 lines…

● Number runner resource and validation lines (shell)
  │ $p='scripts/experiment.py'; $i=0; Get-Content $p | ForEach-Object { $i++; if(($i -ge 35 -and $i
  │ -le 70) -or ($i -ge 125 -and $i -le 165) -or ($i -ge 180 -and $i -le 250)){ '{0}:{1}' -f $i,$_ }
  │ }
  └ 149 lines…

● Number Rust benchmark lines (shell)
  │ $p='examples/ffi_bench.rs'; $i=0; Get-Content $p | ForEach-Object { $i++; '{0}:{1}' -f $i,$_ }
  └ 123 lines…

**IMPORTANT — incomplete numerical evidence validation**  
- **Files:** `scripts/experiment.py:217-225`, `examples/ffi_bench.rs:103-110`, `cpp/tools/kernel_bench.cpp:94-103`  
- **Trigger:** Microbenchmark runs are accepted using only an aggregate floating-point checksum. No per-event result comparison or order-sensitive digest is performed against the independent oracle. A faulty implementation can therefore pass if errors cancel or preserve the aggregate checksum. This does not satisfy H5’s requirement to verify numerical output outside timing.  
- **Smallest resolving experiment:** Emit a deterministic per-event result digest (or all result tuples for the bounded microbenchmark) and compare it against independently computed oracle results with the frozen tolerance and ordering.

**IMPORTANT — native reference corpus provenance is not authenticated**  
- **Files:** `cpp/tools/kernel_bench.cpp:35-45`, `scripts/experiment.py:199-225`  
- **Trigger:** The native benchmark reads only `signals.f64le`; it does not validate the corpus manifest or payload SHA-256. The runner validates `corpus_sha256` only for Rust FFI runs (`experiment.py:223-224`) and relies on the aggregate checksum for native runs. A changed, reordered, or otherwise substituted native payload can affect measurements without being rejected if its aggregate checksum remains compatible.  
- **Smallest resolving experiment:** Validate the native payload SHA-256 against the generated corpus manifest before launch and include that hash in the native summary.

**IMPORTANT — resource-contamination gate is too weak**  
- **Files:** `scripts/experiment.py:38-51`, `scripts/experiment.py:187-205`, `scripts/experiment.py:239-241`  
- **Trigger:** `resources()` records available memory, free swap, load average, CPU counters, and swap activity, but `stable_resources()` checks only CPU count/affinity and total memory/swap. Campaign runs therefore continue under changing memory pressure, swap activity, CPU load, or competing processes. The metadata records snapshots but does not reject or stratify contaminated runs.  
- **Smallest resolving experiment:** Define and enforce campaign-stability thresholds for load, `MemAvailable`, swap deltas, and competing CPU activity, or mark affected repetitions invalid and rerun them under stable conditions.

**Coverage and limitations:** Read-only inspection covered `AGENTS.md`, the benchmark-audit skill, `docs/ARCHITECTURE.md`, `docs/EXPERIMENTS.md`, `docs/reviews/H5_PACKET.md`, the requested benchmark sources, experiment scripts, validators, `cpp/CMakeLists.txt`, and `build.rs` at candidate `32c2730`. No commands were run to build or execute campaigns, and runtime implementation, generated artifacts, host scheduling, and actual WSL conditions were not independently observed.

