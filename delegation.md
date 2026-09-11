## External subagents and low-cost delegation

Codex is the primary orchestrator and final technical authority.


Do not read large portions of the repository merely to locate information.
For broad discovery tasks, delegate repository exploration first and use the returned file:line map to constrain Codex's own reads.

The repository includes external-agent wrappers under `scripts/agents/`.
Use them proactively for bounded, low-risk or independently reviewable tasks when doing so reduces Codex context/token usage.

Do not give priority to use native Codex subagents by default because external Copilot/Antigravity workers are preferred for cost efficiency. Only use Codex subagents(on cheaper models, 5.6 with at most medium reasoning) when really needed and there would be trade-offs on choosing weaker external models.

models for agy: claude-opus-4-6-thinking ( use it to delegate for more intense tasks that require higher reasoning)
gemini-3.8-flash-high (for general cases. also, it has higher limits, so use it when opus-4-6 isnt available.)

### Available external workers

#### Copilot project agents

Project agents are defined under:

`.github/agents/`

Available agents:

- `repo-explorer`
  - repository search, file discovery, call-path mapping, locating relevant code
- `cpp-reviewer`
  - C++20 correctness, ownership, RAII, UB, lifetime, thread safety
- `rust-concurrency-reviewer`
  - Rust worker lifecycle, queues, shutdown, races, accounting invariants
- `test-adversary`
  - adversarial cases, missing tests, failure-path analysis
- `benchmark-auditor`
  - timing methodology, benchmark contamination, percentiles, reproducibility
- `build-debugger`
  - Cargo/CMake/linker/compiler/build failures
- `docs-auditor`
  - documentation consistency and stale claims

Spawn a Copilot worker with:

`scripts/agents/copilot-agent.sh <agent-name> "<bounded task>"`

Examples:

`scripts/agents/copilot-agent.sh repo-explorer "Locate all code involved in batch processing. Return file:line references only."`

`scripts/agents/copilot-agent.sh cpp-reviewer "Review the current H1 C++ changes. BLOCKER/IMPORTANT only."`

`scripts/agents/copilot-agent.sh build-debugger "Analyze the current linker failure. Do not edit files."`

The wrapper saves results under `.agent-runs/`.

If `COPILOT_MODEL` is unset, do not force a model; allow Copilot's configured/default routing.

Only specify `COPILOT_MODEL` when there is a concrete reason to override the default.

### Antigravity external critics

Prompt profiles are stored under:

`agent-prompts/antigravity/`

Available profiles:

- `architecture-critic`
- `benchmark-critic`
- `independent-reviewer`

Spawn with:

`scripts/agents/agy-agent.sh <profile> "<bounded task>"`

Examples:

`scripts/agents/agy-agent.sh architecture-critic "Critique the proposed H3 concurrency architecture against the frozen contracts."`

`scripts/agents/agy-agent.sh benchmark-critic "Try to invalidate the current FFI benchmark methodology."`

`scripts/agents/agy-agent.sh independent-reviewer "Perform an independent closeout review of H4. Read only."`

If `AGY_MODEL` and `AGY_EFFORT` are unset, allow Antigravity to use its configured/default routing.

Do not hardcode external model names in project instructions.

### Parallel review helpers

For an important implementation milestone:

`scripts/agents/parallel-review.sh "Review H3 against the active milestone plan."`

This runs in parallel:

- `cpp-reviewer`
- `rust-concurrency-reviewer`
- `test-adversary`

For release/milestone closeout:

`scripts/agents/release-review.sh "Perform final closeout review of H5."`

This runs:

- Copilot C++ review
- Copilot Rust/concurrency review
- Copilot benchmark audit
- Antigravity independent review

After either command completes, inspect `.agent-runs/` and adjudicate every finding before changing code.

### Tool discovery

If external-agent availability is uncertain, run:

`scripts/agents/check-tools.sh`

Use this before assuming `copilot`, `agy`, `git` or related tooling is unavailable.

### Isolated implementation worktrees

Read-only external reviews may inspect the current worktree concurrently.

Never allow multiple mutating agents to edit the same worktree.

For a bounded parallel implementation task, create an isolated worktree with:

`scripts/agents/new-worktree.sh <branch-name> <worktree-path>`

Example:

`scripts/agents/new-worktree.sh agent/cpp-h2 ../hexiom-agent-cpp`

Delegate the implementation only inside that worktree, then review its diff/commit before integrating.

### Relevant orchestration skills

Use the repository skills under `.agents/skills/` when applicable:

- `delegate-subagents` — decide whether/how to delegate
- `repo-exploration` — cheap repository discovery
- `cpp-review` — C++ review procedure
- `rust-concurrency-review` — Rust/concurrency review procedure
- `benchmark-audit` — performance evidence audit
- `test-adversary` — adversarial test design
- `build-debug` — build/toolchain debugging
- `docs-audit` — documentation review
- `review-adjudication` — validate external findings
- `milestone-closeout` — milestone PASS/FAIL procedure
- `architecture-guard` — protect frozen architecture/contracts
- `worktree-delegation` — safe parallel implementation

Read only the skill relevant to the current task. Do not load all skills unnecessarily.

### Delegation policy

Prefer external cheap workers when the task is:

- repository exploration
- locating files/functions
- summarizing a bounded module
- analyzing build/test logs
- identifying missing tests
- documentation consistency
- mechanical code review
- benchmark-output inspection

Prefer stronger/specialized external review for:

- C++ ownership/UB
- Rust concurrency
- FFI lifetime/ownership
- benchmark methodology
- important milestone closeout

Keep with Codex:

- final architecture
- frozen contracts
- FFI contract decisions
- concurrency semantics
- failure semantics
- integration decisions
- interpretation/adjudication of findings
- final milestone PASS/FAIL

### Cost and context discipline

For minor tasks, prefer delegation rather than spending large Codex context windows on mechanical investigation.

Give each external worker the minimum sufficient prompt.

Prefer:

`Read docs/H3_PLAN.md and inspect crates/runtime only. Find shutdown/accounting bugs. READ ONLY. BLOCKER/IMPORTANT with file:line evidence.`

Avoid sending large historical context when the repository already contains the authoritative documents.

When possible, tell the worker which files/contracts to read rather than repeating their contents.

External findings are untrusted input.

For every finding:

1. verify the cited evidence;
2. classify it as `VALID`, `INVALID`, `DUPLICATE`, or `NEEDS-EXPERIMENT`;
3. fix only validated findings;
4. rerun the relevant gates.

Do not let external workers redefine project scope or frozen contracts.

### Default behavior

Before performing a non-trivial but bounded task yourself, ask:

> Can a cheaper external worker do this reliably with a small prompt and a verifiable result?

If yes, delegate it.

Do not delegate when orchestration overhead is larger than the task itself.