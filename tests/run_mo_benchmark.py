import csv
import io
import hashlib
import json
import os
import platform
import statistics
import subprocess
import tempfile
from collections import defaultdict
from pathlib import Path
root=Path(__file__).resolve().parents[1]
b=root.joinpath('tests/support/mo_baseline.rs').read_text()
start=b.index('    const ROT_DELTA')
end=b.index('    fn get_ord',start)
loop='''    #[inline]
    fn hilbert_order(x: u32, y: u32, pow: u32, mut rot: u32) -> u64 {
        let mut ord = 0;
        let mut flip = 0;
        for p in (0..pow).rev() {
            let rx = (x >> p) & 1;
            let ry = (y >> p) & 1;
            let seg = ((ry * 3 ^ rx) + rot) & 3;
            rot = (rot + [3, 0, 0, 1][seg as usize]) & 3;
            ord = (ord << 2) | ((seg ^ flip) as u64);
            if seg == 0 || seg == 3 { flip ^= 3; }
        }
        ord
    }

'''
i=b[:start]+loop+b[end:]
s=i
st=s.index('            while l > query.l')
en=s.index('            res[query.id]',st)
slices='''            if l > query.l {
                for (offset, value) in self.data[query.l..l].iter().enumerate().rev() {
                    state.add_left(query.l + offset, value);
                }
                l = query.l;
            }
            if r < query.r {
                for (offset, value) in self.data[r..query.r].iter().enumerate() {
                    state.add_right(r + offset, value);
                }
                r = query.r;
            }
            if l < query.l {
                for (offset, value) in self.data[l..query.l].iter().enumerate() {
                    state.sub_left(l + offset, value);
                }
                l = query.l;
            }
            if r > query.r {
                for (offset, value) in self.data[query.r..r].iter().enumerate().rev() {
                    state.sub_right(query.r + offset, value);
                }
                r = query.r;
            }
'''
s=s[:st]+slices+s[en:]
u=i.replace('        self.query.push(', '        assert!(l <= r && r <= self.data.len());\n        self.query.push(')
u=u.replace('&self.data[l]', 'unsafe { self.data.get_unchecked(l) }').replace('&self.data[r]', 'unsafe { self.data.get_unchecked(r) }')
f=s.replace('query: Vec<(MoQuery, M::Query)>','query: Vec<(MoQuery, M::Query, u64)>').replace('}, qd));','}, qd, 0));')
a=f.index('    fn get_ord'); z=f.index('    pub fn solve',a)
f=f[:a]+f[z:]
f=f.replace('        let ord = self.get_ord();','''        let mx = self.query.iter().map(|(q, _, _)| q.l.max(q.r)).max().unwrap_or(0);
        let pow = usize::BITS - mx.leading_zeros();
        for (q, _, key) in &mut self.query {
            *key = Self::hilbert_order(q.l as u32, q.r as u32, pow, 0);
        }
        self.query.sort_unstable_by_key(|q| q.2);''').replace('        for qi in ord {\n            let (query, qd) = &self.query[qi];', '        for (query, qd, _) in &self.query {')
variants={'baseline':b,'iterative':i,'slices':s,'unchecked':u,'flat':f}
temporary=tempfile.TemporaryDirectory(prefix='mo_benchmark_')
out=Path(temporary.name)
for name,code in variants.items(): (out/(name+'.rs')).write_text(code)
v=s.replace('        let mut res: Vec<Option<M::Ans>> = (0..q).map(|_| None).collect();\n        let ord = self.get_ord();','''        let ord = self.get_ord();
        let mut res = MoAnswers {
            values: (0..q).map(|_| std::mem::MaybeUninit::uninit()).collect::<Vec<_>>(),
            order: &ord,
            initialized: 0,
        };''').replace('        for qi in ord {','        for &qi in &ord {').replace('            res[query.id] = Some(state.ans(query.id, qd));','''            res.values[query.id].write(state.ans(query.id, qd));
            res.initialized += 1;''').replace('        res.into_iter().map(|x| x.unwrap()).collect()','''        let mut res = std::mem::ManuallyDrop::new(res);
        unsafe {
            Vec::from_raw_parts(res.values.as_mut_ptr().cast::<M::Ans>(), q, res.values.capacity())
        }''')
v+='''
struct MoAnswers<'a, A> {
    values: Vec<std::mem::MaybeUninit<A>>,
    order: &'a [usize],
    initialized: usize,
}
impl<A> Drop for MoAnswers<'_, A> {
    fn drop(&mut self) {
        for &id in &self.order[..self.initialized] {
            unsafe { self.values[id].assume_init_drop(); }
        }
    }
}
'''
(out/'uninit.rs').write_text(v)

(out/'adopted.rs').write_text(root.joinpath('src/OfflineQuery/mo.rs').read_text())
env = dict(os.environ, MO_CANDIDATE_DIR=str(out))
subprocess.run(['rustc', '--edition=2021', '-O', str(root/'tests/support/benchmark_mo.rs'), '-o', str(out/'benchmark')], env=env, check=True)
raw = subprocess.check_output([str(out/'benchmark')], text=True)
fields = ['case','n','q','payload_u64s','repeat','variant','ns','checksum']
rows = [dict(zip(fields,row)) for row in csv.reader(io.StringIO(raw))]
for row in rows:
    for field in fields[1:5] + fields[6:]: row[field] = int(row[field])
groups = defaultdict(lambda: defaultdict(list))
for row in rows: groups[(row['case'],row['payload_u64s'])][row['variant']].append(row['ns'])
summary = []
for (case,payload), variants in groups.items():
    medians = {name: statistics.median(times) for name,times in variants.items()}
    ratios = {name: medians['baseline']/ns for name,ns in medians.items()}
    summary.append(dict(case=case,payload_u64s=payload,median_ns=medians,speedup=ratios))
    print(case, payload, {name:round(ratio,3) for name,ratio in ratios.items()})
result = dict(platform=platform.platform(),machine=platform.machine(),rustc=subprocess.check_output(['rustc','--version'],text=True).strip(),timing='construction, query registration, ordering, movement, answer construction, checksum; excludes final object destruction',repeats=7,source_sha256={name:hashlib.sha256((out/(name+'.rs')).read_bytes()).hexdigest() for name in [*variants,'uninit','adopted']},summary=summary,measurements=rows)
(root/'tests/mo_benchmark_results.json').write_text(json.dumps(result,indent=2)+'\n')
temporary.cleanup()
