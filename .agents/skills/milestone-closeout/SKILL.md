---
name: milestone-closeout
description: Close a milestone with gates, specialist reviews, adjudication and PASS/FAIL.
---
# Milestone Closeout
Preconditions: documented scope, explicit acceptance criteria, identifiable frozen contracts.

Sequence: inspect implementation -> local gates -> specialist reviews -> adjudicate -> fix valid findings -> rerun gates -> external model-diverse review for release-critical changes -> PASS/FAIL from evidence.

Routing:
- C++ -> cpp-reviewer
- Rust queues/workers/shutdown -> rust-concurrency-reviewer
- benchmarks -> benchmark-auditor
- non-trivial behavior -> test-adversary
- build/linker -> build-debugger
- docs/evidence -> docs-auditor

PASS requires all acceptance criteria supported and no unresolved BLOCKER/IMPORTANT invalidating the milestone.
