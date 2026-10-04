#!/usr/bin/env python3
"""最大桁の全桁最大・carry連鎖・冪付近を、閉じた形の期待出力で比較。"""
from pathlib import Path
import argparse, hashlib, json, subprocess, time
p=argparse.ArgumentParser();p.add_argument('--binary',required=True,type=Path);p.add_argument('--report',type=Path,default=Path(__file__).with_name('hard_results.json'));args=p.parse_args()
results=[]
for radix,n in [(10,2000000),(16,1600000)]:
    digit='9' if radix==10 else 'F';penultimate='8' if radix==10 else 'E';mode='dec' if radix==10 else 'hex'
    maximum=digit*n;power='1'+'0'*n
    cases=[
        ('mul',maximum,maximum,digit*(n-1)+penultimate+'0'*(n-1)+'1','all_max_square'),
        ('add',maximum,'1',power,'full_carry'),
        ('sub',power,'1',maximum,'full_borrow'),
        ('div',power,maximum,'1 1','power_over_previous'),
        ('div',maximum,power,'0 '+maximum,'smaller_dividend'),
    ]
    for op,a,b,expected,name in cases:
        data=('1\n'+a+' '+b+'\n').encode();start=time.perf_counter()
        actual=subprocess.check_output([str(args.binary.resolve()),op,mode],input=data)
        elapsed=time.perf_counter()-start;assert actual==expected.encode()+b'\n',name
        row={'radix':radix,'digits':n,'case':name,'operation':op,'seconds':round(elapsed,6),'status':'PASS','output_sha256':hashlib.sha256(actual).hexdigest()}
        results.append(row);print(f'{radix} / {name}: PASS {elapsed:.3f}s',flush=True)
args.report.write_text(json.dumps({'date':'2026-10-02','cases':results},indent=2)+'\n')
