"""H5 design and independent acceptance helpers (outside measured intervals)."""
import itertools
import math
from pathlib import Path

import numpy as np
import pyarrow.parquet as pq

from oracle import features
from validate import load_corpus


def pipeline_design():
    axes = {(w, 256, 16) for w in (1, 2, 4)}
    axes |= {(2, q, 16) for q in (64, 256, 1024)}
    axes |= {(2, 256, b) for b in (1, 16, 64)}
    return [dict(kind="pipeline", samples=256, workers=w, queue_capacity=q,
                 batch_size=b, policy=p, ffi=f)
            for (w, q, b), p, f in itertools.product(sorted(axes),
                                                    ("block", "drop-new"), ("event", "batch"))]


def micro_design():
    return [dict(kind="micro", samples=n, batch_size=b, ffi=f,
                 preparation=p, engine=e)
            for n, b, f, p, e in itertools.product(
                (64, 256, 4096), (1, 4, 16, 64), ("event", "batch"),
                ("prepared", "pack"), ("rust_ffi", "native"))]


def accounting(summary, code):
    expected = 2 if summary["dropped"] else 0
    assert code == expected, (code, expected)
    assert summary["status"] == ("completed_with_drops" if expected else "completed")
    assert summary["produced"] == summary["config"]["events"]
    assert summary["produced"] == summary["accepted"] + summary["dropped"] + summary["not_admitted"]
    assert summary["accepted"] == summary["processed"] + summary["failed"] + summary["aborted"]
    assert summary["processed"] == summary["written"] + summary["unwritten"]
    assert all(summary[k] == 0 for k in ("not_admitted", "failed", "aborted", "unwritten"))
    if summary["config"]["policy"] == "block":
        assert summary["dropped"] == 0
    assert summary["output"]["state"] == "finalized"
    assert summary["duration_ns"] > 0
    for counter in ("processed", "written"):
        expected_rate = summary[counter] * 1e9 / summary["duration_ns"]
        assert math.isclose(summary[f"throughput_{counter}_per_second"], expected_rate, rel_tol=1e-12)
    metrics = summary["metrics"]
    assert metrics["latency_observations"] == summary["processed"]
    assert metrics["latency_recorded"] + metrics["latency_overflow"] == summary["processed"]
    assert len(metrics["actual_batch_sizes"]) == 65 and metrics["actual_batch_sizes"][0] == 0
    assert sum(i * n for i, n in enumerate(metrics["actual_batch_sizes"])) == summary["processed"]


def validate_rows(summary, corpus):
    manifest, data = load_corpus(corpus)
    assert summary["corpus_sha256"] == manifest["sha256"]
    table = pq.read_table(Path(summary["output"]["file"]))
    assert len(table) == summary["written"]
    ids = table["event_id"].to_numpy()
    assert len(np.unique(ids)) == len(ids)
    assert np.all(ids < summary["produced"])
    assert np.array_equal(table["channel_id"].to_numpy(), ids % 4)
    assert np.array_equal(table["sequence"].to_numpy(), ids // 4)
    reference = [features(row, manifest["baseline_samples"]) for row in data]
    indices = (ids % manifest["rows"]).astype(np.intp)
    for key in ("baseline", "peak_amplitude", "integral", "peak_index"):
        expected = np.array([row[key] for row in reference])[indices]
        actual = table[key].to_numpy()
        if key == "peak_index":
            assert np.array_equal(actual, expected)
        else:
            assert np.all(np.isfinite(actual))
            assert np.all(np.abs(actual - expected) <= 1e-10 + 1e-10 * np.abs(expected)), key
    latency = table["latency_ns"].to_numpy()
    assert np.all(latency <= summary["duration_ns"])
    overflow = int(np.count_nonzero(latency > summary["metrics"]["latency_max_trackable_ns"]))
    assert overflow == summary["metrics"]["latency_overflow"]
    return dict(population=len(latency), small_population=len(latency) < 1000,
                histogram_overflow=overflow,
                **{f"p{p}_ns": int(np.quantile(latency, p / 100, method="inverted_cdf"))
                   if len(latency) else None for p in (50, 95, 99)})


def expected_checksum(corpus, events):
    manifest, data = load_corpus(corpus)
    values = []
    for row in data:
        value = features(row, manifest["baseline_samples"])
        assert value["status"] == 0
        values.append(value["baseline"] + value["peak_amplitude"] + value["integral"] + value["peak_index"])
    # Match the documented observable consumption order, outside measurement.
    checksum = 0.0
    for index in range(events):
        checksum += values[index % len(values)]
    return checksum


def validate_micro(summary, corpus):
    manifest, data = load_corpus(corpus)
    assert summary["repeat_consistent"] is True
    verification = summary["verification"]
    assert len(verification) == min(summary["events"], manifest["rows"])
    for actual, samples in zip(verification, data):
        expected = features(samples, manifest["baseline_samples"])
        assert len(actual) == 5 and actual[0] == expected["status"] == 0
        assert actual[3] == expected["peak_index"]
        for index, key in ((1, "baseline"), (2, "peak_amplitude"), (4, "integral")):
            assert math.isfinite(actual[index])
            assert abs(actual[index] - expected[key]) <= 1e-10 + 1e-10 * abs(expected[key]), key
