"""H4 CLI failure acceptance checks.

The cases intentionally assert accounting and output invariants, not scheduling
dependent admission counts or concurrent batch grouping.
"""

import argparse
import hashlib
import json
import resource
import shutil
import signal
import subprocess
import tempfile
import time
from pathlib import Path

import numpy as np
import pyarrow as pa
import pyarrow.parquet as pq

from generate_data import generate
from oracle import assert_result, features
from validate import load_corpus


SCHEMA = pa.schema(
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


def _accounting(summary):
    assert summary["produced"] == summary["accepted"] + summary["dropped"] + summary["not_admitted"]
    assert summary["accepted"] == summary["processed"] + summary["failed"] + summary["aborted"]
    assert summary["processed"] == summary["written"] + summary["unwritten"]
    assert len(summary["diagnostics"]) <= 16
    assert isinstance(summary["worker_metrics"], list)
    metrics = summary["metrics"]
    assert metrics["latency_observations"] == summary["processed"]
    assert metrics["latency_recorded"] + metrics["latency_overflow"] == summary["processed"]
    if summary["config"]["execution"] == "concurrent":
        assert len(summary["worker_metrics"]) == summary["config"]["workers"]
        assert sum(worker["latency_observations"] for worker in summary["worker_metrics"]) == summary["processed"]


def _persist_failure(output, result):
    output.mkdir(parents=True, exist_ok=True)
    (output / "finalstdout.json").write_text(result.stdout)
    (output / "finalstderr.txt").write_text(result.stderr)


def _run(binary, corpus, output, options, expected=None, preexec_fn=None, timeout=120):
    command = [
        str(binary),
        "run",
        "--corpus",
        str(corpus),
        "--out",
        str(output),
        *map(str, options),
    ]
    result = None
    try:
        result = subprocess.run(
            command,
            capture_output=True,
            text=True,
            timeout=timeout,
            preexec_fn=preexec_fn,
        )
    except BaseException:
        if result is None:
            raise
    _persist_failure(output, result)
    assert result.stdout.strip(), (result.returncode, result.stderr)
    summary = json.loads(result.stdout)
    _accounting(summary)
    assert result.returncode == {"completed": 0, "completed_with_drops": 2, "failed": 1, "interrupted": 130, "invalid_configuration": 64}[summary["status"]]
    if (output / "summary.json").exists():
        assert json.loads((output / "summary.json").read_text()) == summary
    if expected is not None:
        assert result.returncode == expected, (result.returncode, result.stdout, result.stderr)
    return summary


def _check_output(summary, corpus, expected_ids=None, partial=False):
    path = Path(summary["output"]["file"])
    assert path.is_file()
    if partial:
        assert path.name == "events.partial.parquet"
        assert not list(path.parent.glob("*.incomplete"))
    else:
        assert not path.name.endswith(".incomplete") and path.name != "events.partial.parquet"
    if not partial:
        assert not list(path.parent.glob("*.incomplete"))
    table = pq.read_table(path)
    assert table.schema.equals(SCHEMA), table.schema
    rows = sorted(table.to_pylist(), key=lambda row: row["event_id"])
    ids = [row["event_id"] for row in rows]
    if expected_ids is not None:
        assert ids == list(expected_ids)
    assert len(ids) == len(set(ids)) == summary["written"]
    manifest, data = load_corpus(corpus)
    assert summary["corpus_sha256"] == manifest["sha256"]
    assert summary["estimated_data_bytes"] <= 256 * 1024 * 1024
    for row in rows:
        event_id = row["event_id"]
        assert 0 <= row["latency_ns"] <= summary["duration_ns"]
        assert row["channel_id"] == event_id % 4
        assert row["sequence"] == event_id // 4
        assert_result(
            dict(row, status=0),
            features(data[event_id % manifest["rows"]], summary["config"]["baseline_samples"]),
        )
    return rows


def _check_status(summary, status, state):
    assert summary["status"] == status
    assert summary["output"]["state"] == state
    assert summary["failed"] >= 0


def _limit_file(size):
    def apply():
        signal.signal(signal.SIGXFSZ, signal.SIG_IGN)
        resource.setrlimit(resource.RLIMIT_FSIZE, (size, size))

    return apply


def _drop_new(binary, corpus, root):
    cases = 0
    for ffi in ("event", "batch"):
        output = root / f"drop-{ffi}"
        summary = _run(
            binary,
            corpus,
            output,
            [
                "--execution",
                "concurrent",
                "--workers",
                "1",
                "--events",
                "50000",
                "--batch-size",
                "1",
                "--queue-capacity",
                "1",
                "--policy",
                "drop-new",
                "--ffi",
                ffi,
            ],
            timeout=180,
        )
        assert summary["status"] in ("completed", "completed_with_drops")
        assert summary["output"]["state"] == "finalized"
        assert summary["failed"] == summary["aborted"] == summary["unwritten"] == 0
        assert summary["dropped"] == 0 or summary["status"] == "completed_with_drops"
        assert summary["status"] == ("completed_with_drops" if summary["dropped"] else "completed")
        assert summary["not_admitted"] == 0
        assert summary["produced"] == 50000
        rows = _check_output(summary, corpus, partial=False)
        ids = {row["event_id"] for row in rows}
        expected = set(range(summary["produced"]))
        assert ids <= expected
        assert len(expected - ids) == summary["dropped"]
        assert summary["accepted"] == len(ids)
        cases += 1
    return cases


def _writer_failure(binary, corpus, root):
    output = root / "writer-failure"
    summary = _run(
        binary,
        corpus,
        output,
        [
            "--execution",
            "concurrent",
            "--workers",
            "1",
            "--events",
            "50000",
            "--batch-size",
            "16",
            "--queue-capacity",
            "1",
            "--policy",
            "block",
            "--ffi",
            "batch",
        ],
        expected=1,
        preexec_fn=_limit_file(16 * 1024),
        timeout=180,
    )
    _check_status(summary, "failed", "incomplete")
    assert summary["failed"] == 0
    assert summary["written"] == 0
    assert summary["unwritten"] == summary["processed"] == summary["accepted"]
    assert summary["produced"] == summary["accepted"] + summary["dropped"] + summary["not_admitted"]
    assert summary["dropped"] == 0
    assert Path(summary["output"]["file"]).name.endswith(".incomplete")
    return 1


def _nan_corpus(source, destination):
    shutil.copytree(source, destination)
    manifest, data = load_corpus(destination)
    mutated = data.copy()
    mutated[0, 0] = np.nan
    payload = mutated.astype("<f8", copy=False).tobytes(order="C")
    (destination / "signals.f64le").write_bytes(payload)
    manifest["sha256"] = hashlib.sha256(payload).hexdigest()
    (destination / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")


def _nan_cases(binary, corpus, root):
    invalid = root / "nan-corpus"
    _nan_corpus(corpus, invalid)
    manifest, _ = load_corpus(invalid)
    cases = 0
    for ffi in ("event", "batch"):
        output = root / f"nan-{ffi}"
        summary = _run(
            binary,
            invalid,
            output,
            [
                "--execution",
                "concurrent",
                "--workers",
                "2",
                "--events",
                "65",
                "--batch-size",
                "16",
                "--queue-capacity",
                "4",
                "--policy",
                "block",
                "--ffi",
                ffi,
            ],
            expected=1,
        )
        _check_status(summary, "failed", "partial")
        assert summary["produced"] == summary["accepted"] == 65
        assert summary["accepted"] == summary["processed"] + summary["failed"]
        assert summary["failed"] > 0 and summary["aborted"] == 0
        assert summary["processed"] == summary["written"] and summary["unwritten"] == 0
        valid_ids = [event_id for event_id in range(summary["accepted"]) if event_id % manifest["rows"] != 0]
        assert summary["failed"] == summary["accepted"] - len(valid_ids)
        _check_output(summary, invalid, valid_ids, partial=True)
        cases += 1
    return cases


def _interrupt(binary, corpus, root, execution):
    output = root / f"sigint-{execution}"
    command = [
        str(binary),
        "run",
        "--corpus",
        str(corpus),
        "--out",
        str(output),
        "--execution",
        execution,
        "--workers",
        "2" if execution == "concurrent" else "1",
        "--events",
        "1000000",
        "--batch-size",
        "16",
        "--queue-capacity",
        "1",
        "--policy",
        "block",
        "--ffi",
        "batch",
    ]
    process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    try:
        incomplete = output / "events.parquet.incomplete"
        deadline = time.monotonic() + 30
        while time.monotonic() < deadline and (
            not incomplete.exists() or incomplete.stat().st_size <= 4
        ):
            time.sleep(0.05)
        assert incomplete.exists() and incomplete.stat().st_size > 4
        process.send_signal(signal.SIGINT)
        stdout, stderr = process.communicate(timeout=30)
        assert process.returncode == 130, (process.returncode, stdout, stderr)
        _persist_failure(output, subprocess.CompletedProcess(command, process.returncode, stdout, stderr))
        summary = json.loads(stdout)
        _accounting(summary)
        if (output / "summary.json").exists():
            assert json.loads((output / "summary.json").read_text()) == summary
        assert summary["status"] == "interrupted"
        assert summary["output"]["state"] == "partial"
        assert summary["dropped"] == summary["failed"] == summary["aborted"] == summary["unwritten"] == 0
        assert summary["accepted"] == summary["processed"] == summary["written"]
        _check_output(summary, corpus, range(summary["accepted"]), partial=True)
        return 1
    finally:
        if process.poll() is None:
            process.kill()
            process.wait(timeout=10)


def validate(binary, corpus, artifacts):
    root = Path(tempfile.mkdtemp(prefix="h4-", dir=artifacts))
    generated = root / "sigint-corpus"
    generate(generated, rows=32, samples=4096, baseline_samples=32)
    cases = _drop_new(binary, corpus, root)
    cases += _writer_failure(binary, corpus, root)
    cases += _nan_cases(binary, corpus, root)
    cases += _interrupt(binary, generated, root, "sequential")
    cases += _interrupt(binary, generated, root, "concurrent")
    print(
        json.dumps(
            {
                "status": "PASS",
                "failure_cases": cases,
                "artifact_directory": str(root),
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
