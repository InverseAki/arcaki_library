import re

def inline(s, always=False):
    start=s.index('impl<')
    return s[:start]+re.sub(r'(?m)^(    )(pub )?fn ', lambda m: m[1]+'#[inline'+('(always)' if always else '')+']\n'+m[0], s[start:])

def search(s):
    for op in ['M::op', 'self.m.op', 'F::op']:
        for a,b,pred in [('ac','self.data[l]','f'),('self.data[r]','ac','f')]:
            if op=='F::op': pred='g'
            expr=f'{op}(&{a}, &{b})'
            check=expr if pred=='g' else '&'+expr
            s=s.replace(f'if !{pred}({check}) {{',f'let next = {expr};\n            if !{pred}('+('next.clone()' if pred=='g' else '&next')+') {')
            s=s.replace(f'ac = {expr};', 'ac = next;')
    s=s.replace('while l % 2 == 0 {\n                l >>= 1;\n            }','l >>= l.trailing_zeros();')
    s=s.replace('while l % 2 == 0{\n                l >>= 1;\n            }','l >>= l.trailing_zeros();')
    return s

def flags(s):
    s=s.replace('    log: usize,','    log: usize,\n    pending: Vec<bool>,')
    s=s.replace('n, log, data, lazy,','n, log, data, lazy, pending: vec![false; n],')
    s=s.replace('if k < self.n { self.lazy[k] = F::composition(&f, &self.lazy[k]) }','''if k < self.n {
            self.lazy[k] = if self.pending[k] { F::composition(&f, &self.lazy[k]) } else { f };
            self.pending[k] = true;
        }''')
    s=s.replace('    fn push(&mut self, k: usize) {','    fn push(&mut self, k: usize) {\n        if !self.pending[k] { return; }\n        self.pending[k] = false;')
    return s

def boundary(s):
    # Only different boundary ancestors need propagation/reconstruction.
    s=s.replace('if ((r >> i) << i) != r {','if ((r >> i) << i) != r && (((l >> i) << i) == l || (l >> i) != ((r - 1) >> i)) {')
    s=s.replace('if ((right >> i) << i) != right {','if ((right >> i) << i) != right && (((left >> i) << i) == left || (left >> i) != ((right - 1) >> i)) {')
    return s

def sliced(s):
    start=s.index('        l += self.n; r += self.n;', s.index('pub fn get_slice'))
    return s[:start]+'''        if l >= r { return Vec::new(); }
        l += self.n; r += self.n;
        for h in (1..=self.log).rev() {
            for k in (l >> h)..=((r - 1) >> h) {
                self.push(k);
            }
        }
        self.data[l..r].to_vec()
    }
}
'''

def owned(s):
    return s.replace('for (i, v) in a.iter().enumerate() {\n            data[i + n] = v.clone();','for (i, v) in a.into_iter().enumerate() {\n            data[i + n] = v;')

def half(s):
    return s.replace('vec![F::identity(); n << 1]','vec![F::identity(); n]').replace('vec![F::identity(); n<<1]','vec![F::identity(); n]')

def borrowed(s):
    s=s.replace('k: usize, f: F::F)', 'k: usize, f: &F::F)')
    s=s.replace('F::map(&f,', 'F::map(f,').replace('F::composition(&f,','F::composition(f,')
    s=s.replace('else { f };','else { f.clone() };')
    s=s.replace('self.inner_apply(l, f.clone())','self.inner_apply(l, &f)').replace('self.inner_apply(r, f.clone())','self.inner_apply(r, &f)')
    s=s.replace('        self.inner_apply(2 * k, self.lazy[k].clone());\n        self.inner_apply(2 * k + 1, self.lazy[k].clone());\n        self.lazy[k] = F::identity();','        let f = std::mem::replace(&mut self.lazy[k], F::identity());\n        self.inner_apply(2 * k, &f);\n        self.inner_apply(2 * k + 1, &f);')
    return s

def unchecked(s):
    # Experimental comparison only; indices have the same preconditions as baseline.
    s=re.sub(r'&self.data\[([^\[\]]+)\]',r'unsafe { self.data.get_unchecked(\1) }',s)
    return s

def optional(s):
    s=half(s).replace('lazy: Vec<F::F>','lazy: Vec<Option<F::F>>')
    s=s.replace('vec![F::identity(); n]','vec![None; n]')
    s=s.replace('if k < self.n { self.lazy[k] = F::composition(&f, &self.lazy[k]) }','''if k < self.n {
            self.lazy[k] = Some(match &self.lazy[k] {
                Some(g) => F::composition(&f, g),
                None => f,
            });
        }''')
    start=s.index('        self.inner_apply(2 * k, self.lazy[k].clone());')
    end=s.index('\n    }',start)
    s=s[:start]+'''        if let Some(f) = self.lazy[k].take() {
            self.inner_apply(2 * k, f.clone());
            self.inner_apply(2 * k + 1, f);
        }'''+s[end:]
    return s

def fast_prod(s,lazy=False):
    start=s.index('        ',s.index('\n',s.index('pub fn prod')))
    # Insert after the existing empty-range guard/assertion.
    if lazy:
        start=s.index('\n',s.index('if r <= l { return F::id_e() }',start))+1
        extra='        if l == 0 && r == self.n { return self.data[1].clone(); }\n        if r == l + 1 { return self.get(l); }\n'
    else:
        if 'assert!(l <= r && r <= self.n);' in s[start:]: start=s.index('\n',s.index('assert!(l <= r && r <= self.n);',start))+1
        extra='        if l == r { return M::identity(); }\n        if l == 0 && r == self.n { return self.data[1].clone(); }\n        if r == l + 1 { return self.get(l); }\n'
    return s[:start]+extra+s[start:]

def paired(s):
    for op in ['M::op','self.m.op','F::op']:
        s=s.replace(f'{op}(&self.data[p << 1], &self.data[(p << 1) | 1])',f'{{ let children = &self.data[p * 2..p * 2 + 2]; {op}(&children[0], &children[1]) }}')
        s=s.replace(f'{op}(&self.data[2 * k], &self.data[2 * k + 1])',f'{{ let children = &self.data[k * 2..k * 2 + 2]; {op}(&children[0], &children[1]) }}')
    return s

def flag_compose(s):
    s=flags(half(s))
    s=s.replace('self.lazy[k] = if self.pending[k] { F::composition(&f, &self.lazy[k]) } else { f };','self.lazy[k] = F::composition(&f, &self.lazy[k]);')
    return s

def boundary_loops(s):
    start='''        for i in (1..=self.log).rev() {
            if ((l >> i) << i) != l {
                self.push(l >> i);
            }'''
    for right in ['(r-1)', '(r - 1)']:
        old=start+f'''
            if ((r >> i) << i) != r {{
                self.push({right} >> i);
            }}
        }}'''
        new='''        let lz = l.trailing_zeros() as usize;
        let rz = r.trailing_zeros() as usize;
        for i in (lz + 1..=self.log).rev() {
            self.push(l >> i);
        }
        for i in (rz + 1..=self.log).rev() {
            self.push((r - 1) >> i);
        }'''
        s=s.replace(old,new)
    s=s.replace('if ((left >> i) << i) != left {','if i > lz {').replace('if ((right >> i) << i) != right {','if i > rz {')
    return s

def fullonly(s,lazy=False):
    marker='        if r <= l { return F::id_e() }' if lazy else ('        assert!(l <= r && r <= self.n);' if 'assert!(l <= r && r <= self.n);' in s else '    pub fn prod(&self, l: usize, r: usize) -> M::S {')
    return s.replace(marker,marker+'\n        if r - l == self.n { return self.data[1].clone(); }')

def construct(s,lazy=False):
    if lazy:
        s=s.replace('let mut data = vec![F::id_e(); n << 1];\n        data[n..(n + vec.len())].clone_from_slice(vec);','''let mut data = Vec::with_capacity(n << 1);
        data.resize(n, F::id_e());
        data.extend_from_slice(vec);
        data.resize_with(n << 1, F::id_e);''')
    else:
        start=s.index('        let mut data = vec![',s.index('    pub fn from'))
        end=s.index('        for i in (1..n).rev()',start)
        s=s[:start]+'''        let mut data = Vec::with_capacity(2 * n);
        data.resize(n, M::identity());
        data.extend(a);
        data.resize_with(2 * n, M::identity);
'''+s[end:]
    if not lazy and "fn op(&self" in s:
        s=s.replace("data.resize(n, M::identity());", "data.resize(n, e);")
    return s

def variants(base):
    p, s, l = base
    l = l.replace('while r % 2 != 0 {', 'while r > 1 && r % 2 != 0 {')
    base = (p, s, l)
    common = lambda f: tuple(f(src) for src in base)
    reuse = lambda src: search(src).replace('l >>= l.trailing_zeros();', 'while l % 2 == 0 { l >>= 1; }')
    combined = (search(owned(p)), search(s), sliced(boundary(borrowed(half(flags(search(l)))))))
    prefix = (construct(reuse(p)), construct(reuse(s)))
    core = borrowed(flag_compose(construct(reuse(l), True)))
    return {
        'baseline': base,
        'inline': common(inline),
        'always_inline': common(lambda src: inline(src, True)),
        'search': common(search),
        'owned': (owned(p), s, l),
        'lazy_half': (p, s, half(l)),
        'lazy_flags': (p, s, flags(half(l))),
        'lazy_boundary': (p, s, boundary(l)),
        'lazy_borrow': (p, s, borrowed(l)),
        'lazy_slice': (p, s, sliced(l)),
        'unchecked': common(unchecked),
        'combined': combined,
        'combined_inline': tuple(inline(src) for src in combined),
        'lazy_option': (p, s, optional(l)),
        'fast_prod': (fast_prod(p), fast_prod(s), fast_prod(l, True)),
        'paired': common(paired),
        'combined_option': (search(owned(p)), search(s), sliced(boundary(optional(search(l))))),
        'combined_fastprod': tuple(fast_prod(src, i == 2) for i, src in enumerate(combined)),
        'search_reuse': common(reuse),
        'lazy_flag_compose': (p, s, flag_compose(l)),
        'conservative': (fast_prod(owned(reuse(p))), fast_prod(reuse(s)), sliced(boundary(borrowed(half(reuse(l)))))) ,
        'boundary_loops': (p, s, boundary_loops(l)),
        'fullonly': (fullonly(p), fullonly(s), fullonly(l, True)),
        'minimal': (owned(p), s, sliced(boundary(borrowed(half(l))))),
        'construct': (construct(p), construct(s), construct(l, True)),
        'minimal_construct': (construct(p), construct(s), sliced(boundary(borrowed(half(construct(l, True)))))),
        'final_flags': (*prefix, sliced(core)),
        'final_boundary': (*prefix, sliced(boundary(core))),
        'final_loops': (*prefix, sliced(boundary_loops(core))),
    }

measured_variants = variants

def variants(base):
    v = measured_variants(base)
    p, s, l = v['final_flags']
    l = l.replace('        if r <= l { return F::id_e() }', '        if r <= l { return F::id_e() }\n        if l == 0 && r == self.n { return self.data[1].clone(); }')
    l = l.replace('        if l >= r { return; }', '        if l >= r { return; }\n        if l == 0 && r == self.n { self.inner_apply(1, &f); return; }')
    v['selected'] = (p, s, l)
    return v
