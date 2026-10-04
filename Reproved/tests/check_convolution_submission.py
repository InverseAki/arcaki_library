#!/usr/bin/env python3
"""修正版の提出コードを小ケースとストリーム入出力で検証。--fullは最大入力とRSS計測。"""
from pathlib import Path
import argparse
import os
import random
import subprocess
import sys
import tempfile
import time

ROOT=Path(__file__).resolve().parents[1]
WORKSPACE=ROOT.parents[1]
parser=argparse.ArgumentParser()
parser.add_argument('--full',action='store_true')
parser.add_argument('--source',type=Path,default=WORKSPACE/'librarychecker/submissions/convolution_998244353_memory_fixed.rs')
args=parser.parse_args()
source=args.source.read_text()
with tempfile.TemporaryDirectory(dir=ROOT,prefix='submission-check-') as tmp:
    p=Path(tmp);binary=p/'solution'
    subprocess.run(['rustc','--edition=2021','-O','-Awarnings',str(args.source),'-o',str(binary)],check=True)
    rng=random.Random(71893)
    for n,m in [(0,0),(0,5),(1,1),(4,7),(33,35),(63,65),(100,123),(300,511)]:
        a=[rng.randrange(998244353)for _ in range(n)];b=[rng.randrange(998244353)for _ in range(m)]
        expected=[0]*(n+m-1) if n and m else []
        for i,x in enumerate(a):
            for j,y in enumerate(b):expected[i+j]=(expected[i+j]+x*y)%998244353
        # CRLF、空白、末尾改行なしも含める。
        data=f'{n} {m}\r\n'+ '\t'.join(map(str,a))+'\r\n'+' '.join(map(str,b))
        result=subprocess.run([str(binary)],input=data.encode(),stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=15,check=True)
        assert list(map(int,result.stdout.split()))==expected,(n,m)
    # 入力バッファ境界をまたぐ整数列。
    n=100000
    data=f'{n} 1\n'+'998244352 '*n+'\n1'
    result=subprocess.run([str(binary)],input=data.encode(),stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=15,check=True)
    assert list(map(int,result.stdout.split()))==[998244352]*n
    print('submission small/buffer-boundary cases: PASS',flush=True)
    if args.full:
        # 入力と期待値はストリーム処理し、巨大な入力/出力ファイルは作らない。
        scanner=source[source.index('pub struct Input {'):source.index('// Stringを要素数分')]
        helper='use std::io::{Read,Write};\n'+scanner+r'''
fn main(){
    let args:Vec<String>=std::env::args().collect();let n=1usize<<24;
    if args[1]=="generate" {
        let mut out=std::io::BufWriter::with_capacity(1<<20,std::io::stdout().lock());
        writeln!(out,"{n} {n}").unwrap();let chunk="998244352 ".repeat(4096);
        for _ in 0..2*n/4096{out.write_all(chunk.as_bytes()).unwrap();}
        out.write_all(b"\n").unwrap();out.flush().unwrap();
    }else{
        let mut input=Input::new();
        for i in 0..2*n-1 {
            let actual=input.u32();let expected=(i+1).min(2*n-1-i)as u32;
            assert_eq!(actual,expected,"coefficient {i}");
        }
        while let Some(b)=input.byte(){assert!(b.is_ascii_whitespace(),"unexpected trailing output");}
        eprintln!("all {} coefficients verified",2*n-1);
    }
}
'''
        (p/'helper.rs').write_text(helper)
        subprocess.run(['rustc','--edition=2021','-O','-Awarnings',str(p/'helper.rs'),'-o',str(p/'helper')],check=True)
        start=time.monotonic()
        generate=subprocess.Popen([str(p/'helper'),'generate'],stdout=subprocess.PIPE)
        solution=subprocess.Popen([str(binary)],stdin=generate.stdout,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
        generate.stdout.close()
        checker=subprocess.Popen([str(p/'helper'),'check'],stdin=solution.stdout,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
        solution.stdout.close()
        _,status,usage=os.wait4(solution.pid,0)
        solution.returncode=os.waitstatus_to_exitcode(status)
        error=solution.stderr.read().decode();solution.stderr.close()
        checked=checker.communicate(timeout=120)
        assert generate.wait(timeout=120)==0
        assert solution.returncode==0,error
        assert checker.returncode==0,checked[1].decode()
        rss=usage.ru_maxrss/(2**20 if sys.platform=='darwin' else 1024)
        print(checked[1].decode().strip())
        print(f'full submitted program: PASS; peak RSS={rss:.2f} MiB; pipeline wall={time.monotonic()-start:.2f}s')
