#!/usr/bin/env python3
"""Time equivalent submissions on official decimal division inputs, validating every output."""
from pathlib import Path
import argparse, datetime, hashlib, json, platform, statistics, subprocess, tempfile, time, tomllib
p=argparse.ArgumentParser()
p.add_argument('--repo', type=Path, required=True)
p.add_argument('--before', type=Path, required=True)
p.add_argument('--after', type=Path, required=True)
p.add_argument('--report', type=Path, required=True)
p.add_argument('--repeats', type=int, default=3)
p.add_argument('--case')
p.add_argument('--baseline-source',type=Path)
p.add_argument('--environment', default='native aarch64')
a=p.parse_args()
def sha(path):
    h=hashlib.sha256()
    with path.open('rb') as f:
        for block in iter(lambda:f.read(1<<20), b''): h.update(block)
    return h.hexdigest()
folder=a.repo.resolve()/'big_integer/division_of_big_integers'
spec=tomllib.loads((folder/'info.toml').read_text())
hashes=json.loads((folder/'hash.json').read_text())
(folder/'params.h').write_text('\n'.join(f'#define {k} (long long){v}' for k,v in spec['params'].items())+'\n')
root=Path(__file__).resolve().parents[2]
report={'date':datetime.date.today().isoformat(),'platform':platform.platform(),'environment':a.environment,
 'rustc':subprocess.check_output(['rustc','--version'],text=True).strip(),
 'official_commit':subprocess.check_output(['git','-C',str(a.repo),'rev-parse','HEAD'],text=True).strip(),
 'source_sha256':sha(root/'src/NumberTheory/big_integer.rs'), 'baseline_sha256':sha(a.baseline_source if a.baseline_source else root/'tests/support/big_integer_ntt_baseline.rs'),
 'binary_sha256':{k:sha(v) for k,v in [('before',a.before),('after',a.after)]},
 'method':'input/output included; alternating order; each run matches official output SHA256; no online submission',
 'repeats':a.repeats,'cases':[]}
report['compiler_flags']=['--edition=2021','-O','-Awarnings']
report['cpu_info']=platform.processor()
with tempfile.TemporaryDirectory(prefix='bigint-official-bench-') as tmp:
    tmp=Path(tmp)
    for entry in spec['tests']:
        source=folder/'gen'/entry['name'];exe=tmp/source.stem
        seeds=[i for i in range(entry['number']) if not a.case or a.case in f'{source.stem}_{i:02}']
        if not seeds:continue
        if source.suffix=='.cpp':subprocess.run(['c++','-O2','-std=c++17','-I',str(a.repo/'common'),str(source),'-o',str(exe)],check=True,capture_output=True)
        for seed in seeds:
            case=f'{source.stem}_{seed:02}';inp=tmp/'input';out=tmp/'output'
            with inp.open('wb') as f:
                if source.suffix=='.cpp':subprocess.run([str(exe),str(seed)],stdout=f,check=True)
                else:f.write((source.parent/(case+'.in')).read_bytes())
            assert sha(inp)==hashes[case+'.in'],case
            samples={'before':[],'after':[]}
            for i in range(a.repeats):
                for key in (['before','after'] if i%2==0 else ['after','before']):
                    binary=getattr(a,key).resolve();start=time.perf_counter()
                    with inp.open('rb') as f,out.open('wb') as g:subprocess.run([str(binary)],stdin=f,stdout=g,check=True)
                    samples[key].append(time.perf_counter()-start)
                    assert sha(out)==hashes[case+'.out'],(case,key)
            median={k:statistics.median(v) for k,v in samples.items()}
            report['cases'].append({'case':case,'input_bytes':inp.stat().st_size,'samples':samples,'median_seconds':median,
                'speedup':median['before']/median['after'],'output_sha256':hashes[case+'.out'],'status':'PASS'})
            print(f"{case}: {median['before']:.4f} -> {median['after']:.4f}s ({median['before']/median['after']:.2f}x)",flush=True)
            a.report.write_text(json.dumps(report,indent=2)+'\n')
