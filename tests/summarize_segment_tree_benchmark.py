#!/usr/bin/env python3
"""Summarize repeated baseline/current timings with bootstrap intervals."""
import json
import math
import random
import statistics
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
path = ROOT / 'tests/segment_tree_benchmark_results.json'
data = json.loads(path.read_text())
cases = defaultdict(dict)
for row in data['measurements']:
    key = (row['tree'], row['n'], row['mode'], row['pattern'])
    cases[key].setdefault(row['variant'], []).append(row['ns'])

rng = random.Random(562301)
results = []
for key, variants in cases.items():
    old, new = variants['baseline'], variants['current']
    samples = sorted(
        statistics.median(rng.choices(old, k=len(old)))
        / statistics.median(rng.choices(new, k=len(new)))
        for _ in range(2000)
    )
    results.append(dict(
        tree=key[0], n=key[1], mode=key[2], pattern=key[3],
        speedup=statistics.median(old)/statistics.median(new),
        ci95=[samples[50], samples[1949]],
    ))

aggregates = []
for tree in dict.fromkeys(r['tree'] for r in results):
    for mode in dict.fromkeys(r['mode'] for r in results if r['tree'] == tree):
        group = [r for r in results if r['tree'] == tree and r['mode'] == mode]
        for patterns in ['all', 'random']:
            selected = group if patterns == 'all' else [r for r in group if r['pattern'] == patterns]
            if not selected:
                continue
            ratios = [r['speedup'] for r in selected]
            aggregates.append(dict(tree=tree, mode=mode, patterns=patterns,
                minimum=min(ratios), median=statistics.median(ratios), maximum=max(ratios),
                geomean=math.exp(statistics.mean(math.log(r) for r in ratios))))

output = dict(
    note='Speedup = median baseline / median current; independent resampling of repeated timings, 2000 draws, percentile 95% intervals. Per-case exploratory intervals, without correction for multiple comparisons. Baseline includes the min_left correctness fix. These are local workload timings including deterministic index generation and dispatch, excluding I/O.',
    cases=results, aggregates=aggregates,
    clear_gains=sum(r['ci95'][0] > 1.05 for r in results),
    clear_regressions=sum(r['ci95'][1] < 0.95 for r in results),
)
(ROOT/'tests/segment_tree_benchmark_summary.json').write_text(json.dumps(output, indent=2)+'\n')
for row in aggregates:
    if row['patterns'] == 'random':
        print(row['tree'], row['mode'], *(round(row[k], 3) for k in ['minimum', 'median', 'maximum']))
print('clear gains / regressions:', output['clear_gains'], output['clear_regressions'])
for row in results:
    if row['ci95'][1] < 0.95:
        print('regression:', row)
