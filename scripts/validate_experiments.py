"""Adversarial H5 runner/analysis checks; synthetic fixtures are not measurements."""
import copy
import json
import tempfile
import unittest
from pathlib import Path

from experiment import BUDGET, ROOT, authenticate_payload, check_budget, execute, schedule, sha256, stable_resources
from experiment_analysis import analyze
from experiment_support import accounting, micro_design, pipeline_design, validate_micro
from oracle import features
from validate import load_corpus


class Experiments(unittest.TestCase):
    def test_micro_validation_rejects_checksum_preserving_errors_and_reordering(self):
        corpus = ROOT / "fixtures/small"
        manifest, data = load_corpus(corpus)
        tuples = []
        for row in data:
            value = features(row, manifest["baseline_samples"])
            tuples.append([value[k] for k in ("status", "baseline", "peak_amplitude", "peak_index", "integral")])
        summary = dict(events=len(data), repeat_consistent=True, verification=tuples)
        validate_micro(summary, corpus)
        corrupt = copy.deepcopy(summary)
        corrupt["verification"][0][1] += 1
        corrupt["verification"][0][2] -= 1
        with self.assertRaises(AssertionError):
            validate_micro(corrupt, corpus)
        reordered = copy.deepcopy(summary)
        reordered["verification"].reverse()
        with self.assertRaises(AssertionError):
            validate_micro(reordered, corpus)
        with self.assertRaises(AssertionError):
            validate_micro(dict(summary, repeat_consistent=False), corpus)

    def test_payload_reordering_is_rejected_even_with_same_values(self):
        with tempfile.TemporaryDirectory() as tmp:
            payload = Path(tmp) / "payload"
            payload.write_bytes(b"aaaabbbb")
            expected = sha256(payload)
            authenticate_payload(payload, expected)
            payload.write_bytes(b"bbbbaaaa")
            with self.assertRaises(RuntimeError):
                authenticate_payload(payload, expected)

    def test_single_axis_union_and_randomized_repetitions(self):
        configs = pipeline_design()
        self.assertEqual(len(configs), 28)
        self.assertEqual(len({json.dumps(c, sort_keys=True) for c in configs}), 28)
        for c in configs:
            self.assertLessEqual(sum((c["workers"] != 2, c["queue_capacity"] != 256, c["batch_size"] != 16)), 1)
        self.assertEqual(len(micro_design()), 96)
        jobs = schedule(False, 17)
        self.assertEqual(len(jobs), 744)
        self.assertEqual(jobs, schedule(False, 17))
        self.assertNotEqual(jobs, schedule(False, 18))
        self.assertEqual(sum(j["phase"] == "warmup" for j in jobs), 124)
        for rep in range(6):
            self.assertEqual(len({json.dumps(j["config"], sort_keys=True) for j in jobs if j["repetition"] == rep}), 124)

    def test_code_two_is_losses_and_failures_cannot_be_relabelled(self):
        summary = dict(dropped=3, produced=10, accepted=7, processed=7, written=7,
                       not_admitted=0, failed=0, aborted=0, unwritten=0,
                       config=dict(events=10, policy="drop-new"), status="completed_with_drops",
                       output=dict(state="finalized"), duration_ns=1000000000,
                       throughput_processed_per_second=7, throughput_written_per_second=7,
                       metrics=dict(latency_observations=7, latency_recorded=7, latency_overflow=0,
                                    actual_batch_sizes=[0, 7] + [0] * 63))
        accounting(summary, 2)
        for code in (0, 1, 130):
            with self.assertRaises(AssertionError):
                accounting(summary, code)
        for key, value in (("status", "completed"), ("accepted", 8), ("written", 6), ("aborted", 1)):
            changed = dict(summary, **{key: value})
            with self.assertRaises(AssertionError):
                accounting(changed, 2)
        summary["config"]["policy"] = "block"
        with self.assertRaises(AssertionError):
            accounting(summary, 2)

    def test_budget_and_resource_change_rejected(self):
        check_budget(BUDGET - 10, 10)
        with self.assertRaises(RuntimeError):
            check_budget(BUDGET - 10, 11)
        before = dict(cpus=8, affinity=list(range(8)), mem_total="8 GiB", swap_total="2 GiB", loadavg="1")
        self.assertTrue(stable_resources(before, dict(before, loadavg="2")))
        for key, value in (("cpus", 4), ("affinity", [0, 1]), ("mem_total", "4 GiB"), ("swap_total", "0 GiB")):
            self.assertFalse(stable_resources(before, dict(before, **{key: value})))

    def test_timeout_retains_logs_and_terminates_process_group(self):
        with tempfile.TemporaryDirectory() as tmp:
            prefix = Path(tmp) / "timeout"
            result = execute(["sleep", "10"], prefix, timeout=0.05, measured=True)
            self.assertTrue(result["timed_out"])
            self.assertNotEqual(result["exit_code"], 0)
            self.assertTrue(prefix.with_suffix(".command.json").exists())
            self.assertTrue(prefix.with_suffix(".stderr").exists())

    def test_analysis_excludes_warmup_and_rejects_incomplete_or_failed_runs(self):
        jobs = schedule(True, 17)
        metadata = dict(status="completed", smoke=True, repetitions=1, schedule=jobs, commit="synthetic-test-only")
        records = []
        for job in jobs:
            # Deliberately enormous warmup values detect contamination.
            amount = 999999 if job["phase"] == "warmup" else 10
            records.append(dict(job, validated=True, peak_rss_kib=amount,
                                summary=dict(events=1, duration_ns=amount,
                                    throughput_processed_per_second=amount,
                                    throughput_written_per_second=amount, dropped=0, produced=1),
                                raw_latency=dict(population=1, small_population=True,
                                    histogram_overflow=0, p50_ns=1, p95_ns=1, p99_ns=1)))
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            def write(meta, rows):
                (root / "campaign.json").write_text(json.dumps(meta))
                (root / "runs.jsonl").write_text("\n".join(json.dumps(r) for r in rows))
            write(metadata, records)
            result = analyze(root)
            self.assertEqual(result["evidence"], "smoke_only")
            self.assertTrue(all(c["peak_rss_kib"]["median"] == 10 for c in result["configurations"]))
            for meta, rows in ((dict(metadata, status="failed"), records),
                               (metadata, records[:-1]),
                               (dict(metadata, repetitions=5), records)):
                write(meta, rows)
                with self.assertRaises(AssertionError):
                    analyze(root)
            failed = copy.deepcopy(records)
            failed[10]["validated"] = False
            write(metadata, failed)
            with self.assertRaises(AssertionError):
                analyze(root)
            duplicate = copy.deepcopy(records)
            duplicate[-1] = duplicate[-2]
            write(metadata, duplicate)
            with self.assertRaises(AssertionError):
                analyze(root)


if __name__ == "__main__":
    unittest.main()
