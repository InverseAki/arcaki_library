#!/usr/bin/env python3
import argparse
import datetime
import json
import os
from pathlib import Path
import platform
import subprocess

parser = argparse.ArgumentParser()
parser.add_argument("--mode", choices=["candidates", "adaptive", "blocked", "final"], default="final")
parser.add_argument("--full", action="store_true")
parser.add_argument("--all-types", action="store_true")
parser.add_argument("--import-log", type=Path)
args = parser.parse_args()
root = Path(__file__).resolve().parents[1]
env = os.environ.copy()
for key in ["SORT_FULL", "SORT_ADAPTIVE", "SORT_BLOCK", "SORT_SELECTED", "SORT_ALL_TYPES"]:
    env.pop(key, None)
if args.full:
    env["SORT_FULL"] = "1"
if args.all_types:
    env["SORT_ALL_TYPES"] = "1"
if args.mode != "candidates":
    env[{"adaptive": "SORT_ADAPTIVE", "blocked": "SORT_BLOCK", "final": "SORT_SELECTED"}[args.mode]] = "1"
command = ["cargo", "test", "--offline", "--release", "--test", "benchmark_integer_sort", "--", "--ignored", "--nocapture", "--test-threads=1"]
if args.import_log:
    log = args.import_log.read_text()
else:
    log = subprocess.check_output(command, cwd=root, env=env, text=True)
    print(log, end="")
records = []
for line in log.splitlines():
    if line.startswith("RESULT,"):
        _, ty, n, distribution, method, median = line.split(",")
        records.append(dict(type=ty, n=int(n), distribution=distribution, method=method, median_us=float(median)))
if not records or "test result: ok." not in log:
    raise RuntimeError("No complete successful benchmark result")
result = dict(
    date=datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=9))).isoformat(),
    platform=platform.platform(),
    rustc=subprocess.check_output(["rustc", "-Vv"], text=True).strip(),
    mode=args.mode,
    full=args.full,
    all_types=args.all_types,
    seed=20261021,
    repetitions=5 if args.full else 3,
    warmup=1,
    batch="max(1, 8192 // max(n, 1))",
    unit="microseconds",
    timing="sort only, including allocation, initialization, analysis, and final copy; excludes input generation, input reset, validation, and I/O",
    reused="buffer allocated by untimed warmup, then retained across repetitions of the same case",
    order="methods in fixed order; one process and one test thread; every method checked against sort_unstable before timing",
    command=command,
    records=records,
)
output = root / "tests" / f"integer_sort_benchmark_{args.mode}.json"
output.write_text(json.dumps(result, indent=2) + "\n")
print(f"Saved {len(records)} measurements to {output}")
