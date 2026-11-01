#!/usr/bin/env python3
"""Repeatable CLI smoke benchmark. All generated data lives in a private temporary directory.
Not a UI benchmark. Warm OS caches, process launch included; small sample tails are not SLAs.
"""
import argparse
import json
import math
import pathlib
import statistics
import subprocess
import tempfile
import time


def measure(command, trials):
    samples = []
    for _ in range(trials):
        start = time.perf_counter_ns()
        subprocess.run(command, check=True, stdout=subprocess.DEVNULL)
        samples.append((time.perf_counter_ns() - start) / 1_000_000)
    samples.sort()
    return {
        "trials": trials,
        "p50_ms": round(statistics.median(samples), 3),
        "p95_ms": round(samples[math.ceil(trials * 0.95) - 1], 3),
        "p99_ms": round(samples[math.ceil(trials * 0.99) - 1], 3),
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", default="target/release/aurex-cli")
    parser.add_argument("--entries", type=int, default=1000)
    parser.add_argument("--trials", type=int, default=11)
    args = parser.parse_args()
    if not 1 <= args.entries <= 50000 or not 1 <= args.trials <= 1000:
        parser.error("entries must be 1..50000, trials 1..1000")
    binary = str(pathlib.Path(args.binary).resolve(strict=True))
    with tempfile.TemporaryDirectory(prefix="aurex-benchmark-") as temp:
        root = pathlib.Path(temp)
        directory = root / "files"
        directory.mkdir()
        for n in range(args.entries):
            (directory / f"file-{n:05}.txt").write_bytes(b"x" * 1024)
        (root / "payload").write_bytes(b"aurex\n" * 1_398_101)  # ~8 MiB
        metrics = {"entries": args.entries, "notes": "warm-cache CLI; includes process startup; not UI latency"}
        metrics["help_startup"] = measure([binary, "--help"], args.trials)
        metrics["directory_listing"] = measure([binary, temp, "list", "files"], args.trials)
        samples = []
        for n in range(args.trials):
            samples.append(measure([binary, temp, "--write", "copy", "payload", f"copy-{n}"], 1)["p50_ms"])
            (root / f"copy-{n}").unlink()
        samples.sort()
        metrics["verified_copy_approx_8_mib"] = {
            "trials": args.trials,
            "p50_ms": round(statistics.median(samples), 3),
            "p95_ms": samples[math.ceil(args.trials * 0.95) - 1],
            "p99_ms": samples[math.ceil(args.trials * 0.99) - 1],
        }
        print(json.dumps(metrics, indent=2))


if __name__ == "__main__":
    main()
