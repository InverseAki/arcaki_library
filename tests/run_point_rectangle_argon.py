"""Reproducible candidate ablations; registration/build and solve included."""
import argparse, hashlib, json, os, platform, statistics, subprocess, tempfile
from pathlib import Path
R=Path(__file__).resolve().parent
P=R/'support/point_rectangle_argon'
a=argparse.ArgumentParser(); a.add_argument('--final',action='store_true'); a.add_argument('--refine',action='store_true'); args=a.parse_args()
configs={'argon':(16,False,0), 'prefix':(16,True,0), 'clear4':(4,False,0), 'clear64':(64,False,0), 'clear256':(256,False,0), 'cut16':(16,False,16), 'cut32':(16,False,32), 'cut64':(16,False,64), 'safe':(16,False,0), 'branch':(16,False,0)}
if args.refine: configs={'c16':(64,False,16),'c32':(64,False,32),'c64':(64,False,64),'prefix32':(64,True,32),'reset':(64,False,32),'binary':(64,False,32),'prerank':(64,False,32),'fullsearch':(64,False,32)}
if args.final: configs={'final':(64,False,32)}
bins={}
for name,(clear,prefix,cutoff) in configs.items():
    s=(P/'candidate.rs').read_text()
    if name=='reset':
        s=s.replace('bit.add(e.y_or_payload as usize, e.w_or_lohi.wrapping_neg());','''let mut idx = e.y_or_payload as usize + 1;
                        while idx <= bit.n {
                            bit.vec[idx] = 0;
                            idx += idx & idx.wrapping_neg();
                        }''')
    if name=='binary':
        s=s.replace('ys.partition_point(|y| y < &initial[p].1)', 'ys.binary_search(&initial[p].1).unwrap()')
        s=s.replace('ys.partition_point(|v| v < &y)', 'ys.binary_search(&y).unwrap()')
    if name=='prerank':
        s=s.replace('        // 動的Addが一切ない場合', '''        let initial: Vec<_> = initial.into_iter().map(|(x,y,w)| (x,ys.binary_search(&y).unwrap(),w)).collect();
        // 動的Addが一切ない場合''')
        s=s.replace('ys.partition_point(|y| y < &initial[p].1)', 'initial[p].1')
    if name=='fullsearch':
        s=s.replace('lo + ys[lo..].partition_point(|y| y < &ry)', 'ys.partition_point(|y| y < &ry)')
    if name=='safe':
        s=s.replace('unsafe { *ptr.add(r) }','self.vec[r]').replace('unsafe { *ptr.add(l) }','self.vec[l]')
        s=s.replace('let p = ptr.add(idx);','let p = &mut self.vec[idx];')
        s=s.replace('*left_ptr.add(li) = e;', 'dst_left[li] = e;').replace('*right_ptr.add(ri) = e;', 'dst_right[ri] = e;')
        s=s.replace('*block_counts.get_unchecked_mut(b)', 'block_counts[b]')
        s=s.replace('*block_offsets.get_unchecked(m) - *block_offsets.get_unchecked(l)','block_offsets[m] - block_offsets[l]')
        s=s.replace('ans.get_unchecked_mut(idx)', '&mut ans[idx]')
    if name=='branch':
        s=s.replace('let delta = (sum ^ mask).wrapping_sub(mask);','let delta = if payload >= 0 { sum.wrapping_neg() } else { sum };')
    if args.final: s=(R/'../src/OfflineQuery/point_add_rectangle_sum.rs').read_text()
    (P/'config.rs').write_text(f'const CLEAR_FACTOR: usize={clear};\nconst PREFIX: bool={str(prefix).lower()};\nconst CUTOFF: usize={cutoff};\n'+s)
    b=Path(tempfile.gettempdir())/f'rect_argon_{name}'
    subprocess.run(['rustc','--edition=2021','-Awarnings','-O',str(R/'point_rectangle_argon_bench.rs'),'-o',str(b)],check=True)
    bins[name]=b
bins['baseline']=next(iter(bins.values()))
cases=[(0,200000,m) for m in ['mixed','add_first','query_first','ties','add_heavy','query_heavy','blocks','narrow','full','sorted','empty','outside','zero']]+[(100000,100000,m) for m in ['static','mixed']]
if args.final: cases += [(0,1000000,m) for m in ['mixed','add_first','ties','add_heavy','query_heavy','blocks','narrow','full']]+[(500000,500000,m) for m in ['static','mixed']]
if args.refine: cases=[(0,300000,m) for m in ['mixed','ties','add_heavy','query_heavy','blocks','full','sorted']] + [(150000,150000,m) for m in ['static','mixed']]
rows=[]
for n,q,mode in cases:
    results={name:[] for name in bins}; checks=set()
    names=list(bins)
    for rep in range(3 if not args.final else 5):
        for name in names[rep%len(names):]+names[:rep%len(names)]:
            proc=subprocess.Popen([str(bins[name]),'baseline' if name=='baseline' else 'candidate',mode,str(n),str(q)],stdout=subprocess.PIPE,text=True)
            _, status, usage=os.wait4(proc.pid,0); proc.returncode=os.waitstatus_to_exitcode(status)
            assert proc.returncode==0
            elapsed, checksum, count=proc.stdout.read().split();proc.stdout.close()
            checks.add((checksum,count)); results[name].append({'seconds':float(elapsed),'rss_bytes':usage.ru_maxrss,'checksum':checksum})
    assert len(checks)==1,(n,q,mode,checks)
    row={'initial':n,'operations':q,'mode':mode,'methods':{k:{'median':statistics.median(v['seconds'] for v in vs),'runs':vs} for k,vs in results.items()}}
    rows.append(row)
    print(n,q,mode,{k:round(v['median'],5) for k,v in row['methods'].items()},flush=True)
output={'platform':platform.platform(),'rustc':subprocess.check_output(['rustc','-V'],text=True).strip(),'timing':'build + registration + solve; excludes input generation, process startup, I/O','configs':configs,'results':rows}
output['source_sha256'] = {str(path.relative_to(R.parent)): hashlib.sha256(path.read_bytes()).hexdigest() for path in [P/'baseline.rs', R/'../src/OfflineQuery/point_add_rectangle_sum.rs' if args.final else P/'candidate.rs']}
(R/('point_rectangle_argon_final.json' if args.final else 'point_rectangle_argon_refined.json' if args.refine else 'point_rectangle_argon_candidates.json')).write_text(json.dumps(output,indent=2)+'\n')

(P/"config.rs").write_text("const CLEAR_FACTOR: usize = 16;\nconst PREFIX: bool = false;\nconst CUTOFF: usize = 0;\ninclude!(\"candidate.rs\");\n")
