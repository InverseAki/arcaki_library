#!/usr/bin/env python3
"""Local ABC467 G examples, differential checks, and optional maximum-scale cases.
No external submission. Seed is read from stdin (empty stdin uses 467).
"""
from pathlib import Path
import argparse
import json
import platform
import random
import re
import subprocess
import sys
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
SAMPLES = [
    ("""7 5
8 2 4 1 7 3 6
1 1 4 7 9
5 2 1 3 8
6 4 1 5 9
6 5 3 5 1
7 9 4 6 4
""", [2, -1, 4, 1, 1]),
    ("""15 10
993115119 576136368 21553212 219853538 853822501 687675302 281611653 844033520 423210108 339630584 780395612 207907746 285523486 359061085 14767613
6 13801767 1 3 667406485
7 672229269 5 7 855399219
11 2367096 1 10 4016308479
6 5951398 8 8 413598120
6 196639646 5 11 2483790193
14 105322777 6 7 610670157
7 416730828 2 14 1236755516
9 838827476 5 14 4350335977
1 894919681 3 7 1132967029
6 932707551 6 15 1694677398
""", [1, 2, 6, 1, 4, 1, 2, -1, 2, 2]),
]


def encode(initial, queries):
    lines = [f"{len(initial)} {len(queries)}", " ".join(map(str, initial))]
    lines.extend(f"{i+1} {x} {l+1} {r} {k}" for i, x, l, r, k in queries)
    return "\n".join(lines) + "\n"


def naive(values, target):
    total = 0
    for count, value in enumerate(sorted(values, reverse=True), 1):
        total += value
        if total >= target:
            return count
    return -1


def execute(binary, data, measured=False):
    cmd = [str(binary)]
    if measured:
        # A fresh measuring parent has only this binary in RUSAGE_CHILDREN.
        # macOS time -l needs a sysctl unavailable in the workspace sandbox.
        runner = """import resource, subprocess, sys
run = subprocess.run([sys.argv[1]], stdout=subprocess.PIPE, text=True, check=True)
sys.stdout.write(run.stdout)
rss = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss
if sys.platform != 'darwin': rss *= 1024
print(f'RSS_BYTES {rss}', file=sys.stderr)
"""
        cmd = [sys.executable, "-c", runner, str(binary)]
    start = time.perf_counter()
    run = subprocess.run(cmd, input=data, text=True, capture_output=True, check=True)
    elapsed = time.perf_counter() - start
    rss = re.search(r"RSS_BYTES (\d+)", run.stderr)
    return list(map(int, run.stdout.split())), elapsed, int(rss[1]) if rss else None


def random_checks(binary, seed, count=100):
    rng = random.Random(seed)
    for case in range(count):
        n = rng.randrange(1, 31)
        initial = [rng.randrange(1, 101) for _ in range(n)]
        a = initial.copy()
        queries, expected = [], []
        for step in range(100):
            i, x = rng.randrange(n), rng.randrange(1, 101)
            l, r = sorted(rng.sample(range(n + 1), 2))
            a[i] = x
            total = sum(a[l:r])
            target = [1, total, total + 1, rng.randrange(1, total + 30)][step % 4]
            queries.append((i, x, l, r, target))
            expected.append(naive(a[l:r], target))
        data = encode(initial, queries)
        got, _, _ = execute(binary, data)
        if got != expected:
            failure = ROOT / "tests" / "waveletmatrix_offline_failure.in"
            failure.write_text(data)
            raise AssertionError(f"case {case} failed; input saved at {failure}")
    return count * 100


def benchmark_cases(seed):
    n = q = 100_000
    rng = random.Random(seed)
    # Full duplicate values, unchanged updates, exact/just-unreachable targets.
    initial = [10**9] * n
    queries, expected = [], []
    for t in range(q):
        l, r = sorted(rng.sample(range(n + 1), 2))
        k = [1, (r-l)*10**9, (r-l)*10**9+1][t % 3]
        queries.append((rng.randrange(n), 10**9, l, r, k))
        expected.append((k-1)//10**9+1 if k <= (r-l)*10**9 else -1)
    yield "all_equal", initial, queries, expected

    # 100,000 distinct future values at one position: nearly every candidate inactive.
    initial = [1] * n
    queries, expected = [], []
    i = n // 2
    for t in range(q):
        x = 10**9 - t
        l, r = sorted(rng.sample(range(n + 1), 2))
        total = (r-l) + (x-1 if l <= i < r else 0)
        k = [1, total, total+1, x+1, x][t % 5]
        queries.append((i, x, l, r, k))
        if k > total:
            ans = -1
        elif l <= i < r:
            ans = 1 if k <= x else 1 + (k-x)
        else:
            ans = k
        expected.append(ans)
    yield "concentrated_candidates", initial, queries, expected

    # Sorted values and unchanged updates; independent arithmetic-progression oracle.
    initial = list(range(1, n+1))
    queries, expected = [], []
    for t in range(q):
        l, r = sorted(rng.sample(range(n + 1), 2))
        total = (l+1+r)*(r-l)//2
        k = [1, total, total+1, (total+1)//2][t % 4]
        i = rng.randrange(n)
        queries.append((i, i+1, l, r, k))
        if total < k:
            expected.append(-1)
        else:
            low, high = 0, r-l
            while low < high:
                mid = (low+high)//2
                if mid*(2*r-mid+1)//2 >= k:
                    high = mid
                else:
                    low = mid+1
            expected.append(low)
    yield "ascending", initial, queries, expected

    # Large arbitrary intervals and genuine value changes; all answers get total-sum
    # and cardinality checks, plus 128 exact independently sorted interval checks.
    initial = [rng.randrange(1, 10**9+1) for _ in range(n)]
    a = initial.copy()
    bit = [0] * (n+1)
    for j, value in enumerate(a, 1):
        bit[j] += value
        p = j + (j & -j)
        if p <= n:
            bit[p] += bit[j]
    def prefix(r):
        result = 0
        while r:
            result += bit[r]
            r &= r-1
        return result
    exact_steps = set(rng.sample(range(q), 128))
    queries, checks = [], []
    for t in range(q):
        i, x = rng.randrange(n), rng.randrange(1, 10**9+1)
        delta = x-a[i]
        a[i] = x
        j = i+1
        while j <= n:
            bit[j] += delta
            j += j & -j
        l, r = sorted(rng.sample(range(n+1), 2))
        total = prefix(r)-prefix(l)
        k = [1, total, total+1, (total+1)//2][t % 4]
        queries.append((i,x,l,r,k))
        if t in exact_steps:
            checks.append((r-l,total,naive(a[l:r],k)))
        else:
            checks.append((r-l,total,None))
    yield "random_candidates", initial, queries, checks


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--benchmark", action="store_true")
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    seed_text = sys.stdin.read().strip()
    seed = int(seed_text) if seed_text else 467
    results = {
        "seed": seed,
        "environment": {
            "os": sys.platform,
            "architecture": platform.machine(),
            "rustc": subprocess.run(["rustc", "--version"], text=True, capture_output=True, check=True).stdout.strip(),
            "benchmark_compile_flags": ["--edition=2021", "-O"],
        },
        "samples_each_profile": len(SAMPLES),
        "profiles": {},
    }
    with tempfile.TemporaryDirectory(prefix="wavelet-offline-") as directory:
        binaries = {}
        for mode, flags in [("debug", []), ("release", ["-O"])]:
            binary = Path(directory) / mode
            subprocess.run(["rustc", "--edition=2021", *flags, str(ROOT / "examples/abc467_g.rs"), "-o", str(binary)], check=True)
            binaries[mode] = binary
            for data, expected in SAMPLES:
                got, _, _ = execute(binary, data)
                assert got == expected, (mode, got, expected)
            count = random_checks(binary, seed)
            results["profiles"][mode] = {"random_queries": count, "passed": True}
            print(f"{mode}: 2 samples + {count} differential queries OK", flush=True)
        if args.benchmark:
            results["benchmarks"] = []
            for name, initial, queries, expected in benchmark_cases(seed):
                got, elapsed, rss = execute(binaries["release"], encode(initial,queries), measured=True)
                assert len(got) == len(queries)
                if name == "random_candidates":
                    for ans, (_,_,_,_,k), (length,total,exact) in zip(got,queries,expected):
                        assert (ans == -1) == (total < k)
                        assert ans == -1 or 1 <= ans <= length
                        if exact is not None:
                            assert ans == exact, (ans,exact)
                else:
                    assert got == expected, name
                entry = {"case":name,"n":len(initial),"q":len(queries),"seconds":round(elapsed,4),"rss_bytes":rss,"verification":"128 exact + all sum/cardinality checks" if name == "random_candidates" else "all queries exact"}
                results["benchmarks"].append(entry)
                print(json.dumps(entry), flush=True)
    if args.output:
        args.output.write_text(json.dumps(results, indent=2) + "\n")


if __name__ == "__main__":
    main()
