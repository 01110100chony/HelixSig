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
