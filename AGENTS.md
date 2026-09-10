# Helix development contract

This is an academic systems-engineering experiment, not a commercial product.
User instructions take precedence. Read docs/ARCHITECTURE.md, DEVELOPMENT.md,
ROADMAP.md and the active milestone review before changing code.

- Work sequentially through G0 and H0-H6. Do not claim a gate passed without evidence.
- Only one agent writes production code at a time. Freeze the candidate during review.
- Use a separate-context AI reviewer at each gate; identify this as AI review.
- Preserve numerical tolerance, safety checks, bounded memory, accounting and tests.
- Do not add networking, hardware, GPU, web UI, recovery, async runtime or C++ threads.
- Do not change frozen contracts or remove mandatory scope without user approval.
- Optional integrated sanitizers are timeboxed; standalone ASan/UBSan are mandatory.
- Never fabricate benchmark results, human learning approval or publication approval.
- Code and public documentation are English; the personal study guide is Portuguese.

## Subagent orchestration

Act as the primary technical orchestrator. Your goal is not to waste tokens on minor tasks with minor risks.

When a task is non-trivial, evaluate whether independent delegation would materially improve:
- parallelism,
- context isolation,
- independent verification,
- specialist review,
- or inference cost.

If yes, use subagents proactively without waiting for the user to request them.

Use MAINLY external workers through the repository wrappers when model diversity is valuable:
- Copilot CLI for cheap/specialized repository workers;
- Antigravity for independent external review.
To understand how to use them , read: delegation.md . all instructions with scripts are there.

Prefer native Codex subagents ( when a real cheap model can be used for that. dont waste tokens on high frontier models for explaning.) for:
- repository exploration 
- bounded implementation tasks,
- test/log analysis,
- parallel code inspection.

In copilot just use auto model. In AGY, prefer 3.8 flash high.

Before delegating, give each worker:
1. one bounded objective;
2. authoritative files to read;
3. explicit read-only or mutating scope;
4. expected output format.

Do not delegate final decisions about:
- architecture,
- frozen contracts,
- FFI ownership,
- concurrency semantics,
- failure semantics,
- milestone PASS/FAIL.
- anything risky which could result in scope creep.

For important milestones, proactively obtain at least one independent review before declaring PASS.

Never let multiple mutating agents edit the same working tree concurrently.
