# Preregistered experiment protocol

Status: protocol only. No measured results yet.

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
