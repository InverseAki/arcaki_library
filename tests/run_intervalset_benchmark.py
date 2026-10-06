from pathlib import Path
import json, statistics, subprocess
root = Path(__file__).resolve().parents[1]
binary = Path('/tmp/benchmark_intervalset')
subprocess.run(['rustc', '--edition=2021', '-O', str(root/'tests/benchmark_intervalset.rs'), '-o', str(binary)], check=True)
rows=[]
for mode in ['records', 'plain']:
    for case in ['noop', 'shorten', 'random', 'bulk_1', 'bulk_8', 'bulk_256', 'bulk_65536']:
        samples={v: [] for v in ['baseline','candidate']}
        for trial in range(5):
            for version in (['baseline','candidate'] if trial%2==0 else ['candidate','baseline']):
                output=subprocess.check_output([str(binary),version,case,mode],text=True)
                update,total,checksum=output.split()
                samples[version].append({'update_ms':float(update),'total_ms':float(total),'checksum':int(checksum)})
        assert len({s['checksum'] for values in samples.values() for s in values}) == 1
        median={v: {key: statistics.median(s[key] for s in values) for key in ['update_ms','total_ms']} for v,values in samples.items()}
        row={'mode':mode,'case':case,'median':median,'samples':samples}
        rows.append(row)
        print(mode,case,median,'update speedup',round(median['baseline']['update_ms']/median['candidate']['update_ms'],2),flush=True)
(root/'tests/intervalset_benchmark_results.json').write_text(json.dumps({'rustc':subprocess.check_output(['rustc','--version'],text=True).strip(),'repetitions':5,'rows':rows},indent=2)+'\n')
