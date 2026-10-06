"""Local deterministic benchmark; subprocess peak RSS via wait4 (macOS bytes)."""
import json
import os
from pathlib import Path
import platform
import statistics
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parent
BINARY = Path(tempfile.gettempdir()) / "point_rectangle_bench"
subprocess.run(["rustc", "--edition=2021", "-O", str(ROOT / "point_add_rectangle_sum_research.rs"), "-o", str(BINARY)], check=True)
results = []
for n, modes in [
    (200000, ["mixed", "initial", "ties", "sorted", "add_heavy", "query_heavy", "query_first", "only_add", "only_query"]),
    (1000000, ["mixed", "initial", "add_heavy", "query_heavy"]),
]:
    grouped = {method: [] for method in ["baseline", "presort", "add_only", "compressed", "shared"]}
    for repeat in range(3):
        for method in list(grouped)[repeat:] + list(grouped)[:repeat]:
            for mode in modes:
                process = subprocess.Popen([str(BINARY), method, mode, str(n)], stdout=subprocess.PIPE, text=True)
                _, status, usage = os.wait4(process.pid, 0)
                process.returncode = os.waitstatus_to_exitcode(status)
                assert process.returncode == 0
                elapsed, checksum, answers = process.stdout.read().split()
                process.stdout.close()
                grouped[method].append({"mode": mode, "seconds": float(elapsed), "rss_bytes": usage.ru_maxrss, "checksum": checksum, "answers": int(answers)})
    for mode in modes:
        row = {"operations": n, "mode": mode, "methods": {}}
        checks = set()
        for method, runs in grouped.items():
            runs = [r for r in runs if r["mode"] == mode]
            checks.update((r["checksum"], r["answers"]) for r in runs)
            row["methods"][method] = {
                "median_seconds": statistics.median(r["seconds"] for r in runs),
                "max_rss_mib": max(r["rss_bytes"] for r in runs) / 1024**2,
                "runs": runs,
            }
        assert len(checks) == 1, row
        results.append(row)
        print(json.dumps({"operations": n, "mode": mode, **{m: round(v["median_seconds"], 6) for m, v in row["methods"].items()}}), flush=True)
output = {"platform": platform.platform(), "rustc": subprocess.check_output(["rustc", "-V"], text=True).strip(), "seed": 123456789, "repetitions": 3, "timing": "operation registration/conversion plus solve; excludes generation, compile and process startup", "results": results}
(ROOT / "point_add_rectangle_sum_research_results.json").write_text(json.dumps(output, indent=2) + "\n")
