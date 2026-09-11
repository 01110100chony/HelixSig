# Copilot subagent: benchmark-auditor

- timestamp: 20260910-153608
- mode: read-only
- task: READ ONLY re-review of frozen H5 e6fa29e. Read docs/reviews/H5.md for adjudication, docs/reviews/H5_PACKET.md, docs/EXPERIMENTS.md. Inspect the corrected examples/ffi_bench.rs, cpp/tools/kernel_bench.cpp and scripts/experiment*.py plus scripts/validate_experiments.py. Verify H5-R1 full result capture/repeat consistency plus oracle validation and H5-R2 pre/post payload authentication; find remaining concrete BLOCKER/IMPORTANT defects within the preregistered scope. Evaluate H5-R3 against the actual record-load/stable-config contract, not a new constant-load requirement. Read-only local tools, no edits/delegation. Return finding status, file:line evidence, coverage and limitations; no milestone verdict.

## Response

● skill(benchmark-audit)

I’ll inspect the governing instructions, H5 adjudication/packet, experiment contract, and all specified benchmark/validation files, then trace the relevant capture, oracle, authentication, and record-load behavior without modifying the tree.

/ Search (glob)
  │ "cpp\tools\kernel_bench.cpp"
  └ 1 file found

/ Search (glob)
  │ "docs\reviews\H5.md"
  └ 1 file found

/ Search (glob)
  │ "docs\reviews\H5_PACKET.md"
  └ 1 file found

/ Search (glob)
  │ "scripts\validate_experiments.py"
  └ 1 file found

/ Search (glob)
  │ "docs\EXPERIMENTS.md"
  └ 1 file found

/ Search (glob)
  │ "examples\ffi_bench.rs"
  └ 1 file found

/ Search (glob)
  │ "scripts\experiment*.py"
  └ 3 files found

/ Search (glob)
  │ "AGENTS.md"
  └ 1 file found

● Read H5.md
  │ docs\reviews\H5.md
  └ 46 lines read

● Read experiment_support.py
  │ scripts\experiment_support.py
  └ 109 lines read

● Read experiment_analysis.py
  │ scripts\experiment_analysis.py
  └ 97 lines read

● Read kernel_bench.cpp
  │ cpp\tools\kernel_bench.cpp
  └ 132 lines read

● Read experiment.py
  │ scripts\experiment.py
  └ 272 lines read

● Read AGENTS.md
  └ 60 lines read

● Read H5_PACKET.md
  │ docs\reviews\H5_PACKET.md
  └ 56 lines read

● Read ffi_bench.rs
  │ examples\ffi_bench.rs
  └ 147 lines read

● Read EXPERIMENTS.md
  │ docs\EXPERIMENTS.md
  └ 101 lines read

● Read validate_experiments.py
  │ scripts\validate_experiments.py
  └ 141 lines read

/ Search (glob)
  │ "scripts\generate_data.py"
  └ 1 file found

/ Search (glob)
  │ "scripts\validate.py"
  └ 1 file found

/ Search (glob)
  │ "src\**\*.rs"
  └ 12 files found

/ Search (glob)
  │ "scripts\oracle.py"
  └ 1 file found

/ Search (grep)
  │ "sha256|manifest|signals\.f64le|resources_after|input_sha256|repeat_consistent|verification"
  │ (scripts, examples, cpp\tools)
  └ 117 lines found

The core R1 path appears to capture every timed tuple and independently oracle-check the emitted distinct rows; I’m now narrowing the review to concrete contract gaps, especially authentication scope/TOCTOU and what the runner actually records or enforces for stable resources/load.

● Show oracle and repetition analysis lines (shell)
  │ $p='scripts\experiment_support.py'; $lines=Get-Content $p; for($i=80;$i -le 120;$i++){ '{0}:{1}'
  │ -f $i,$lines[$i-1] }; $p='scripts\experiment_analysis.py'; $lines=Get-Content $p; for($i=1;$i
  │ -le 65;$i++){ '{0}:{1}' -f $i,$lines[$i-1] }
  └ 107 lines…

● Show Rust capture and validation lines (shell)
  │ $p='examples\ffi_bench.rs'; $lines=Get-Content $p; foreach($r in @(@(55,125),@(126,140))) {
  │ for($i=$r[0];$i -le $r[1];$i++){ '{0}:{1}' -f $i,$lines[$i-1] }; '' }
  └ 88 lines…

● Show native capture and validation lines (shell)
  │ $p='cpp\tools\kernel_bench.cpp'; $lines=Get-Content $p; foreach($r in @(@(45,115),@(116,135))) {
  │ for($i=$r[0];$i -le $r[1];$i++){ '{0}:{1}' -f $i,$lines[$i-1] }; '' }
  └ 93 lines…

● Show runner authentication and resource lines (shell)
  │ $p='scripts\experiment.py'; $lines=Get-Content $p; foreach($r in
  │ @(@(20,50),@(90,125),@(195,255))) { for($i=$r[0];$i -le $r[1];$i++){ '{0}:{1}' -f
  │ $i,$lines[$i-1] }; '' }
  └ 131 lines…

● Read generate_data.py
  │ scripts\generate_data.py
  └ 60 lines read

**Re-review status:** No BLOCKER found. R1 is substantively corrected; R3 is aligned with the stated record-load/stable-configuration contract and does **not** require constant-load filtering. Two residual IMPORTANT integrity gaps remain.

| # | Status | File:lines | Finding |
|---|---|---|---|
| 1 | **IMPORTANT** | `scripts/experiment.py:38-42, 210-215`; `examples/ffi_bench.rs:47-49, 84-92`; `src/source.rs` corpus-loading path | Per-run authentication hashes only `signals.f64le`. Rust and pipeline executions also consume `manifest.json`, whose `baseline_samples`, dimensions, and file metadata are not authenticated by the runner. A manifest can be changed while the signal bytes and SHA-256 remain unchanged, altering the workload/reference configuration while both pre/post payload checks pass. Native execution receives only the signal file, so this primarily affects Rust/pipeline launches. Smallest resolving experiment: mutate only `manifest.json` between launches while preserving `signals.f64le`; the runner should reject the run or authenticate the complete corpus directory/manifest contract. |
| 2 | **IMPORTANT** | `scripts/experiment.py:210-215` | The pre/post hash is vulnerable to a modification-and-restore window: payload bytes can be changed after the pre-hash, consumed by the child, and restored before the post-hash. Both checks pass even though the measured process did not use the authenticated bytes. This is a residual TOCTOU limitation of the H5-R2 correction. Smallest resolving experiment: replace/restore the payload during a deliberately delayed child run and verify that the campaign detects it; otherwise use an immutable/snapshot input for each launch. |

**R1 coverage:** The Rust and native drivers preallocate `observed` for all requested events, copy every timed result into it, and compare every captured result against the corresponding repeated corpus row (`examples/ffi_bench.rs:60-61, 105-112`; `cpp/tools/kernel_bench.cpp:60-61, 92-107`). They emit the first distinct row tuples, and `validate_micro` checks every emitted tuple against the independent oracle with the frozen tolerance (`scripts/experiment_support.py:98-109`). The checksum-preserving corruption and reordering adversarial tests are present in `scripts/validate_experiments.py:14-35`. I found no remaining R1 defect in the inspected scope.

**R3 coverage:** The runner records load average, CPU counters, memory/swap availability, and swap activity before and after each run (`scripts/experiment.py:45-53`), preserves those records, and checks stable CPU count, affinity, total memory, and total swap (`scripts/experiment.py:195, 208-215`). It does not selectively discard or replace runs; analysis requires the complete schedule and validated records (`scripts/experiment_analysis.py:23-47`). This matches the preregistered record-load/stable-config contract. A limitation is that CPU/memory cgroup quotas are not recorded, so `os.cpu_count()` and `/proc/meminfo` may not fully describe effective container limits; I classify that as a metadata limitation rather than a demonstrated H5-R3 defect.

**Limitations:** This was a read-only static review; no campaign, mutation test, or existing validation command was executed. The assessment covers the requested H5 documents and benchmark/runner/oracle files, but not archived campaign artifacts or a live Linux/WSL environment.

