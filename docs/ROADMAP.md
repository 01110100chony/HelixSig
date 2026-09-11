# Milestone ledger

| Gate | Objective | Status | Required acceptance |
|---|---|---|---|
| G0 | Linux/toolchain/mixed-build feasibility | PASS | Environment + native/Rust/CXX smoke |
| H0 | Deterministic signals and C++ correctness | TECH_PASS | Candidate 3e6ca9f; H0-N1 fixed; 533 oracle cases and sanitizers; independent re-review |
| H1 | Safe event/batch FFI | TECH_PASS | Equivalent results, tails, borrowing, concurrent calls |
| H2 | Sequential complete pipeline | TECH_PASS | Candidate cb35222; 4101-event readback per FFI mode, 42 pipeline cases, 16 Rust tests, independent AI review |
| H3 | Bounded runtime | TECH_PASS | Candidate 1f5aa05; 18 Rust tests, 42 H2 cases, 24 concurrent/reference runs, external AI review |
| H4 | Overload and failures | TECH_PASS | Candidate 4211e93; full H4 gate and external AI concurrency review |
| H5 | Reproducible experiments | TECH_PASS | Candidate 1d8724c; 744 runs (620 measured), 96 micro + 28 pipeline configs, 4 AI reviews, docs audit PASS |
| H6 | Release candidate and study | NOT_STARTED | Clean checkout, CI, docs, prior reviews, human approval |

Each gate has its detailed packet and independent review in reviews/Hn.md.

First cuts: plot polish; additional long-waveform scenario; campaign expansion;
optional integrated sanitizer; additional profiling. Never silently remove core
C++, FFI, concurrency, both overload policies, correctness or basic experiments.

Risks: build/link integration, shutdown/accounting, misleading benchmarks, SSD
bottleneck, optional sanitizer integration,
scope growth. Mitigate with small gates, early build smoke, deterministic fault
tests, controlled measurement.
