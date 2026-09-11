"""H3 acceptance: bounded concurrent NumPy/Parquet equivalence checks."""

import argparse
import json
import math
import os
import subprocess
import tempfile
from pathlib import Path

import pyarrow as pa
import pyarrow.parquet as pq

from oracle import assert_result, features
from validate import load_corpus


def run(binary, corpus, output, options, expected=0):
    result = subprocess.run(
        [str(binary), "run", "--corpus", str(corpus), "--out", str(output), *options],
        capture_output=True,
        text=True,
        timeout=90,
    )
    assert result.returncode == expected, (result.returncode, result.stdout, result.stderr)
    summary = json.loads(result.stdout)
    assert summary["schema_version"] == 1
    assert summary["produced"] == summary["accepted"] + summary["dropped"] + summary["not_admitted"]
    assert summary["accepted"] == summary["processed"] + summary["failed"] + summary["aborted"]
    assert summary["processed"] == summary["written"] + summary["unwritten"]
    assert summary["status"] == "completed"
    assert summary["output"]["state"] == "finalized"
    assert summary["produced"] == summary["accepted"] == summary["processed"] == summary["written"]
    assert all(summary[name] == 0 for name in ("dropped", "not_admitted", "failed", "aborted", "unwritten"))
    assert len(summary["diagnostics"]) <= 16
    assert summary["metrics"]["latency_observations"] == summary["processed"]
    assert (
        summary["metrics"]["latency_recorded"] + summary["metrics"]["latency_overflow"]
        == summary["processed"]
    )
    assert json.loads((output / "summary.json").read_text()) == summary
    assert summary["duration_ns"] > 0
    for counter in ("processed", "written"):
        assert math.isclose(summary[f"throughput_{counter}_per_second"], summary[counter] * 1e9 / summary["duration_ns"])
    return summary


def read_rows(summary, corpus, events):
    manifest, data = load_corpus(corpus)
    path = Path(summary["output"]["file"])
    assert path.is_file() and not path.name.endswith(".incomplete")
    assert not list(path.parent.glob("*.incomplete"))
    parquet = pq.ParquetFile(path)
    expected_schema = pa.schema(
        [
            pa.field("event_id", pa.uint64(), nullable=False),
            pa.field("channel_id", pa.uint32(), nullable=False),
            pa.field("sequence", pa.uint64(), nullable=False),
            pa.field("baseline", pa.float64(), nullable=False),
            pa.field("peak_amplitude", pa.float64(), nullable=False),
            pa.field("peak_index", pa.uint32(), nullable=False),
            pa.field("integral", pa.float64(), nullable=False),
            pa.field("latency_ns", pa.uint64(), nullable=False),
        ]
    )
    table = parquet.read()
    assert table.schema.equals(expected_schema), table.schema
    assert table.num_rows == events == summary["written"]
    assert summary["corpus_sha256"] == manifest["sha256"]
    assert summary["estimated_data_bytes"] <= 256 * 1024 * 1024
    rows = sorted(table.to_pylist(), key=lambda row: row["event_id"])
    ids = [row["event_id"] for row in rows]
    assert ids == list(range(events)) and len(ids) == len(set(ids))
    for row in rows:
        event_id = row["event_id"]
        assert row["channel_id"] == event_id % 4
        assert row["sequence"] == event_id // 4
        assert 0 <= row["latency_ns"] <= summary["duration_ns"]
        assert_result(
            dict(row, status=0),
            features(data[event_id % manifest["rows"]], summary["config"]["baseline_samples"]),
        )
    for group in range(parquet.num_row_groups):
        metadata = parquet.metadata.row_group(group)
        assert metadata.num_rows <= 4096
        for index in range(metadata.num_columns):
            column = metadata.column(index)
            assert column.compression == "UNCOMPRESSED"
            assert not any("DICTIONARY" in encoding for encoding in column.encodings)
    return [{key: value for key, value in row.items() if key != "latency_ns"} for row in rows]


def check_metrics(summary, workers, batch_size):
    metrics = summary["metrics"]
    histogram = metrics["actual_batch_sizes"]
    assert len(histogram) > batch_size
    assert all(count == 0 for count in histogram[batch_size + 1 :])
    assert len(histogram) == 65 and histogram[0] == 0
    assert sum(index * count for index, count in enumerate(histogram)) == summary["accepted"]
    assert sum(index * count for index, count in enumerate(histogram)) == metrics["latency_observations"]

    worker_metrics = summary["worker_metrics"]
    assert len(worker_metrics) == workers
    totals = {"latency_observations": 0, "latency_recorded": 0, "latency_overflow": 0}
    merged_histogram = [0] * len(histogram)
    for worker in worker_metrics:
        for key in totals:
            assert key in worker
            totals[key] += worker[key]
        worker_histogram = worker["actual_batch_sizes"]
        assert len(worker_histogram) <= len(histogram)
        assert all(count == 0 for count in worker_histogram[batch_size + 1 :])
        assert len(worker_histogram) == 65 and worker_histogram[0] == 0
        assert sum(index * count for index, count in enumerate(worker_histogram)) == worker[
            "latency_observations"
        ]
        for index, count in enumerate(worker_histogram):
            merged_histogram[index] += count
        if worker["latency_recorded"] == 0:
            for key, value in worker.items():
                if "latency_p" in key or key.startswith("p"):
                    assert value is None, (key, value)
    assert totals["latency_observations"] == metrics["latency_observations"]
    assert totals["latency_recorded"] == metrics["latency_recorded"]
    assert totals["latency_overflow"] == metrics["latency_overflow"]
    assert merged_histogram == histogram


def validate(binary, corpus, artifacts):
    load_corpus(corpus)
    available = len(os.sched_getaffinity(0)) if hasattr(os, "sched_getaffinity") else (os.cpu_count() or 1)
    workers = [worker for worker in (1, 2, 4) if worker <= available]
    # Two cases per available worker plus the required boundary counts keeps this matrix bounded.
    matrix = [
        (1, 1, "event", workers[0]),
        (3, 16, "batch", workers[0]),
        (65, 64, "event", workers[min(1, len(workers) - 1)]),
        (67, 1, "batch", workers[min(1, len(workers) - 1)]),
        (4101, 16, "event", workers[-1]),
        (65, 64, "batch", workers[-1]),
        (1, 64, "batch", workers[-1]),
        (3, 1, "event", workers[-1]),
        (67, 16, "event", workers[0]),
        (4101, 1, "batch", workers[-1]),
        (65, 16, "event", workers[min(1, len(workers) - 1)]),
        (67, 64, "batch", workers[-1]),
    ]
    cases = 0
    root = Path(tempfile.mkdtemp(prefix="h3-", dir=artifacts))
    for index, (events, batch_size, ffi, worker_count) in enumerate(matrix):
        queue_capacity = 1 if index % 2 == 0 else 256
        common = [
            "--events",
            str(events),
            "--batch-size",
            str(batch_size),
            "--queue-capacity",
            str(queue_capacity),
            "--ffi",
            ffi,
            "--policy",
            "block",
        ]
        concurrent = run(
            binary,
            corpus,
            root / f"concurrent-{index}",
            [
                "--execution",
                "concurrent",
                "--workers",
                str(worker_count),
                *common,
            ],
        )
        check_metrics(concurrent, worker_count, batch_size)
        actual = read_rows(concurrent, corpus, events)

        sequential = run(
            binary,
            corpus,
            root / f"sequential-{index}",
            ["--execution", "sequential", "--workers", "1", *common],
        )
        assert sequential["worker_metrics"] == []
        expected = read_rows(sequential, corpus, events)
        assert actual == expected
        cases += 2
    print(
        json.dumps(
            {
                "status": "PASS",
                "concurrent_cases": len(matrix),
                "case_count": cases,
                "artifact_directory": str(root),
                "worker_counts": workers,
                "atol": 1e-10,
                "rtol": 1e-10,
            }
        )
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--corpus", required=True, type=Path)
    parser.add_argument("--artifacts", default=Path("artifacts"), type=Path)
    args = parser.parse_args()
    args.artifacts.mkdir(parents=True, exist_ok=True)
    validate(args.binary.resolve(), args.corpus.resolve(), args.artifacts.resolve())
