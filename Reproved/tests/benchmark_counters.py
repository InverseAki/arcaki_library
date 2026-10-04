#!/usr/bin/env python3
"""統合前の保存版と現行srcを同一操作列で簡易比較（rustc -O、5回の中央値）。"""
from pathlib import Path
import subprocess
import tempfile

root = Path(__file__).resolve().parents[2]
code = 'use std::{time::Instant, hint::black_box};\n'
variants = [
    ('old_c', 'Counter', 'Reproved/tests/support/couter_baseline.rs'),
    ('new_c', 'Counter', 'src/Basic/couter.rs'),
    ('old_h', 'HashCounter', 'Reproved/tests/support/hashcounter_baseline.rs'),
    ('new_h', 'HashCounter', 'src/Basic/hashcounter.rs'),
]
for module, _, file in variants:
    code += f'mod {module} {{ use std::{{collections::{{BTreeMap,HashMap,btree_map::Range as BTreeRange}},ops::RangeBounds,hash::Hash,mem::swap}};\n'
    code += (root / file).read_text() + '\n}\n'
code += 'fn main(){\n'
for module, typ, _ in variants:
    code += '''{
        let mut samples=Vec::new();
        for _ in 0..5 {
            let mut c=MODULE::TYPE::new();
            for i in 0..4096u64 { c.add(i,4); }
            let mut seed=123u64;
            let start=Instant::now();
            for _ in 0..500000 {
                seed^=seed<<13; seed^=seed>>7; seed^=seed<<17;
                let x=seed&4095;
                c.one_sub(black_box(x)); c.one_add(black_box(x));
            }
            black_box(c);
            samples.push(start.elapsed().as_secs_f64()*1000.0);
        }
        samples.sort_by(|a,b|a.partial_cmp(b).unwrap());
        println!("MODULE median {:.2} ms",samples[2]);
    }\n'''.replace('MODULE', module).replace('TYPE', typ)
code += '}\n'
with tempfile.TemporaryDirectory(dir=root/'Reproved', prefix='counter-bench-') as tmp:
    p = Path(tmp)
    (p/'bench.rs').write_text(code)
    subprocess.run(['rustc', '--edition=2021', '-O', '-Awarnings', str(p/'bench.rs'), '-o', str(p/'bench')], check=True)
    subprocess.run([str(p/'bench')], check=True)
