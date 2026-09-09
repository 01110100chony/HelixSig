---
name: review-adjudication
description: Verify and adjudicate AI reviewer findings before modifying code.
---
# Review Adjudication
For every finding: locate evidence, reconstruct the claim independently, classify it as
VALID / INVALID / DUPLICATE / NEEDS-EXPERIMENT, assign severity against actual contracts,
and fix only VALID in-scope findings.

For NEEDS-EXPERIMENT, design the smallest falsifiable test/benchmark and reclassify from evidence.
Rerun affected gates. Closeout should list reviewers, classifications, fixes, rerun gates and remaining limitations.
