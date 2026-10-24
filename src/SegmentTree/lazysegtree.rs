// Source: https://github.com/InverseAki/arcaki_library/blob/main/src/SegmentTree/lazysegtree.rs
pub fn bit_length(x: usize) -> usize {
    usize::BITS as usize - x.saturating_sub(1).leading_zeros() as usize
}

pub trait SegtreeMonoid {
    type S: Clone;
    fn identity() -> Self::S;
    fn op(a: &Self::S, b: &Self::S) -> Self::S;
}

pub trait LazySegtreeMonoid {
    type M: SegtreeMonoid;
    type F: Clone;
    fn id_e() -> <Self::M as SegtreeMonoid>::S {
        <Self::M as SegtreeMonoid>::identity()
    }
    fn op(
        a: &<Self::M as SegtreeMonoid>::S,
        b: &<Self::M as SegtreeMonoid>::S,
    ) -> <Self::M as SegtreeMonoid>::S {
        <Self::M>::op(a, b)
    }
    fn identity() -> Self::F;
    fn map(f: &Self::F, x: &<Self::M as SegtreeMonoid>::S) -> <Self::M as SegtreeMonoid>::S;
    fn composition(f: &Self::F, g: &Self::F) -> Self::F;
}

pub struct LazySegtree<F>
where
    F: LazySegtreeMonoid,
{
    n: usize,
    log: usize,
    pending: Vec<bool>,
    data: Vec<<F::M as SegtreeMonoid>::S>,
    lazy: Vec<F::F>,
}

impl<F: LazySegtreeMonoid> LazySegtree<F> {
    pub fn new(n: usize) -> Self {
        let n = n.next_power_of_two();
        let log = bit_length(n);
        let lazy = vec![F::identity(); n];
        let data = vec![F::id_e(); n << 1];
        LazySegtree {
            n,
            log,
            data,
            lazy,
            pending: vec![false; n],
        }
    }

    pub fn build(vec: &Vec<<F::M as SegtreeMonoid>::S>) -> Self {
        let n = vec.len().next_power_of_two();
        let log = bit_length(n);
        let lazy = vec![F::identity(); n];
        let mut data = Vec::with_capacity(n << 1);
        data.resize(n, F::id_e());
        data.extend_from_slice(vec);
        data.resize_with(n << 1, F::id_e);
        let mut res = LazySegtree {
            n,
            log,
            data,
            lazy,
            pending: vec![false; n],
        };
        for i in (1..n).rev() {
            res.update(i);
        }
        res
    }

    pub fn set(&mut self, mut p: usize, x: <F::M as SegtreeMonoid>::S) {
        p += self.n;
        for i in (1..=self.log).rev() {
            self.push(p >> i);
        }
        self.data[p] = x;
        for i in 1..=self.log {
            self.update(p >> i);
        }
    }

    fn update(&mut self, k: usize) {
        self.data[k] = F::op(&self.data[2 * k], &self.data[2 * k + 1]);
    }

    fn inner_apply(&mut self, k: usize, f: &F::F) {
        self.data[k] = F::map(f, &self.data[k]);
        if k < self.n {
            self.lazy[k] = F::composition(f, &self.lazy[k]);
            self.pending[k] = true;
        }
    }

    fn push(&mut self, k: usize) {
        if !self.pending[k] {
            return;
        }
        self.pending[k] = false;
        let f = std::mem::replace(&mut self.lazy[k], F::identity());
        self.inner_apply(2 * k, &f);
        self.inner_apply(2 * k + 1, &f);
    }

    pub fn get(&mut self, mut p: usize) -> <F::M as SegtreeMonoid>::S {
        p += self.n;
        for i in (1..=self.log).rev() {
            self.push(p >> i);
        }
        self.data[p].clone()
    }

    pub fn prod(&mut self, mut l: usize, mut r: usize) -> <F::M as SegtreeMonoid>::S {
        if r <= l {
            return F::id_e();
        }
        if l == 0 && r == self.n {
            return self.data[1].clone();
        }
        l += self.n;
        r += self.n;
        for i in (1..=self.log).rev() {
            if ((l >> i) << i) != l {
                self.push(l >> i);
            }
            if ((r >> i) << i) != r {
                self.push((r - 1) >> i);
            }
        }
        let mut acl = F::id_e();
        let mut acr = F::id_e();
        while l < r {
            if l & 1 != 0 {
                acl = F::op(&acl, &self.data[l]);
                l += 1;
            }
            if r & 1 != 0 {
                r -= 1;
                acr = F::op(&self.data[r], &acr);
            }
            l >>= 1;
            r >>= 1;
        }
        F::op(&acl, &acr)
    }

    pub fn all_prod(&mut self) -> <F::M as SegtreeMonoid>::S {
        self.data[1].clone()
    }

    pub fn apply_range(&mut self, mut l: usize, mut r: usize, f: F::F) {
        if l >= r {
            return;
        }
        if l == 0 && r == self.n {
            self.inner_apply(1, &f);
            return;
        }
        l += self.n;
        r += self.n;
        for i in (1..=self.log).rev() {
            if ((l >> i) << i) != l {
                self.push(l >> i);
            }
            if ((r >> i) << i) != r {
                self.push((r - 1) >> i);
            }
        }
        let left = l;
        let right = r;
        while l < r {
            if l & 1 != 0 {
                self.inner_apply(l, &f);
                l += 1;
            }
            if r & 1 != 0 {
                r -= 1;
                self.inner_apply(r, &f);
            }
            l >>= 1;
            r >>= 1;
        }
        for i in 1..=self.log {
            if ((left >> i) << i) != left {
                self.update(left >> i);
            }
            if ((right >> i) << i) != right {
                self.update((right - 1) >> i);
            }
        }
    }

    pub fn max_right<G>(&mut self, mut l: usize, g: G) -> usize
    where
        G: Fn(<F::M as SegtreeMonoid>::S) -> bool,
    {
        assert!(g(F::id_e()));
        if l >= self.n {
            return self.n;
        }
        l += self.n;
        for i in (1..=self.log).rev() {
            self.push(l >> i);
        }
        let mut ac = F::id_e();
        while {
            while l % 2 == 0 {
                l >>= 1;
            }
            let next = F::op(&ac, &self.data[l]);
            if !g(next.clone()) {
                while l < self.n {
                    self.push(l);
                    l *= 2;
                    let res = F::op(&ac, &self.data[l]);
                    if g(res.clone()) {
                        ac = res;
                        l += 1;
                    }
                }
                return l - self.n;
            }
            ac = next;
            l += 1;
            let left = l as isize;
            (left & -left) != left
        } {}
        self.n
    }

    pub fn min_left<G>(&mut self, mut r: usize, g: G) -> usize
    where
        G: Fn(<F::M as SegtreeMonoid>::S) -> bool,
    {
        assert!(g(F::id_e()));
        if r == 0 {
            return 0;
        }
        r += self.n;
        for i in (1..=self.log).rev() {
            self.push((r - 1) >> i);
        }
        let mut ac = F::id_e();
        while {
            r -= 1;
            while r > 1 && r % 2 != 0 {
                r >>= 1;
            }
            let next = F::op(&self.data[r], &ac);
            if !g(next.clone()) {
                while r < self.n {
                    self.push(r);
                    r = 2 * r + 1;
                    let res = F::op(&self.data[r], &ac);
                    if g(res.clone()) {
                        ac = res;
                        r -= 1;
                    }
                }
                return r + 1 - self.n;
            }
            ac = next;
            let right = r as isize;
            (right & -right) != right
        } {}
        0
    }

    pub fn get_slice(&mut self, mut l: usize, mut r: usize) -> Vec<<F::M as SegtreeMonoid>::S> {
        if l >= r {
            return Vec::new();
        }
        l += self.n;
        r += self.n;
        for h in (1..=self.log).rev() {
            for k in (l >> h)..=((r - 1) >> h) {
                self.push(k);
            }
        }
        self.data[l..r].to_vec()
    }
}
