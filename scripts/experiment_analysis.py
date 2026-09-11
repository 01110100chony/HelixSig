"""Analyze complete H5 campaigns without treating smoke as performance evidence."""
import argparse
import json
from collections import defaultdict
from pathlib import Path

import numpy as np
from experiment_support import micro_design, pipeline_design


def dispersion(values):
    values = np.asarray(values, dtype=float)
    assert len(values) and np.all(np.isfinite(values))
    return dict(n=len(values), minimum=float(values.min()),
                q1=float(np.quantile(values, .25)), median=float(np.median(values)),
                q3=float(np.quantile(values, .75)), maximum=float(values.max()))


def analyze(directory):
    directory = Path(directory)
    metadata = json.loads((directory / "campaign.json").read_text())
    records = [json.loads(line) for line in (directory / "runs.jsonl").read_text().splitlines()]
    assert metadata["status"] == "completed", "incomplete campaign cannot support conclusions"
    assert len(records) == len(metadata["schedule"])
    assert metadata["repetitions"] == (1 if metadata["smoke"] else 5)
    expected_configs = micro_design() + [c for c in pipeline_design() if not metadata["smoke"] or
        (c["workers"], c["queue_capacity"], c["batch_size"]) == (2, 256, 16)]
    expected_keys = {json.dumps(c, sort_keys=True) for c in expected_configs}
    assert len(records) == len(expected_configs) * (metadata["repetitions"] + 1)
    groups = defaultdict(list)
    seen = set()
    for record, scheduled in zip(records, metadata["schedule"]):
        assert record["id"] == scheduled["id"] and record["config"] == scheduled["config"]
        assert record["phase"] == scheduled["phase"]
        assert record["repetition"] == scheduled["repetition"]
        key = json.dumps(record["config"], sort_keys=True)
        assert (key, record["repetition"]) not in seen, "duplicate repetition"
        seen.add((key, record["repetition"]))
        assert record["phase"] == ("warmup" if record["repetition"] == 0 else "measured")
        assert record["validated"], "failed runs must not be silently filtered"
        if record["phase"] == "measured":
            groups[json.dumps(record["config"], sort_keys=True)].append(record)
    assert set(groups) == expected_keys
    assert seen == {(key, rep) for key in expected_keys for rep in range(metadata["repetitions"] + 1)}
    results = []
    for key, runs in groups.items():
        assert len(runs) == metadata["repetitions"]
        config = json.loads(key)
        result = dict(config=config, peak_rss_kib=dispersion([r["peak_rss_kib"] for r in runs]))
        if config["kind"] == "micro":
            result["ns_per_event"] = dispersion([r["summary"]["duration_ns"] / r["summary"]["events"] for r in runs])
        else:
            for metric in ("throughput_processed_per_second", "throughput_written_per_second"):
                result[metric] = dispersion([r["summary"][metric] for r in runs])
            result["drop_fraction"] = dispersion([r["summary"]["dropped"] / r["summary"]["produced"] for r in runs])
            # Individual empirical percentiles, never an average of percentiles.
            result["latency_by_run"] = [dict(id=r["id"], **r["raw_latency"]) for r in runs]
        results.append(result)
    output = dict(schema_version=1, evidence="smoke_only" if metadata["smoke"] else "measured",
                  commit=metadata["commit"], repetitions=metadata["repetitions"],
                  runs=len(records), configurations=results)
    (directory / "analysis.json").write_text(json.dumps(output, indent=2, allow_nan=False) + "\n")
    lines = ["# H5 experiment analysis", "", f"Evidence: **{output['evidence']}**. Commit: `{output['commit']}`.",
             f"{len(records)} runs; {len(results)} configurations; {metadata['repetitions']} measured repetitions.", "",
             "Median [Q1, Q3] across measured runs. Warmups are excluded. No minimum speedup is required.", "",
             "## Pipeline", "", "| W | Q | B | Policy | FFI | Written events/s | Drop fraction |",
             "|---|---|---|---|---|---|---|"]
    for result in results:
        c = result["config"]
        if c["kind"] == "pipeline":
            rate = result["throughput_written_per_second"]
            loss = result["drop_fraction"]
            lines.append(f"| {c['workers']} | {c['queue_capacity']} | {c['batch_size']} | {c['policy']} | {c['ffi']} | "
                         f"{rate['median']:.0f} [{rate['q1']:.0f}, {rate['q3']:.0f}] | {loss['median']:.4f} [{loss['q1']:.4f}, {loss['q3']:.4f}] |")
    lines += ["", "## Microbenchmark", "", "| N | B | Engine | FFI | Preparation | ns/event |", "|---|---|---|---|---|---|"]
    for result in results:
        c = result["config"]
        if c["kind"] == "micro":
            cost = result["ns_per_event"]
            lines.append(f"| {c['samples']} | {c['batch_size']} | {c['engine']} | {c['ffi']} | {c['preparation']} | "
                         f"{cost['median']:.1f} [{cost['q1']:.1f}, {cost['q3']:.1f}] |")
    lines += ["", "## Interpretation limits", "",
              "These are local finite-replay observations. WSL scheduling, competing host load, cache state and the writer affect results. "
              "The pipeline includes writing; the microbenchmark excludes I/O and allocation, includes result consumption, "
              "and pack mode includes the row-to-flat copy. Differences between executables are not exact isolated FFI overhead. "
              "Single-axis comparisons do not establish interactions. Five runs do not characterize production tails or hard real time.", "",
              "Raw Parquet retains successful-event latencies only. Per-run empirical p50/p95/p99, population, small-population flags "
              "and histogram overflow are in analysis.json; percentiles are never averaged across runs. Loss fractions must accompany "
              "drop-new throughput. Peak Linux process RSS is not a whole-system or Windows-host memory measure.", ""]
    (directory / "analysis.md").write_text("\n".join(lines))
    return output


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("campaign", type=Path)
    print(json.dumps(analyze(parser.parse_args().campaign), allow_nan=False))
