#!/usr/bin/env python3
"""Compare pasted baseline and current library: arithmetic timing excludes parse/format."""
from pathlib import Path
import subprocess,tempfile,argparse,json,hashlib,platform,statistics
ROOT=Path(__file__).resolve().parents[2]
p=argparse.ArgumentParser();p.add_argument('--repeats',type=int,default=5);p.add_argument('--output',type=Path);p.add_argument('--target');p.add_argument('--before-source',type=Path);p.add_argument('--after-source',type=Path);p.add_argument('--workload',action='append');p.add_argument('--quick',action='store_true');a=p.parse_args()
code='#![allow(dead_code)]\n'
body=r'''
pub fn run(op:&str,n:usize,m:usize,base:u32)->(f64,String) {
 fn go<const B:u32>(op:&str,n:usize,m:usize)->(f64,String){
  let radix=if B==10000 {10u64}else{16};let mut seed=271891u64;
  let mut data=|len|{let mut s=String::with_capacity(len);for i in 0..len {seed^=seed<<13;seed^=seed>>7;seed^=seed<<17;let d=if i==0 {1+seed%(radix-1)} else {seed%radix};s.push(b"0123456789ABCDEF"[d as usize]as char);}s};
  let xs=data(n);let ys=data(m);let x:RadixBigInt<B>=xs.parse().unwrap();let y:RadixBigInt<B>=ys.parse().unwrap();
  let start=std::time::Instant::now();
  if op=="format" {let result=format!("{} {}",x,y);let t=start.elapsed().as_secs_f64();return(t,result);}
  if op=="parse" {let x:RadixBigInt<B>=xs.parse().unwrap();let y:RadixBigInt<B>=ys.parse().unwrap();let t=start.elapsed().as_secs_f64();return(t,format!("{} {}",x,y));}
  let x=std::hint::black_box(&x);let y=std::hint::black_box(&y);
   if op=="mul" || op=="add" || op=="sub" {let r=match op {"mul"=>x*y,"add"=>x+y,_=>x-y};let elapsed=start.elapsed().as_secs_f64();let result=r.to_string();std::hint::black_box(r);return(elapsed,result);}
   else {let(q,r)=x.div_rem(y);let elapsed=start.elapsed().as_secs_f64();let result=format!("{} {}",q,r);std::hint::black_box((q,r));return(elapsed,result);}
 } if base==10 {go::<10000>(op,n,m)}else {go::<65536>(op,n,m)}
}
'''
files={'before':a.before_source or Path('tests/support/big_integer_ntt_baseline.rs'),'after':a.after_source or Path('src/NumberTheory/big_integer.rs')}
for label,f in files.items():code+=f'mod {label} {{ include!({json.dumps(str(ROOT/f))}); {body} }}\n'
code+=r'''fn main(){let a:Vec<_>=std::env::args().collect();if a[1]=="features" {
#[cfg(target_arch="x86_64")] println!("avx2={}",std::is_x86_feature_detected!("avx2"));
#[cfg(target_arch="aarch64")] println!("neon={}",std::arch::is_aarch64_feature_detected!("neon"));return;}let n=a[3].parse().unwrap();let m=a[4].parse().unwrap();let b=a[5].parse().unwrap();let(t,s)=if a[1]=="before"{before::run(&a[2],n,m,b)}else{after::run(&a[2],n,m,b)};println!("{t}");println!("{s}");}'''
work=[('parse',200000,100000),('format',200000,100000),('add',200000,100000),('sub',200000,100000),('mul',64,64),('div',128,64),('div',4096,64),('div',4096,4000),('mul',1024,1024),('mul',32768,32768),('div',4096,2048),('div',65536,32768),('div',200000,100000),('div',200000,1000)]
if not a.quick:work += [('div',2000000,1000000),('div',2000000,200000),('div',2000000,1000)]
report={'platform':platform.platform(),'target':a.target,'method':'operation timing; arithmetic excludes parse/format; explicit parse and format workloads are timed separately; alternating order; exact output equality','rustc':subprocess.check_output(['rustc','--version'],text=True).strip(),'repeats':a.repeats,'sha256':{k:hashlib.sha256((ROOT/f).read_bytes()).hexdigest()for k,f in files.items()},'workloads':{}}
with tempfile.TemporaryDirectory(prefix='bigint-ntt-bench-') as tmp:
 src=Path(tmp)/'bench.rs';binary=Path(tmp)/'bench';src.write_text(code)
 flags=['--target',a.target] if a.target else []
 subprocess.run(['rustc','--edition=2021','-O',str(src),'-o',str(binary),*flags],check=True)
 report['runtime_simd']=subprocess.check_output([str(binary),"features"],text=True).strip()
 for base in [10,16]:
  for op,n,m in work:
   if a.workload and f'{base}_{op}_{n}_{m}' not in a.workload:continue
   samples={'before':[],'after':[]};expected=None
   for i in range(a.repeats):
    for label in (['before','after'] if i%2==0 else ['after','before']):
     text=subprocess.check_output([str(binary),label,op,str(n),str(m),str(base)],text=True);t,result=text.split('\n',1)
     if expected is None:expected=result
     assert expected==result,(base,op,n,m,label)
     samples[label].append(float(t))
   med={k:statistics.median(v) for k,v in samples.items()};speed=med['before']/med['after']
   report['workloads'][f'{base}_{op}_{n}_{m}']={'seconds':med,'samples':samples,'speedup':speed,'result_sha256':hashlib.sha256(expected.encode()).hexdigest()}
   print(base,op,n,m,f"{med['before']*1000:.3f} -> {med['after']*1000:.3f} ms ({speed:.2f}x)",flush=True)
if a.output:a.output.write_text(json.dumps(report,indent=2)+'\n')
