#!/usr/bin/env python3
import argparse
import csv
import hashlib
import io
import json
import platform
import statistics
import subprocess
import tempfile
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
p = argparse.ArgumentParser()
p.add_argument('--toolchain', default='1.89.0')
args = p.parse_args()
rustc = ['rustc'] + (['+' + args.toolchain] if args.toolchain else [])
with tempfile.TemporaryDirectory(prefix='intmap-') as tmp:
    exe = str(Path(tmp) / 'benchmark')
    subprocess.run(rustc + ['--edition=2021', '-O', str(ROOT / 'tests/benchmark_intmap.rs'), '-o', exe], check=True)
    result = subprocess.run([exe], text=True, capture_output=True, check=True)
rows = list(csv.DictReader(io.StringIO(result.stdout)))
assert len(rows) == 270, len(rows)
(ROOT / 'tests/intmap_benchmark_results.csv').write_text(result.stdout)
summary = []
for n in ['1024', '100000', '1000000']:
    for operation in dict.fromkeys(r['operation'] for r in rows):
        for patterns in [('sequential', 'random'), ('shifted',)]:
            ratios = [float(r['speedup']) for r in rows if r['n'] == n and r['operation'] == operation and r['pattern'] in patterns]
            summary.append(dict(n=int(n), operation=operation, patterns=patterns, minimum=min(ratios), median=statistics.median(ratios), maximum=max(ratios)))
metadata = dict(
    measured_at_utc=datetime.now(timezone.utc).isoformat(),
    compiler=subprocess.check_output(rustc + ['-Vv'], text=True),
    platform=platform.platform(), machine=platform.machine(),
    flags=['--edition=2021', '-O'],
    repetitions=6, warmups=1,
    note='Alternating backend order; median of six measurements after one warmup; setup and destruction excluded. All keys and miss probes are disjoint. Checksums and final contents checked by commutative checksums. Speedup = FxMap time / IntMap time.',
    sources={str(path): hashlib.sha256((ROOT / path).read_bytes()).hexdigest() for path in [Path('src/Basic/fxhash.rs'), Path('src/DataStructure/intmap.rs'), Path('tests/benchmark_intmap.rs')]},
    summary=summary,
)
(ROOT / 'tests/intmap_benchmark_results.json').write_text(json.dumps(metadata, ensure_ascii=False, indent=2) + '\n')
for row in summary:
    if row['patterns'] == ('sequential', 'random'):
        print(row['n'], row['operation'], f"{row['minimum']:.2f} / {row['median']:.2f} / {row['maximum']:.2f}", flush=True)
