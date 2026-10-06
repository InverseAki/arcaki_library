// Copy this file together with Basic/waveletmatrix.rs (or keep both files adjacent).
// Positions and value/rank ranges are half-open; equal keys keep original index order.
mod wavelet_monoid_detail {
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

/// Associative operation with identity. COMMUTATIVE=true promises op(a,b)=op(b,a).
pub trait WaveletMonoid {
    type S: Clone;
    const COMMUTATIVE: bool = false;
    fn identity() -> Self::S;
    fn op(&self, a: &Self::S, b: &Self::S) -> Self::S;
}
struct WaveletAggregateTree<S> {
    size: usize,
    data: Vec<S>,
}
impl<S: Clone> WaveletAggregateTree<S> {
    fn new<M: WaveletMonoid<S = S>>(a: Vec<S>, m: &M) -> Self {
        let size = a.len().max(1).next_power_of_two();
        let mut data = vec![M::identity(); size * 2];
        for (i, x) in a.into_iter().enumerate() {
            data[size + i] = x;
        }
        for i in (1..size).rev() {
            data[i] = m.op(&data[i * 2], &data[i * 2 + 1]);
        }
        Self { size, data }
    }
    fn set<M: WaveletMonoid<S = S>>(&mut self, i: usize, x: S, m: &M) {
        let mut p = self.size + i;
        self.data[p] = x;
        while p > 1 {
            p /= 2;
            self.data[p] = m.op(&self.data[p * 2], &self.data[p * 2 + 1]);
        }
    }
    fn prod<M: WaveletMonoid<S = S>>(&self, l: usize, r: usize, m: &M) -> S {
        let (mut l, mut r) = (l + self.size, r + self.size);
        let (mut a, mut b) = (M::identity(), M::identity());
        while l < r {
            if l & 1 == 1 {
                a = m.op(&a, &self.data[l]);
                l += 1;
            }
            if r & 1 == 1 {
                r -= 1;
                b = m.op(&self.data[r], &b);
            }
            l /= 2;
            r /= 2;
        }
        m.op(&a, &b)
    }
}

/// Fixed usize keys and mutable arbitrary payloads. Aggregation order is key ascending,
/// then original index ascending, including for noncommutative monoids.
pub struct WaveletMatrixMonoid<M: WaveletMonoid> {
    wm: wavelet_monoid_detail::WaveletMatrix,
    trees: Vec<WaveletAggregateTree<M::S>>,
    leaf_indices: Vec<usize>,
    m: M,
}
impl<M: WaveletMonoid> WaveletMatrixMonoid<M> {
    pub fn new(keys: &[usize], data: &[M::S], m: M) -> Self {
        assert_eq!(keys.len(), data.len());
        let (wm, orders) = wavelet_monoid_detail::build(keys);
        let leaf_indices = orders.last().unwrap().clone();
        let trees = orders
            .into_iter()
            .map(|order| {
                WaveletAggregateTree::new(order.into_iter().map(|i| data[i].clone()).collect(), &m)
            })
            .collect();
        Self {
            wm,
            trees,
            leaf_indices,
            m,
        }
    }
    pub fn matrix(&self) -> &wavelet_monoid_detail::WaveletMatrix {
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
    pub fn get(&self, i: usize) -> M::S {
        assert!(i < self.len());
        self.trees[0].data[self.trees[0].size + i].clone()
    }
    /// Replace payload at an original index; keys remain fixed.
    pub fn set(&mut self, mut i: usize, x: M::S) {
        assert!(i < self.len());
        self.trees[0].set(i, x.clone(), &self.m);
        for d in 0..wavelet_monoid_detail::depth(&self.wm) {
            let (a, b, c, _) = wavelet_monoid_detail::children(&self.wm, d, i, i + 1);
            i = if b > a { a } else { c };
            self.trees[d + 1].set(i, x.clone(), &self.m);
        }
    }
    pub fn kth_index(&self, mut l: usize, mut r: usize, mut k: usize) -> usize {
        self.check(l, r);
        assert!(k < r - l);
        for d in 0..wavelet_monoid_detail::depth(&self.wm) {
            let (a, b, c, e) = wavelet_monoid_detail::children(&self.wm, d, l, r);
            if k < b - a {
                l = a;
                r = b;
            } else {
                k -= b - a;
                l = c;
                r = e;
            }
        }
        self.leaf_indices[l + k]
    }
    pub fn kth_data(&self, l: usize, r: usize, k: usize) -> M::S {
        self.get(self.kth_index(l, r, k))
    }
    pub fn prod_sorted(&self, l: usize, r: usize, start: usize, end: usize) -> M::S {
        self.check(l, r);
        assert!(start <= end && end <= r - l);
        self.fold(0, l, r, start, end)
    }
    pub fn prefix_prod(&self, l: usize, r: usize, k: usize) -> M::S {
        self.prod_sorted(l, r, 0, k)
    }
    /// Aggregate all selected payloads in stable ascending key order.
    pub fn prod(&self, l: usize, r: usize) -> M::S {
        self.check(l, r);
        self.prod_sorted(l, r, 0, r - l)
    }
    pub fn prod_less(&self, l: usize, r: usize, upper: usize) -> M::S {
        self.check(l, r);
        self.prefix_prod(l, r, self.wm.range_freq(l, r, upper))
    }
    pub fn prod_between(&self, l: usize, r: usize, lower: usize, upper: usize) -> M::S {
        self.check(l, r);
        if lower >= upper {
            return M::identity();
        }
        self.prod_sorted(
            l,
            r,
            self.wm.range_freq(l, r, lower),
            self.wm.range_freq(l, r, upper),
        )
    }
    fn fold(&self, d: usize, l: usize, r: usize, start: usize, end: usize) -> M::S {
        if start == end {
            return M::identity();
        }
        if d == wavelet_monoid_detail::depth(&self.wm) {
            return self.trees[d].prod(l + start, l + end, &self.m);
        }
        if M::COMMUTATIVE && start == 0 && end == r - l {
            return self.trees[d].prod(l, r, &self.m);
        }
        let (a, b, c, e) = wavelet_monoid_detail::children(&self.wm, d, l, r);
        let z = b - a;
        let left = self.fold(d + 1, a, b, start.min(z), end.min(z));
        let right = self.fold(d + 1, c, e, start.saturating_sub(z), end.saturating_sub(z));
        self.m.op(&left, &right)
    }
}
