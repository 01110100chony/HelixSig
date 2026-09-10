# H0 numerical correction packet

Prerequisite finding independently reproduced in Ubuntu with NumPy 2.5.3:
samples=[0,1e16,1,-1e16,-1,1e15,-1e15,0,0], K=1. Native integral=-1;
NumPy integral=0. Original candidate ebc8edb312b2eab125dcc3b286fc880668a5b2a1.

Leader classification: VALID, blocks H0. Preserve formula, float64 types, status
semantics and tolerance. No scope expansion or tolerance relaxation.

## Allowed edits
Only cpp/src/kernel.cpp, cpp/tests/kernel_tests.cpp, scripts/validate.py.
Do not edit public headers, oracle.py, other files, or any Linux checkout.
Do not commit, delegate, install tools or run builds. Leader runs Linux checks.

## Required implementation
Use an independently written private blocked pairwise reduction for baseline
and integral, matching the reduction grouping of the pinned NumPy oracle:
- Fewer than 8 values: sequential accumulation initialized to negative zero.
- 8..128 values: initialize eight scalar lanes from the first eight values;
  accumulate complete subsequent groups of eight lane-wise; combine as
  ((s0+s1)+(s2+s3))+((s4+s5)+(s6+s7)); then add remaining values in order.
- Above 128: split near half, rounded down to a multiple of eight, recurse and
  add the halves. Recursion is bounded by the maximum signal size 4096.
- Allow subtracting the baseline while reading values for integral; no heap
  allocation, copied waveform, custom SIMD, mutable globals or fast-math.
- Continue rejecting nonfinite input/corrected values and nonfinite final sums.
- Compute peak with the existing strict first-maximum rule.

This fixes the numerical reduction order before H0 freezes; it is not performance
tuning. NumPy's public sum documentation describes partial pairwise summation:
https://numpy.org/doc/stable/reference/generated/numpy.sum.html . The native kernel
remains independently compiled and does not call NumPy or use Python expectations.

## Regression checks to add
- Native analytical regression for the exact counterexample above.
- Independent Python/native comparisons with finite cancellation for both baseline
  and corrected integral, covering short tails and widths around 8 and 128 plus
  recursive sizes through 4096. Include deterministic permutations with scales
  1, 1e8 and 1e16; use the existing oracle unchanged.
- Preserve all existing overflow/NaN/Inf/shape checks.
- No use of asserts in C++ tests that disappears under Release/NDEBUG.

## Required response
List changed files, concise rationale and checks actually run. No gate verdict.
