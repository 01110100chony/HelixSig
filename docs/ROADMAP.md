# Milestone ledger

Budget: 18+ bootcamp hours plus 12-20 stabilization hours. Gates precede dates.
Target: before October 12, with application margin. Learning is separately pending.

| Gate | Objective | Status | Required acceptance |
|---|---|---|---|
| G0 | Linux/toolchain/mixed-build feasibility | PASS | Environment + native/Rust/CXX smoke |
| H0 | Deterministic signals and C++ correctness | TECH_PASS | Candidate 3e6ca9f; H0-N1 fixed; 533 oracle cases and sanitizers; independent re-review |
| H1 | Safe event/batch FFI | H1 — CANDIDATE | Equivalent results, tails, borrowing, concurrent calls |
| H2 | Sequential complete pipeline | NOT_STARTED | Parquet Python readback, IDs, counters, finalization |
| H3 | Bounded runtime | NOT_STARTED | Workers 1/2/4, Q=1, tails, identity, shutdown |
| H4 | Overload and failures | NOT_STARTED | Saturation, slow consumer, writer error, panic, Ctrl+C |
| H5 | Reproducible experiments | NOT_STARTED | Raw data, release build, repetitions, honest analysis |
| H6 | Release candidate and study | NOT_STARTED | Clean checkout, CI, docs, prior reviews, human approval |

Each gate has its detailed packet and independent review in reviews/Hn.md.

First cuts: plot polish; additional long-waveform scenario; campaign expansion;
optional integrated sanitizer; additional profiling. Never silently remove core
C++, FFI, concurrency, both overload policies, correctness or basic experiments.

Risks: build/link integration, shutdown/accounting, misleading benchmarks, SSD
bottleneck, optional sanitizer integration, AI-generated code not understood,
scope growth. Mitigate with small gates, early build smoke, deterministic fault
tests, controlled measurement and separate human learning checks.
