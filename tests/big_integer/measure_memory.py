#!/usr/bin/env python3
"""wait4で代表的な公式最大級ケースのRSSと時間を記録。"""
from pathlib import Path
import argparse, json, os, subprocess, sys, tempfile, time
p=argparse.ArgumentParser();p.add_argument('--repo',required=True,type=Path);p.add_argument('--binary',required=True,type=Path);args=p.parse_args()
results=[]
with tempfile.TemporaryDirectory(prefix='arcaki-bigint-memory-') as tmp:
    tmp=Path(tmp)
    for problem,case,seed,op in [('multiplication_of_big_integers','max_max',0,'mul'),('division_of_big_integers','a_max_b_random',2,'div'),('division_of_big_integers','length_ratio_integer',0,'div')]:
        source=args.repo/'big_integer'/problem/'gen'/(case+'.cpp');exe=tmp/'gen';inp=tmp/'input'
        subprocess.run(['c++','-O2','-std=c++17','-I',str(args.repo/'common'),str(source),'-o',str(exe)],check=True,capture_output=True)
        with inp.open('wb') as f:subprocess.run([str(exe),str(seed)],stdout=f,check=True)
        with inp.open('rb') as f:
            start=time.perf_counter()
            child=subprocess.Popen([str(args.binary.resolve()),op,'dec'],stdin=f,stdout=subprocess.DEVNULL)
            _,status,usage=os.wait4(child.pid,0)
            child.returncode=os.waitstatus_to_exitcode(status)
            if child.returncode:
                raise subprocess.CalledProcessError(child.returncode,child.args)
            elapsed=time.perf_counter()-start
        rss=usage.ru_maxrss*(1 if sys.platform=='darwin' else 1024)
        row={'problem':problem,'case':f'{case}_{seed:02}','peak_rss_bytes':rss,'seconds':elapsed};results.append(row);print(row)
Path(__file__).with_name('memory_results.json').write_text(json.dumps({'date':'2026-10-02','method':'POSIX wait4 rusage.ru_maxrss; macOS bytes, Linux KiB converted to bytes','cases':results},indent=2)+'\n')
