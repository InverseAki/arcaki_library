#!/usr/bin/env python3
"""Python intと比較。両基数・符号・閾値・逆数・大きさ比・構造的難例。"""
from pathlib import Path
import argparse, random, subprocess, sys, tempfile, time
sys.set_int_max_str_digits(0)
ROOT=Path(__file__).resolve().parents[2]
p=argparse.ArgumentParser();p.add_argument('--binary',type=Path);p.add_argument('--large',action='store_true');args=p.parse_args()
rng=random.Random(20261002)

def fmt(x,radix):
    if radix==10:return str(x)
    return ('-' if x<0 else '')+format(abs(x),'X')
def random_int(digits,radix):
    return int(''.join(rng.choice('0123456789ABCDEF'[:radix]) for _ in range(digits)),radix)
def run(binary,op,pairs,radix):
    data=str(len(pairs))+'\n'+''.join(f'{fmt(a,radix)} {fmt(b,radix)}\n' for a,b in pairs)
    result=subprocess.run([str(binary),op,'hex' if radix==16 else 'dec'],input=data,text=True,capture_output=True,check=True)
    lines=result.stdout.splitlines();assert len(lines)==len(pairs)
    for i,((a,b),line) in enumerate(zip(pairs,lines)):
        if op=='div':
            r=a%abs(b);q=(a-r)//b
            expected=f'{fmt(q,radix)} {fmt(r,radix)}'
        else:expected=fmt({'add':lambda:a+b,'sub':lambda:a-b,'mul':lambda:a*b}[op](),radix)
        if line!=expected:
            fail=ROOT/'tests/big_integer/failure.txt'
            fail.write_text(f'operation={op} radix={radix}\n1\n{fmt(a,radix)} {fmt(b,radix)}\nactual={line}\nexpected={expected}\n')
            raise AssertionError(f'{op} radix={radix} case={i}, details={fail}')

def check(binary):
    total=0
    for radix in [10,16]:
        base=radix**4;pairs=[]
        for _ in range(12000):pairs.append((rng.randrange(-10**30,10**30),rng.randrange(-10**30,10**30)))
        for n in [1,2,3,4,31,32,33,47,48,49,63,64,65,95,96,97,127,128,129,255,256,257,511,512,513,1023]:
            for m in sorted({1,2,31,32,33,48,49,n,max(1,n//2),max(1,n-32),max(1,n-33)}):
                for _ in range(2):
                    a=random_int(4*n,radix);b=random_int(4*m,radix)
                    pairs.append((a if rng.randrange(2) else -a,b if rng.randrange(2) else -b))
            # 最大桁・最小上位・carryとborrowの連鎖
            for x in [base**n-1,base**n,base**n+1,(base//2)*base**(n-1),base**(n-1)+1]:
                for y in [1,base-1,base+1,x,x-1,x+1]:pairs.append((x,y))
        for op in ['add','sub','mul','div']:
            pp=pairs if op!='div' else [(a,b) for a,b in pairs if b]
            t=time.perf_counter();run(binary,op,pp,radix);total+=len(pp)
            print(f'radix={radix} {op}: {len(pp)} cases, {time.perf_counter()-t:.2f}s',flush=True)
        division=[]
        for n in [33,49,64,65,127,128,129,255,256,257,1023,2048]:
            for top in [1,base//2,base-1]:
                b=top*base**(n-1)+random_int(4*(n-1),radix)
                for k in [1,31,32,33,n//2,n,n+1,2*n+1,4*n+1]:
                    q=random_int(4*k,radix)
                    for r in [0,1,b-1]:division.append((b*q+r,b))
        run(binary,'div',division,radix);total+=len(division)
        print(f'radix={radix} constructed division: {len(division)} cases',flush=True)
        if args.large:
            pairs=[(random_int(n,radix),random_int(m,radix)) for n,m in [(50000,50000),(100000,51000),(200000,70000),(200000,131),(200000,20000)]]
            for op in ['mul','div']:
                t=time.perf_counter();run(binary,op,pairs,radix);total+=len(pairs)
                print(f'radix={radix} large {op}: {time.perf_counter()-t:.2f}s',flush=True)
    print(f'PASS: {total} differential cases')

if args.binary:check(args.binary.resolve())
else:
    with tempfile.TemporaryDirectory(prefix='arcaki-bigint-check-') as tmp:
        binary=Path(tmp)/'driver'
        subprocess.run(['rustc','--edition=2021','-O','-Awarnings',str(ROOT/'tests/big_integer/driver.rs'),'-o',str(binary)],check=True)
        check(binary)
