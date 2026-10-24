#!/usr/bin/env python3
import re, argparse, csv, hashlib, importlib.util, io, json, platform, random, statistics, subprocess, tempfile
from pathlib import Path
from datetime import datetime, timezone
ROOT=Path(__file__).resolve().parents[1]
SUPPORT=ROOT/'tests/support/segment_tree'
spec=importlib.util.spec_from_file_location('candidates',SUPPORT/'candidates.py'); mod=importlib.util.module_from_spec(spec);spec.loader.exec_module(mod)
p=argparse.ArgumentParser();p.add_argument('--sizes',default='1024,65536,262144');p.add_argument('--validate',action='store_true');p.add_argument('--repeats',type=int,default=7);p.add_argument('--variants',default='');p.add_argument('--output',default='segment_tree_benchmark_results.json');p.add_argument('--toolchain',default='1.89.0');args=p.parse_args()
base=tuple((SUPPORT/f'{x}_baseline.rs').read_text() for x in ['segtree','stateful','lazy'])
variants=mod.variants(base)
variants['current']=tuple((ROOT/'src/SegmentTree'/x).read_text() for x in ['segtree.rs','segtree_with_monoid.rs','lazysegtree.rs'])
if args.variants:variants={k:variants[k] for k in args.variants.split(',')}
rustc=['rustc']+(['+'+args.toolchain] if args.toolchain else [])
rows=[]
with tempfile.TemporaryDirectory(prefix='segment-tree-') as tmp:
    tmp=Path(tmp);executables={}
    for name,sources in variants.items():
        directory=tmp/name;directory.mkdir()
        for part,src in zip(['plain','state','lazy'],sources):(directory/f'{part}.rs').write_text(src)
        bench='\n'.join(f'mod {part};' for part in ['plain','state','lazy'])+'\n'+(SUPPORT/'benchmark.rs').read_text().replace('#![allow(dead_code)]','')
        bench=re.sub(r'for n in \[\s*1024,\s*65536,\s*262144\s*\]', f'for n in [{args.sizes}]', bench)
        (directory/'main.rs').write_text('#![allow(dead_code)]\n'+bench)
        if args.validate:
            test=(ROOT/'tests/segment_tree_optimization.rs').read_text()
            test=re.sub(r'#\[path\s*=\s*"[^"]+"\]\s*mod (plain|state|lazy);', lambda m:f'#[path="{directory/m[1]}.rs"] mod {m[1]};',test)
            (directory/'test.rs').write_text(test)
            subprocess.run(rustc+['--edition=2021','-O','--test',str(directory/'test.rs'),'-o',str(directory/'test')],check=True,capture_output=True)
            subprocess.run([str(directory/'test')],check=True,capture_output=True)
        exe=directory/'bench';subprocess.run(rustc+['--edition=2021','-O',str(directory/'main.rs'),'-o',str(exe)],check=True,capture_output=True);executables[name]=exe
    for rep in range(args.repeats+1):
        order=list(variants);random.Random(634+rep).shuffle(order)
        for name in order:
            result=subprocess.check_output([str(executables[name])],text=True)
            if rep:
                for row in csv.reader(io.StringIO(result)):
                    tree,n,mode,pattern,q,ns,check=row
                    rows.append(dict(variant=name,repeat=rep,tree=tree,n=int(n),mode=mode,pattern=pattern,q=int(q),ns=int(ns),checksum=check))
            print(f'{rep}/{args.repeats} {name}',flush=True)
checks={}
for r in rows:
    key=tuple(r[k] for k in ['tree','n','mode','pattern'])
    if key in checks:assert checks[key]==r['checksum'],(key,r)
    checks[key]=r['checksum']
groups={}
for r in rows:
    key=tuple(r[k] for k in ['variant','tree','n','mode','pattern']);groups.setdefault(key,[]).append(r['ns'])
summary=[]
for key,times in groups.items():
    name,*case=key;baseline=groups.get(('baseline',*case))
    summary.append(dict(variant=name,tree=case[0],n=case[1],mode=case[2],pattern=case[3],median_ns=statistics.median(times),min_ns=min(times),max_ns=max(times),speedup=statistics.median(baseline)/statistics.median(times) if baseline else None))
output=dict(measured_at=datetime.now(timezone.utc).isoformat(),compiler=subprocess.check_output(rustc+['-Vv'],text=True),platform=platform.platform(),repeats=args.repeats,flags=['--edition=2021','-O'],source_hashes={name:[hashlib.sha256(s.encode()).hexdigest() for s in src] for name,src in variants.items()},note='Baseline includes the min_left root guard correction; original snapshots retained. One warmup; seeded shuffled variant order per repetition; setup excluded except build; allocation/destruction included in build and slice. Same deterministic streams and checksums verified for every case. Local ARM64 evidence, no judge results.',sizes=args.sizes,validation=args.validate,harness_hashes={str(path.relative_to(ROOT)):hashlib.sha256(path.read_bytes()).hexdigest() for path in [Path(__file__), SUPPORT/'benchmark.rs',SUPPORT/'candidates.py']},summary=summary,measurements=rows)
(ROOT/'tests'/args.output).write_text(json.dumps(output,indent=2)+'\n')
for name in variants:
    for tree in ['plain','state','matrix','boxed','add','affine']:
        ratios=[r['speedup'] for r in summary if r['variant']==name and r['tree']==tree and r['speedup'] is not None]
        if ratios:print(name,tree,'min/median/max',*[round(f(ratios),3) for f in [min,statistics.median,max]])
