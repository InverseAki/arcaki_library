import argparse
import csv
import hashlib
import json
import math
import pathlib
import platform
import statistics
import subprocess
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
SUPPORT = ROOT / 'tests/support/predecessor64_argon'
BASE = (SUPPORT / 'baseline.rs').read_text()


def replace_function(src, name, replacement):
    start = src.index('    fn ' + name) if '    fn ' + name in src else src.index('    pub fn ' + name)
    brace = src.index('{', start)
    depth = 1
    end = brace + 1
    while depth:
        depth += (src[end] == '{') - (src[end] == '}')
        end += 1
    return src[:start] + replacement + src[end:]


def guarded(src):
    src = src.replace('Self::layout(mx.div_ceil(64));', 'Self::layout(mx.max(1).div_ceil(64));').replace('            mx,\n            data: vec!', '            mx: mx.max(1),\n            data: vec!')
    for name in ['insert', 'remove']:
        src = src.replace(f'pub fn {name}(&mut self, mut p: usize) {{', f'pub fn {name}(&mut self, mut p: usize) {{\n        assert!(p < self.mx);')
    src = src.replace('pub fn include(&self, p: usize) -> bool {', 'pub fn include(&self, p: usize) -> bool {\n        if p >= self.mx { return false; }')
    for name, result in [('search_prev', 'self.max()'), ('search_next', '!0')]:
        src = src.replace(f'fn {name}<const INCLUSIVE: bool>(&self, mut p: usize) -> usize {{', f'fn {name}<const INCLUSIVE: bool>(&self, mut p: usize) -> usize {{\n        if p >= self.mx {{ return {result}; }}')
    return src


def peeled(src):
    for direction in ['prev', 'next']:
        src = replace_function(src, 'search_' + direction, search(direction))
    for name in ['insert', 'remove']:
        begin = 'let old = *word;\n            *word |= 1 << (p & 63);\n            if old != 0' if name == 'insert' else 'let old = *word;\n            *word &= !(1 << (p & 63));\n            if *word != 0 || old == 0'
        body = begin + ' { return; }'
        body = body.replace('            ', '        ')
        fn = f'''    pub fn {name}(&mut self, mut p: usize) {{
        assert!(p < self.mx);
        let word = &mut self.data[p >> 6];
        {body}
        for i in 1..self.levels {{
            p >>= 6;
            let word = &mut self.data[self.offset[i] + (p >> 6)];
            {body}
        }}
    }}'''
        src = replace_function(src, name, fn)
    return src


def search(direction):
    prev = direction == 'prev'
    strict = '(1u64 << (p & 63)) - 1' if prev else '(u64::MAX << (p & 63)) << 1'
    inclusive = 'u64::MAX >> (63 - (p & 63))' if prev else 'u64::MAX << (p & 63)'
    bit = '63 - word.leading_zeros() as usize' if prev else 'word.trailing_zeros() as usize'
    subbit = '63 - self.data[self.offset[j] + res].leading_zeros() as usize' if prev else 'self.data[self.offset[j] + res].trailing_zeros() as usize'
    return f'''    fn search_{direction}<const INCLUSIVE: bool>(&self, mut p: usize) -> usize {{
        if p >= self.mx {{ return {'self.max()' if prev else '!0'}; }}
        let mask = if INCLUSIVE {{ {inclusive} }} else {{ {strict} }};
        let word = self.data[p >> 6] & mask;
        if word != 0 {{ return (p & !63) | ({bit}); }}
        for i in 1..self.levels {{
            p >>= 6;
            let word = self.data[self.offset[i] + (p >> 6)] & ({strict});
            if word != 0 {{
                let mut res = (p & !63) | ({bit});
                for j in (0..i).rev() {{ res = (res << 6) | ({subbit}); }}
                return res;
            }}
        }}
        !0
    }}'''


def unchecked(src):
    for expr in ['self.offset[i] + (p >> 6)', 'p >> 6']:
        src = src.replace('&mut self.data[' + expr + ']', 'unsafe { self.data.get_unchecked_mut(' + expr + ') }')
    for expr in ['self.offset[self.levels - 1]', 'self.offset[j] + res', 'self.offset[i] + (p >> 6)', 'self.offset[i] + p', 'leaf_index', 'p >> 6']:
        src = src.replace('self.data[' + expr + ']', '(unsafe { *self.data.get_unchecked(' + expr + ') })')
    return src


def nostore(src):
    # Only the leaf needs to check duplicates: a propagated insertion/deletion
    # always changes the corresponding upper bit by the hierarchy invariant.
    for name in ['insert', 'remove']:
        start = src.index('    pub fn ' + name)
        end = src.index('        for i in 1..self.levels', start)
        part = src[start:end]
        test = '!= 0' if name == 'insert' else '== 0'
        part = part.replace('let old = *word;', f'let old = *word;\n        if old & (1u64 << (p & 63)) {test} {{ return; }}')
        src = src[:start] + part + src[end:]
    return src


def root(src):
    start = src.index('        let leaf_index =')
    end = src.index('        let mut word =', start)
    return src[:start] + src[end:]


def branchless(src):
    return src.replace('if v != 0 {\n                        bits |= 1 << j;\n                    }', 'bits |= ((v != 0) as u64) << j;')


def chunks(src):
    start = src.index('            for (chunk, word)')
    end = src.index('\n        }\n        Self {', start)
    return src[:start] + '''            let mut chunks = lower.chunks_exact(64);
            let mut words = upper.iter_mut();
            for (chunk, word) in (&mut chunks).zip(&mut words) {
                let mut bits = 0;
                for j in 0..64 { bits |= ((chunk[j] != 0) as u64) << j; }
                *word = bits;
            }
            if let Some(word) = words.next() {
                let mut bits = 0;
                for (j, &v) in chunks.remainder().iter().enumerate() {
                    bits |= ((v != 0) as u64) << j;
                }
                *word = bits;
            }''' + src[end:]


def packed(src):
    return replace_function(src, 'from_01_string', '''    pub fn from_01_string(s: String) -> Self {
        let mut base = Vec::with_capacity(s.len().div_ceil(64));
        for chunk in s.as_bytes().chunks(64) {
            let mut word = 0;
            let mut groups = chunk.chunks_exact(8);
            for (i, group) in (&mut groups).enumerate() {
                let mut bytes = [0; 8];
                for j in 0..8 { bytes[j] = (group[j] == b'1') as u8; }
                let bits = u64::from_le_bytes(bytes).wrapping_mul(0x0102_0408_1020_4080) >> 56;
                word |= bits << (i * 8);
            }
            let start = chunk.len() & !7;
            for (j, &c) in groups.remainder().iter().enumerate() {
                word |= ((c == b'1') as u64) << (start + j);
            }
            base.push(word);
        }
        Self::from_vec_u64(base)
    }''')


def unroll(src):
    for direction in ['prev', 'next']:
        fn = 'search_' + direction
        src = src.replace(f'fn {fn}<const INCLUSIVE: bool>', f'fn {fn}_depth<const INCLUSIVE: bool, const LEVELS: usize>')
        start = src.index('    fn ' + fn + '_depth')
        end = src.index('\n    }', start) + 6
        part = src[start:end].replace('self.levels', 'LEVELS')
        wrapper = f'''    fn {fn}<const INCLUSIVE: bool>(&self, p: usize) -> usize {{
        match self.levels {{
            1 => self.{fn}_depth::<INCLUSIVE, 1>(p),
            2 => self.{fn}_depth::<INCLUSIVE, 2>(p),
            3 => self.{fn}_depth::<INCLUSIVE, 3>(p),
            4 => self.{fn}_depth::<INCLUSIVE, 4>(p),
            5 => self.{fn}_depth::<INCLUSIVE, 5>(p),
            _ => self.{fn}_depth::<INCLUSIVE, 11>(p),
        }}
    }}
    #[inline(always)]
'''
        # Keep a dynamic fallback so levels 6..11 remain valid.
        fallback = part.replace(fn + '_depth', fn + '_dynamic').replace(', const LEVELS: usize', '').replace('LEVELS', 'self.levels')
        wrapper = wrapper.replace(f'self.{fn}_depth::<INCLUSIVE, 11>(p)', f'self.{fn}_dynamic::<INCLUSIVE>(p)')
        src = src[:start] + wrapper + part + '\n    #[inline(always)]\n' + fallback + src[end:]
    return src


def topdown(src):
    # Preserve the same tree and offsets, reversing the order of the levels.
    src = src.replace('        Self {\n            mx,\n            data: base,', '''        let mut reversed = Vec::with_capacity(total);
        let mut new_offset = [0; 11];
        for i in (0..levels).rev() {
            new_offset[i] = reversed.len();
            let end = if i + 1 == levels { total } else { offset[i + 1] };
            reversed.extend_from_slice(&base[offset[i]..end]);
        }
        let offset = new_offset;
        let base = reversed;
        Self {
            mx,
            data: base,''')
    # new() only contains zeroes, so offsets suffice.
    src = src.replace('        (offset, levels, total)', '''        let original = offset;
        let mut pos = 0;
        for i in (0..levels).rev() {
            offset[i] = pos;
            pos += if i + 1 == levels { total - original[i] } else { original[i + 1] - original[i] };
        }
        (offset, levels, total)''')
    # Constructor needs bottom-up layout during aggregation.
    src = src.replace('let (offset, levels, total) = Self::layout(base.len());', '''let (mut offset, levels, total) = Self::layout(base.len());
        for i in 0..levels { offset[i] = total - offset[i] - if i + 1 == levels { 1 } else { (base.len() >> (i * 6)) + usize::from(base.len() & ((1usize << (i * 6)) - 1) != 0) }; }''')
    src = src.replace('self.data[p >> 6]', 'self.data[self.offset[0] + (p >> 6)]')
    src = src.replace('self.data[leaf_index]', 'self.data[self.offset[0] + leaf_index]')
    return src


def shifted(src):
    for direction in ['prev', 'next']:
        prev = direction == 'prev'
        start = src.index('        let mask = if INCLUSIVE', src.index('    fn search_' + direction))
        end = src.index('        for i in 1..self.levels', start)
        shift = '<< (63 - (p & 63))' if prev else '>> (p & 63)'
        extra = '<< 1' if prev else '>> 1'
        count = 'leading_zeros' if prev else 'trailing_zeros'
        answer = 'p - usize::from(!INCLUSIVE) - word.' + count + '() as usize' if prev else 'p + usize::from(!INCLUSIVE) + word.' + count + '() as usize'
        part = f'''        let word = self.data[p >> 6] {shift};
        let word = if INCLUSIVE {{ word }} else {{ word {extra} }};
        if word != 0 {{ return {answer}; }}
'''
        src = src[:start] + part + src[end:]
    return src


def propagation(src):
    start = src.index('        for i in 1..self.levels', src.index('    pub fn remove'))
    end = src.index('\n    }', start)
    part = src[start:end].replace('            let old = *word;\n', '').replace(' || old == 0', '')
    return src[:start] + part + src[end:]


def iterated(src):
    for name in ['insert', 'remove']:
        start = src.index('    pub fn ' + name)
        end = src.index('\n    }', start)
        part = src[start:end].replace('for i in 1..self.levels', 'for &offset in self.offset.iter().take(self.levels).skip(1)').replace('self.offset[i]', 'offset')
        src = src[:start] + part + src[end:]
    for name in ['search_prev', 'search_next']:
        start = src.index('    fn ' + name)
        end = src.index('\n    }', start)
        part = src[start:end].replace('for i in 1..self.levels', 'for (i, &offset) in self.offset.iter().enumerate().take(self.levels).skip(1)').replace('self.offset[i]', 'offset')
        part = part.replace('for j in (0..i).rev()', 'for &sub_offset in self.offset.iter().take(i).rev()').replace('self.offset[j]', 'sub_offset')
        src = src[:start] + part + src[end:]
    return src


def candidates():
    g = guarded(BASE)
    p = peeled(g)
    combined = packed(chunks(unchecked(p)))
    safe = packed(chunks(p))
    return {'propagation': propagation(combined), 'iterated': iterated(combined),
            'iterated_propagation': iterated(propagation(combined)),
            'shifted': shifted(p), 'shifted_unchecked': unchecked(shifted(p)),
            'combined': combined, 'combined_safe': safe,
            'combined_shifted': packed(chunks(unchecked(shifted(p)))),
            'combined_shifted_safe': packed(chunks(shifted(p))),
            'reserved': combined.replace('Vec::with_capacity(s.len().div_ceil(64))', 'Vec::with_capacity(Self::layout(s.len().div_ceil(64)).2)'),
            'baseline': BASE, 'guarded': g, 'peeled': p, 'unchecked': unchecked(p),
            'nostore': nostore(p), 'root': root(p), 'branchless': branchless(p),
            'chunks': chunks(p), 'packed': packed(p),
            'xor': p.replace('63 - word.leading_zeros()', '63 ^ word.leading_zeros()').replace('63 - self.data[self.offset[j] + res].leading_zeros()', '63 ^ self.data[self.offset[j] + res].leading_zeros()'),
            'unroll': unroll(p), 'topdown': topdown(p),
            'compact': p.replace('[usize; 11]', '[usize; 6]').replace('[0; 11]', '[0; 6]')}


def prepare(directory, selected):
    versions = candidates()
    versions['current'] = (ROOT / 'src/DataStructure/predecessor64.rs').read_text()
    names = selected.split(',')
    common = (ROOT / 'tests/predecessor64_performance/common.rs').read_text()
    common = common[common.index('trait Set:'):common.index('impl_set!(baseline')]
    common += '\nfn rand(' + (ROOT / 'tests/predecessor64_performance/common.rs').read_text().split('fn rand(')[1]
    modules = ''
    for name in names:
        (directory / (name + '.rs')).write_text(versions[name])
        modules += f'mod {name};\nimpl_set!({name}::Predecessor64);\n'
    (directory / 'common.rs').write_text(common + modules)
    for kind in ['bench', 'edges', 'constructors', 'noops']:
        text = (SUPPORT / (kind + '.rs')).read_text()
        arms = '\n'.join(f'{i} => ("{name}", run::<{name}::Predecessor64>(n, &initial, &ops{", kind" if kind in ["edges", "noops"] else ""})),' for i, name in enumerate(names))
        if kind == 'constructors':
            arms = '\n'.join(f'{i} => ("{name}", build::<{name}::Predecessor64>(constructor, &base, &string)),' for i, name in enumerate(names))
        text = text.replace('VARIANTS', str(len(names))).replace('MATCH_ARMS', arms)
        if kind == 'constructors':
            impls = '\n'.join(f'''impl Build for {name}::Predecessor64 {{
                fn from_words(v: Vec<u64>) -> Self {{ Self::from_vec_u64(v) }}
                fn from_usize(v: Vec<usize>) -> Self {{ Self::from_vec_usize(v) }}
                fn from_string(v: String) -> Self {{ Self::from_01_string(v) }}
            }}''' for name in names)
            text = text.replace('BUILD_IMPLS', impls)
        (directory / (kind + '.rs')).write_text(text)
    model = (ROOT / 'tests/predecessor64_performance/model.rs').read_text()
    model = model.replace('include!("model_checks.rs");', (ROOT / 'tests/predecessor64_performance/model_checks.rs').read_text())
    model = model[:model.index('#[test] fn baseline_model')]
    model += '\n'.join(f'#[test] fn {name}_model() {{ verify::<{name}::Predecessor64>(); }}' for name in names)
    (directory / 'model.rs').write_text(model)
    return names


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--variants', default=','.join(candidates()))
    parser.add_argument('--count', type=int, default=500_000)
    parser.add_argument('--repeats', type=int, default=7)
    parser.add_argument('--output', default='predecessor64_argon_candidates')
    parser.add_argument('--toolchain', default='1.89.0')
    parser.add_argument('--native', action='store_true')
    parser.add_argument('--model', action='store_true')
    parser.add_argument('--only', choices=['bench', 'edges', 'constructors', 'noops'])
    args = parser.parse_args()
    output = ROOT / 'tests' / args.output
    records = []
    with tempfile.TemporaryDirectory(prefix='predecessor64-') as tmp:
        directory = pathlib.Path(tmp)
        names = prepare(directory, args.variants)
        rustc = ['rustc', '+' + args.toolchain, '--edition=2021', '-Awarnings']
        if args.model:
            for mode in [[], ['-O']]:
                subprocess.run(rustc + mode + ['--test', str(directory / 'model.rs'), '-o', str(directory / 'model')], check=True)
                subprocess.run([str(directory / 'model')], check=True)
        for kind in ([args.only] if args.only else ['bench', 'edges', 'constructors', 'noops']):
            print('running', kind, flush=True)
            flags = ['-O'] + (['-C', 'target-cpu=native'] if args.native else [])
            binary = directory / kind
            subprocess.run(rustc + flags + [str(directory / (kind + '.rs')), '-o', str(binary)], check=True)
            result = subprocess.run([str(binary), str(args.count), str(args.repeats)], check=True, text=True, capture_output=True)
            rows = list(csv.DictReader(result.stdout.splitlines()))
            for row in rows:
                row['suite'] = kind
                records.append(row)
        fields = list(dict.fromkeys(k for r in records for k in r))
        with output.with_suffix('.csv').open('w') as f:
            writer = csv.DictWriter(f, fields)
            writer.writeheader()
            writer.writerows(records)
        groups = {}
        for row in records:
            key = tuple((k, v) for k, v in row.items() if k not in ['repeat', 'ns_per_op', 'ns_per_build', 'checksum'])
            groups.setdefault(key, []).append(float(row.get('ns_per_op', row.get('ns_per_build'))))
        medians = [{**dict(k), 'median_ns': statistics.median(v), 'min_ns': min(v), 'max_ns': max(v)} for k, v in groups.items()]
        output.with_suffix('.json').write_text(json.dumps({'rustc': subprocess.check_output(rustc + ['--version', '--verbose'], text=True), 'machine': platform.platform(), 'args': vars(args), 'source_sha256': {name: hashlib.sha256((directory / (name + '.rs')).read_bytes()).hexdigest() for name in names}, 'medians': medians}, indent=2) + '\n')
        first = names[0]
        for suite in ['bench', 'edges', 'constructors', 'noops']:
            for name in names[1:]:
                for workload in sorted({m.get('workload', m.get('constructor')) for m in medians if m['suite'] == suite}):
                    ratios = []
                    for m in medians:
                        if m['suite'] != suite or m['implementation'] != name or m.get('workload', m.get('constructor')) != workload: continue
                        key = {k: v for k, v in m.items() if k not in ['implementation', 'median_ns', 'min_ns', 'max_ns']}
                        old = next(v for v in medians if v['implementation'] == first and all(v.get(k) == value for k, value in key.items()))
                        ratios.append(m['median_ns'] / old['median_ns'])
                    if ratios: print(suite, name, workload, f'{math.exp(statistics.mean(map(math.log, ratios))):.3f}', f'range={min(ratios):.3f}..{max(ratios):.3f}')

if __name__ == '__main__':
    main()
