#!/usr/bin/env python3
import json
from pathlib import Path
import platform
import subprocess

root = Path(__file__).resolve().parents[1]
records = []
for test in ["tune", "density"]:
    log = subprocess.check_output(["cargo", "test", "--offline", "--release", "--test", "benchmark_gf2", test, "--", "--ignored", "--nocapture", "--test-threads=1"], cwd=root, text=True)
    shape = None
    for line in log.splitlines():
        for marker in ("shape,", "density_mode,"):
            if marker in line:
                line = line[line.index(marker):]
                break
        fields = line.split(",")
        if fields[0] == "shape":
            shape = list(map(int, fields[1:]))
            records.append(dict(shape=shape, median_ms={}))
        elif fields[0] == "density_mode":
            records.append(dict(shape=[4096] * 3, density_mode=int(fields[1]), median_ms={}))
        elif len(fields) == 2 and records and (fields[0] in ("old", "rowxor", "selected") or "/" in fields[0]):
            records[-1]["median_ms"][fields[0]] = float(fields[1])
    print(log)
result = dict(platform=platform.platform(), rustc=subprocess.check_output(["rustc", "-V"], text=True).strip(), repetitions=3, warmup=1, unit="ms", includes="product allocation and preprocessing; excludes input/output", records=records)
(root / "tests/gf2_benchmark_results.json").write_text(json.dumps(result, indent=2) + "\n")
