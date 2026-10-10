#!/usr/bin/env python3
"""Deterministic local differential tests against Python Fraction and math.gcd."""
from pathlib import Path
from fractions import Fraction
import random, math, subprocess, tempfile, json, hashlib, platform, sys, argparse, time
sys.set_int_max_str_digits(0)
ROOT=Path(__file__).resolve().parents[1]
p=argparse.ArgumentParser();p.add_argument('--output',type=Path);args=p.parse_args()
rng=random.Random(20261004)
cases=[]

def fit(q,bits):
    if bits and not (-(1<<(bits-1))<=q.numerator<(1<<(bits-1)) and q.denominator<(1<<(bits-1))):return 'overflow'
    return f'{q.numerator} {q.denominator}'

def emit(backend,op,n,d,m=None,e=None,k=None):
    bits={'i64':64,'i128':128,'big':0}[backend]
    line=f'{backend} {op} {n} {d}'
    if m is not None:line+=f' {m} {e}'
    if k is not None:line+=f' {k}'
    if d==0:
        expected='indeterminate' if n==0 else f'{1 if n>0 else -1} 0'
    else:
        a=Fraction(n,d);expected=fit(a,bits)
        if expected!='overflow':
            if op=='new':pass
            elif op=='neg':expected=fit(-a,bits)
            elif op=='inv':expected=fit(1/a,bits) if a else '1 0'
            elif op=='pow':expected=fit(a**k,bits)
            elif op=='round':
                trunc=abs(a.numerator)//a.denominator*(-1 if a<0 else 1)
                expected=f'{a.numerator//a.denominator} {-(-a.numerator//a.denominator)} {trunc}'
            else:
                b=Fraction(m,e)
                if fit(b,bits)=='overflow':expected='overflow'
                elif op=='cmp':expected=str((a>b)-(a<b))
                elif op=='div' and not b:expected='indeterminate' if not a else f'{1 if a>0 else -1} 0'
                else:expected=fit({'add':lambda:a+b,'sub':lambda:a-b,'mul':lambda:a*b,'div':lambda:a/b}[op](),bits)
    cases.append((line,expected))

for backend,bits in [('i64',64),('i128',128),('big',128)]:
    edge=[-(1<<(bits-1)),-(1<<(bits-1))+1,-2,-1,0,1,2,(1<<(bits-1))-2,(1<<(bits-1))-1]
    for n in edge:
        for d in edge:
            emit(backend,'new',n,d)
            if d:
                for op in ['neg','inv','round']:emit(backend,op,n,d)
    for i in range(2400):
        width=rng.choice([4,16,31,62,bits-1])
        nums=[rng.randrange(-(1<<width),(1<<width)) for _ in range(4)]
        n,d,m,e=nums;d=d or 1;e=e or 1
        for op in ['add','sub','mul','div','cmp']:emit(backend,op,n,d,m,e)
        if i%6==0:
            if n:emit(backend,'mul',n,d,d,n)
            emit(backend,'sub',n,d,n,d)
            emit(backend,'pow',n,d,k=rng.randrange(0,6))
    # Wide products followed by a large GCD: normalized final result still fits.
    for t in [8,16,31,32,48,62]:
        if 2*t+3>=bits:continue
        g=1<<(t+2);a=(1<<t)+1;b=(1<<t)+3
        n=g*a-1;m=g*(b-1)+(b*pow(a,-1,g))%g
        for op in ['add','sub','cmp']:emit(backend,op,n,g*a,m,g*b)

for digits in [40,127,128,129,191,192,193,255,256,257,1000,3000]:
    for pattern in range(3):
        n=int(''.join(rng.choice('0123456789') for _ in range(digits))) or 1
        d=10**digits-1 if pattern==0 else (10**digits+1 if pattern==1 else n+1)
        factor=10**(digits//2)+7
        for op in ['new','neg','inv','round']:emit('big',op,n*factor,-d*factor)
        for op in ['add','sub','mul','div','cmp']:
            emit('big',op,n,d,d,n)
            emit('big',op,n*factor,d*factor,-n*factor,d*factor)
        emit('big','pow',n,d,k=2)
        cases.append((f'gcd {-n*factor} {d*factor}',str(math.gcd(n*factor,d*factor))))
# Euclid's slow quotient pattern and constructed common factors.
x=y=1
for step in range(6000):
    x,y=y,x+y
    if step in [127,128,255,256,511,512,1023,2047,4095,5999]:
        factor=10**200+39
        cases.append((f'gcd {-x*factor} {y*factor}',str(factor)))
for i in range(250):
    x=rng.getrandbits(rng.choice([0,1,64,127,128,129,512,2048,4096]))
    y=rng.getrandbits(rng.choice([0,1,64,127,128,129,512,2048,4096]))
    if i%2:x=-x
    cases.append((f'gcd {x} {y}',str(math.gcd(x,y))))

report={'seed':20261004,'platform':platform.platform(),'cases_per_build':len(cases),'oracle':'Python fractions.Fraction and math.gcd','rustc':subprocess.check_output(['rustc','--version'],text=True).strip(),'sha256':{f:hashlib.sha256((ROOT/f).read_bytes()).hexdigest() for f in ['src/NumberTheory/ratio.rs','src/NumberTheory/big_ratio.rs','src/NumberTheory/big_integer.rs']},'builds':{}}
with tempfile.TemporaryDirectory(prefix='ratio-check-') as tmp:
    for label,flags in [('debug',[]),('release',['-O'])]:
        binary=Path(tmp)/label
        subprocess.run(['rustc','--edition=2021','-Awarnings',str(ROOT/'tests/support/ratio_driver.rs'),'-o',str(binary),*flags],check=True)
        start=time.perf_counter()
        result=subprocess.run([str(binary)],input='\n'.join(c for c,e in cases)+'\n',text=True,capture_output=True,check=True)
        lines=result.stdout.splitlines();assert len(lines)==len(cases)
        for i,((case,expected),actual) in enumerate(zip(cases,lines)):
            if actual!=expected:
                failure=ROOT/'tests/ratio_failure.txt';failure.write_text(f'seed=20261004 build={label} case={i}\n{case}\nexpected={expected}\nactual={actual}\n')
                raise AssertionError(f'mismatch, saved to {failure}')
        elapsed=time.perf_counter()-start
        report['builds'][label]={'passed':len(cases),'seconds':elapsed}
        print(f'{label}: PASS {len(cases)} cases, {elapsed:.2f}s',flush=True)
if args.output:args.output.write_text(json.dumps(report,indent=2)+'\n')
