import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import random
import statistics
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]


def program_prefix_and_suffix(submission):
    if submission:
        source = submission.read_text()
        begin = source.index('// Source: https://github.com/InverseAki/arcaki_library/blob/main/src/DataStructure/sparsetable.rs')
        end = source.index('pub fn solve()')
        return source[:begin], source[end:]
    prefix = (ROOT / 'src/Basic/basic_io.rs').read_text() + '\n'
    prefix += (ROOT / 'src/Basic/basic_output.rs').read_text() + '\n'
    suffix = '''fn main() {
        let mut ip = Input::new();
        let mut o = Output::new();
        let n = ip.usize(); let q = ip.usize();
        let st = SparseTableMI::build(&(0..n).map(|_| ip.i32()).collect::<Vec<_>>());
        for _ in 0..q { let l = ip.usize(); let r = ip.usize(); o.i32_line(st.query(l,r)); }
    }'''
    return prefix, suffix


def source_variants(submission):
    prefix, suffix = program_prefix_and_suffix(submission)
    old = (ROOT / 'tests/support/sparsetable_nested_baseline.rs').read_text()
    current = (ROOT / 'src/DataStructure/sparsetable.rs').read_text()
    candidates = (ROOT / 'tests/support/sparsetable_candidates.rs').read_text()
    owned = suffix.replace('SparseTableMI::build(&', 'SparseTableMI::build(')
    variants = {'submitted': prefix + old + suffix, 'current': prefix + current + owned}
    for label, kind in [('flat_formula', 'Flat<T,true,false>'),
                        ('prefix16', 'PrefixSuffix<T,16>'),
                        ('prefix32', 'PrefixSuffix<T,32>'),
                        ('prefix64', 'PrefixSuffix<T,64>')]:
        query = 'self.0.query::<true>(l,r,|x,y|x.min(y))' if label == 'flat_formula' else 'self.0.query(l,r,|x,y|x.min(y))'
        constructor = kind.replace('<', '::<', 1)
        adapter = f'''struct SparseTableMI<T:Copy>(candidates::{kind});
            impl<T:Ord+Copy> SparseTableMI<T> {{
                fn build(a:Vec<T>)->Self {{Self(candidates::{constructor}::build(a,|x,y|x.min(y)))}}
                #[inline] fn query(&self,l:usize,r:usize)->T {{{query}}}
            }}'''
        variants[label] = prefix + '\nmod candidates {\n' + candidates + '\n}\n' + adapter + owned
    return variants


def make_input(path, n, q, distribution):
    rng = random.Random(20261021 + n + q)
    with path.open('w') as f:
        f.write(f'{n} {q}\n')
        f.write(' '.join(str(rng.randrange(-2147483648, 2147483648)) for _ in range(n)) + '\n')
        for _ in range(q):
            l = rng.randrange(n)
            if distribution == 'wide':
                r = rng.randrange(l + 1, n + 1)
            elif distribution == 'short':
                r = min(n, l + rng.randrange(1, 65))
            else:
                r = rng.randrange(l + 1, min(n, (l // 16 + 1) * 16) + 1)
            f.write(f'{l} {r}\n')


def timed_run(binary, input_path):
    with input_path.open('rb') as f:
        start = time.perf_counter()
        proc = subprocess.Popen([str(binary)], stdin=f, stdout=subprocess.DEVNULL)
        _, status, usage = os.wait4(proc.pid, 0)
        proc.returncode = os.waitstatus_to_exitcode(status)
        if proc.returncode:
            raise RuntimeError(f'{binary.name}: exit {proc.returncode}')
        elapsed = (time.perf_counter() - start) * 1000
        rss = usage.ru_maxrss * (1 if platform.system() == 'Darwin' else 1024)
        return elapsed, rss


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--submission', type=Path)
    parser.add_argument('--toolchain', default='1.89.0')
    parser.add_argument('--rounds', type=int, default=9)
    parser.add_argument('--output', type=Path, default=ROOT / 'tests/sparsetable_benchmark_results.json')
    parser.add_argument('--export', type=Path)
    args = parser.parse_args()
    compiler = ['rustc', '+' + args.toolchain]
    version = subprocess.check_output(compiler + ['--version'], text=True).strip()
    sources = source_variants(args.submission)
    if args.export:
        args.export.write_text(sources['current'])
    results = {'compiler': version, 'platform': platform.platform(), 'flags': ['--edition=2021', '-O'],
               'io_source': 'submission' if args.submission else 'current library', 'warmups': 2,
               'rounds': args.rounds, 'rows': []}
    datasets = [('wide500k', 500_000, 500_000, 'wide'),
                ('wide1m', 1_000_000, 1_000_000, 'wide'),
                ('short500k', 500_000, 500_000, 'short'),
                ('queries1m', 100_000, 1_000_000, 'wide'),
                ('sameblock500k', 500_000, 500_000, 'sameblock')]
    with tempfile.TemporaryDirectory(prefix='sparsetable-') as directory:
        work = Path(directory)
        for name, source in sources.items():
            path = work / (name + '.rs')
            path.write_text(source)
            subprocess.run(compiler + ['--edition=2021', '-O', '-Awarnings', str(path), '-o', str(work / name)], check=True)
        for label, n, q, distribution in datasets:
            input_path = work / (label + '.in')
            make_input(input_path, n, q, distribution)
            expected = None
            for name in sources:
                with input_path.open('rb') as f:
                    output = subprocess.check_output([str(work / name)], stdin=f)
                digest = hashlib.sha256(output).hexdigest()
                if expected is None:
                    expected = digest
                if digest != expected:
                    raise AssertionError((label, name, digest, expected))
            names = list(sources)
            samples = {name: [] for name in names}
            for round_index in range(args.rounds + 2):
                shift = round_index % len(names)
                for name in names[shift:] + names[:shift]:
                    elapsed, rss = timed_run(work / name, input_path)
                    if round_index >= 2:
                        samples[name].append({'ms': elapsed, 'peak_rss_bytes': rss})
            for name in names:
                row = {'dataset': label, 'n': n, 'q': q, 'variant': name,
                       'ms': statistics.median(s['ms'] for s in samples[name]),
                       'peak_rss_bytes': statistics.median(s['peak_rss_bytes'] for s in samples[name]),
                       'samples': samples[name], 'sha256': expected}
                results['rows'].append(row)
                print(f"{label:16} {name:14} {row['ms']:8.3f} ms {row['peak_rss_bytes']/2**20:7.2f} MiB", flush=True)
    args.output.write_text(json.dumps(results, indent=2) + '\n')


if __name__ == '__main__':
    main()
