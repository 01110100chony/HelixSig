# Preregistered experiment protocol

Status: H5 tooling implemented; campaign validation and measurement pending.

## Questions

How do W, Q, B and FFI granularity change throughput, latency and event loss?
No expected speedup, hard-real-time claim or minimum performance target.

## FFI microbenchmark

N in {64,256,4096}, B in {1,4,16,64}. Compare event/batch calls on identical
prepared buffers; additionally measure pack+call and direct native C++ reference.
Consume results observably. Same release settings, no LTO or fast-math.
One warmup, five measurements. Report per-event cost and dispersion, not a claim
that differences between executables are exact isolated FFI overhead.

## Pipeline

Center N=256,W=2,Q=256,B=16. Union of single-axis W={1,2,4}, Q={64,256,1024},
B={1,16,64}; both policies and FFI modes; remove duplicates, not full factorial.
Optional additional central N=4096 scenario. 100000 events/run, one warmup,
five measured repetitions, seeded randomized configuration order. Budget 4 GiB.

Record commit and clean-tree state, corpus hash, hardware, OS/kernel/WSL, tool
versions, build flags, effective resources and config. Preserve raw outcomes,
latencies, batch sizes, timings, peak Linux process RSS and exit codes. Report
medians/dispersion across runs; never average percentiles. Code 2 preserves drops.
Mark small latency populations and histogram overflow; empty percentiles are null.

WSL2 Ryzen 5 3400G results characterize that environment only. Source/build/data
must be on the Linux filesystem. Planned resources: 4 CPUs, 8 GiB RAM, 2 GiB swap;
actual configuration must be recorded and stable across a campaign. Record swap
and competing load. The SSD/writer can limit the full pipeline; microbenchmark
does not include output. Do not infer interactions not covered by this design.

## Results

NOT_RUN. Raw files, numerical conclusions and performance claims must only be
added after H4 passes and a clean candidate has been measured.

## Executable protocol

`scripts/experiment.py` builds matched GCC Release C++ kernels and the Rust
driver with optimization level 3, LTO disabled and no fast-math. It saves verbose
Cargo logs, CMake compile commands, tool versions, exact command lines and binary
hashes. Corpus loading, allocation and printing are outside microbenchmark
timing; result checksum consumption, batch counters and a copy of every result
into a preallocated verification buffer are inside both drivers. This common
instrumentation is part of the reported per-event cost. After timing, the drivers
check all repeated-row results for exact consistency and emit every distinct
row's full result tuple. Python checks each tuple against the independent oracle
with the frozen tolerance; the checksum alone is not numerical acceptance.
`prepared` borrows a contiguous preloaded corpus segment; `pack` copies owned
rows into a reusable flat buffer before the same event/batch calls. All 256
corpus rows are replayed cyclically, including a final partial batch. The native
reference includes both call granularities and preparation modes.

The full schedule contains 96 microbenchmark configurations and 28 pipeline
configurations, one warmup round followed by five measurement rounds. Each round
shuffles all configurations with a recorded seed (default 20260910). Both families
process 100000 events per run. The separate smoke mode uses 257 events, all 96
microbenchmark configurations and four central pipeline configurations, one warmup
and one measured round. Its results are labelled `smoke_only` and cannot support
performance claims. Python validates output after each timed child terminates.

The runner authenticates each payload SHA-256 before and after every child,
including the native reference, and retains the input hash in its run record.
It checks the clean commit, Linux filesystem placement, available CPUs,
stable CPU affinity/memory/swap limits, binary hashes and artifact budget. It
records Linux load, CPU counters and swap activity before/after each run, plus
process lists at campaign boundaries. Actual resources supersede the planned
configuration in the report; the runner does not alter WSL settings. Windows
host activity is not fully observable from this Linux metadata. Resource-limit
stability does not mean constant runtime load. No post hoc load threshold filters
or selectively replaces measured repetitions; observed load/swap remain visible.

GNU time records each child's peak Linux process RSS; wall duration separately
includes process startup. Pipeline throughput uses the runtime's documented
duration including finalization. Microbenchmark cost uses internal elapsed
nanoseconds divided by actual processed events. Raw per-run summaries retain
actual batch sizes and histogram coverage. Parquet retains every successful
event's latency. Empirical per-run percentiles are stored separately from the
bounded HDR estimates, with null empty populations and a flag below 1000 samples.
The analysis reports medians, quartiles and ranges of run-level costs/throughputs;
it keeps latency percentiles per run instead of averaging them. Failures stop a
campaign with the failed record preserved; analysis rejects incomplete schedules.

Commands (new output directory, clean Linux checkout for a full campaign):

```bash
HELIX_PYTHON=.venv/bin/python scripts/verify.sh H5
.venv/bin/python scripts/experiment.py --out artifacts/h5-campaign
.venv/bin/python scripts/experiment_analysis.py artifacts/h5-campaign
```

Raw evidence lives under the chosen output directory. `campaign.json` records
provenance and schedule, `runs.jsonl` records every warmup/measurement and exit
code, and `analysis.json`/`analysis.md` contain derived results. Retain the entire
directory for reproducibility. No networking, publication or global environment
reconfiguration is part of this command.
