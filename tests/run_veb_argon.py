"""Generate, model-test and benchmark independently selectable vEB candidates."""
import argparse, csv, hashlib, json, math, platform, statistics, subprocess, tempfile
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
SUPPORT = ROOT / 'tests/support/veb_argon'
BASE = (SUPPORT / 'baseline.rs').read_text()

def replace_fn(src, name, body):
    start = src.index('    pub fn '+name) if '    pub fn '+name in src else src.index('    fn '+name)
    brace = src.index('{', start)
    end, depth = brace+1, 1
    while depth:
        depth += (src[end]=='{') - (src[end]=='}'); end += 1
    return src[:start]+body+src[end:]

def shifted(s):
    s=replace_fn(s,'prev', '''    pub fn prev(&self, p: usize) -> usize {
        if p == 0 { usize::MAX } else { self.inprev(p - 1) }
    }''')
    return replace_fn(s,'next', '''    pub fn next(&self, p: usize) -> usize { self.innext(p.saturating_add(1)) }''')
def masks(s):
    return s.replace('(u64::MAX << (p & 63)) << 1','!1u64 << (p & 63)').replace('((u64::MAX << ((p >> 6) & 63)) << 1)','(!1u64 << ((p >> 6) & 63))').replace('((u64::MAX << p) << 1)','(!1u64 << p)').replace('(63 - (p & 63))','(!p & 63)')
def remove_one(s):
    return s.replace('''        *word &= !(1 << (p & 63));
        if *word == 0 && old != 0 {''','''        let mask = 1u64 << (p & 63);
        *word = old & !mask;
        if old == mask {''')
def sentinel(s):
    pos=s.index('impl VebNode');a,b=s[:pos],s[pos:]
    a=a.replace('let p = self.summary.max;\n        if p == usize::MAX', 'let p = self.summary.max;\n        if self.summary.min == usize::MAX').replace('if self.summary.min == usize::MAX {\n            p','if self.summary.min == usize::MAX {\n            usize::MAX')
    b=b.replace('max: usize::MAX','max: 0').replace('self.max = usize::MAX','self.max = 0').replace('self.min == self.max {','self.min == usize::MAX || self.min == self.max {').replace('self.min == usize::MAX || p >= self.max','p >= self.max').replace('self.min == usize::MAX || p <= self.min','p <= self.min').replace('child.min != usize::MAX && lo < child.max','lo < child.max')
    b=b.replace('self.max = if hi == usize::MAX', 'self.max = if self.summary.as_ref().unwrap().min == usize::MAX')
    return a+b

def unchecked(s):
    s=s.replace('&mut self.middle[p >> 12]', 'unsafe { self.middle.get_unchecked_mut(p >> 12) }')
    for x in ['p >> 12','p','i']:
        s=s.replace('self.middle['+x+']', '(unsafe { *self.middle.get_unchecked('+x+') })')
    return s

def shift6(s):
    return s.replace('node.shift = width / 2;', 'node.shift = (width / 2).div_ceil(6) * 6;')
def swar(s):
    a=s.index('                let mut bytes = [0; 8];');b=s.index('                word |= bits',a)
    return s[:a]+'''                let raw = u64::from_le_bytes(group.try_into().unwrap());
                let x = raw ^ 0x3131_3131_3131_3131;
                let low = 0x7f7f_7f7f_7f7f_7f7f;
                let eq = !(((x & low).wrapping_add(low)) | x | low) >> 7;
                let bits = (eq & 0x0101_0101_0101_0101).wrapping_mul(0x0102_0408_1020_4080) >> 56;
'''+s[b:]
def exact(s):
    a=s.index('        for (i, chunk) in data.chunks(64)');b=s.index('        let mut summary',a)
    return s[:a]+'''        let mut chunks = data.chunks_exact(64);
        let mut words = middle.iter_mut();
        for (chunk, word) in (&mut chunks).zip(&mut words) {
            let mut bits = 0;
            for j in 0..64 { bits |= ((chunk[j] != 0) as u64) << j; }
            *word = bits;
        }
        if let Some(word) = words.next() {
            for (j, &v) in chunks.remainder().iter().enumerate() { *word |= ((v != 0) as u64) << j; }
        }
'''+s[b:]
def variants():
    v={'baseline':BASE,'current':(ROOT/'src/DataStructure/veb_tree.rs').read_text()}
    for name,fn in [('shifted',shifted),('masks',masks),('xor',lambda s:s.replace('63 - ', '63 ^ ')),('remove_one',remove_one),('sentinel',sentinel),('unchecked',unchecked),('shift6',shift6),('swar',swar),('exact',exact)]: v[name]=fn(BASE)
    v.update({p.stem:p.read_text() for p in SUPPORT.glob('*.rs') if p.stem!='baseline'})
    return v

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--variants',default='baseline,shifted,masks,xor,remove_one,sentinel,unchecked,shift6')
    parser.add_argument('--count',type=int,default=200000)
    parser.add_argument('--repeats',type=int,default=3)
    parser.add_argument('--model',action='store_true')
    parser.add_argument('--only',choices=['bench','constructors','edges','noops'],default='bench')
    parser.add_argument('--large',action='store_true')
    parser.add_argument('--output',default='veb_argon_screen')
    args=parser.parse_args(); vs=variants(); names=args.variants.split(',')
    sources={n:vs[n] for n in names if n!='pred'}
    if 'pred' in names: sources['pred']=(ROOT/'src/DataStructure/predecessor64.rs').read_text()
    types={n:('Predecessor64' if n=='pred' else 'VebTree') for n in names}
    raw=(ROOT/'tests/predecessor64_performance/common.rs').read_text()
    common=raw[raw.index('trait Set:'):raw.index('impl_set!(baseline')]+raw[raw.index('fn rand('):]
    with tempfile.TemporaryDirectory(prefix='veb-argon-') as tmp:
        tmp=Path(tmp)
        for name,src in sources.items(): (tmp/(name+'.rs')).write_text(src)
        (tmp/'common.rs').write_text(common+'\n'+'\n'.join(f'mod {n}; impl_set!({n}::{types[n]});' for n in names))
        compiler=['rustc','+1.89.0','--edition=2021','-Awarnings']
        if args.model:
            model=f'#[path = "{ROOT / "src/DataStructure/predecessor64.rs"}"] mod model_reference;\ninclude!("common.rs");\n'+(ROOT/'tests/predecessor64_performance/model_checks.rs').read_text()
            checks=(ROOT/'tests/predecessor64_performance/constructor_checks.rs').read_text()
            large=(ROOT/'tests/veb_tree.rs').read_text().split('#[test]\nfn large_sparse_recursive_boundaries')[1]
            for n in names:
                model+=f'\nmod test_{n} {{ use super::*; type TestedSet = {n}::{types[n]};\n#[test] fn model() {{ verify::<TestedSet>(); }}\n'+checks+'\n#[test]\nfn large_sparse_recursive_boundaries'+large.replace('current::', 'model_reference::')+'\n}\n'
            (tmp/'model.rs').write_text(model)
            for flags in [[],['-O']]:
                subprocess.run(compiler+flags+['--test',str(tmp/'model.rs'),'-o',str(tmp/'model')],check=True)
                subprocess.run([str(tmp/'model'),'--test-threads=1'],check=True)
        kind=args.only
        code=(ROOT/f'tests/support/predecessor64_argon/{kind}.rs').read_text()
        arms='\n'.join(f'{i} => ("{n}", run::<{n}::{types[n]}>(n, &initial, &ops'+(', kind' if kind in ['edges','noops'] else '')+')),' for i,n in enumerate(names))
        if kind=='constructors':
            arms='\n'.join(f'{i} => ("{n}", build::<{n}::{types[n]}>(constructor, &base, &string)),' for i,n in enumerate(names))
            code=code.replace('BUILD_IMPLS','\n'.join(f'impl Build for {n}::{types[n]} {{ fn from_words(v:Vec<u64>)->Self {{ Self::from_vec_u64(v) }} fn from_usize(v:Vec<usize>)->Self {{ Self::from_vec_usize(v) }} fn from_string(v:String)->Self {{ Self::from_01_string(v) }} }}' for n in names))
        code=code.replace('VARIANTS',str(len(names))).replace('MATCH_ARMS',arms)
        if kind=='bench':
            code=code.replace('for op in black_box(ops) {','for op in black_box(ops) {\n        let set = black_box(&mut set);')
            code=code.replace('[2, 64, 4096]', '[2, 64, 4096, n].into_iter().collect::<std::collections::BTreeSet<_>>()')
            code=code.replace('["strict", "inclusive", "update", "mixed"]','["strict", "inclusive", "update", "mixed", "extrema", "contains", "empty", "boundary"]')
            code=code.replace('let mut state =','let saved = initial.clone();\n                if workload == "empty" { initial.clear(); }\n                let mut state =')
            code=code.replace('            }\n        }\n    }\n}', '                initial = saved;\n            }\n        }\n    }\n}')
            code=code.replace('_ => set.include(op.p) as usize,','7 => set.min(), 8 => set.max(), _ => set.include(op.p) as usize,')
            code=code.replace('"strict" =>','"extrema" => 7 + (r % 2) as u8, "contains" => 6, "empty" | "boundary" | "strict" =>')
            code=code.replace('p: rand(&mut seed) % n','p: if workload == "boundary" { ((rand(&mut seed) % n) & !63) | ((r & 1) * 63) } else { rand(&mut seed) % n }')
            if args.large:
                code=code.replace('[64, 4096, 262_144, 16_777_216]', '[16_777_216, 268_435_456]')
                code=code.replace('[2, 64, 4096, n]', '[4096, 65536, 1048576, n]')
        (tmp/'bench.rs').write_text(code)
        subprocess.run(compiler+['-O',str(tmp/'bench.rs'),'-o',str(tmp/'bench')],check=True)
        result=subprocess.check_output([str(tmp/'bench'),str(args.count),str(args.repeats)],text=True)
    out=ROOT/'tests'/args.output;out.with_suffix('.csv').write_text(result)
    rows=list(csv.DictReader(result.splitlines()));keys=[k for k in rows[0] if k not in ['implementation','repeat','ns_per_op','checksum']]
    groups={}
    for row in rows: groups.setdefault(tuple(row[k] for k in keys),{}).setdefault(row['implementation'],[]).append(float(row['ns_per_op']))
    medians=[dict(zip(keys,key),**{n:statistics.median(v) for n,v in value.items()}) for key,value in groups.items()]
    out.with_suffix('.json').write_text(json.dumps(dict(platform=platform.platform(),rustc=subprocess.check_output(compiler[:2]+['-Vv'],text=True),args=vars(args),source_sha256={k:hashlib.sha256(v.encode()).hexdigest() for k,v in sources.items()},medians=medians),indent=2)+'\n')
    ref=names[0]
    for name in names[1:]:
        ratios=[r[ref]/r[name] for r in medians]
        print(name,'min/geomean/max',round(min(ratios),3),round(math.exp(statistics.mean(map(math.log,ratios))),3),round(max(ratios),3),flush=True)
if __name__=='__main__': main()
