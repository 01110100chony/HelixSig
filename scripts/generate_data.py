"""Generate a bounded synthetic corpus and independent numerical reference."""

import argparse
import hashlib
import json
import platform
from pathlib import Path

import numpy as np

from oracle import features


def generate(output, rows=256, samples=256, baseline_samples=32, seed=20260909):
    if not 2 <= samples <= 4096 or not 1 <= baseline_samples < samples:
        raise ValueError("require 2 <= samples <= 4096 and 1 <= baseline_samples < samples")
    if rows < 1 or rows * samples * 8 > 64 * 1024 * 1024:
        raise ValueError("corpus exceeds 64 MiB or has no rows")
    output = Path(output)
    output.mkdir(parents=True, exist_ok=False)
    rng = np.random.Generator(np.random.PCG64(seed))
    index = np.arange(samples)
    data = np.empty((rows, samples), dtype="<f8")
    for row in range(rows):
        baseline = rng.uniform(-10, 10)
        amplitude = rng.uniform(10, 100)
        position = rng.uniform(baseline_samples, samples - 1)
        width = max(1.0, samples / 32)
        pulse = amplitude * np.exp(-0.5 * ((index - position) / width) ** 2)
        pulse[:baseline_samples] = 0
        data[row] = baseline + rng.normal(0, 0.2, samples) + pulse
    # Reproducible exact analytical cases alongside stochastic fixtures.
    data[0] = -2.0
    if rows > 1:
        data[1] = 3.0
        data[1, baseline_samples] = 8.0
    payload = data.tobytes(order="C")
    (output / "signals.f64le").write_bytes(payload)
    manifest = dict(schema_version=1, file="signals.f64le", rows=rows, samples=samples,
                    baseline_samples=baseline_samples, dtype="float64-le", seed=seed,
                    sha256=hashlib.sha256(payload).hexdigest(), generator="PCG64",
                    python_version=platform.python_version(), numpy_version=np.__version__,
                    signal=dict(baseline_range=[-10, 10], amplitude_range=[10, 100],
                                noise_sigma=0.2, pulse_width=max(1.0, samples / 32),
                                pulse_zero_before=baseline_samples, analytical_rows=min(2, rows)))
    (output / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    reference = [features(row, baseline_samples) for row in data]
    (output / "reference.json").write_text(json.dumps(reference, indent=2, allow_nan=False) + "\n")
    return manifest


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--rows", type=int, default=256)
    parser.add_argument("--samples", type=int, default=256)
    parser.add_argument("--baseline-samples", type=int, default=32)
    parser.add_argument("--seed", type=int, default=20260909)
    args = parser.parse_args()
    print(json.dumps(generate(args.out, args.rows, args.samples, args.baseline_samples, args.seed)))
