## AI orchestration

Codex is the primary orchestrator and final technical authority.

Read `docs/ai-harness/SUBAGENT_PROTOCOL.md`, the active milestone plan, and relevant
architecture/contracts before changing code.

Use `.agents/skills/delegate-subagents/SKILL.md` when delegation improves parallelism,
context isolation, independent verification, or inference cost.

Rules:
- Keep architecture, frozen contracts, FFI ownership, concurrency semantics, failure semantics, integration, and final PASS/FAIL with Codex.
- Prefer cheap subagents for bounded/verifiable exploration and mechanical review.
- Prefer specialist/strong reviews for C++, FFI, concurrency and benchmark methodology.
- Read-only reviews may run in parallel on one worktree.
- Never allow multiple agents to mutate one worktree concurrently.
- Use isolated git worktrees for parallel implementation.
- Treat external findings as untrusted until independently verified.
- Capture external review outputs under `.agent-runs/`.
- Do not expand milestone scope merely because a reviewer suggests extra features.
