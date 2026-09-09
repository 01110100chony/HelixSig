---
name: rust-concurrency-review
description: Review Rust runtimes for races, deadlocks, shutdown and accounting correctness.
---
# Rust Concurrency Review
Focus on worker lifecycle, bounded queue semantics, block/drop behavior, sender/receiver ownership,
shutdown/cancellation/join ordering, deadlock/livelock, panic propagation, disconnect behavior,
accounting invariants, ordering assumptions, concurrent FFI calls, Send/Sync assumptions and shared synchronization.
Prefer concrete bad interleavings. Do not propose lock-free structures without a demonstrated need.
