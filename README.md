# Helix

Bounded concurrent signal processing. Rust runtime, C++20 numerical kernel, Python oracle.

![milestone: H5 ✓](https://img.shields.io/badge/milestone-H5_%E2%9C%93-blue)
![platform: Linux / WSL2](https://img.shields.io/badge/platform-Linux_%2F_WSL2-lightgrey)
![license: MIT](https://img.shields.io/badge/license-MIT-green)

## What and why

Helix is an academic systems-engineering experiment. It replays synthetic
waveform events through a mixed-language pipeline, Rust for the runtime and
I/O, C++20 for the numerical kernel, Python/NumPy as an independent oracle
and measures throughput, latency and event loss under bounded concurrency.

The goal is to learn and apply skills that matter for scientific
computing infrastructure: safe FFI ownership, bounded-queue backpressure,
reproducible measurement, and honest reporting of what the data does and
doesn't show.

## Architecture

```
Python corpus + independent oracle
               │
Rust load/validate → producer → bounded queue → W Rust workers
                                                      │
                                              CXX bridge → C++20 kernel
                                                      │
Rust collector/writer ← bounded result queue ←────────┘
        │
  Parquet + JSON → Python validation & analysis
```

Sequential mode reuses the same components without channels or threads.
C++ owns no threads, mutable globals, files or runtime timestamps.
Full contracts and rationale in [ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Skills demonstrated

| Area | What Helix exercises | Where |
|---|---|---|
| **Rust / C++ interop** | Safe `cxx` bridge, Rust-owned buffers, no shared mutable state | [`bridge.rs`](src/bridge.rs), [`cpp/`](cpp/) |
| **Bounded concurrency** | Producer–worker–collector with crossbeam channels, backpressure, graceful shutdown | [`concurrent.rs`](src/concurrent.rs), [`runtime.rs`](src/runtime.rs) |
| **Numerical correctness** | Blocked pairwise reduction matching a pinned NumPy oracle within documented tolerance | [`cpp/src/`](cpp/src/), [`oracle.py`](scripts/oracle.py), [`numerical.rs`](tests/numerical.rs) |
| **Overload handling** | Block and drop-new policies, SIGINT drain, worker panic classification | [`runtime.rs`](src/runtime.rs), [`validate_failures.py`](scripts/validate_failures.py) |
| **Reproducible experiments** | 744-run preregistered campaign, provenance tracking, no selective discarding | [`experiment.py`](scripts/experiment.py), [EXPERIMENTS.md](docs/EXPERIMENTS.md) |
| **Structured output** | Parquet event records, JSON summaries, bounded HDR latency histograms | [`output.rs`](src/output.rs), [`metrics.rs`](src/metrics.rs) |
| **Build integration** | CMake C++20 kernel + Cargo `build.rs` + CXX codegen, single `cargo build` | [`build.rs`](build.rs), [`CMakeLists.txt`](cpp/CMakeLists.txt) |

## Milestones

| Gate | Focus | Status |
|---|---|---|
| G0 | Toolchain and mixed-language build | ✓ PASS |
| H0 | Deterministic signals, C++ correctness, 533 oracle cases | ✓ PASS |
| H1 | Safe FFI — event and batch modes, borrowing, concurrent calls | ✓ PASS |
| H2 | Sequential pipeline — corpus through Parquet output, 42 integration cases | ✓ PASS |
| H3 | Bounded concurrent runtime — workers, queues, shutdown | ✓ PASS |
| H4 | Overload and failure paths — drops, panics, SIGINT, writer errors | ✓ PASS |
| H5 | Reproducible experiments — 744 runs, 4 independent AI reviews | ✓ PASS |
| H6 | Release candidate — clean checkout, CI, study guide | Planned |

Each milestone has a detailed gate review in [`docs/reviews/`](docs/reviews/).

## Quick start

Ubuntu 24.04 / WSL2 with Rust stable, GCC/G++, CMake and Python 3:

```bash
python3 -m venv .venv && .venv/bin/python -m pip install -r requirements.txt
.venv/bin/python scripts/generate_data.py --out artifacts/corpus
cargo run --locked -- run --corpus artifacts/corpus --out artifacts/run --events 4101
HELIX_PYTHON=.venv/bin/python scripts/verify.sh H2
```

For concurrent mode, experiment campaigns and full CLI reference, see
[DEVELOPMENT.md](docs/DEVELOPMENT.md) and [EXPERIMENTS.md](docs/EXPERIMENTS.md).

## Project structure

```
src/           Rust runtime — config, source, processing, bridge, concurrent, output
cpp/           Independent C++20 signal-processing kernel + CXX adapter
scripts/       Data generation, validation, experiment runner, analysis
tests/         Rust integration tests
docs/          Architecture, roadmap, experiments, gate reviews, study guide (pt-BR)
```

## Documentation

| Document | Contents |
|---|---|
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | Frozen contracts, data flow, numerical spec, runtime semantics |
| [DEVELOPMENT.md](docs/DEVELOPMENT.md) | AI-driven development governance, gate protocol, change rules |
| [ROADMAP.md](docs/ROADMAP.md) | Milestone ledger with acceptance criteria |
| [EXPERIMENTS.md](docs/EXPERIMENTS.md) | Preregistered protocol, results, interpretation limits |
| [Study guide (pt-BR)](docs/STUDY_GUIDE.pt-BR.md) | Interview-preparation checkpoints — in Portuguese |

## Scope and honesty

This is academic work, not production software. Code review was performed by
AI agents in separate contexts and is identified as such throughout. No
hardware, network, GPU, web UI or crash-recovery features are included.

Primary environment is Ubuntu 24.04 under WSL2. Source, builds and measured
data belong on the Linux filesystem; Windows is an editing interface only.

There is no promised speedup, no minimum throughput target, and the experiment
results characterize one specific environment. Claims are limited to what the
data supports.

## License

[MIT](LICENSE)
