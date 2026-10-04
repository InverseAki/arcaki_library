#!/usr/bin/env python3
"""Library Checker 6問用の、単体で提出できるmain.rsを生成する。"""
from pathlib import Path
import argparse
ROOT=Path(__file__).resolve().parents[2]
PROBLEMS={f'{op}_of_{radix}big_integers':(operation,'HexBigInt' if radix else 'BigInt')
          for op,operation in [('addition','add'),('multiplication','mul'),('division','div')]
          for radix in ['', 'hex_']}
p=argparse.ArgumentParser();p.add_argument('--output-dir',type=Path,required=True);p.add_argument('--problem',choices=list(PROBLEMS));args=p.parse_args()
args.output_dir.mkdir(parents=True,exist_ok=True)
library=(ROOT/'src/NumberTheory/big_integer.rs').read_text()
for problem in ([args.problem] if args.problem else PROBLEMS):
    operation,kind=PROBLEMS[problem]
    body={'add':'(&a + &b).append_to(&mut output);','mul':'(&a * &b).append_to(&mut output);',
          'div':"let (q,r)=a.div_rem(&b); q.append_to(&mut output); output.push(' '); r.append_to(&mut output);"}[operation]
    main=f'''
fn main() {{
    use std::io::{{Read,Write}};
    let mut input=String::new();std::io::stdin().read_to_string(&mut input).unwrap();
    let mut tokens=input.split_whitespace();let t:usize=tokens.next().unwrap().parse().unwrap();
    let mut output=String::with_capacity(input.len()*2);
    for _ in 0..t {{
        let a:{kind}=tokens.next().unwrap().parse().unwrap();
        let b:{kind}=tokens.next().unwrap().parse().unwrap();
        {body}
        output.push('\\n');
    }}
    std::io::stdout().lock().write_all(output.as_bytes()).unwrap();
}}
'''
    dest=args.output_dir/(problem+'.rs');dest.write_text(library+main);print(dest)
