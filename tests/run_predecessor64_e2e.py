"""Compare the same input/output code with the frozen and optimized sets.
Large cases follow Library Checker-like distributions and are generated locally.
"""
import bisect
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
N, Q, REPEATS = 10_000_000, 1_000_000, 9
rows = []
with tempfile.TemporaryDirectory(prefix='predecessor64-e2e-') as tmp:
    tmp = Path(tmp)
    example = (ROOT / 'examples/predecessor_problem.rs').read_text()
    for name, source in [('baseline', ROOT / 'tests/support/predecessor64_argon/baseline.rs'), ('current', ROOT / 'src/DataStructure/predecessor64.rs')]:
        code = example.replace('../src/Basic/', str(ROOT / 'src/Basic') + '/').replace('../src/DataStructure/predecessor64.rs', str(source))
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
    for name in ['baseline', 'current']:
        assert subprocess.run([str(tmp / name)], input=data, capture_output=True, check=True).stdout == expected
    for pattern in ['random', 'sparse', 'all0', 'all1', 'query012']:
        rng = random.Random(12345)
        if pattern in ['random', 'query012']:
            initial = rng.randbytes(N).translate(bytes(48 + (i & 1) for i in range(256)))
        elif pattern == 'sparse':
            initial = bytearray(b'0' * N)
            for p in rng.sample(range(N), 10): initial[p] = ord('1')
        else: initial = (b'0' if pattern == 'all0' else b'1') * N
        path = tmp / 'input'
        with path.open('wb') as f:
            f.write(f'{N} {Q}\n'.encode())
            f.write(initial)
            f.write(b'\n')
            for _ in range(Q):
                kind = rng.randrange(3, 5) if pattern == 'sparse' else rng.randrange(2, 5) if pattern == 'all0' else rng.randrange(1, 5) if pattern == 'all1' else rng.randrange(3) if pattern == 'query012' else rng.randrange(5)
                f.write(f'{kind} {rng.randrange(N)}\n'.encode())
        expected = None
        for repeat in range(REPEATS):
            for name in (['baseline', 'current'] if repeat % 2 == 0 else ['current', 'baseline']):
                with path.open('rb') as f:
                    start = time.perf_counter()
                    result = subprocess.run([str(tmp / name)], stdin=f, capture_output=True, check=True)
                    elapsed = (time.perf_counter() - start) * 1000
                if expected is None: expected = result.stdout
                else: assert expected == result.stdout, (pattern, name)
                rows.append({'pattern': pattern, 'implementation': name, 'repeat': repeat, 'wall_ms': elapsed})
        medians = {name: statistics.median(r['wall_ms'] for r in rows if r['pattern'] == pattern and r['implementation'] == name) for name in ['baseline', 'current']}
        print(pattern, medians, flush=True)
with (ROOT / 'tests/predecessor64_argon_e2e.csv').open('w') as f:
    writer = csv.DictWriter(f, rows[0].keys())
    writer.writeheader()
    writer.writerows(rows)
(ROOT / 'tests/predecessor64_argon_e2e.json').write_text(json.dumps({'rustc': subprocess.check_output(['rustc', '+1.89.0', '-Vv'], text=True), 'n': N, 'q': Q, 'repeats': REPEATS, 'source_sha256': {name: hashlib.sha256(path.read_bytes()).hexdigest() for name, path in [('baseline', ROOT / 'tests/support/predecessor64_argon/baseline.rs'), ('current', ROOT / 'src/DataStructure/predecessor64.rs')]}, 'note': 'Local generated cases, not official tests. Same I/O code; includes process startup, input, output capture and destruction.'}, indent=2) + '\n')
