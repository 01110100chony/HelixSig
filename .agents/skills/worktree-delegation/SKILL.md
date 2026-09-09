---
name: worktree-delegation
description: Safely delegate mutating tasks to isolated git worktrees.
---
# Worktree Delegation
Use only for independent mutating tasks. One worker per worktree, one bounded task per worker,
no contract changes without Codex approval, and require a clean diff/commit for review.

Create with:
`scripts/agents/new-worktree.sh agent/<task> ../<repo>-agent-<task>`

Worker prompt must state milestone, allowed files/directories, frozen contracts, required tests and non-goals.
Inspect diff and rerun appropriate review/gates before integration.
