"""Reproduce candidates, validation, and benchmarks without Cargo dependencies."""
import argparse
import csv
import json
import math
import platform
import statistics
import subprocess
import tempfile
import hashlib
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent
parser = argparse.ArgumentParser()
parser.add_argument('--toolchain', default='1.89.0')
parser.add_argument('--native', action='store_true')
parser.add_argument('--count', type=int, default=1_000_000)
parser.add_argument('--repeats', type=int, default=7)
parser.add_argument('--edges', action='store_true', help='Also measure empty/singleton/boundary/extrema and no-op updates')
parser.add_argument('--current-only', action='store_true', help='Compare public implementation with the original baseline')
args = parser.parse_args()
subprocess.run(['python3', str(ROOT / 'prepare.py')], check=True)
compiler = ['rustc', '+' + args.toolchain]
flags = ['--edition=2021', '-Awarnings']
if args.native:
    flags += ['-C', 'target-cpu=native']
name = args.toolchain + ('_native' if args.native else '_default')
if args.current_only:
    name += '_current'
metadata = {
    'compiler': subprocess.check_output(compiler + ['-Vv'], text=True),
    'platform': platform.platform(),
    'flags': flags + ['-O'],
    'count_per_case': args.count,
    'repeats': args.repeats,
    'date': '2026-10-07',
    'source_sha256': {str(p.name): hashlib.sha256(p.read_bytes()).hexdigest() for p in [ROOT / 'baseline.rs', ROOT.parents[1] / 'src/Basic/predecessor64.rs']},
}
with tempfile.TemporaryDirectory(prefix='predecessor64-') as tmp:
    for mode, extra in [('debug', []), ('release', ['-O'])]:
        binary = str(Path(tmp) / ('model-' + mode))
        model = ROOT.parent / 'predecessor64.rs' if args.current_only else ROOT / 'model.rs'
        subprocess.run(compiler + flags + extra + ['--test', str(model), '-o', binary], check=True)
        subprocess.run([binary], check=True)
    binary = str(Path(tmp) / 'bench')
    subprocess.run(compiler + flags + ['-O', str(ROOT / 'bench.rs'), '-o', binary], check=True)
    with (ROOT / ('results_' + name + '.csv')).open('w') as out:
        subprocess.run([binary, str(args.count), str(args.repeats), 'current' if args.current_only else 'candidates'], stdout=out, check=True)
    if args.edges:
        for experiment in ['edges', 'noops']:
            binary = str(Path(tmp) / experiment)
            subprocess.run(compiler + flags + ['-O', str(ROOT / (experiment + '.rs')), '-o', binary], check=True)
            with (ROOT / (experiment + '_' + name + '.csv')).open('w') as out:
                subprocess.run([binary] + (['current'] if args.current_only else []), stdout=out, check=True)

rows = list(csv.DictReader((ROOT / ('results_' + name + '.csv')).open()))
groups = defaultdict(list)
for row in rows:
    key = (int(row['n']), int(row['stride']), row['workload'], row['implementation'])
    groups[key].append(float(row['ns_per_op']))
cases = []
for (n, stride, workload, impl), timings in sorted(groups.items()):
    baseline = statistics.median(groups[n, stride, workload, 'baseline'])
    median = statistics.median(timings)
    cases.append(dict(n=n, stride=stride, workload=workload, implementation=impl,
                      median_ns=median, min_ns=min(timings), max_ns=max(timings), ratio=median / baseline))
metadata['cases'] = cases
(ROOT / ('summary_' + name + '.json')).write_text(json.dumps(metadata, indent=2) + '\n')
for workload in ['strict', 'inclusive', 'update', 'mixed']:
    print(workload, flush=True)
    for impl in (['current'] if args.current_only else ['update_only', 'incremental', 'leaf_first', 'flat', 'shifted', 'flat_shifted']):
        ratios = [r['ratio'] for r in cases if r['workload'] == workload and r['implementation'] == impl]
        geometric = math.exp(sum(map(math.log, ratios)) / len(ratios))
        print(f'  {impl}: geometric ratio={geometric:.3f}, range={min(ratios):.3f}..{max(ratios):.3f}', flush=True)
