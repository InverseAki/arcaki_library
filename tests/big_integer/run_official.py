#!/usr/bin/env python3
"""公開公式生成器の全ケースを実行し、公式hash.jsonの入出力SHA256と照合。
ネットワーク取得はしない。--repo に取得済み library-checker-problems を指定。
"""
from pathlib import Path
import datetime, argparse, hashlib, json, platform, subprocess, tempfile, time, tomllib
ROOT=Path(__file__).resolve().parents[2]
p=argparse.ArgumentParser();p.add_argument('--repo',required=True,type=Path);p.add_argument('--binary',type=Path);p.add_argument('--report',type=Path,default=ROOT/'tests/big_integer/official_results.json');p.add_argument('--problem',action='append');p.add_argument('--case');args=p.parse_args()
repo=args.repo.resolve()
problems=['addition_of_big_integers','multiplication_of_big_integers','division_of_big_integers','addition_of_hex_big_integers','multiplication_of_hex_big_integers','division_of_hex_big_integers']
if args.problem:problems=args.problem
results=[]
def sha(path):
    h=hashlib.sha256()
    with path.open('rb') as f:
        for chunk in iter(lambda:f.read(1<<20),b''):h.update(chunk)
    return h.hexdigest()
with tempfile.TemporaryDirectory(prefix='arcaki-bigint-official-') as tmp:
    tmp=Path(tmp);binary=args.binary.resolve() if args.binary else tmp/'driver'
    if not args.binary:subprocess.run(['rustc','--edition=2021','-O','-Awarnings',str(ROOT/'tests/big_integer/driver.rs'),'-o',str(binary)],check=True)
    for name in problems:
        folder=repo/'big_integer'/name
        spec=tomllib.loads((folder/'info.toml').read_text());hashes=json.loads((folder/'hash.json').read_text())
        (folder/'params.h').write_text('\n'.join(f'#define {k} (long long){v}' for k,v in spec['params'].items())+'\n')
        op={'addition':'add','multiplication':'mul','division':'div'}[name.split('_')[0]]
        radix='hex' if '_hex_' in name else 'dec'
        for entry in spec['tests']:
            source=folder/'gen'/entry['name'];exe=tmp/(name+'-'+source.stem)
            selected=[i for i in range(entry['number']) if not args.case or args.case in f'{source.stem}_{i:02}']
            if not selected:continue
            if source.suffix=='.cpp':
                subprocess.run(['c++','-O2','-std=c++17','-I',str(repo/'common'),str(source),'-o',str(exe)],check=True,capture_output=True)
            for seed in selected:
                case=f'{source.stem}_{seed:02}';inp=tmp/'input';out=tmp/'output'
                with inp.open('wb') as f:
                    if source.suffix=='.cpp':subprocess.run([str(exe),str(seed)],stdout=f,check=True)
                    else:f.write((source.parent/(case+'.in')).read_bytes())
                assert sha(inp)==hashes[case+'.in'],f'generated input hash mismatch: {name}/{case}'
                start=time.perf_counter()
                with inp.open('rb') as f,out.open('wb') as g:
                    subprocess.run([str(binary),op,radix],stdin=f,stdout=g,check=True,timeout=60)
                elapsed=time.perf_counter()-start
                assert sha(out)==hashes[case+'.out'],f'output hash mismatch: {name}/{case}'
                row={'problem':name,'case':case,'seconds':round(elapsed,6),'input_bytes':inp.stat().st_size,'output_sha256':sha(out),'status':'PASS','time_limit':spec['timelimit']}
                results.append(row)
                print(f'{name}/{case}: PASS {elapsed:.3f}s / {spec["timelimit"]}s',flush=True)
                report={'date':datetime.date.today().isoformat(),'platform':platform.platform(),'processor':platform.processor(),'rustc':subprocess.check_output(['rustc','--version'],text=True).strip(),'official_commit':subprocess.check_output(['git','-C',str(repo),'rev-parse','HEAD'],text=True).strip(),'source_sha256':sha(ROOT/'src/NumberTheory/big_integer.rs'),'method':'official generator + exact input/output SHA256 from official hash.json; local execution, no online submission','cases':results}
                args.report.write_text(json.dumps(report,indent=2)+'\n')
    print(f'PASS: {len(results)} official test files')
