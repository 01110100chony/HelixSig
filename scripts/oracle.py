"""Independent NumPy specification: no native code or Helix runtime imports."""

import numpy as np

ATOL = 1e-10
RTOL = 1e-10


def features(samples, baseline_samples):
    x = np.asarray(samples, dtype=np.float64)
    empty = dict(status=0, baseline=0.0, peak_amplitude=0.0, peak_index=0, integral=0.0)
    if x.ndim != 1 or not 2 <= x.size <= 4096:
        return empty | {"status": 1}
    if not 1 <= baseline_samples < x.size:
        return empty | {"status": 2}
    if not np.isfinite(x).all():
        return empty | {"status": 3}
    with np.errstate(over="ignore", invalid="ignore"):
        baseline = float(np.mean(x[:baseline_samples]))
        corrected = x[baseline_samples:] - baseline
        integral = float(np.sum(corrected))
    if not np.isfinite(baseline) or not np.isfinite(corrected).all() or not np.isfinite(integral):
        return empty | {"status": 4}
    peak = int(np.argmax(corrected))
    return dict(status=0, baseline=baseline, peak_amplitude=float(corrected[peak]),
                peak_index=baseline_samples + peak, integral=integral)


def assert_result(actual, reference):
    assert int(actual["status"]) == reference["status"], (actual, reference)
    if reference["status"]:
        return
    assert int(actual["peak_index"]) == reference["peak_index"], (actual, reference)
    for field in ("baseline", "peak_amplitude", "integral"):
        value, expected = float(actual[field]), reference[field]
        assert np.isfinite(value) and abs(value - expected) <= ATOL + RTOL * abs(expected), (
            field, value, expected)
