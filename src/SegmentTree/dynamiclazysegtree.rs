pub trait DynamicSegtreeMonoid {
    type S: Clone;
    fn identity() -> Self::S;
    fn op(a: &Self::S, b: &Self::S) -> Self::S;
}

pub trait DynamicLazySegtreeMonoid {
    type M: DynamicSegtreeMonoid;
    type F: Clone;

    fn id_e() -> Self::F;

    fn mapping(
        f: &Self::F,
        x: &<Self::M as DynamicSegtreeMonoid>::S,
        len: usize,
    ) -> <Self::M as DynamicSegtreeMonoid>::S;

    fn composition(f: &Self::F, g: &Self::F) -> Self::F;
}

pub struct DynamicSegtreeNode<MM>
where
    MM: DynamicLazySegtreeMonoid,
{
    val: <MM::M as DynamicSegtreeMonoid>::S,
    lazy: MM::F,
    len: u32,
    left: u32,
    right: u32,
}

pub struct DynamicLazySegtree<MM>
where
    MM: DynamicLazySegtreeMonoid,
{
    n: usize,
    data: Vec<DynamicSegtreeNode<MM>>,
}

impl<MM> DynamicLazySegtree<MM>
where
    MM: DynamicLazySegtreeMonoid,
{
    pub fn new(n: usize) -> Self {
        let mut data = Vec::new();

        data.push(DynamicSegtreeNode::<MM> {
            val: <MM::M as DynamicSegtreeMonoid>::identity(),
            lazy: MM::id_e(),
            len: 0,
            left: 0,
            right: 0,
        });

        data.push(DynamicSegtreeNode::<MM> {
            val: <MM::M as DynamicSegtreeMonoid>::identity(),
            lazy: MM::id_e(),
            len: n as u32,
            left: 0,
            right: 0,
        });

        Self { n, data }
    }

    #[inline]
    fn new_node(&mut self, len: usize) -> u32 {
        let idx = self.data.len() as u32;
        self.data.push(DynamicSegtreeNode::<MM> {
            val: <MM::M as DynamicSegtreeMonoid>::identity(),
            lazy: MM::id_e(),
            len: len as u32,
            left: 0,
            right: 0,
        });
        idx
    }

    #[inline]
    fn left_len(len: usize) -> usize {
        len >> 1
    }

    #[inline]
    fn right_len(len: usize) -> usize {
        len - (len >> 1)
    }

    #[inline]
    fn ensure_children(&mut self, k: u32) {
        let len = self.data[k as usize].len as usize;
        if len <= 1 {
            return;
        }
        let llen = Self::left_len(len);
        let rlen = Self::right_len(len);

        if self.data[k as usize].left == 0 {
            let c = self.new_node(llen);
            self.data[k as usize].left = c;
        }
        if self.data[k as usize].right == 0 {
            let c = self.new_node(rlen);
            self.data[k as usize].right = c;
        }
    }

    #[inline]
    fn all_apply(&mut self, k: u32, f: &MM::F) {
        let len = self.data[k as usize].len as usize;
        let new_val = MM::mapping(f, &self.data[k as usize].val, len);
        let new_lazy = MM::composition(f, &self.data[k as usize].lazy);
        self.data[k as usize].val = new_val;
        self.data[k as usize].lazy = new_lazy;
    }

    #[inline]
    fn push(&mut self, k: u32) {
        let len = self.data[k as usize].len as usize;
        if len <= 1 {
            return;
        }
        self.ensure_children(k);

        let f = self.data[k as usize].lazy.clone();
        let lch = self.data[k as usize].left;
        let rch = self.data[k as usize].right;

        self.all_apply(lch, &f);
        self.all_apply(rch, &f);

        self.data[k as usize].lazy = MM::id_e();
    }

    #[inline]
    fn pull(&mut self, k: u32) {
        let lch = self.data[k as usize].left;
        let rch = self.data[k as usize].right;

        let lv = if lch == 0 {
            <MM::M as DynamicSegtreeMonoid>::identity()
        } else {
            self.data[lch as usize].val.clone()
        };
        let rv = if rch == 0 {
            <MM::M as DynamicSegtreeMonoid>::identity()
        } else {
            self.data[rch as usize].val.clone()
        };

        self.data[k as usize].val = <MM::M as DynamicSegtreeMonoid>::op(&lv, &rv);
    }

    pub fn apply_range(&mut self, l: usize, r: usize, f: MM::F) {
        assert!(l <= r && r <= self.n);
        self.apply_range_rec(1, 0, self.n, l, r, &f);
    }

    fn apply_range_rec(&mut self, k: u32, nl: usize, nr: usize, ql: usize, qr: usize, f: &MM::F) {
        if qr <= nl || nr <= ql {
            return;
        }
        if ql <= nl && nr <= qr {
            self.all_apply(k, f);
            return;
        }
        self.push(k);
        let mid = nl + ((nr - nl) >> 1);
        let lch = self.data[k as usize].left;
        let rch = self.data[k as usize].right;
        self.apply_range_rec(lch, nl, mid, ql, qr, f);
        self.apply_range_rec(rch, mid, nr, ql, qr, f);
        self.pull(k);
    }

    pub fn prod(&mut self, l: usize, r: usize) -> <MM::M as DynamicSegtreeMonoid>::S {
        assert!(l <= r && r <= self.n);
        self.prod_rec(1, 0, self.n, l, r)
    }

    fn prod_rec(
        &mut self,
        k: u32,
        nl: usize,
        nr: usize,
        ql: usize,
        qr: usize,
    ) -> <MM::M as DynamicSegtreeMonoid>::S {
        if qr <= nl || nr <= ql {
            return <MM::M as DynamicSegtreeMonoid>::identity();
        }
        if ql <= nl && nr <= qr {
            return self.data[k as usize].val.clone();
        }
        self.push(k);
        let mid = nl + ((nr - nl) >> 1);
        let lch = self.data[k as usize].left;
        let rch = self.data[k as usize].right;
        let lv = self.prod_rec(lch, nl, mid, ql, qr);
        let rv = self.prod_rec(rch, mid, nr, ql, qr);
        <MM::M as DynamicSegtreeMonoid>::op(&lv, &rv)
    }

    pub fn set(&mut self, p: usize, x: <MM::M as DynamicSegtreeMonoid>::S) {
        assert!(p < self.n);
        self.set_rec(1, 0, self.n, p, x);
    }

    fn set_rec(
        &mut self,
        k: u32,
        nl: usize,
        nr: usize,
        p: usize,
        x: <MM::M as DynamicSegtreeMonoid>::S,
    ) {
        if nr - nl == 1 {
            self.data[k as usize].val = x;
            self.data[k as usize].lazy = MM::id_e();
            return;
        }
        self.push(k);
        let mid = nl + ((nr - nl) >> 1);
        let lch = self.data[k as usize].left;
        let rch = self.data[k as usize].right;
        if p < mid {
            self.set_rec(lch, nl, mid, p, x);
        } else {
            self.set_rec(rch, mid, nr, p, x);
        }
        self.pull(k);
    }

    pub fn get(&mut self, p: usize) -> <MM::M as DynamicSegtreeMonoid>::S {
        assert!(p < self.n);
        self.get_rec(1, 0, self.n, p)
    }

    fn get_rec(&mut self, k: u32, nl: usize, nr: usize, p: usize) -> <MM::M as DynamicSegtreeMonoid>::S {
        if nr - nl == 1 {
            return self.data[k as usize].val.clone();
        }
        self.push(k);
        let mid = nl + ((nr - nl) >> 1);
        let lch = self.data[k as usize].left;
        let rch = self.data[k as usize].right;
        if p < mid {
            self.get_rec(lch, nl, mid, p)
        } else {
            self.get_rec(rch, mid, nr, p)
        }
    }

    pub fn apply_point(&mut self, p: usize, f: MM::F) {
        self.apply_range(p, p + 1, f);
    }

    pub fn all_prod(&self) -> <MM::M as DynamicSegtreeMonoid>::S {
        self.data[1].val.clone()
    }
}

struct M;
impl DynamicSegtreeMonoid for M{
    type S = MI;

    fn identity() -> Self::S {
        MI::new(0)
    }

    fn op(&a: &Self::S, &b: &Self::S) -> Self::S {
        a+b
    }
}

struct MM;
impl DynamicLazySegtreeMonoid for MM{
    type M = M;

    type F = (MI, MI);

    fn id_e() -> Self::F {
        (MI::new(1), MI::new(0))
    }

    fn mapping(
        f: &Self::F,
        &x: &<Self::M as DynamicSegtreeMonoid>::S,
        len: usize,
    ) -> <Self::M as DynamicSegtreeMonoid>::S {
        x*f.0+f.1*len
    }

    fn composition(f: &Self::F, g: &Self::F) -> Self::F {
        (f.0*g.0, f.0*g.1+f.1)
    }
}
