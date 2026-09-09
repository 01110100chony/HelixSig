# Helix

A Linux-first laboratory for bounded concurrent signal processing in Rust with an
independent C++20 numerical library and a Python/NumPy oracle.

**Status: bootstrap in progress. No milestone or performance claim is approved yet.**

The experiment studies worker count, queue capacity, processing batch size and
per-event versus batched FFI. It is finite synthetic replay, not a real-time DAQ
system or physical simulation. There is no required speedup.

See [architecture](docs/ARCHITECTURE.md), [development governance](docs/DEVELOPMENT.md),
[milestones](docs/ROADMAP.md), [experiments](docs/EXPERIMENTS.md), and the
[Portuguese study guide](docs/STUDY_GUIDE.pt-BR.md).

Primary environment: Ubuntu 24.04 in WSL2. Source, builds and measured data belong
on the Linux filesystem. Windows is an editing interface, not a supported target.
