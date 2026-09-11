# Preregistered experiment protocol

Status: full H5 campaign measured and validated on 2026-09-10 at clean candidate
`1d8724c9521e7d88a43e6f751c4681e1fbbf75c6`.

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

The complete campaign ran on a Ryzen 5 3400G under Ubuntu 24.04 / WSL2,
Linux 6.18.33.2, with 8 logical CPUs, approximately 7.7 GiB RAM and 2 GiB swap.
Those effective limits remained stable; no swap-out activity was observed.
These are the actual resources, not the earlier planned four-CPU allocation.
All 744 runs validated: 124 warmups and 620 measurements. There were 660 exit-0
runs and 84 exit-2 runs, with losses retained. No runs were selectively discarded.
Campaign artifacts occupied 830648366 bytes (about 0.77 GiB), below 4 GiB.

For the central N=256, Q=256, B=16, batch-FFI, block-policy case, median written
throughput with W=1,2,4 was respectively 368418, 507085 and 509060 events/s.
The W=2 and W=4 medians were close relative to their between-run spread; this
single-axis observation does not establish a general scaling law. At the W=2
center, drop-new batch-FFI had median loss fraction 0.89246 and median written
throughput 402177 events/s. That outcome must not be presented as lossless
throughput.

At N=256/B=16 with prepared buffers, Rust event/batch medians were 423.4/414.1
ns per event, with overlapping interquartile intervals [421.6,435.5] and
[409.9,448.2]. Native event/batch medians were 437.6/405.4 ns. The instrumentation
and different executables prevent interpreting these differences as exact FFI
overhead or a universal batching speedup. No optimization was made from these data.

The [full analysis](evidence/H5/analysis.md) and [machine-readable results](evidence/H5/analysis.json)
preserve every configuration and its dispersion. [Provenance](evidence/H5/campaign.json),
[verification](evidence/H5/verification.log), [aggregate counts](evidence/H5/summary.json)
and the [raw-file/archive SHA-256 index](evidence/H5/raw-index.json) are retained.
Peak child-process RSS was 35120 KiB; no successful-event histogram overflow or
latency population below 1000 occurred in this campaign. These observations do
not bound future latency or RSS.

Full original artifacts: `/root/helix-h5-a346652/artifacts/h5-campaign` in WSL.
Portable raw data, logs and measured executables: `artifacts/h5-1d8724c-raw.tar.gz`
in the Windows editing checkout (106993609 bytes). Every archived file was read
back and verified against the index. Rebuild-only intermediate objects remain in
the original Linux directory; source and locked dependencies are identified by
the candidate commit. Publication and the H6 clean-checkout release gate remain
separate. Optional extra N=4096 pipeline runs and plot polish were not performed.

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

The runner authenticates payload and manifest SHA-256 before and after every child,
including the native reference, and retains both hashes in its run record.
It owns the generated corpus directory for the campaign. Concurrent hostile
replacement of local files is outside this experiment's trusted-input scope;
pre/post hashing does not claim filesystem snapshot isolation.
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
