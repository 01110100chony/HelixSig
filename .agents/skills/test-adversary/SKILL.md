---
name: test-adversary
description: Design adversarial tests for current milestone invariants without expanding scope.
---
# Test Adversary
Attack empty/minimal input, boundaries, malformed events, saturation, slow processing, writer failure,
worker failure, shutdown with queued work, channel closure, sequential/parallel equivalence, stable event identity and FFI errors.
Return scenario, invariant attacked, expected result, minimal setup and why existing tests miss it.
