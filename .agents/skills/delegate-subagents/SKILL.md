---
name: delegate-subagents
description: Delegate bounded development subtasks while preserving Codex architectural authority.
---
# Delegate Subagents
Use when delegation improves cost, context isolation, parallelism or independent verification.

Codex retains final authority over architecture, contracts, FFI ownership/lifetime,
concurrency/failure semantics, scope, integration and PASS/FAIL.

Prefer cheap workers for exploration, code location, call graphs, logs, simple tests,
docs and mechanical checks. Prefer strong/specialist reviewers for C++, FFI,
concurrency, benchmarks and difficult debugging.

External commands:
- `scripts/agents/copilot-agent.sh <agent> "<task>"`
- `scripts/agents/agy-agent.sh <profile> "<task>"`

Keep prompts minimal: authoritative files, exact scope, read-only/mutating and expected output.
Read-only reviews may run in parallel. Capture outputs in `.agent-runs/` and then use
`review-adjudication`.

Never allow parallel mutation in one worktree. Use `worktree-delegation`.
Do not spawn agents for trivial work, accept findings automatically, let workers redefine
architecture, or create recursive swarms without a concrete need.
