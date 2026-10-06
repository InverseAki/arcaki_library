#!/usr/bin/env python3
"""Local old/new matrix comparison. No downloads; bundled MI extracted locally."""
from pathlib import Path
import argparse, subprocess, tempfile, json, statistics, platform, hashlib
ROOT=Path(__file__).resolve().parents[1]
p=argparse.ArgumentParser()
p.add_argument('--sizes',default='2,4,8,16,32,64,128,256')
p.add_argument('--repeats',type=int,default=5)
p.add_argument('--output',type=Path)
p.add_argument('--acl-rlib',type=Path)
a=p.parse_args()
sizes=[int(x) for x in a.sizes.split(',')]
assert all(n>0 for n in sizes) and a.repeats>0
source=(ROOT.parent/'librarychecker/src/main.rs').read_text()
start=source.index('pub type ModInt1000000007 =')
end=source.index('#[allow(unused_imports)]\nuse std::{',start)
code='use std::{marker::PhantomData,ops::*,str::FromStr};\n'+source[start:end]
if a.acl_rlib: code='use ac_library::*;\n'
files=['tests/support/doubling_matrix_baseline.rs','tests/support/mint_matrix_baseline.rs','src/Basic/matrix.rs','src/Basic/doubling_matrix.rs','src/Basic/mint_matrix.rs']
for label,path in [('old_d','tests/support/doubling_matrix_baseline.rs'),('new_d','src/Basic/doubling_matrix.rs'),('old_m','tests/support/mint_matrix_baseline.rs'),('new_m','src/Basic/mint_matrix.rs')]:
    body=(ROOT/path).read_text() if label=='old_m' else f'include!({json.dumps(str(ROOT/path))});'
    # Only fix the pre-existing inverse borrow-check error; product stays unchanged.
    body=body.replace('a[(row, j)] -= factor * a[(col, j)];','let v = a[(col,j)]; a[(row,j)] -= factor * v;').replace('b[(row, j)] -= factor * b[(col, j)];','let v = b[(col,j)]; b[(row,j)] -= factor * v;')
    code+=f'mod {label} {{ const MOD:i64=998244353; type MI=super::ModInt998244353; {body} }}\n' 
code+=f'mod raw {{ include!({json.dumps(str(ROOT/"src/Basic/matrix.rs"))}); }}\n'
code+=r'''
use std::{hint::black_box,time::Instant};
fn data(n:usize,sparse:bool)->Vec<u32>{let mut seed=127918u64;(0..n*n).map(|i|{seed^=seed<<13;seed^=seed>>7;seed^=seed<<17;if sparse && i%10!=0 {0} else {(seed%998244353)as u32}}).collect()}
fn main(){let args:Vec<_>=std::env::args().collect();let label=&args[1];let n:usize=args[2].parse().unwrap();let q:usize=args[3].parse().unwrap();let v=data(n,args[4]=="sparse");
 if label.ends_with("min") {
  let iv:Vec<_>=v.iter().map(|&x|if x==0 {1i64<<60} else {(x%1000)as i64}).collect();
  let old=old_d::DoublingMatrix::<old_d::MinPlusMonoid>::new(n,&iv);
  let new=new_d::DoublingMatrix::<new_d::MinPlusMonoid>::new(n,&iv);
  let expected=old.prod(&old);let actual=new.prod(&new);
  for i in 0..n {for j in 0..n {assert_eq!(expected.get(i,j),actual.get(i,j));}}
  let start=Instant::now();let mut checksum=0u64;
  for _ in 0..q {let value=if label=="old_min" {let r=black_box(&old).prod(black_box(&old));let v=*r.get(n/2,n/2);black_box(r);v} else {let r=black_box(&new).prod(black_box(&new));let v=*r.get(n/2,n/2);black_box(r);v};checksum=checksum.wrapping_add(value as u64);}
  println!("{} {}",start.elapsed().as_secs_f64()/q as f64,checksum);return;
 }
 let iv:Vec<_>=v.iter().map(|&x|x as i64).collect();let mv:Vec<_>=v.iter().map(|&x|ModInt998244353::new(x)).collect();
 // Check every cell before timing, including both preserved entrypoints.
 let reference=old_d::DoublingMatrix::<old_d::AddMulMonoid>::new(n,&iv).prod(&old_d::DoublingMatrix::<old_d::AddMulMonoid>::new(n,&iv));
 macro_rules! measure {($mat:expr,$mul:ident,$value:expr)=>{{let mat=$mat;let check=mat.$mul(&mat);for i in 0..n {for j in 0..n {assert_eq!(($value)(&check,i,j),*reference.get(i,j) as u32);}}let start=Instant::now();let mut checksum=0u64;for _ in 0..q {let r=black_box(&mat).$mul(black_box(&mat));checksum=checksum.wrapping_add(($value)(black_box(&r),n/2,n/2)as u64);black_box(r);}println!("{} {}",start.elapsed().as_secs_f64()/q as f64,checksum);}}}
 match label.as_str(){
 "old_d"=>measure!(old_d::DoublingMatrix::<old_d::AddMulMonoid>::new(n,&iv),prod,|r:&old_d::DoublingMatrix<old_d::AddMulMonoid>,i,j|*r.get(i,j)as u32),
 "new_d"=>measure!(new_d::DoublingMatrix::<new_d::AddMulMonoid>::new(n,&iv),prod,|r:&new_d::DoublingMatrix<new_d::AddMulMonoid>,i,j|*r.get(i,j)as u32),
 "old_m"=>measure!(old_m::Matrix::new(n,mv),mul,|r:&old_m::Matrix,i,j|r[(i,j)].val()),
 "new_m"=>measure!(new_m::Matrix::new(n,mv),mul,|r:&new_m::Matrix,i,j|r[(i,j)].val()),
 "raw"=>measure!(raw::SquareMatrix::<raw::ModMatrixMonoid<998244353>>::new(n,v),mul,|r:&raw::SquareMatrix<raw::ModMatrixMonoid<998244353>>,i,j|r[(i,j)]),
 _=>panic!()}}
'''
report={'platform':platform.platform(),'rustc':subprocess.check_output(['rustc','--version'],text=True).strip(),'repeats':a.repeats,'flags':['-O'],'backend':'ACL' if a.acl_rlib else 'bundled MI','sha256':{f:hashlib.sha256((ROOT/f).read_bytes()).hexdigest() for f in files},'workloads':{}}
with tempfile.TemporaryDirectory(prefix='matrix-bench-') as d:
    src=Path(d)/'bench.rs';binary=Path(d)/'bench';src.write_text(code)
    flags=[]
    if a.acl_rlib:
        lib=a.acl_rlib.resolve();flags=['--extern',f'ac_library={lib}','-L',f'dependency={lib.parent}']
    subprocess.run(['rustc','--edition=2021','-O','-Awarnings',str(src),'-o',str(binary),*flags],check=True)
    for pattern in ['dense','sparse']:
        for n in sizes:
            q=max(1,min(20000,4000000//(n**3)))
            samples={k:[] for k in ['old_d','new_d','old_m','new_m','raw','old_min','new_min']};checks={'mod':set(),'min':set()}
            for trial in range(a.repeats):
                labels=list(samples)
                if trial%2: labels.reverse()
                for label in labels:
                    sec,check=subprocess.check_output([str(binary),label,str(n),str(q),pattern],text=True).split();samples[label].append(float(sec));checks['min' if label.endswith('min') else 'mod'].add(int(check))
            assert all(len(x)==1 for x in checks.values())
            median={k:statistics.median(v) for k,v in samples.items()}
            result={'seconds':median,'samples':samples,'iterations':q,'doubling_speedup':median['old_d']/median['new_d'],'mint_speedup':median['old_m']/median['new_m'],'raw_vs_old_mint':median['old_m']/median['raw'],'minplus_speedup':median['old_min']/median['new_min']}
            report['workloads'][f'{pattern}_{n}']=result
            print(pattern,n,json.dumps({k:round(v*1e6,2)for k,v in median.items()}),'us',flush=True)
if a.output: a.output.write_text(json.dumps(report,indent=2)+'\n')
