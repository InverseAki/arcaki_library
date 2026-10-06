#!/usr/bin/env python3
"""MI integration tests: local bundled MI and optional installed ACL. No downloads."""
from pathlib import Path
import argparse, subprocess, tempfile, json
ROOT=Path(__file__).resolve().parents[1]
p=argparse.ArgumentParser();p.add_argument('--acl-rlib',type=Path);a=p.parse_args()
s=(ROOT.parent/'librarychecker/src/main.rs').read_text();start=s.index('pub type ModInt1000000007 =');end=s.index('#[allow(unused_imports)]\nuse std::{',start)
for backend,prefix,flags in [('bundled','use std::{marker::PhantomData,ops::*,str::FromStr};\n'+s[start:end],[])]+([('ACL','use ac_library::*;', ['--extern',f'ac_library={a.acl_rlib.resolve()}','-L',f'dependency={a.acl_rlib.resolve().parent}'])] if a.acl_rlib else []):
    code='#![allow(dead_code,unused_imports)]\n'+prefix+'\n'
    for label,mi in [('p998','ModInt998244353'),('p1e9','ModInt1000000007')]:
        code+=f'mod {label} {{ type MI=super::{mi}; include!({json.dumps(str(ROOT/"src/Basic/mint_matrix.rs"))});\n'+r'''
#[test] fn product_power_inverse(){
 let m=(MI::new(0)-MI::new(1)).val() as u64+1;
 let mut seed=1238918u64;
 for n in [0,1,2,8,15,16,17,31,32,33,63,64,65,129] {
  let values:Vec<_>=(0..n*n).map(|_|{seed^=seed<<13;seed^=seed>>7;seed^=seed<<17;MI::new((seed%m)as u32)}).collect();
  let a=Matrix::new(n,values);let b=a.mul(&a);
  for i in 0..n {for j in 0..n {
   let sum=(0..n).fold(0u128,|s,k|s+a[(i,k)].val()as u128*a[(k,j)].val()as u128);
   assert_eq!(b[(i,j)].val(),(sum%m as u128)as u32);
  }}
  assert_eq!(a.pow(0),Matrix::identity(n));assert_eq!(a.pow(2),b);
 }
 let a=Matrix::new(2,vec![MI::new(0),MI::new(1),MI::new(1),MI::new(3)]);
 assert_eq!(a.mul(&a.inv()),Matrix::identity(2));
 assert!(Matrix::zeros(2).try_inv().is_none());assert_eq!(Matrix::zeros(0).inv(),Matrix::identity(0));
}
}
'''
    # The old usize-only Mint also exercises the full-u32 conversion branch.
    code+=f'mod wide {{ include!({json.dumps(str(ROOT/"src/NumberTheory/mint.rs"))}); type MI=Mint<4294967291>; include!({json.dumps(str(ROOT/"src/Basic/mint_matrix.rs"))});\n'+r'''
#[test] fn wide_modulus(){let n=17;let m=4294967291usize;let a=Matrix::new(n,vec![MI::new(m-1);n*n]);let b=a.mul(&a);assert!(b.as_slice().iter().all(|x|x.val()==n));}
}
'''
    with tempfile.TemporaryDirectory(prefix='matrix-check-') as d:
        src=Path(d)/'check.rs';src.write_text(code)
        for label,opt in [('debug',[]),('release',['-O'])]:
            binary=Path(d)/label
            subprocess.run(['rustc','--edition=2021','--test',str(src),'-o',str(binary),*opt,*flags],check=True)
            subprocess.run([str(binary)],check=True)
            print(backend,label,'passed',flush=True)
