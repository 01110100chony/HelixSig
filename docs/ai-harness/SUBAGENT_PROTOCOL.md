# Subagent Protocol

## Authority model
Codex owns architecture, frozen contracts, interfaces, FFI ownership/lifetime,
concurrency semantics, failure semantics, milestone scope, integration, acceptance
criteria and final PASS/FAIL.

Subagents are workers and critics, not co-architects.

## Delegate when
- work is separable;
- clean context improves review quality;
- a cheaper model can complete a bounded/verifiable task;
- independent verification is materially useful;
- noisy output would pollute the main context;
- parallel read-only review saves time.

Do not delegate tiny work merely because a worker exists.

## Minimal prompt contract
State:
1. authoritative files to read;
2. exact scope;
3. read-only or mutating;
4. expected output;
5. severity/output format if reviewing.

Example:

> Read AGENTS.md, docs/CONTRACTS.md and the active H2 plan. READ ONLY. Review only the Rust/C++ FFI boundary for ownership, lifetime, thread safety, error propagation and unnecessary copies. Return BLOCKER/IMPORTANT with file:line evidence. Do not expand scope.

## Finding taxonomy
- BLOCKER — invalidates correctness, safety, contract or milestone acceptance.
- IMPORTANT — substantial defect/risk before closeout.
- MINOR — useful non-blocking issue.
- NOTE — observation/question/optional improvement.

## Adjudication
Codex classifies each finding:
- VALID
- INVALID
- DUPLICATE
- NEEDS-EXPERIMENT

Only VALID findings are remediated automatically.

## Mutation policy
Read-only reviewers may share a worktree and run concurrently.
Mutating workers need a bounded task and isolated worktree/branch when parallel.
They must not redefine contracts and must return a diff/commit for Codex review.

## Context hygiene
Point agents to repository sources of truth. Do not paste full project history.
Prefer concise output and file:line evidence.

## Cost routing
Cheap: exploration, call graph, log triage, docs audit, test ideas, repetitive checks.
Strong: concurrency, FFI ownership, C++ UB, performance interpretation, architecture critique.

## Closeout
1. implementation;
2. local gates;
3. parallel specialist reviews;
4. Codex adjudication;
5. valid fixes;
6. gates rerun;
7. external model-diverse review if warranted;
8. PASS/FAIL.
