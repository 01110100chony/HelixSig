"""Build, execute and preserve the preregistered H5 Linux experiment campaign."""
import argparse
import hashlib
import json
import math
import os
import platform
import random
import shutil
import signal
import subprocess
import sys
import time
from pathlib import Path

from experiment_analysis import analyze
from experiment_support import accounting, expected_checksum, micro_design, pipeline_design, validate_rows, validate_micro
from generate_data import generate

ROOT = Path(__file__).resolve().parents[1]
BUDGET = 4 * 1024**3


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2, allow_nan=False) + "\n")


def query(command):
    return subprocess.run(command, cwd=ROOT, check=True, capture_output=True,
                          text=True, timeout=30).stdout.strip()


def sha256(path):
    with Path(path).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def authenticate_payload(path, expected):
    actual = sha256(path)
    if actual != expected:
        raise RuntimeError("corpus SHA-256 differs from the generated manifest")
    return actual


def authenticate_corpus(corpus, expected_payload, expected_manifest):
    return dict(input_sha256=authenticate_payload(corpus / "signals.f64le", expected_payload),
                input_manifest_sha256=authenticate_payload(corpus / "manifest.json", expected_manifest))


def resources():
    mem = dict(line.split(":", 1) for line in Path("/proc/meminfo").read_text().splitlines())
    vm = dict(line.split() for line in Path("/proc/vmstat").read_text().splitlines())
    return dict(cpus=os.cpu_count(), affinity=sorted(os.sched_getaffinity(0)),
                mem_total=mem["MemTotal"].strip(), swap_total=mem["SwapTotal"].strip(),
                mem_available=mem["MemAvailable"].strip(), swap_free=mem["SwapFree"].strip(),
                swap_in_pages=int(vm["pswpin"]), swap_out_pages=int(vm["pswpout"]),
                loadavg=Path("/proc/loadavg").read_text().strip(),
                linux_cpu_counters=Path("/proc/stat").read_text().splitlines()[0],
                time_utc=time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()))


def stable_resources(before, after):
    return all(before[k] == after[k] for k in ("cpus", "affinity", "mem_total", "swap_total"))


def disk_bytes(path):
    return sum(p.stat().st_size for p in path.rglob("*") if p.is_file())


def check_budget(used, reserve=0):
    if used + reserve > BUDGET:
        raise RuntimeError("campaign would exceed the 4 GiB artifact budget")


def linux_path(path):
    path = Path(path).resolve()
    while not path.exists():
        path = path.parent
    kind = query(["findmnt", "-n", "-o", "FSTYPE", "-T", str(path)])
    if str(path).startswith("/mnt/") or kind in ("9p", "drvfs", "ntfs", "ntfs3", "fuseblk"):
        raise RuntimeError("performance source/build/data must be on the Linux filesystem")


def execute(command, prefix, *, timeout, env=None, measured=False):
    """Bounded process group; a timeout cannot leave its measured child running."""
    prefix.parent.mkdir(parents=True, exist_ok=True)
    actual = command
    if measured:
        actual = ["/usr/bin/time", "-f", "%M", "-o", str(prefix.with_suffix(".rss")), "--", *command]
    write_json(prefix.with_suffix(".command.json"), dict(argv=actual, cwd=str(ROOT)))
    start = time.monotonic_ns()
    timed_out = False
    with prefix.with_suffix(".stdout").open("w") as stdout, prefix.with_suffix(".stderr").open("w") as stderr:
        process = subprocess.Popen(actual, cwd=ROOT, env=env, stdout=stdout, stderr=stderr,
                                   start_new_session=True)
        try:
            code = process.wait(timeout=timeout)
        except BaseException as error:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait(timeout=10)
            if not isinstance(error, subprocess.TimeoutExpired):
                raise
            code, timed_out = process.returncode, True
    rss_file = prefix.with_suffix(".rss")
    rss = None
    if measured and rss_file.exists():
        lines = rss_file.read_text().splitlines()
        if lines and lines[-1].isdigit():
            rss = int(lines[-1])
    return dict(exit_code=code, timed_out=timed_out, wall_duration_ns=time.monotonic_ns() - start,
                peak_rss_kib=rss, command=command)


def schedule(smoke, seed):
    configs = micro_design() + [c for c in pipeline_design() if not smoke or
        (c["workers"], c["queue_capacity"], c["batch_size"]) == (2, 256, 16)]
    rng = random.Random(seed)
    jobs = []
    for repetition in range(2 if smoke else 6):
        order = list(configs)
        rng.shuffle(order)
        for config in order:
            jobs.append(dict(id=f"run-{len(jobs):04d}", config=config,
                             phase="warmup" if repetition == 0 else "measured", repetition=repetition))
    return jobs


def campaign(out, smoke=False, seed=20260910):
    if sys.platform != "linux":
        raise RuntimeError("H5 requires Linux")
    out = Path(out).resolve()
    dirty = query(["git", "status", "--porcelain"])
    commit = query(["git", "rev-parse", "HEAD"])
    if not smoke:
        if dirty:
            raise RuntimeError("performance evidence requires a clean committed source tree")
        linux_path(ROOT)
        linux_path(out)
    initial_resources = resources()
    if len(initial_resources["affinity"]) < (2 if smoke else 4):
        raise RuntimeError("insufficient available CPUs for the required worker design")
    if shutil.disk_usage(out.parent).free < BUDGET + 1024**3:
        raise RuntimeError("insufficient free space for campaign budget and headroom")
    out.mkdir(exist_ok=False)
    events = 257 if smoke else 100000
    jobs = schedule(smoke, seed)
    check_budget(sum(events * 80 + 2 * 1024**2 for j in jobs if j["config"]["kind"] == "pipeline") + 1024**3)
    compiler = shutil.which("g++")
    if not compiler:
        raise RuntimeError("g++ is required for the matched C++ release builds")
    overrides = dict(CXX=compiler, CXXFLAGS="-O3 -ffp-contract=off", CFLAGS="-O3",
                     RUSTFLAGS="-C opt-level=3 -C target-cpu=x86-64 -C lto=off",
                     CARGO_TARGET_DIR=str(out / "build-rust"), CARGO_BUILD_JOBS="2",
                     CARGO_PROFILE_RELEASE_LTO="false", CARGO_PROFILE_RELEASE_OPT_LEVEL="3")
    env = {k: v for k, v in os.environ.items() if not k.startswith(("CARGO_PROFILE_", "CMAKE_"))
           and k not in ("CARGO_ENCODED_RUSTFLAGS", "RUSTFLAGS", "CXXFLAGS", "CFLAGS", "CPPFLAGS", "LDFLAGS")}
    env.update(overrides)
    metadata = dict(schema_version=1, status="running", smoke=smoke, seed=seed, events=events,
                    repetitions=1 if smoke else 5, warmups=1, budget_bytes=BUDGET,
                    commit=commit, dirty_tree=dirty, source=str(ROOT), raw_directory=str(out),
                    initial_resources=initial_resources, schedule=jobs, build_environment=overrides,
                    platform=platform.platform(), kernel=platform.release(),
                    python=sys.version, cpu=query(["lscpu"]),
                    competing_processes_start=query(["ps", "-eo", "pid,comm,pcpu,pmem", "--sort=-pcpu"]),
                    versions={name: query(command) for name, command in {
                        "rustc": ["rustc", "-vV"], "cargo": ["cargo", "--version"],
                        "cpp": [compiler, "--version"], "cmake": ["cmake", "--version"],
                        "python_packages": [sys.executable, "-m", "pip", "freeze"]}.items()})
    write_json(out / "campaign.json", metadata)
    try:
        commands = [
            ["cargo", "build", "--release", "--locked", "--bin", "helix", "--example", "ffi_bench", "-j", "2", "-vv"],
            ["cmake", "-S", "cpp", "-B", str(out / "build-native"), "-DCMAKE_BUILD_TYPE=Release",
             f"-DCMAKE_CXX_COMPILER={compiler}", "-DCMAKE_CXX_FLAGS_RELEASE=-O3 -DNDEBUG",
             "-DCMAKE_INTERPROCEDURAL_OPTIMIZATION=OFF", "-DCMAKE_EXPORT_COMPILE_COMMANDS=ON",
             "-DHELIX_BENCHMARKS=ON", "-DBUILD_TESTING=OFF"],
            ["cmake", "--build", str(out / "build-native"), "-j", "2"]]
        for i, command in enumerate(commands):
            result = execute(command, out / f"build-{i}", timeout=1200, env=env)
            assert result["exit_code"] == 0 and not result["timed_out"], result
        binaries = dict(pipeline=out / "build-rust/release/helix",
                        rust_ffi=out / "build-rust/release/examples/ffi_bench",
                        native=out / "build-native/kernel_bench")
        metadata["binaries"] = {name: dict(path=str(path), sha256=sha256(path)) for name, path in binaries.items()}
        metadata["corpora"] = {}
        metadata["manifest_sha256"] = {}
        checksums = {}
        for width in (64, 256, 4096):
            corpus = out / "corpora" / str(width)
            metadata["corpora"][str(width)] = generate(corpus, samples=width)
            metadata["manifest_sha256"][str(width)] = sha256(corpus / "manifest.json")
            checksums[width] = expected_checksum(corpus, events)
        write_json(out / "campaign.json", metadata)
        with (out / "runs.jsonl").open("x") as records:
            for job in jobs:
                config = job["config"]
                prefix = out / "runs" / job["id"]
                corpus = out / "corpora" / str(config["samples"])
                reserve = events * 80 + 2 * 1024**2 if config["kind"] == "pipeline" else 65536
                check_budget(disk_bytes(out), reserve)
                before = resources()
                assert stable_resources(initial_resources, before), "effective resources changed"
                if config["kind"] == "pipeline":
                    command = [str(binaries["pipeline"]), "run", "--corpus", str(corpus), "--out", str(prefix),
                               "--execution", "concurrent", "--events", str(events)]
                    for key in ("workers", "queue_capacity", "batch_size", "policy", "ffi"):
                        command += ["--" + key.replace("_", "-"), str(config[key])]
                elif config["engine"] == "rust_ffi":
                    command = [str(binaries["rust_ffi"]), "--corpus", str(corpus), "--events", str(events)]
                    for key in ("batch_size", "ffi", "preparation"):
                        command += ["--" + key.replace("_", "-"), str(config[key])]
                else:
                    command = [str(binaries["native"]), str(corpus / "signals.f64le"), str(config["samples"]),
                               "32", str(config["batch_size"]), str(events), config["ffi"], config["preparation"]]
                record = dict(job, validated=False, resources_before=before)
                try:
                    record.update(authenticate_corpus(corpus,
                        metadata["corpora"][str(config["samples"])]["sha256"],
                        metadata["manifest_sha256"][str(config["samples"])]))
                    record.update(execute(command, prefix, timeout=180, measured=True))
                    authenticate_corpus(corpus, record["input_sha256"], record["input_manifest_sha256"])
                    record["resources_after"] = resources()
                    assert stable_resources(before, record["resources_after"]), "effective resources changed"
                    assert not record["timed_out"] and record["peak_rss_kib"] is not None
                    summary = json.loads(prefix.with_suffix(".stdout").read_text())
                    record["summary"] = summary
                    if config["kind"] == "pipeline":
                        assert json.loads((prefix / "summary.json").read_text()) == summary
                        assert all(summary["config"][key] == config[key] for key in
                                   ("workers", "queue_capacity", "batch_size", "policy", "ffi"))
                        assert summary["config"]["events"] == events
                        accounting(summary, record["exit_code"])
                        record["raw_latency"] = validate_rows(summary, corpus)
                    else:
                        assert record["exit_code"] == 0 and summary["events"] == events and summary["duration_ns"] > 0
                        assert all(summary[key] == config[key] for key in ("engine", "samples", "batch_size", "ffi", "preparation"))
                        batches = summary["actual_batch_sizes"]
                        assert len(batches) == 65 and batches[0] == 0
                        assert sum(i * n for i, n in enumerate(batches)) == events
                        assert not any(batches[config["batch_size"] + 1:])
                        if config["engine"] == "rust_ffi":
                            assert summary["corpus_sha256"] == metadata["corpora"][str(config["samples"])]["sha256"]
                        assert math.isclose(summary["checksum"], checksums[config["samples"]], rel_tol=1e-10, abs_tol=1e-10)
                        validate_micro(summary, corpus)
                    record["validated"] = True
                except BaseException as error:
                    record["error"] = repr(error)
                    raise
                finally:
                    records.write(json.dumps(record, allow_nan=False) + "\n")
                    records.flush()
                if int(job["id"].split("-")[1]) % 25 == 0:
                    print(f"{job['id']}/{len(jobs)} validated ({job['phase']})", flush=True)
        assert query(["git", "rev-parse", "HEAD"]) == commit
        if not smoke:
            assert not query(["git", "status", "--porcelain"]), "source changed during measurement"
        assert all(sha256(path) == metadata["binaries"][name]["sha256"] for name, path in binaries.items())
        metadata["final_resources"] = resources()
        metadata["competing_processes_end"] = query(["ps", "-eo", "pid,comm,pcpu,pmem", "--sort=-pcpu"])
        assert stable_resources(initial_resources, metadata["final_resources"])
        metadata["artifact_bytes"] = disk_bytes(out)
        check_budget(metadata["artifact_bytes"])
        metadata["status"] = "completed"
        write_json(out / "campaign.json", metadata)
        analyze(out)
    except BaseException as error:
        metadata["status"] = "failed"
        metadata["error"] = repr(error)
        write_json(out / "campaign.json", metadata)
        raise
    print(json.dumps(dict(status="completed", smoke=smoke, runs=len(jobs), artifacts=str(out))))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", required=True, type=Path)
    parser.add_argument("--smoke", action="store_true")
    parser.add_argument("--seed", type=int, default=20260910)
    args = parser.parse_args()
    campaign(args.out, args.smoke, args.seed)
