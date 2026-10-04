#!/usr/bin/env python3
"""同一の全幅ランダム入力で旧版/新版の実時間と全係数を比較する。"""
from pathlib import Path
import argparse
import subprocess
import tempfile
import sys
import os
root=Path(__file__).resolve().parents[1]
parser=argparse.ArgumentParser()
parser.add_argument('--powers',type=int,nargs='+',default=[12,18,22,24])
parser.add_argument('--repeats',type=int,default=3)
parser.add_argument('--skip-baseline',action='store_true')
parser.add_argument('--right-length',type=int,help='右入力長（省略時は左と同じ）')
parser.add_argument('--measure-memory',action='store_true',help='wait4で子プロセスの最大RSSを記録（POSIX）')
args=parser.parse_args()
if args.right_length is not None and not 1<=args.right_length<=1<<24: parser.error('invalid right length')
if args.repeats<1 or any(p<1 or p>24 for p in args.powers): parser.error('invalid powers/repeats')
s=(root.parents[1]/'librarychecker/src/main.rs').read_text()
start=s.index('pub type ModInt998244353 =');end=s.index('#[allow(unused_imports)]\nuse std::{',start)
code='use std::{marker::PhantomData,ops::*,str::FromStr};\n'+s[start:end]
for name,path in [('baseline','tests/support/convolution_998244353_baseline.rs'),('candidate','../src/Fps/convolution_mod998244353.rs')]:
    body=(root/path).read_text()
    code+=f'mod {name} {{ use super::*;\n'+body+'\n}\n'
code+='''
fn main(){
 let args:Vec<String>=std::env::args().collect();
 let power:usize=args[1].parse().unwrap();let repeats:usize=args[2].parse().unwrap();let baseline=&args[3]=="yes";
 let n=1usize<<power;let m:usize=args[4].parse().unwrap();let mut seed=123456789u64;
 let mut make=|len| (0..len).map(|_|{seed^=seed<<13;seed^=seed>>7;seed^=seed<<17;ModInt998244353::new(seed%998244353)}).collect::<Vec<_>>();
 let a=make(n);let b=make(m);
 let expected=if baseline {
  let start=std::time::Instant::now();let c=baseline::convolution_mod998244353(std::hint::black_box(&a),std::hint::black_box(&b));
  println!("n=2^{power},m={m} baseline {:.6}s",start.elapsed().as_secs_f64());Some(c)
 }else{None};
 let mut samples=Vec::new();
 for _ in 0..repeats {
  let start=std::time::Instant::now();let c=candidate::convolution_mod998244353(std::hint::black_box(&a),std::hint::black_box(&b));
  let elapsed=start.elapsed().as_secs_f64();samples.push(elapsed);
  if let Some(ref expected)=expected {assert!(c.iter().zip(expected).all(|(a,b)|a.val()==b.val()));assert_eq!(c.len(),expected.len());}
  std::hint::black_box(&c);println!("n=2^{power},m={m} candidate {elapsed:.6}s");
 }
 samples.sort_by(|a,b|a.partial_cmp(b).unwrap());println!("n=2^{power},m={m} candidate median {:.6}s",samples[repeats/2]);
}
'''
with tempfile.TemporaryDirectory(dir=root,prefix='conv-bench-') as tmp:
    p=Path(tmp);(p/'bench.rs').write_text(code)
    subprocess.run(['rustc','--edition=2021','-O','-Awarnings',str(p/'bench.rs'),'-o',str(p/'bench')],check=True)
    for power in args.powers:
        command=[str(p/'bench'),str(power),str(args.repeats),'no' if args.skip_baseline else 'yes',str(args.right_length or (1<<power))]
        if args.measure_memory:
            if not hasattr(os, 'wait4'): parser.error('--measure-memory requires POSIX wait4')
            process=subprocess.Popen(command)
            _,status,usage=os.wait4(process.pid,0)
            process.returncode=os.waitstatus_to_exitcode(status)
            if process.returncode: raise subprocess.CalledProcessError(process.returncode,command)
            rss_bytes=usage.ru_maxrss if sys.platform=='darwin' else usage.ru_maxrss*1024
            print(f'process peak RSS: {rss_bytes/2**20:.2f} MiB',flush=True)
        else:
            subprocess.run(command,check=True)
