#!/usr/bin/env python3
"""同じ列操作を通常版・ProdMonoid・NilMonoidで比較する。"""
from pathlib import Path
import argparse
import hashlib
import json
import platform
import statistics
import subprocess
import tempfile
ROOT=Path(__file__).resolve().parents[1]
p=argparse.ArgumentParser()
p.add_argument('--n',type=int,default=65536)
p.add_argument('--operations',type=int,default=200000)
p.add_argument('--repeats',type=int,default=5)
p.add_argument('--output',type=Path)
a=p.parse_args()
if a.n<8 or a.operations<1 or a.repeats<1: p.error('n>=8, operations>=1, repeats>=1 required')
code='#![allow(dead_code)]\ninclude!("'+str(ROOT/'src/SegmentTree/splay.rs')+'");\n'+r'''
struct SumValues;
impl SplayMonoid for SumValues {
 type S=i64;
 fn identity()->i64 {0}
 fn op(a:&i64,b:&i64)->i64 {a+b}
 fn reverse_prod(_: &mut i64) {}
}
struct Scale;
impl SplayLazyMonoid for Scale {
 type M=SumValues; type F=i64;
 fn identity()->i64 {1}
 fn map(f:&i64,x:&i64)->i64 {f*x}
 fn composition(f:&i64,g:&i64)->i64 {f*g}
}
struct Concat;
impl SplayMonoid for Concat {
 type S=String;
 fn identity()->String {String::new()}
 fn op(a:&String,b:&String)->String {format!("{a}{b}")}
 fn reverse_prod(x: &mut String) {*x=x.chars().rev().collect();}
}
struct NoChange;
impl SplayLazyMonoid for NoChange {
 type M=Concat; type F=i64;
 fn identity()->i64 {0}
 fn map(_: &i64,x:&String)->String {x.clone()}
 fn composition(f:&i64,g:&i64)->i64 {f+g}
}
fn run<C: SplaySpec<Value=i64>>(work:&str,n:usize,q:usize)->(f64,u64) {
 let mut t=SplayTree::<C>::from_vec((0..n).map(|i|i as i64).collect());
 let mut seed=71829u64;let mut checksum=0u64;
 let start=std::time::Instant::now();
 for i in 0..q {
  seed^=seed<<13;seed^=seed>>7;seed^=seed<<17;
  let k=seed as usize%n;
  match work {
   "random_get" => {checksum=checksum.wrapping_mul(37).wrapping_add(t.get(k) as u64);}
   "local_get" => {checksum=checksum.wrapping_mul(37).wrapping_add(t.get(n/2+i%8) as u64);}
   "mixed" => match i%3 {
    0=> t.set(k,(seed%1000000)as i64),
    1=> {let r=k+1+(seed>>32)as usize%(n-k);t.reverse(k,r);}
    _=>{checksum=checksum.wrapping_mul(37).wrapping_add(t.get(k) as u64);}
   },
   "churn"=> {checksum=checksum.wrapping_mul(37).wrapping_add(t.remove(k) as u64);t.insert(k,(seed%1000000)as i64);}
   _=>panic!(),
  }
 }
 let elapsed=start.elapsed().as_secs_f64();
 for x in t.to_vec() {checksum=checksum.wrapping_mul(37).wrapping_add(x as u64);}
 (elapsed,checksum)
}
fn main(){
 let a:Vec<String>=std::env::args().collect();
 if a[1]=="layout" {
  println!("{} {} {} {} {} {}",
   std::mem::size_of::<Node<Scale>>(),std::mem::size_of::<Node<ProdMonoid<SumValues>>>(),std::mem::size_of::<Node<NilMonoid<i64>>>(),
   std::mem::size_of::<Node<NoChange>>(),std::mem::size_of::<Node<ProdMonoid<Concat>>>(),std::mem::size_of::<Node<NilMonoid<String>>>());
  return;
 }
 let n=a[3].parse().unwrap();let q=a[4].parse().unwrap();
 let(s,c)=match a[1].as_str(){
 "lazy"=>run::<Scale>(&a[2],n,q),
 "prod"=>run::<ProdMonoid<SumValues>>(&a[2],n,q),
 "nil"=>run::<NilMonoid<i64>>(&a[2],n,q),
 _=>panic!()};
 println!("{s:.9} {c}");
}
'''
report={'n':a.n,'operations':a.operations,'repeats':a.repeats,'platform':platform.platform(),'rustc':subprocess.check_output(['rustc','--version'],text=True).strip(),'source_sha256':hashlib.sha256((ROOT/'src/SegmentTree/splay.rs').read_bytes()).hexdigest(),'workloads':{}}
with tempfile.TemporaryDirectory(prefix='splay-modes-') as d:
    src=Path(d)/'bench.rs';binary=Path(d)/'bench';src.write_text(code)
    subprocess.run(['rustc','--edition=2021','-O',str(src),'-o',str(binary)],check=True)
    layout=list(map(int,subprocess.check_output([str(binary),'layout'],text=True).split()))
    report['node_bytes']={typ:dict(zip(['lazy','prod','nil'],sizes)) for typ,sizes in [('i64',layout[:3]),('String',layout[3:])]}
    print('node_bytes',report['node_bytes'],flush=True)
    for work in ['random_get','local_get','mixed','churn']:
        samples={label:[] for label in ['lazy','prod','nil']};checks={}
        for trial in range(a.repeats):
            labels=list(samples)
            # 実行順を循環させ、特定の型が常に先頭にならないようにする。
            labels=labels[trial%3:]+labels[:trial%3]
            for label in labels:
                s,c=subprocess.check_output([str(binary),label,work,str(a.n),str(a.operations)],text=True).split()
                samples[label].append(float(s));checks[label]=int(c)
                assert len(set(checks.values()))==1,(work,checks)
        med={label:statistics.median(x) for label,x in samples.items()}
        report['workloads'][work]={'seconds':med,'samples_seconds':samples,'checksum':checks['lazy'],'speedup_vs_lazy':{label:med['lazy']/med[label] for label in ['prod','nil']}}
        print(work,json.dumps(report['workloads'][work]),flush=True)
if a.output: a.output.write_text(json.dumps(report,indent=2)+'\n')
