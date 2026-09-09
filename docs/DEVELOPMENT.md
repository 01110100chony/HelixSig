# AI-driven development governance

## Authority and workflow

The user owns scope, budget, learning verification and publication. The leader
owns contracts, work packets and technical gates. One implementer changes code;
a separate-context reviewer examines a frozen candidate without editing code.
Use Astra when available. Call it AI review, never independent human review.

Work packet -> implementation -> relevant checks -> candidate commit -> review
-> corrections/re-review -> TECH_PASS -> integration -> next milestone.

Each packet defines objective, contracts, allowed edits, exclusions, required
tests and exact acceptance command. Record it in docs/reviews/Hn.md. Code changes
after review invalidate affected evidence. Two inconclusive rounds trigger a
minimal reproducer and leader decision, not an indefinite agent loop.

## Gates

G0: verify WSL2, Windows compatibility, virtualization, at least approximately
10 GiB available, tools and a minimal mixed-language build. Do not silently
substitute a Windows implementation if this fails.

Every technical gate requires mandatory behavior, relevant regression checks,
separate-context review, no blocking findings, accurate docs and evidence tied
to the candidate. Missing evidence is NOT_RUN, not PASS.

Blocking/high: UB, wrong numbers, deadlock, silent loss, false evidence or absent
requirements: no advance. Moderate issues that affect reproducibility/acceptance
also block. Optional style improvements may be deferred explicitly.

Human learning is separate and can remain pending during implementation. No AI
can mark user learning or public-release approval as passed on the user's behalf.

## Change rules

Proceed automatically with in-contract fixes, relevant tests/docs and integration
after technical approval. User decision is needed for mandatory scope cuts,
incompatible frozen-contract changes, budget expansion, purchases and publication.
Do not weaken tests, loosen tolerances or remove mandatory sanitizer checks.
Record contract-change rationale and invalidate affected gates. New dependencies
must have a concrete purpose. Optimization requires before/after measurements.

Scripts define commands; architecture defines contracts; review documents define
evidence. Full logs live in ignored artifacts and durable release evidence later.
Performance evidence requires an identified commit and clean tracked tree.

## Review template

Each H0-H6 review contains Scope and frozen contracts; Implementation handoff
(candidate code commit, changes, limitations); Acceptance matrix (requirement,
evidence, result); Verification (command, environment, exit code, reference);
Findings (ID, severity, location, reproduction, correction, status); Re-review
history; Technical verdict; Learning checkpoint; Handoff.

## CI

Linux formatting/lint/tests + NumPy/Parquet integration; standalone Clang ASan
and UBSan; benchmark smoke only. CI starts at H0. Integrated native sanitizer
smoke is optional with a one-hour investigation limit; TSan is deferred. No
throughput threshold and no full benchmark campaign in routine CI.
