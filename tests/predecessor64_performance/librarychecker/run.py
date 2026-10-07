"""Full-process timing on reproducible cases following the official generator distributions.
These are local generated cases, not downloaded official test files.
"""
import bisect
import argparse
import csv
import json
import platform
import random
import statistics
import subprocess
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent
parser = argparse.ArgumentParser()
parser.add_argument('--library-io', action='store_true', help='Compare the reusable input/output libraries with the pasted and earlier optimized submissions')
args = parser.parse_args()
names = ['original', 'pack_digits', 'library_io'] if args.library_io else ['original', 'fmt', 'digits', 'pack', 'pack_digits']
compiler = ['rustc', '+1.89.0', '--edition=2021', '-O', '-Awarnings']
rows = []
with tempfile.TemporaryDirectory(prefix='predecessor64-judge-') as temp:
    temp = Path(temp)
    for name in names:
        print('compile', name, flush=True)
        source = ROOT.parents[2] / 'examples/predecessor_problem.rs' if name == 'library_io' else ROOT / (name + '.rs')
        subprocess.run(compiler + [str(source), '-o', str(temp / name)], check=True)
    rng = random.Random(12345)
    n, q = 129, 5000
    initial = [rng.randrange(2) for _ in range(n)]
    oracle = [i for i, b in enumerate(initial) if b]
    ops, expected = [], []
    for _ in range(q):
        t, k = rng.randrange(5), rng.randrange(n)
        ops.append(f'{t} {k}\n')
        at = bisect.bisect_left(oracle, k)
        if t == 0:
            if at == len(oracle) or oracle[at] != k: oracle.insert(at, k)
        elif t == 1:
            if at < len(oracle) and oracle[at] == k: oracle.pop(at)
        elif t == 2: expected.append(str(int(at < len(oracle) and oracle[at] == k)))
        elif t == 3: expected.append(str(oracle[at] if at < len(oracle) else -1))
        else:
            at = bisect.bisect_right(oracle, k) - 1
            expected.append(str(oracle[at] if at >= 0 else -1))
    small = f'{n} {q}\n' + ''.join(map(str, initial)) + '\n' + ''.join(ops)
    expected = ('\n'.join(expected) + '\n').encode()
    for name in names:
        result = subprocess.run([str(temp / name)], input=small.encode(), capture_output=True, check=True)
        assert result.stdout == expected, name
    print('small independent oracle passed', flush=True)

    n, q = 10_000_000, 1_000_000
    for pattern in ['random', 'sparse', 'all0', 'all1', 'query012']:
        print('case', pattern, flush=True)
        rng = random.Random(12345)
        if pattern in ['random', 'query012']:
            initial = rng.randbytes(n).translate(bytes(48 + (i & 1) for i in range(256)))
        elif pattern == 'sparse':
            initial = bytearray(b'0' * n)
            for p in rng.sample(range(n), 10): initial[p] = ord('1')
        else: initial = (b'0' if pattern == 'all0' else b'1') * n
        input_file = temp / (pattern + '.in')
        with input_file.open('wb') as out:
            out.write(f'{n} {q}\n'.encode()); out.write(initial); out.write(b'\n')
            for _ in range(q):
                t = rng.randrange(3, 5) if pattern == 'sparse' else rng.randrange(2, 5) if pattern == 'all0' else rng.randrange(1, 5) if pattern == 'all1' else rng.randrange(3) if pattern == 'query012' else rng.randrange(5)
                out.write(f'{t} {rng.randrange(n)}\n'.encode())
        expected = None
        for repeat in range(7):
            for j in range(len(names)):
                name = names[(repeat + j) % len(names)]
                with input_file.open('rb') as inp:
                    start = time.perf_counter()
                    result = subprocess.run([str(temp / name)], stdin=inp, capture_output=True, check=True)
                    wall = (time.perf_counter() - start) * 1000
                if expected is None: expected = result.stdout
                else: assert result.stdout == expected, (pattern, name)
                profile = result.stderr.decode().strip()
                phases = list(map(float, profile.split(','))) if profile else [None] * 4
                rows.append([pattern, name, repeat, wall, *phases, len(result.stdout)])
        for name in names:
            sample = [r for r in rows if r[0] == pattern and r[1] == name]
            print(name, 'wall_ms', round(statistics.median(r[3] for r in sample), 3),
                  'read/build/query+format/write', [round(statistics.median(r[i] for r in sample), 3) for i in range(4, 8)] if sample[0][4] is not None else 'not instrumented', flush=True)

suffix = '_library_io' if args.library_io else ''
with (ROOT / ('results' + suffix + '.csv')).open('w') as out:
    writer = csv.writer(out)
    writer.writerow(['pattern', 'implementation', 'repeat', 'wall_ms', 'read_ms', 'build_ms', 'query_and_format_ms', 'write_ms', 'output_bytes'])
    writer.writerows(rows)
metadata = {'compiler': subprocess.check_output(['rustc', '+1.89.0', '-Vv'], text=True),
            'platform': platform.platform(), 'flags': compiler[2:], 'n': n, 'q': q, 'repeats': 7,
            'note': 'Local generated cases following official distributions; not official files. Process startup/drop/output capture included in wall_ms.'}
(ROOT / ('metadata' + suffix + '.json')).write_text(json.dumps(metadata, indent=2) + '\n')
