#[path = "../Basic/rollbackvector.rs"]
mod rollback_segtree_vector;

/// op は結合的、identity は両側単位元。可換性は不要。
pub trait RollbackSegtreeMonoid {
    type S: Clone;
    fn identity() -> Self::S;
    fn op(a: &Self::S, b: &Self::S) -> Self::S;
}

/// 区間は 0-indexed の [l, r)。構築 O(n)、更新・区間積・探索 O(log n)。
/// rollbackは直前のset/pushを戻す。O(log n)、領域O(n + 保存中の更新数 * log n)。
pub struct RollbackSegtree<M: RollbackSegtreeMonoid> {
    n: usize,
    size: usize,
    data: rollback_segtree_vector::RollbackVector<M::S>,
    hist: Vec<usize>,
}

impl<M: RollbackSegtreeMonoid> RollbackSegtree<M> {
    pub fn new(n: usize) -> Self {
        let size = n.next_power_of_two();
        let data = vec![M::identity(); 2 * size];
        RollbackSegtree {
            n,
            size,
            data: rollback_segtree_vector::RollbackVector::from_vec(data),
            hist: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.n
    }

    pub fn is_empty(&self) -> bool {
        self.n == 0
    }

    pub fn set(&mut self, i: usize, x: M::S) {
        assert!(i < self.n);
        self.hist.push(self.data.history_len());
        let mut p = i + self.size;
        self.data.set(p, x);
        while p > 1 {
            p /= 2;
            self.data
                .set(p, M::op(&self.data[p << 1], &self.data[(p << 1) | 1]));
        }
    }

    pub fn from(a: Vec<M::S>) -> Self {
        let len = a.len();
        let n = len.next_power_of_two();
        let mut data = vec![M::identity(); 2 * n];
        for (i, v) in a.iter().enumerate() {
            data[i + n] = v.clone();
        }
        for i in (1..n).rev() {
            data[i] = M::op(&data[2 * i], &data[2 * i + 1]);
        }
        RollbackSegtree {
            n: len,
            size: n,
            data: rollback_segtree_vector::RollbackVector::from_vec(data),
            hist: Vec::new(),
        }
    }

    pub fn get(&self, p: usize) -> M::S {
        assert!(p < self.n);
        self.data[self.size + p].clone()
    }

    /// A[i] を op(A[i], x) に更新する。
    pub fn push(&mut self, i: usize, x: M::S) {
        self.set(i, M::op(&self.get(i), &x));
    }

    /// 直前のset/pushを取り消す。履歴がなければ何もしない。
    pub fn rollback(&mut self) {
        if let Some(len) = self.hist.pop() {
            self.data.rollback(len);
        }
    }
    /// 現在の状態を基準にして履歴を捨てる。
    pub fn snapshot(&mut self) {
        self.hist.clear();
        self.data.clear_history();
    }
    pub fn all_back(&mut self) {
        while !self.hist.is_empty() {
            self.rollback();
        }
    }

    pub fn prod(&self, l: usize, r: usize) -> M::S {
        assert!(l <= r && r <= self.n);
        let mut p_l = l + self.size;
        let mut p_r = r + self.size;
        let mut res_l = M::identity();
        let mut res_r = M::identity();
        while p_l < p_r {
            if p_l & 1 == 1 {
                res_l = M::op(&res_l, &self.data[p_l]);
                p_l += 1;
            }
            if p_r & 1 == 1 {
                p_r -= 1;
                res_r = M::op(&self.data[p_r], &res_r);
            }
            p_l >>= 1;
            p_r >>= 1;
        }
        M::op(&res_l, &res_r)
    }

    pub fn all_prod(&self) -> M::S {
        self.data[1].clone()
    }

    /// f(prod(l, r)) が true となる最大の r。f(identity) は true、
    /// r を伸ばしたとき true → false の単調性を持ち、同じ引数には同じ結果を返すこと。
    pub fn max_right<F>(&self, mut l: usize, f: F) -> usize
    where
        F: Fn(&M::S) -> bool,
    {
        assert!(l <= self.n);
        assert!(f(&M::identity()));
        if l == self.n {
            return self.n;
        }
        l += self.size;
        let mut ac = M::identity();
        while {
            while l % 2 == 0 {
                l >>= 1;
            }
            if !f(&M::op(&ac, &self.data[l])) {
                while l < self.size {
                    l <<= 1;
                    let res = M::op(&ac, &self.data[l]);
                    if f(&res) {
                        ac = res;
                        l += 1;
                    }
                }
                return l - self.size;
            }
            ac = M::op(&ac, &self.data[l]);
            l += 1;
            !l.is_power_of_two()
        } {}
        self.n
    }

    /// f(prod(l, r)) が true となる最小の l。f(identity) は true、
    /// l を縮めたとき true → false の単調性を持ち、同じ引数には同じ結果を返すこと。
    pub fn min_left<F>(&self, mut r: usize, f: F) -> usize
    where
        F: Fn(&M::S) -> bool,
    {
        assert!(r <= self.n);
        assert!(f(&M::identity()));
        if r == 0 {
            return 0;
        }
        r += self.size;
        let mut ac = M::identity();
        while {
            r -= 1;
            while r > 1 && r % 2 == 1 {
                r >>= 1;
            }
            if !f(&M::op(&self.data[r], &ac)) {
                while r < self.size {
                    r = 2 * r + 1;
                    let res = M::op(&self.data[r], &ac);
                    if f(&res) {
                        ac = res;
                        r -= 1;
                    }
                }
                return r + 1 - self.size;
            }
            ac = M::op(&self.data[r], &ac);
            !r.is_power_of_two()
        } {}
        0
    }
}
