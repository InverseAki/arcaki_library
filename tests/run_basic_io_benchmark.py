import argparse
import json
import pathlib
import random
import statistics
import subprocess
import tempfile
import time

ROOT = pathlib.Path(__file__).resolve().parents[1]


def output_candidate(source):
    table = ''.join(f'{i:02}' for i in range(100))
    source = source.replace('macro_rules! impl_unsigned_output {',
        f'const OUTPUT_DIGIT_PAIRS: &[u8; 200] = b"{table}";\n\nmacro_rules! impl_unsigned_output {{')
    for value in ['value', 'magnitude']:
        old = f'''                    loop {{
                        at -= 1;
                        buf[at] = b'0' + ({value} % 10) as u8;
                        {value} /= 10;
                        if {value} == 0 {{ break; }}
                    }}'''
        new = f'''                    while {value} >= 100 {{
                        let pair = ({value} % 100) as usize * 2;
                        {value} /= 100;
                        at -= 2;
                        buf[at..at + 2].copy_from_slice(&OUTPUT_DIGIT_PAIRS[pair..pair + 2]);
                    }}
                    if {value} >= 10 {{
                        let pair = {value} as usize * 2;
                        at -= 2;
                        buf[at..at + 2].copy_from_slice(&OUTPUT_DIGIT_PAIRS[pair..pair + 2]);
                    }} else {{
                        at -= 1;
                        buf[at] = b'0' + {value} as u8;
                    }}'''
        assert old in source
        source = source.replace(old, new)
    return source


def input_candidate(source):
    # Preserve the existing whitespace, EOF, and non-digit behavior.
    for sign in ['+', '-']:
        old = f"res = res * 10 {sign} (buf[i] - b'0') as $t;\n"
        # Every digit loop already proves i < len; get() guards the lookahead.
        new = f'''if let Some(&next) = buf.get(i + 1).filter(|b| b.is_ascii_digit()) {{
                            res = res * 100 {sign} ((buf[i] - b'0') * 10 + next - b'0') as $t;
                            i += 2;
                            continue;
                        }}
                        {old}'''
        assert old in source
        source = source.replace(old, new)
    return source


def output_groups4(source):
    source = output_candidate(source)
    for value in ['value', 'magnitude']:
        old = f'                    while {value} >= 100 {{'
        new = f'''                    while {value} >= 10000 {{
                        let group = ({value} % 10000) as usize;
                        {value} /= 10000;
                        let high = group / 100 * 2;
                        let low = group % 100 * 2;
                        at -= 4;
                        buf[at..at + 2].copy_from_slice(&OUTPUT_DIGIT_PAIRS[high..high + 2]);
                        buf[at + 2..at + 4].copy_from_slice(&OUTPUT_DIGIT_PAIRS[low..low + 2]);
                    }}
{old}'''
        # u8/i8 magnitudes cannot represent 10000; use a widened comparison and division.
        new = new.replace(f'{value} >= 10000', f'{value} as u128 >= 10000')
        new = new.replace(f'({value} % 10000)', f'({value} as u128 % 10000)')
        new = new.replace(f'{value} /= 10000;', f'{value} = ({value} as u128 / 10000) as _;')
        source = source.replace(old, new)
    return source


def input_iterator(source):
    for sign in ['+', '-']:
        old = f'''while buf[i].is_ascii_digit() {{
                        res = res * 10 {sign} (buf[i] - b'0') as $t;
                        i += 1;
                    }}'''
        new = f'''for &digit in &buf[i..] {{
                        if !digit.is_ascii_digit() {{ break; }}
                        res = res * 10 {sign} (digit - b'0') as $t;
                        i += 1;
                    }}'''
        # Signed loops have one additional level of indentation.
        for pad in ['', '    ']:
            before = old.replace('\n', '\n' + pad)
            after = new.replace('\n', '\n' + pad)
            source = source.replace(before, after)
    assert 'while buf[i].is_ascii_digit()' not in source
    return source


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--repeats', type=int, default=7)
    parser.add_argument('--size', type=int, default=500000)
    args = parser.parse_args()
    inp = (ROOT / 'tests/support/basic_io_baseline.rs').read_text()
    out = (ROOT / 'tests/support/basic_output_baseline.rs').read_text()
    assert (ROOT / 'src/Basic/basic_output.rs').read_text() == output_candidate(out)
    solve = '''fn main() {
        let mut ip = Input::new(); let n = ip.usize(); let q = ip.usize();
        let mut ac = vec![0i64; n+1];
        for i in 0..n { ac[i+1] = ac[i] + ip.i64(); }
        let mut o = Output::new();
        for _ in 0..q { let l = ip.usize(); let r = ip.usize(); o.i64_line(ac[r]-ac[l]); }
        o.flush().unwrap();
    }'''
    variants = {'baseline': (inp, out), 'output_pairs': (inp, output_candidate(out)),
                'input_pairs': (input_candidate(inp), out),
                'both_pairs': (input_candidate(inp), output_candidate(out)),
                'input_iterator': (input_iterator(inp), out),
                'iterator_output_pairs': (input_iterator(inp), output_candidate(out)),
                'output_groups4': (inp, output_groups4(out))}
    results = []
    rng = random.Random(20261021)
    with tempfile.TemporaryDirectory() as temp:
        temp = pathlib.Path(temp)
        for name, (i, o) in variants.items():
            src = temp / f'{name}.rs'
            src.write_text('#![allow(dead_code)]\n' + i + '\n' + o + '\n' + solve)
            subprocess.run(['rustc', '+1.89.0', '-O', '--edition=2021', str(src), '-o', str(temp/name)], check=True)
        n = args.size
        for case in ['random', 'small', 'zero', 'negative', 'full_range', 'short']:
            values = [rng.randrange(10**9) for _ in range(n)]
            if case == 'small': values = [v % 100 for v in values]
            if case == 'zero': values = [0] * n
            if case == 'negative': values = [-v for v in values]
            if case == 'short': values = [v % 10 for v in values]
            prefix = [0]
            for v in values: prefix.append(prefix[-1] + v)
            queries = []
            for _ in range(n):
                l = rng.randrange(n)
                r = rng.randrange(l+1, n+1)
                if case == 'full_range': l, r = 0, n
                if case == 'short': r = l + 1
                queries.append((l, r))
            data = f'{n} {n}\n' + ' '.join(map(str, values)) + '\n'
            data += ''.join(f'{l} {r}\n' for l, r in queries)
            path = temp / 'input.txt'
            path.write_text(data)
            expected = ''.join(f'{prefix[r]-prefix[l]}\n' for l, r in queries).encode()
            samples = {name: [] for name in variants}
            for rep in range(args.repeats + 1):
                names = list(variants)
                names = names[rep % len(names):] + names[:rep % len(names)]
                for name in names:
                    with path.open('rb') as stream:
                        start = time.perf_counter_ns()
                        proc = subprocess.run([str(temp/name)], stdin=stream, stdout=subprocess.PIPE, check=True)
                        elapsed = (time.perf_counter_ns()-start)/1e6
                    assert proc.stdout == expected, (case, name)
                    if rep: samples[name].append(elapsed)
            row = {'case': case, 'milliseconds': samples,
                   'median_ms': {k: statistics.median(v) for k, v in samples.items()}}
            results.append(row)
            print(case, row['median_ms'], flush=True)
    report = {'size': n, 'repeats': args.repeats, 'selected': 'output_pairs',
              'rustc': subprocess.check_output(['rustc', '+1.89.0', '-Vv'], text=True),
              'timing': 'process start through exit, file stdin and captured stdout; every output checked against Python',
              'results': results}
    (ROOT / 'tests/basic_io_benchmark_results.json').write_text(json.dumps(report, indent=2)+'\n')


if __name__ == '__main__':
    main()
