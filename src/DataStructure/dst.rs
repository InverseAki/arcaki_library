pub trait DSTSemiGroup {
    type S: Clone;
    fn op(&self, a: &Self::S, b: &Self::S) -> Self::S;
}

pub struct DST<G: DSTSemiGroup> {
    n: usize,
    data: Vec<G::S>,
    m: G,
}

impl<G: DSTSemiGroup> DST<G> {
    pub fn new(a: Vec<G::S>, m: G) -> Self {
        assert!(!a.is_empty());
        let n = a.len();
        let d = if n == 1 {1} else {(usize::BITS - (n - 1).leading_zeros()) as usize};
        let mut data = Vec::with_capacity(d * n);
        data.extend(a.iter().cloned());
        for k in 1..d {
            let hs = 1usize << k;
            let bs = hs << 1;
            let base = data.len();
            data.extend(a.iter().cloned());
            let layer = &mut data[base..base + n];
            for s in (0..n).step_by(bs) {
                let mid = (s + hs).min(n);
                let end = (s + bs).min(n);
                if s < mid {
                    for i in (s..mid - 1).rev() {
                        let x = m.op(&a[i], &layer[i + 1]);
                        layer[i] = x;
                    }
                }
                if mid < end {
                    for i in mid + 1..end {
                        let x = m.op(&layer[i - 1], &a[i]);
                        layer[i] = x;
                    }
                }
            }
        }
        Self { n, data, m }
    }

    #[inline(always)]
    pub fn query(&self, l: usize, r: usize) -> G::S {
        debug_assert!(l < r && r <= self.n);
        if r - l == 1 {return self.data[l].clone();}
        let x = l ^ (r - 1);
        let k = (usize::BITS - 1 - x.leading_zeros()) as usize;
        self.m.op(
            &self.data[self.n * k + l],
            &self.data[self.n * k + r - 1],
        )
    }
}
