---
name: architecture-guard
description: Prevent AI-driven implementation from drifting beyond frozen architecture and milestone scope.
---
# Architecture Guard
Before non-trivial work read architecture, contracts and active milestone; identify interfaces that must not change.
Prefer the smallest design satisfying acceptance criteria. Reject speculative abstractions and technology additions without requirements.
If an architecture change is necessary, stop dependent implementation and record current contract, blocker, minimum change, alternatives and migration impact. Final decision stays with Codex.
