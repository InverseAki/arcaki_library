mod wavelet_group_detail {
    include!("waveletmatrix.rs");

    pub(super) fn build(keys: &[usize]) -> (WaveletMatrix, Vec<Vec<usize>>) {
        let wm = WaveletMatrix::new(keys);
        let mut order: Vec<_> = (0..keys.len()).collect();
        let mut orders = vec![order.clone()];
        for d in 0..wm.max_log {
            let bit = wm.max_log - 1 - d;
            let mut next = Vec::with_capacity(keys.len());
            next.extend(order.iter().copied().filter(|&i| (keys[i] >> bit) & 1 == 0));
            next.extend(order.iter().copied().filter(|&i| (keys[i] >> bit) & 1 == 1));
            order = next;
            orders.push(order.clone());
        }
        (wm, orders)
    }
    pub(super) fn depth(w: &WaveletMatrix) -> usize {
        w.max_log
    }
    pub(super) fn children(
        w: &WaveletMatrix,
        d: usize,
        l: usize,
        r: usize,
    ) -> (usize, usize, usize, usize) {
        let a = w.bitvecs[d].rank0(l);
        let b = w.bitvecs[d].rank0(r);
        (a, b, w.mids[d] + l - a, w.mids[d] + r - b)
    }
}

pub trait WaveletGroup {
    type S: Clone;
    fn identity() -> Self::S;
    fn op(&self, a: &Self::S, b: &Self::S) -> Self::S;
    fn difference(&self, total: &Self::S, prefix: &Self::S) -> Self::S;
}

pub struct WaveletSum<T>(std::marker::PhantomData<fn() -> T>);
impl<T> Default for WaveletSum<T> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}
impl<T> WaveletGroup for WaveletSum<T>
where
    T: Clone + Default + std::ops::Add<Output = T> + std::ops::Sub<Output = T>,
{
    type S = T;
    fn identity() -> T {
        T::default()
    }
    fn op(&self, a: &T, b: &T) -> T {
        a.clone() + b.clone()
    }
    fn difference(&self, total: &T, prefix: &T) -> T {
        total.clone() - prefix.clone()
    }
}

pub struct WaveletXor<T>(std::marker::PhantomData<fn() -> T>);
impl<T> Default for WaveletXor<T> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}
impl<T> WaveletGroup for WaveletXor<T>
where
    T: Clone + Default + std::ops::BitXor<Output = T>,
{
    type S = T;
    fn identity() -> T {
        T::default()
    }
    fn op(&self, a: &T, b: &T) -> T {
        a.clone() ^ b.clone()
    }
    fn difference(&self, total: &T, prefix: &T) -> T {
        total.clone() ^ prefix.clone()
    }
}

pub struct WaveletMatrixPrefix<G: WaveletGroup> {
    wm: wavelet_group_detail::WaveletMatrix,
    prefix: Vec<Vec<G::S>>,
    g: G,
}
impl<G: WaveletGroup> WaveletMatrixPrefix<G> {
    pub fn new(keys: &[usize], data: &[G::S], g: G) -> Self {
        assert_eq!(keys.len(), data.len());
        let (wm, orders) = wavelet_group_detail::build(keys);
        let prefix = orders
            .into_iter()
            .map(|order| {
                let mut p = Vec::with_capacity(order.len() + 1);
                p.push(G::identity());
                for i in order {
                    p.push(g.op(p.last().unwrap(), &data[i]));
                }
                p
            })
            .collect();
        Self { wm, prefix, g }
    }
    pub fn matrix(&self) -> &wavelet_group_detail::WaveletMatrix {
        &self.wm
    }
    pub fn len(&self) -> usize {
        self.wm.len()
    }
    pub fn is_empty(&self) -> bool {
        self.wm.is_empty()
    }
    fn check(&self, l: usize, r: usize) {
        assert!(l <= r && r <= self.len());
    }
    fn at(&self, d: usize, l: usize, r: usize) -> G::S {
        self.g.difference(&self.prefix[d][r], &self.prefix[d][l])
    }
    pub fn prod(&self, l: usize, r: usize) -> G::S {
        self.check(l, r);
        self.at(0, l, r)
    }
    pub fn prod_less(&self, mut l: usize, mut r: usize, upper: usize) -> G::S {
        self.check(l, r);
        let depth = wavelet_group_detail::depth(&self.wm);
        if depth < usize::BITS as usize && upper >> depth != 0 {
            return self.at(0, l, r);
        }
        let mut ans = G::identity();
        for d in 0..depth {
            let (a, b, c, e) = wavelet_group_detail::children(&self.wm, d, l, r);
            if (upper >> (depth - 1 - d)) & 1 == 1 {
                ans = self.g.op(&ans, &self.at(d + 1, a, b));
                l = c;
                r = e;
            } else {
                l = a;
                r = b;
            }
        }
        ans
    }
    pub fn prod_between(&self, l: usize, r: usize, lower: usize, upper: usize) -> G::S {
        self.check(l, r);
        if lower >= upper {
            return G::identity();
        }
        self.g
            .difference(&self.prod_less(l, r, upper), &self.prod_less(l, r, lower))
    }
    pub fn prefix_prod(&self, mut l: usize, mut r: usize, mut k: usize) -> G::S {
        self.check(l, r);
        assert!(k <= r - l);
        let mut ans = G::identity();
        for d in 0..wavelet_group_detail::depth(&self.wm) {
            let (a, b, c, e) = wavelet_group_detail::children(&self.wm, d, l, r);
            if k <= b - a {
                l = a;
                r = b;
            } else {
                ans = self.g.op(&ans, &self.at(d + 1, a, b));
                k -= b - a;
                l = c;
                r = e;
            }
        }
        self.g.op(
            &ans,
            &self.at(wavelet_group_detail::depth(&self.wm), l, l + k),
        )
    }
    pub fn prod_sorted(&self, l: usize, r: usize, start: usize, end: usize) -> G::S {
        self.check(l, r);
        assert!(start <= end && end <= r - l);
        self.g
            .difference(&self.prefix_prod(l, r, end), &self.prefix_prod(l, r, start))
    }
}
pub type WaveletMatrixSum<T = i64> = WaveletMatrixPrefix<WaveletSum<T>>;
pub type WaveletMatrixXor<T = u64> = WaveletMatrixPrefix<WaveletXor<T>>;
