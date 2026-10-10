#!/usr/bin/env python3
"""統合前と現行Splay木を同じ操作列で比較。取得・外部依存なし。"""
from pathlib import Path
import argparse
import json
import hashlib
import platform
import statistics
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
p = argparse.ArgumentParser()
p.add_argument('--n',type=int,default=32768)
p.add_argument('--operations',type=int,default=100000)
p.add_argument('--repeats',type=int,default=5)
p.add_argument('--output',type=Path)
args=p.parse_args()
if args.n<2 or args.operations<1 or args.repeats<1: p.error('n>=2, operations>=1, repeats>=1 required')
monoids=r'''
pub struct BenchSum;
impl SplayMonoid for BenchSum {
 type S=(i64,usize);
 fn identity()->Self::S {(0,0)}
 fn op(a:&Self::S,b:&Self::S)->Self::S {((a.0+b.0)%1000000007,a.1+b.1)}
 fn reverse_prod(_: &mut Self::S) {}
}
pub struct BenchAffine;
impl SplayLazyMonoid for BenchAffine {
 type M=BenchSum; type F=(i64,i64);
 fn identity()->Self::F {(1,0)}
 fn map(f:&Self::F,x:&(i64,usize))->(i64,usize) {((f.0*x.0+f.1*x.1 as i64)%1000000007,x.1)}
 fn composition(f:&Self::F,g:&Self::F)->Self::F {((f.0*g.0)%1000000007,(f.0*g.1+f.1)%1000000007)}
}
pub fn run(workload:&str,n:usize,q:usize)->(f64,u64) {
 let mut t=SplayTree::<BenchAffine>::new();
 let build=std::time::Instant::now();
 for i in 0..n {t.insert(i,(i as i64,1));}
 if workload=="build" {return (build.elapsed().as_secs_f64(),t.prod(0,n).0 as u64);}
 let mut seed=71391829u64; let mut checksum=0u64;
 let start=std::time::Instant::now();
 for step in 0..q {
  seed^=seed<<13;seed^=seed>>7;seed^=seed<<17;
  let l=(seed as usize)%n;
  let r=l+1+((seed>>32)as usize)%(n-l);
  match workload {
   "queries"=> {checksum=checksum.wrapping_mul(37).wrapping_add(t.prod(l,r).0 as u64);}
   "local"=> {let k=n/2+step%8;checksum=checksum.wrapping_mul(37).wrapping_add(t.prod(k,k+1).0 as u64);}
   "whole_apply"=> {t.apply(0,n,(1,1));if step%16==0 {checksum=checksum.wrapping_mul(37).wrapping_add(t.prod(l,l+1).0 as u64);}}
   "mixed"=> match step%4 {
     0=>t.apply(l,r,((seed%3)as i64,((seed>>16)%100)as i64)),
     1=>t.reverse(l,r),
     _=> {checksum=checksum.wrapping_mul(37).wrapping_add(t.prod(l,r).0 as u64);}
   },
   "churn"=> {t.erase(l); t.insert(l,((seed%1000000007)as i64,1)); if step%8==0 {checksum=checksum.wrapping_mul(37).wrapping_add(t.prod(l,r).0 as u64);}}
   _=>panic!("unknown workload"),
  }
 }
 let elapsed=start.elapsed().as_secs_f64();
 checksum=checksum.wrapping_mul(37).wrapping_add(t.prod(0,n).0 as u64);
 (elapsed,checksum)
}
'''
code=''
for label,file in [('before','tests/support/splay_baseline.rs'),('after','src/DataStructure/splay.rs')]:
    code+=f'mod {label} {{ use std::{{fmt::Debug,ptr::null_mut,mem::swap}};\n'+(ROOT/file).read_text()+monoids+'\n}\n'
code+=r'''
fn main(){
 let a:Vec<String>=std::env::args().collect();let n=a[3].parse().unwrap();let q=a[4].parse().unwrap();
 let (s,c)=match a[1].as_str(){
 "before"=>before::run(&a[2],n,q),
 "after"=>after::run(&a[2],n,q),
 "bulk"=>{let v=(0..n).map(|i|(i as i64,1)).collect();let start=std::time::Instant::now();let mut t=after::SplayTree::<after::BenchAffine>::from_vec(v);(start.elapsed().as_secs_f64(),t.prod(0,n).0 as u64)},
 _=>panic!()};
 println!("{s:.9} {c}");
}
'''
report={'platform':platform.platform(),'source_sha256':{name:hashlib.sha256((ROOT/file).read_bytes()).hexdigest() for name,file in [('before','tests/support/splay_baseline.rs'),('after','src/DataStructure/splay.rs')]},'n':args.n,'operations':args.operations,'repeats':args.repeats,'rustc':subprocess.check_output(['rustc','--version'],text=True).strip(),'workloads':{}}
with tempfile.TemporaryDirectory(prefix='splay-bench-') as d:
    src=Path(d)/'bench.rs';binary=Path(d)/'bench'
    src.write_text(code)
    subprocess.run(['rustc','--edition=2021','-O','-Awarnings',str(src),'-o',str(binary)],check=True)
    for workload in ['build','queries','local','whole_apply','mixed','churn']:
        samples={label:[] for label in ['before','after']};checks={}
        if workload=='build': samples['bulk']=[]
        # 順序を交互に入れ替え、各バックエンドを独立したプロセスで計測。
        for trial in range(args.repeats):
            labels=list(samples)
            if trial%2: labels.reverse()
            for label in labels:
                result=subprocess.check_output([str(binary),label,workload,str(args.n),str(args.operations)],text=True).split()
                samples[label].append(float(result[0]));checks[label]=int(result[1])
                assert len(set(checks.values()))==1,(workload,checks)
        medians={label:statistics.median(vals) for label,vals in samples.items()}
        report['workloads'][workload]={'seconds':medians,'samples_seconds':samples,'checksum':checks['before'],'speedup':medians['before']/medians['after']}
        print(workload,json.dumps(report['workloads'][workload]),flush=True)
if args.output:
    args.output.write_text(json.dumps(report,indent=2)+'\n')
