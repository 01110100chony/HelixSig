"""Validate native output against NumPy, including independent invalid fixtures."""

import argparse
import csv
import hashlib
import io
import json
import subprocess
import tempfile
from pathlib import Path

import numpy as np

from oracle import assert_result, features


def load_corpus(directory):
    directory = Path(directory)
    manifest = json.loads((directory / "manifest.json").read_text())
    assert manifest["schema_version"] == 1 and manifest["dtype"] == "float64-le"
    assert manifest["file"] == "signals.f64le"
    size = manifest["rows"] * manifest["samples"] * 8
    assert 0 < size <= 64 * 1024 * 1024
    payload = (directory / manifest["file"]).read_bytes()
    assert len(payload) == size
    assert hashlib.sha256(payload).hexdigest() == manifest["sha256"]
    data = np.frombuffer(payload, dtype="<f8").reshape(manifest["rows"], manifest["samples"])
    return manifest, data


def native_rows(driver, path, width, baseline):
    result = subprocess.run([str(driver), str(path), str(width), str(baseline)],
                            check=True, capture_output=True, text=True, timeout=30)
    return list(csv.DictReader(io.StringIO(result.stdout)))


def validate_native(driver, corpus):
    manifest, data = load_corpus(corpus)
    actual = native_rows(driver, Path(corpus) / manifest["file"], manifest["samples"],
                         manifest["baseline_samples"])
    assert len(actual) == len(data)
    for index, (result, row) in enumerate(zip(actual, data)):
        assert int(result["row"]) == index
        assert_result(result, features(row, manifest["baseline_samples"]))
    # Cases assembled independently of both the generator and C++ test expectations.
    cases = np.array([[-2, -2, 1, 5, 5, -3], [3, 3, 1, 2, 0, 1],
                      [0, 0, np.nan, 1, 2, 3], [0, 0, np.inf, 1, 2, 3],
                      [1e308, 1e308, 1, 1, 1, 1], [0, 0, 1e308, 1e308, 0, 0]], dtype="<f8")
    with tempfile.TemporaryDirectory(prefix="helix-oracle-") as folder:
        path = Path(folder) / "invalid.f64le"
        path.write_bytes(cases.tobytes())
        results = native_rows(driver, path, 6, 2)
        assert len(results) == len(cases)
        for result, row in zip(results, cases):
            assert_result(result, features(row, 2))
    return len(data) + len(cases)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--native", type=Path, required=True)
    parser.add_argument("--corpus", type=Path, required=True)
    args = parser.parse_args()
    count = validate_native(args.native, args.corpus)
    print(json.dumps(dict(status="PASS", independent_cases=count, atol=1e-10, rtol=1e-10)))
