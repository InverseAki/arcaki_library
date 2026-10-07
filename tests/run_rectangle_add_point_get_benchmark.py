"""Registration/build + solve comparison; wait4 RSS uses bytes on macOS."""
import json
import os
from pathlib import Path
import platform
import statistics
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parent
BINARY = Path(tempfile.gettempdir()) / "rectangle_add_bench"
subprocess.run(["rustc", "--edition=2021", "-O", str(ROOT / "rectangle_add_point_get.rs"), "-o", str(BINARY)], check=True)
methods = ["original", "prepend", "build"]
results = []
for n, q, mode in [
    (0, 200000, "mixed"),
    (100000, 100000, "static"),
    (100000, 100000, "mixed"),
    (100000, 100000, "ties"),
    (100000, 100000, "add_heavy"),
    (100000, 100000, "query_heavy"),
    (400000, 100000, "mixed"),
    (100000, 400000, "mixed"),
    (500000, 500000, "static"),
    (500000, 500000, "mixed"),
    (1000000, 10000, "mixed"),
]:
    grouped = {m: [] for m in methods}
    checksums = set()
    for repeat in range(3):
        for method in methods[repeat:] + methods[:repeat]:
            p = subprocess.Popen([str(BINARY), method, mode, str(n), str(q)], stdout=subprocess.PIPE, text=True)
            _, status, usage = os.wait4(p.pid, 0)
            p.returncode = os.waitstatus_to_exitcode(status)
            assert p.returncode == 0
            seconds, checksum, answers = p.stdout.read().split()
            p.stdout.close()
            checksums.add((checksum, answers))
            grouped[method].append({"seconds": float(seconds), "rss_mib": usage.ru_maxrss / 1024**2})
    assert len(checksums) == 1, (n, q, mode)
    row = {"initial_rectangles": n, "operations": q, "mode": mode, "methods": {
        m: {"median_seconds": statistics.median(r["seconds"] for r in runs), "max_rss_mib": max(r["rss_mib"] for r in runs), "runs": runs}
        for m, runs in grouped.items()
    }}
    results.append(row)
    print(json.dumps({"N": n, "Q": q, "mode": mode, **{m: round(v["median_seconds"], 6) for m, v in row["methods"].items()}}), flush=True)
output = {"platform": platform.platform(), "rustc": subprocess.check_output(["rustc", "-V"], text=True).strip(), "seed": 123456789, "repetitions": 3, "timing": "initial registration/build and operation registration plus solve; excludes generation, startup and I/O", "results": results}
(ROOT / "rectangle_add_point_get_benchmark_results.json").write_text(json.dumps(output, indent=2) + "\n")
