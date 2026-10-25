"""Compare the same input/output code with the frozen and optimized sets.
Large cases follow Library Checker-like distributions and are generated locally.
"""
import argparse
from run_veb_argon import variants
import bisect
import os
import platform
import csv
import hashlib
import json
from pathlib import Path
import random
import statistics
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser()
parser.add_argument('--variants', default='baseline,pred,flat,flat_shifted,combined_shifted')
parser.add_argument('--n', type=int, default=10_000_000)
parser.add_argument('--q', type=int, default=1_000_000)
parser.add_argument('--repeats', type=int, default=7)
parser.add_argument('--output', default='veb_argon_e2e')
args = parser.parse_args()
N, Q, REPEATS = args.n, args.q, args.repeats
names = args.variants.split(',')
sources = variants()
sources['pred'] = (ROOT / 'src/DataStructure/predecessor64.rs').read_text()

rows = []
with tempfile.TemporaryDirectory(prefix='predecessor64-e2e-') as tmp:
    tmp = Path(tmp)
    example = (ROOT / 'examples/predecessor_problem.rs').read_text()
    for name in names:
        source = tmp / (name + '_module.rs')
        source.write_text(sources[name])
        code = example.replace('../src/Basic/', str(ROOT / 'src/Basic') + '/').replace('../src/DataStructure/predecessor64.rs', str(source))
        if name != 'pred': code = code.replace('predecessor64::Predecessor64', 'predecessor64::VebTree')
        (tmp / (name + '.rs')).write_text(code)
        subprocess.run(['rustc', '+1.89.0', '--edition=2021', '-O', '-Awarnings', str(tmp / (name + '.rs')), '-o', str(tmp / name)], check=True)
    rng = random.Random(12345)
    n, q = 129, 10000
    initial = [rng.randrange(2) for _ in range(n)]
    oracle = [i for i, value in enumerate(initial) if value]
    ops, expected = [], []
    for _ in range(q):
        kind, p = rng.randrange(5), rng.randrange(n)
        ops.append(f'{kind} {p}\n')
        at = bisect.bisect_left(oracle, p)
        if kind == 0:
            if at == len(oracle) or oracle[at] != p: oracle.insert(at, p)
        elif kind == 1:
            if at < len(oracle) and oracle[at] == p: oracle.pop(at)
        elif kind == 2: expected.append(str(int(at < len(oracle) and oracle[at] == p)))
        elif kind == 3: expected.append(str(oracle[at] if at < len(oracle) else -1))
        else:
            at = bisect.bisect_right(oracle, p) - 1
            expected.append(str(oracle[at] if at >= 0 else -1))
    data = (f'{n} {q}\n' + ''.join(map(str, initial)) + '\n' + ''.join(ops)).encode()
    expected = ('\n'.join(expected) + '\n').encode()
    for name in names:
        assert subprocess.run([str(tmp / name)], input=data, capture_output=True, check=True).stdout == expected
    for pattern in ['random', 'sparse', 'all0', 'all1', 'query012', 'spaced']:
        rng = random.Random(12345)
        if pattern in ['random', 'query012']:
            initial = rng.randbytes(N).translate(bytes(48 + (i & 1) for i in range(256)))
        elif pattern == 'sparse':
            initial = bytearray(b'0' * N)
            for p in rng.sample(range(N), 10): initial[p] = ord('1')
        elif pattern == 'spaced':
            initial = bytearray(b'0' * N)
            initial[0:N:4096] = b'1' * len(range(0, N, 4096))
        else: initial = (b'0' if pattern == 'all0' else b'1') * N
        path = tmp / 'input'
        with path.open('wb') as f:
            f.write(f'{N} {Q}\n'.encode())
            f.write(initial)
            f.write(b'\n')
            for _ in range(Q):
                kind = rng.randrange(3, 5) if pattern == 'sparse' else rng.randrange(2, 5) if pattern == 'all0' else rng.randrange(1, 5) if pattern == 'all1' else rng.randrange(3) if pattern == 'query012' else rng.randrange(5)
                position = rng.randrange(N)
                if pattern == 'spaced' and kind < 2: position = (position // 4096) * 4096
                f.write(f'{kind} {position}\n'.encode())
        expected = None
        for repeat in range(REPEATS):
            for j in range(len(names)):
                name = names[(j + repeat) % len(names)]
                with path.open('rb') as f:
                    start = time.perf_counter()
                    with (tmp / 'output').open('wb') as output:
                        process = subprocess.Popen([str(tmp / name)], stdin=f, stdout=output)
                        _, status, usage = os.wait4(process.pid, 0)
                        process.returncode = os.waitstatus_to_exitcode(status)
                        assert process.returncode == 0
                    elapsed = (time.perf_counter() - start) * 1000
                    answer = (tmp / 'output').read_bytes()
                if expected is None: expected = answer
                else: assert expected == answer, (pattern, name)
                rows.append({'pattern': pattern, 'implementation': name, 'repeat': repeat, 'wall_ms': elapsed, 'peak_rss_bytes': usage.ru_maxrss * (1 if platform.system() == 'Darwin' else 1024)})
        medians = {name: statistics.median(r['wall_ms'] for r in rows if r['pattern'] == pattern and r['implementation'] == name) for name in names}
        print(pattern, medians, flush=True)
with (ROOT / 'tests' / (args.output + '.csv')).open('w') as f:
    writer = csv.DictWriter(f, rows[0].keys())
    writer.writeheader()
    writer.writerows(rows)
(ROOT / 'tests' / (args.output + '.json')).write_text(json.dumps({
    'rustc': subprocess.check_output(['rustc', '+1.89.0', '-Vv'], text=True),
    'args': vars(args),
    'source_sha256': {name: hashlib.sha256(sources[name].encode()).hexdigest() for name in names},
    'note': 'Local generated cases, not official tests. Identical I/O. Includes startup, construction, output-file writes and destruction. Per-child peak RSS from wait4. All outputs compared, small case also checked against sorted-list oracle.'
}, indent=2) + '\n')
