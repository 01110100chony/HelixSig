# Helix

### Concurrent Scientific Signal Processing in Rust + C++20

Helix is a Linux-first experimental pipeline for studying **bounded concurrent
signal processing, Rust/C++ interoperability, failure semantics and reproducible
performance measurement**.

It combines a Rust orchestration runtime, an independent allocation-free C++20
numerical kernel, synchronous CXX FFI, bounded worker queues, Parquet output and
an independent Python/NumPy validation oracle.

> Built as an academic systems/scientific-computing project, with emphasis on
> correctness, reproducibility and understanding system behavior under concurrency
> and overload — not merely maximizing benchmark numbers.

**Rust · C++20 · CXX · Python/NumPy · Parquet · Linux · CMake · ASan/UBSan**

---

## At a glance

**~509k events/s observed** · **744 validated experiment runs** ·  
**620 measured repetitions** · **Rust + C++20 + Python validation stack**

> The throughput figure is an observation from the reference H5 environment and
> tested configuration. It is not a general performance guarantee.

---

## What makes Helix interesting?

A waveform corpus flows through a bounded concurrent processing pipeline:

```text
Python corpus + NumPy oracle
            │
            ▼
     Rust validation
            │
            ▼
        Producer
            │
       bounded queue
            │
       ┌────┴────┐
       ▼         ▼
   Worker 0   Worker N
       │         │
       └────┬────┘
            │
          CXX FFI
            │
            ▼
   C++20 numerical kernel
            │
            ▼
     bounded results
            │
            ▼
 Rust collector / writer
            │
        ┌───┴────┐
        ▼        ▼
     Parquet    JSON
            │
            ▼
 Python validation / analysis
```

The project explores several systems problems that are easy to hide in simpler
benchmarks:

- bounded backpressure instead of unbounded queues;
- explicit overload behavior (`block` vs `drop-new`);
- Rust/C++ memory ownership across an FFI boundary;
- graceful shutdown and failure propagation;
- deterministic accounting of accepted, dropped, processed and written work;
- independent numerical validation;
- reproducible performance experiments.

---

## Engineering highlights

### Rust/C++ ownership

Rust owns waveform storage, queues, threads, clocks, metrics and output files.

C++ receives borrowed slices synchronously through CXX, retains no Rust pointers,
creates no runtime threads and performs no heap allocation inside the numerical
kernel.

Both per-event and batched FFI modes use the same sample preparation path,
allowing their behavior to be compared without changing the numerical contract.

### Bounded concurrency

Concurrent execution uses:

```text
1 producer
W Rust workers
1 main collector/writer
bounded input queue
bounded result queue
```

Workers process opportunistic batches up to the configured batch size.

The runtime explicitly distinguishes:

```text
produced = accepted + dropped + not_admitted
accepted = processed + failed + aborted
processed = written + unwritten
```

These identities are checked rather than repaired after execution.

### Numerical validation

The C++20 kernel computes:

```text
baseline
peak amplitude
peak position
signed integral
```

A separate NumPy implementation acts as an executable oracle.

Release verification exercises both optimized Release builds and standalone
ASan/UBSan builds against the independent oracle.

### Failure semantics

Helix explicitly handles and tests:

- queue overload;
- SIGINT;
- numerical failure;
- writer failure;
- contained worker panic;
- partial output;
- invalid input/configuration.

Output files are finalized only after successful writer closure. Interrupted or
failed executions cannot silently masquerade as successful runs.

---

## Experimental results

The H5 experiment campaign executed **744 validated runs**, including
**124 warmups** and **620 measured repetitions**, across microbenchmark and
end-to-end pipeline configurations.

In the reference environment, using the batch FFI and block policy at the tested
center configuration, median written throughput was approximately:

| Workers | Median written throughput |
|---:|---:|
| 1 | ~368k events/s |
| 2 | ~507k events/s |
| 4 | ~509k events/s |

Scaling from one to two workers produced a substantial throughput increase,
while four workers showed a plateau in these tested configurations.

Helix deliberately does **not** claim that this plateau identifies a causal
bottleneck. Storage, scheduling, copying and other effects were not independently
isolated.

The overload experiment is similarly explicit about losses: high
accepted-result throughput is never presented as lossless throughput when
`drop-new` rejects events.

See [`docs/EXPERIMENTS.md`](docs/EXPERIMENTS.md) for the full methodology,
measurements and interpretation limits.

---

## Quick start

Ubuntu 24.04 / WSL2 is the primary supported environment.

```bash
python3 -m venv .venv
.venv/bin/python -m pip install -r requirements.txt

.venv/bin/python scripts/generate_data.py \
  --out artifacts/example-corpus

cargo run --release --locked -- run \
  --corpus artifacts/example-corpus \
  --out artifacts/example-run \
  --events 100000 \
  --execution concurrent \
  --workers 2 \
  --batch-size 16 \
  --ffi batch \
  --policy block
```

The run produces structured JSON metadata and validated Parquet output.

For help:

```bash
cargo run --locked -- run --help
```

---

## Release verification

The complete H6 verification gate is:

```bash
HELIX_PYTHON=.venv/bin/python scripts/verify.sh H6
```

The H6 suite covers:

- Rust unit and integration tests;
- native C++ tests;
- independent NumPy oracle validation;
- optimized Release builds;
- standalone ASan/UBSan builds;
- Rustfmt and Clippy;
- concurrency and accounting checks;
- failure and interrupt scenarios;
- experiment-runner regression tests;
- release smoke campaign.

A nonzero exit means the gate failed.

See [`docs/DEVELOPMENT.md`](docs/DEVELOPMENT.md) for the exact Ubuntu setup and
reproduction procedure.

---

## Sequential and concurrent replay

Sequential execution uses the same source, processing and output components
without runtime threads or channels.

Concurrent mode adds:

- one producer;
- a bounded input queue;
- `W` Rust workers;
- a bounded result queue;
- one main collector/writer.

Workers wait for one event and then try to collect up to the configured batch
size without waiting for the batch to fill completely. Actual batch sizes and
parallel output order therefore depend on scheduling.

Use event IDs when comparing parallel results.

Example:

```bash
cargo run --locked -- run \
  --corpus artifacts/example-corpus \
  --out artifacts/example-concurrent \
  --events 4101 \
  --execution concurrent \
  --workers 2 \
  --batch-size 16 \
  --ffi batch \
  --policy block
```

---

## Backpressure and overload

Helix supports two input admission policies.

### `block`

The producer waits for queue capacity while periodically observing stop
requests.

This mode applies backpressure and does not intentionally drop events.

### `drop-new`

The producer attempts admission once.

If the input queue is full, the event is rejected and counted as dropped.

Only the input queue drops events. The result queue instead applies
backpressure to workers.

---

## Shutdown and output integrity

SIGINT stops new admission and drains already accepted work.

Worker panic requests a fatal stop while preserving accounting for:

- already delivered outcomes;
- in-flight owned work;
- queued accepted work.

Writer failure stops new admission while the collector continues draining worker
results so blocked worker sends cannot deadlock shutdown.

Each run starts with:

```text
running.json
```

Successful output becomes:

```text
events.parquet
summary.json
```

Interrupted or valid partial runs may produce:

```text
events.partial.parquet
```

Writer failure may leave:

```text
events.parquet.incomplete
```

Incomplete artifacts are not considered valid finalized output.

---

## Exit codes

| Code | Meaning |
|---:|---|
| `0` | completed |
| `1` | failed |
| `2` | completed with explicit drops |
| `64` | invalid configuration/input |
| `130` | interrupted |

Code `2` describes a completed replay with explicit event loss and must not be
interpreted as lossless success.

---

## Measurements and metrics

The versioned summary records:

- effective runtime configuration;
- corpus hash;
- final output state;
- failure reason;
- bounded diagnostic examples;
- data-buffer estimate;
- duration;
- processed throughput;
- written throughput;
- worker metrics;
- actual batch-size distribution;
- latency histograms.

Latency measures the interval from admission attempt to native result
availability.

It includes queueing and processing preparation, but excludes output writing.

Raw successful-event latency is also preserved in Parquet.

---

## Reproducible experiments

H5 is the measured performance milestone.

The full campaign performs:

- **744 validated runs**;
- **96 microbenchmark configurations**;
- **28 pipeline configurations**;
- one warmup and five measured repetitions per configuration.

The standard performance workflow is:

```bash
.venv/bin/python scripts/experiment.py \
  --out artifacts/h5-campaign

.venv/bin/python scripts/experiment_analysis.py \
  artifacts/h5-campaign
```

The normal H6 release gate does **not** rerun the full H5 measurement campaign.

Its smoke campaign is correctness/release evidence, not new performance evidence.

---

## Project scope

Helix is an **academic experimental system**, not a production DAQ system,
physical detector simulation or hard-real-time platform.

It intentionally focuses on:

- concurrent event processing;
- bounded queues and backpressure;
- Rust/C++ interoperability;
- numerical correctness;
- output integrity;
- failure semantics;
- reproducible measurement.

The current scope excludes:

- acquisition hardware;
- networking;
- ROOT integration;
- GPU processing;
- distributed execution;
- hard-real-time guarantees;
- hostile same-user file replacement;
- SIGKILL/OOM recovery;
- native fatal-fault recovery;
- stuck-I/O recovery;
- crash recovery;
- power-loss durability.

The measured results characterize the tested environment and experimental design.
They do not establish universal scaling laws or prove a causal bottleneck.

---

## Technical documentation

| Document | Purpose |
|---|---|
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | Architecture and frozen contracts |
| [`docs/H6_STUDY.md`](docs/H6_STUDY.md) | Detailed technical study |
| [`docs/EXPERIMENTS.md`](docs/EXPERIMENTS.md) | Experiment protocol, results and limitations |
| [`docs/DEVELOPMENT.md`](docs/DEVELOPMENT.md) | Build and reproduction environment |
| [`docs/ROADMAP.md`](docs/ROADMAP.md) | Project milestones |
| [`docs/STUDY_GUIDE.pt-BR.md`](docs/STUDY_GUIDE.pt-BR.md) | Portuguese human-learning guide |
| [`docs/reviews/H6.md`](docs/reviews/H6.md) | H6 review and adjudication |
| [`docs/reviews/H6_PACKET.md`](docs/reviews/H6_PACKET.md) | H6 work packet |
| [`docs/reviews/H6_HUMAN_PACKET.md`](docs/reviews/H6_HUMAN_PACKET.md) | Human-review packet |

---

## Verification status

**G0 PASS · H0–H6 TECH_PASS**

H6 release candidate:

```text
3d11ab82d2276897d70d1a0faa3413f3fb88f9fc
```

The technical verdict establishes the verified state of the candidate under the
documented environment and scope.

It does **not** independently imply:

- production readiness;
- hard-real-time behavior;
- general performance guarantees;
- human-learning completion;
- release approval.

---

## Environment

Primary development and verification environment:

- Ubuntu 24.04 LTS under WSL2;
- Rust 1.98.1;
- C++20;
- GCC / Clang;
- CMake;
- Python 3.12;
- NumPy;
- PyArrow.

Source, builds and measured artifacts are intended to live on the Linux
filesystem. Windows is used as an editing/interface environment rather than as a
native supported runtime target.

---

## Motivation

Helix was built as an academic portfolio project in **systems engineering and
scientific computing**.

The goal is not to imitate a production detector pipeline, but to study and make
visible the engineering problems that appear around scientific data processing:

- ownership across language boundaries;
- bounded memory;
- concurrency;
- overload;
- numerical reproducibility;
- failure handling;
- output finalization;
- experimental methodology.

The repository emphasizes evidence and explicit limitations rather than hiding
system behavior behind a single throughput number.
