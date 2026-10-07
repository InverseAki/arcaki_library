"""Four-BIT integer/modint timing and per-process peak RSS (macOS bytes)."""
import json
import os
from pathlib import Path
import platform
import statistics
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parent
BINARY = Path(tempfile.gettempdir()) / "static_rectangle_bench"
subprocess.run(["rustc", "--edition=2021", "-O", str(ROOT / "static_rectangle_add_rectangle_sum.rs"), "-o", str(BINARY)], check=True)
results = []
for n, q, mode in [(200000,200000,"mixed"),(200000,200000,"ties"),(200000,10000,"mixed"),(10000,200000,"mixed")]:
    grouped = {m: [] for m in ["exact", "modular"]}
    checksums = set()
    for repeat in range(3):
        methods = list(grouped) if repeat % 2 == 0 else list(grouped)[::-1]
        for method in methods:
            p = subprocess.Popen([str(BINARY),method,mode,str(n),str(q)],stdout=subprocess.PIPE,text=True)
            _, status, usage = os.wait4(p.pid,0)
            p.returncode = os.waitstatus_to_exitcode(status)
            assert p.returncode == 0
            seconds, checksum, answers = p.stdout.read().split()
            p.stdout.close()
            checksums.add((checksum,answers))
            grouped[method].append({"seconds":float(seconds),"rss_mib":usage.ru_maxrss/1024**2})
    assert len(checksums)==1
    row = {"rectangles":n,"queries":q,"mode":mode,"methods":{
        m:{"median_seconds":statistics.median(v["seconds"] for v in runs),"max_rss_mib":max(v["rss_mib"] for v in runs),"runs":runs}
        for m,runs in grouped.items()
    }}
    results.append(row)
    print(json.dumps(row),flush=True)
output = {"platform":platform.platform(),"rustc":subprocess.check_output(["rustc","-V"],text=True).strip(),"seed":123456789,"repetitions":3,"timing":"registration/build plus solve; excludes generation, process startup, compilation and I/O","results":results}
(ROOT/"static_rectangle_add_rectangle_sum_benchmark_results.json").write_text(json.dumps(output,indent=2)+"\n")
