"""H2 acceptance: independent NumPy/Parquet readback and deterministic failures."""

import argparse
import hashlib
import json
import math
import os
import resource
import shutil
import signal
import subprocess
import tempfile
from pathlib import Path

import numpy as np
import pyarrow as pa
import pyarrow.parquet as pq

from oracle import assert_result, features
from validate import load_corpus


def invoke(binary, corpus, output, extra=(), expected=0, preexec_fn=None):
    result = subprocess.run(
        [str(binary), "run", "--corpus", str(corpus), "--out", str(output),
         "--workers", "1", *extra], capture_output=True, text=True,
        timeout=90, preexec_fn=preexec_fn)
    assert result.returncode == expected, (result.returncode, result.stdout, result.stderr)
    summary = json.loads(result.stdout)
    assert summary["schema_version"] == 1
    assert summary["produced"] == summary["accepted"] + summary["dropped"] + summary["not_admitted"]
    assert summary["accepted"] == summary["processed"] + summary["failed"] + summary["aborted"]
    assert summary["processed"] == summary["written"] + summary["unwritten"]
    assert len(summary["diagnostics"]) <= 16
    assert summary["metrics"]["latency_observations"] == summary["processed"]
    assert summary["metrics"]["latency_recorded"] + summary["metrics"]["latency_overflow"] == summary["processed"]
    if summary["duration_ns"]:
        seconds = summary["duration_ns"] / 1e9
        for counter in ("processed", "written"):
            assert math.isclose(summary[f"throughput_{counter}_per_second"], summary[counter] / seconds)
        assert sum(i * n for i, n in enumerate(summary["metrics"]["actual_batch_sizes"])) == summary["accepted"]
    if (output / "summary.json").exists():
        assert json.loads((output / "summary.json").read_text()) == summary
    return summary


def readback(summary, corpus, expected_ids):
    manifest, data = load_corpus(corpus)
    path = Path(summary["output"]["file"])
    assert path.is_file() and not path.name.endswith(".incomplete")
    assert not list(path.parent.glob("*.incomplete"))
    assert json.loads((path.parent / "running.json").read_text())["status"] == "running"
    parquet = pq.ParquetFile(path)
    table = parquet.read()
    expected_schema = pa.schema([
        pa.field("event_id", pa.uint64(), nullable=False),
        pa.field("channel_id", pa.uint32(), nullable=False),
        pa.field("sequence", pa.uint64(), nullable=False),
        pa.field("baseline", pa.float64(), nullable=False),
        pa.field("peak_amplitude", pa.float64(), nullable=False),
        pa.field("peak_index", pa.uint32(), nullable=False),
        pa.field("integral", pa.float64(), nullable=False),
        pa.field("latency_ns", pa.uint64(), nullable=False),
    ])
    assert table.schema.equals(expected_schema), table.schema
    assert table.num_rows == len(expected_ids) == summary["written"]
    assert summary["corpus_sha256"] == manifest["sha256"]
    assert summary["estimated_data_bytes"] <= 256 * 1024 * 1024
    rows = table.to_pylist()
    assert [r["event_id"] for r in rows] == expected_ids
    for row in rows:
        event_id = row["event_id"]
        assert row["channel_id"] == event_id % 4 and row["sequence"] == event_id // 4
        assert 0 <= row["latency_ns"] <= summary["duration_ns"]
        assert_result(dict(row, status=0), features(data[event_id % manifest["rows"]], summary["config"]["baseline_samples"]))
    for group in range(parquet.num_row_groups):
        metadata = parquet.metadata.row_group(group)
        assert metadata.num_rows <= 4096
        for index in range(metadata.num_columns):
            column = metadata.column(index)
            assert column.compression == "UNCOMPRESSED"
            assert not any("DICTIONARY" in encoding for encoding in column.encodings)
    return rows, parquet


def validate(binary, corpus, work):
    cases = 0
    outputs = []
    for mode in ("event", "batch"):
        summary = invoke(binary, corpus, work / mode, ["--events", "4101", "--ffi", mode])
        assert summary["status"] == "completed" and summary["output"]["state"] == "finalized"
        for counter in ("produced", "accepted", "processed", "written"):
            assert summary[counter] == 4101
        for counter in ("dropped", "not_admitted", "failed", "aborted", "unwritten"):
            assert summary[counter] == 0
        rows, parquet = readback(summary, corpus, list(range(4101)))
        assert [parquet.metadata.row_group(i).num_rows for i in range(parquet.num_row_groups)] == [4096, 5]
        assert summary["metrics"]["actual_batch_sizes"][16] == 256
        assert summary["metrics"]["actual_batch_sizes"][5] == 1
        outputs.append([{k: v for k, v in row.items() if k != "latency_ns"} for row in rows])
        cases += 1
    assert outputs[0] == outputs[1]
    for batch, policy in ((1, "block"), (64, "drop-new")):
        summary = invoke(binary, corpus, work / f"batch-{batch}",
                         ["--events", "65", "--batch-size", str(batch), "--policy", policy,
                          "--queue-capacity", "1", "--baseline-samples", "1"])
        readback(summary, corpus, list(range(65)))
        assert summary["dropped"] == 0
        cases += 1

    invalid_configs = [
        ("events", "0"), ("events", "1000001"), ("batch-size", "0"), ("batch-size", "65"),
        ("queue-capacity", "0"), ("queue-capacity", "4097"), ("workers", "0"),
        ("workers", "999999999"), ("execution", "concurrent"), ("baseline-samples", "0"),
        ("baseline-samples", "4096"), ("baseline-samples", "256"),
    ]
    for index, (key, value) in enumerate(invalid_configs):
        output = work / f"config-{index}"
        # Avoid passing --workers twice (clap correctly rejects duplicate flags).
        extra = [f"--{key}", value]
        if key == "workers":
            result = subprocess.run([str(binary), "run", "--corpus", str(corpus), "--out", str(output), *extra],
                                    capture_output=True, text=True, timeout=90)
            assert result.returncode == 64
            summary = json.loads(result.stdout)
        else:
            summary = invoke(binary, corpus, output, extra, expected=64)
        assert summary["produced"] == 0 and summary["status"] == "invalid_configuration"
        assert not output.exists()
        cases += 1

    existing = work / "existing"
    existing.mkdir()
    (existing / "sentinel").write_bytes(b"preserve me")
    summary = invoke(binary, corpus, existing, expected=64)
    assert summary["produced"] == 0
    assert sorted(p.name for p in existing.iterdir()) == ["sentinel"]
    assert (existing / "sentinel").read_bytes() == b"preserve me"
    cases += 1

    original = json.loads((corpus / "manifest.json").read_text())
    mutations = [
        {"sha256": "0" * 64}, {"rows": 0}, {"rows": 2**64 - 1}, {"samples": 1}, {"samples": 4097},
        {"schema_version": 2}, {"dtype": "float32-le"}, {"file": "../signals.f64le"},
        {"file": "/signals.f64le"}, {"baseline_samples": 0}, {"sha256": "x" * 64},
    ]
    for index, mutation in enumerate(mutations):
        source = work / f"bad-corpus-{index}"
        shutil.copytree(corpus, source)
        (source / "manifest.json").write_text(json.dumps(dict(original, **mutation)))
        output = work / f"bad-corpus-out-{index}"
        summary = invoke(binary, source, output, expected=64)
        assert summary["produced"] == 0 and not output.exists()
        cases += 1
    for mode in ("short", "long", "manifest-large"):
        source = work / mode
        shutil.copytree(corpus, source)
        if mode == "manifest-large":
            (source / "manifest.json").write_text(" " * (64 * 1024 + 1))
        else:
            payload = (source / "signals.f64le").read_bytes()
            (source / "signals.f64le").write_bytes(payload[:-1] if mode == "short" else payload + b"\0")
        output = work / f"{mode}-out"
        summary = invoke(binary, source, output, expected=64)
        assert summary["produced"] == 0 and not output.exists()
        cases += 1

    # WSL's Windows mount cannot create FIFOs; use Linux temporary storage.
    with tempfile.TemporaryDirectory(prefix="helix-fifo-") as temporary:
        source = Path(temporary) / "corpus"
        shutil.copytree(corpus, source)
        (source / "signals.f64le").unlink()
        os.mkfifo(source / "signals.f64le")
        output = work / "fifo-out"
        summary = invoke(binary, source, output, expected=64)
        assert summary["produced"] == 0 and not output.exists()
        cases += 1

    # Sparse on disk; valid zeros still exercise the entire streaming hash/load.
    source = work / "corpus-limit"
    source.mkdir()
    payload_path = source / "signals.f64le"
    with payload_path.open("wb") as file:
        file.truncate(64 * 1024 * 1024)
    with payload_path.open("rb") as file:
        digest = hashlib.file_digest(file, "sha256").hexdigest()
    boundary_manifest = dict(original, rows=2048, samples=4096, baseline_samples=1, sha256=digest)
    encoded = json.dumps(boundary_manifest)
    (source / "manifest.json").write_text(encoded + " " * (64 * 1024 - len(encoded)))
    summary = invoke(binary, source, work / "corpus-limit-ok",
                     ["--events", "1", "--queue-capacity", "1", "--batch-size", "1"])
    readback(summary, source, [0])
    cases += 1
    output = work / "memory-limit"
    summary = invoke(binary, source, output, ["--events", "1", "--queue-capacity", "4096"], expected=64)
    assert "estimated data buffers" in summary["reason"] and not output.exists()
    cases += 1
    (source / "manifest.json").write_text(json.dumps(dict(boundary_manifest, rows=2049)))
    output = work / "corpus-too-large"
    summary = invoke(binary, source, output, expected=64)
    assert "exceeds 64 MiB" in summary["reason"] and not output.exists()
    cases += 1

    for all_invalid in (False, True):
        source = work / f"nan-{all_invalid}"
        shutil.copytree(corpus, source)
        _, data = load_corpus(source)
        data = data.copy()
        data[:, 0] = np.nan if all_invalid else data[:, 0]
        data[0, 0] = np.nan
        payload = data.tobytes()
        (source / "signals.f64le").write_bytes(payload)
        (source / "manifest.json").write_text(json.dumps(dict(original, sha256=hashlib.sha256(payload).hexdigest())))
        for mode in ("event", "batch"):
            summary = invoke(binary, source, work / f"nan-{all_invalid}-{mode}",
                             ["--events", "65", "--ffi", mode], expected=1)
            ids = [] if all_invalid else [i for i in range(65) if i % original["rows"] != 0]
            assert summary["produced"] == summary["accepted"] == 65
            assert summary["failed"] == 65 - len(ids) and summary["unwritten"] == 0
            assert summary["output"]["state"] == "partial"
            readback(summary, source, ids)
            if all_invalid:
                assert summary["metrics"]["latency_p50_ns"] is None
            cases += 1

    summary = invoke(binary, corpus, work / "missing-parent" / "out", expected=1)
    assert summary["status"] == "failed" and summary["produced"] == 0
    assert summary["output"]["state"] == "not_created"
    cases += 1

    def limited_file_size(size):
        def apply():
            signal.signal(signal.SIGXFSZ, signal.SIG_IGN)
            resource.setrlimit(resource.RLIMIT_FSIZE, (size, size))
        return apply

    output = work / "marker-failure"
    summary = invoke(binary, corpus, output, ["--events", "1"], expected=1,
                     preexec_fn=limited_file_size(128))
    assert summary["status"] == "failed" and summary["produced"] == 0
    assert "output startup" in summary["reason"]
    assert output.is_dir() and (output / "running.json").is_file()
    assert summary["output"]["state"] == "not_created"
    assert not (output / "summary.json").exists()
    cases += 1

    # One-row Parquet fits; JSON publication fails. Valid data remains written,
    # and stdout reports failure with a partial filename and reconciled counters.
    output = work / "summary-failure"
    summary = invoke(binary, corpus, output, ["--events", "1"], expected=1,
                     preexec_fn=limited_file_size(1700))
    assert "summary finalization" in summary["reason"]
    assert summary["produced"] == summary["accepted"] == summary["processed"] == summary["written"] == 1
    assert summary["unwritten"] == 0 and summary["output"]["state"] == "partial"
    assert not (output / "summary.json").exists()
    assert (output / "summary.json.incomplete").is_file()
    assert pq.read_table(summary["output"]["file"]).to_pylist()[0]["event_id"] == 0
    cases += 1

    summary = invoke(binary, corpus, work / "write-failure",
                     ["--events", "5000", "--batch-size", "7"], expected=1,
                     preexec_fn=limited_file_size(16 * 1024))
    assert summary["status"] == "failed" and summary["output"]["state"] == "incomplete"
    assert summary["written"] == 0 and summary["unwritten"] == summary["processed"]
    # ceil(4096/7)*7: this pinned PLAIN writer flushes only at a full row group.
    assert summary["processed"] == summary["accepted"] == summary["produced"] == 4102
    assert Path(summary["output"]["file"]).is_file()
    assert not (work / "write-failure" / "events.parquet").exists()
    cases += 1
    print(json.dumps(dict(status="PASS", pipeline_cases=cases, readback_events_per_ffi_mode=4101,
                          artifact_directory=str(work), atol=1e-10, rtol=1e-10)))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--corpus", required=True, type=Path)
    parser.add_argument("--artifacts", default=Path("artifacts"), type=Path)
    args = parser.parse_args()
    args.artifacts.mkdir(parents=True, exist_ok=True)
    work = Path(tempfile.mkdtemp(prefix="h2-", dir=args.artifacts)).resolve()
    validate(args.binary.resolve(), args.corpus.resolve(), work)
