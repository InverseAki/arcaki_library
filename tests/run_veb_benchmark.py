"""Local fixed-seed comparison against the current Predecessor64."""
import argparse
import csv
import hashlib
import json
import os
from pathlib import Path
import platform
import statistics
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser()
parser.add_argument('--count', type=int, default=500_000)
parser.add_argument('--repeats', type=int, default=5)
parser.add_argument('--veb-source', default='src/DataStructure/veb_tree.rs')
parser.add_argument('--output', default='veb_benchmark')
args = parser.parse_args()
sources = {name: ROOT / 'src/DataStructure' / file for name, file in [('current', 'predecessor64.rs'), ('veb', 'veb_tree.rs')]}
sources['veb'] = (ROOT / args.veb_source).resolve()
common = (ROOT / 'tests/predecessor64_performance/common.rs').read_text()
common = common[common.index('trait Set:'):common.index('impl_set!(baseline')]+common[common.index('fn rand('):].replace('impl_set!(veb::VebTree);', '')
common = '\n'.join(f'#[path = "{p}"] mod {name};' for name, p in sources.items()) + '\n' + common
common += '\nimpl_set!(current::Predecessor64);\nimpl_set!(veb::VebTree);\n'
code = (ROOT / 'tests/support/predecessor64_argon/bench.rs').read_text().replace('include!("common.rs");', common)
code = code.replace('VARIANTS', '2').replace('MATCH_ARMS', '0 => ("current", run::<current::Predecessor64>(n, &initial, &ops)),\n1 => ("veb", run::<veb::VebTree>(n, &initial, &ops)),')
code = code.replace('[2, 64, 4096]', '[2, 64, 4096, n].into_iter().collect::<std::collections::BTreeSet<_>>()')
code = code.replace('["strict", "inclusive", "update", "mixed"]', '["strict", "inclusive", "update", "mixed", "extrema", "contains", "empty"]')
code = code.replace('let mut state =', 'let saved = initial.clone();\n                if workload == "empty" { initial.clear(); }\n                let mut state =')
code = code.replace('let mut expected = None;', 'let mut expected = None;')
code = code.replace('            }\n        }\n    }\n}', '                initial = saved;\n            }\n        }\n    }\n}')
code = code.replace('_ => set.include(op.p) as usize,', '7 => set.min(), 8 => set.max(), _ => set.include(op.p) as usize,')
code = code.replace('"strict" =>', '"extrema" => 7 + (r % 2) as u8, "contains" => 6, "empty" => 2 + (r % 2) as u8, "strict" =>')
with tempfile.TemporaryDirectory(prefix='veb-bench-') as temp:
    temp = Path(temp)
    src, exe = temp / 'bench.rs', temp / 'bench'
    src.write_text(code)
    subprocess.run(['rustc', '+1.89.0', '--edition=2021', '-O', '-Awarnings', str(src), '-o', str(exe)], check=True)
    result = subprocess.check_output([str(exe), str(args.count), str(args.repeats)], text=True)
    out = ROOT / 'tests' / args.output
    out.with_suffix('.csv').write_text(result)
    rows = list(csv.DictReader(result.splitlines()))
    groups = {}
    for row in rows:
        key = (int(row['n']), int(row['stride']), row['workload'])
        groups.setdefault(key, {}).setdefault(row['implementation'], []).append(float(row['ns_per_op']))
    medians = []
    for (n, stride, workload), group in groups.items():
        values = {k: statistics.median(v) for k, v in group.items()}
        medians.append(dict(n=n, stride=stride, workload=workload, **values, speedup=values['current']/values['veb']))
    metadata = dict(platform=platform.platform(), rustc=subprocess.check_output(['rustc', '+1.89.0', '-Vv'], text=True), count=args.count, repeats=args.repeats,
                    source_sha256={k: hashlib.sha256(v.read_bytes()).hexdigest() for k,v in sources.items()}, medians=medians)
    out.with_suffix('.json').write_text(json.dumps(metadata, indent=2)+'\n')
    for workload in sorted({r['workload'] for r in medians}):
        values = [r['speedup'] for r in medians if r['workload'] == workload]
        print(workload, 'speedup min/median/max', min(values), statistics.median(values), max(values), flush=True)
