#[path = "../Basic/rollbackvector.rs"]
mod rollback_lazy_vector;

/// opは結合的、id_eは両側単位元。mapはopを保つ作用。
/// composition(f, g)は「gを適用してからf」を表す。
/// map(identity(), x) = x。値に区間長が必要ならSに長さを含める。
pub trait RollbackLazySegtreeMonoid {
    type S: Clone;
    type F: Clone;
    fn id_e() -> Self::S;
    fn op(a: &Self::S, b: &Self::S) -> Self::S;
    fn identity() -> Self::F;
    fn map(f: &Self::F, x: &Self::S) -> Self::S;
    fn composition(f: &Self::F, g: &Self::F) -> Self::F;
}

/// 半開区間[l, r)。構築O(n)、更新・取得・探索O(log n)。
/// rollbackは直前の更新1回を戻し、O(log n)。読み取りは履歴を変更しない。
/// 領域O(n + 保存中の更新数 * log n)。空区間への更新も1操作として記録。
pub struct RollbackLazySegtree<M: RollbackLazySegtreeMonoid> {
    n: usize,
    size: usize,
    // 集約値には自身の遅延作用を反映済み。子には未反映。
    nodes: rollback_lazy_vector::RollbackVector<(M::S, Option<M::F>)>,
    hist: Vec<usize>,
}

impl<M: RollbackLazySegtreeMonoid> RollbackLazySegtree<M> {
    pub fn new(n: usize) -> Self {
        Self::from(vec![M::id_e(); n])
    }
    pub fn from(a: Vec<M::S>) -> Self {
        let n = a.len();
        let size = n.next_power_of_two();
        let mut nodes = vec![(M::id_e(), None); 2 * size];
        for (i, x) in a.into_iter().enumerate() {
            nodes[size + i].0 = x;
        }
        for i in (1..size).rev() {
            nodes[i].0 = M::op(&nodes[2 * i].0, &nodes[2 * i + 1].0);
        }
        Self {
            n,
            size,
            nodes: rollback_lazy_vector::RollbackVector::from_vec(nodes),
            hist: Vec::new(),
        }
    }
    pub fn build(a: &[M::S]) -> Self {
        Self::from(a.to_vec())
    }
    pub fn len(&self) -> usize {
        self.n
    }
    pub fn is_empty(&self) -> bool {
        self.n == 0
    }
    pub fn rollback(&mut self) {
        if let Some(len) = self.hist.pop() {
            self.nodes.rollback(len);
        }
    }
    /// 現在の状態を基準にして履歴を捨てる。
    pub fn snapshot(&mut self) {
        self.hist.clear();
        self.nodes.clear_history();
    }
    pub fn all_back(&mut self) {
        while !self.hist.is_empty() {
            self.rollback();
        }
    }

    fn apply_node(&mut self, k: usize, f: &M::F) {
        let value = M::map(f, &self.nodes[k].0);
        let lazy = if k < self.size {
            Some(match &self.nodes[k].1 {
                Some(g) => M::composition(f, g),
                None => f.clone(),
            })
        } else {
            None
        };
        self.nodes.set(k, (value, lazy));
    }
    fn push(&mut self, k: usize) {
        if let Some(f) = self.nodes[k].1.clone() {
            self.apply_node(2 * k, &f);
            self.apply_node(2 * k + 1, &f);
            self.nodes.set(k, (self.nodes[k].0.clone(), None));
        }
    }
    fn pull(&mut self, k: usize) {
        let value = M::op(&self.nodes[2 * k].0, &self.nodes[2 * k + 1].0);
        self.nodes.set(k, (value, None));
    }
    pub fn set(&mut self, p: usize, x: M::S) {
        assert!(p < self.n);
        self.hist.push(self.nodes.history_len());
        self.set_inner(1, 0, self.size, p, x);
    }
    fn set_inner(&mut self, k: usize, l: usize, r: usize, p: usize, x: M::S) {
        if r - l == 1 {
            self.nodes.set(k, (x, None));
            return;
        }
        self.push(k);
        let mid = (l + r) / 2;
        if p < mid {
            self.set_inner(2 * k, l, mid, p, x);
        } else {
            self.set_inner(2 * k + 1, mid, r, p, x);
        }
        self.pull(k);
    }
    pub fn apply(&mut self, p: usize, f: M::F) {
        assert!(p < self.n);
        self.apply_range(p, p + 1, f);
    }
    pub fn apply_range(&mut self, l: usize, r: usize, f: M::F) {
        assert!(l <= r && r <= self.n);
        self.hist.push(self.nodes.history_len());
        if l < r {
            self.apply_inner(1, 0, self.size, l, r, &f);
        }
    }
    fn apply_inner(&mut self, k: usize, l: usize, r: usize, ql: usize, qr: usize, f: &M::F) {
        if qr <= l || r <= ql {
            return;
        }
        if ql <= l && r <= qr {
            self.apply_node(k, f);
            return;
        }
        self.push(k);
        let mid = (l + r) / 2;
        self.apply_inner(2 * k, l, mid, ql, qr, f);
        self.apply_inner(2 * k + 1, mid, r, ql, qr, f);
        self.pull(k);
    }
    // 祖先の作用は自身の遅延作用より新しい。
    fn descend(&self, k: usize, carry: &Option<M::F>) -> Option<M::F> {
        match (carry, &self.nodes[k].1) {
            (Some(f), Some(g)) => Some(M::composition(f, g)),
            (Some(f), None) => Some(f.clone()),
            (None, g) => g.clone(),
        }
    }
    fn value(&self, k: usize, carry: &Option<M::F>) -> M::S {
        match carry {
            Some(f) => M::map(f, &self.nodes[k].0),
            None => self.nodes[k].0.clone(),
        }
    }
    pub fn get(&self, p: usize) -> M::S {
        assert!(p < self.n);
        self.prod(p, p + 1)
    }
    pub fn prod(&self, l: usize, r: usize) -> M::S {
        assert!(l <= r && r <= self.n);
        if l == r {
            return M::id_e();
        }
        self.prod_inner(1, 0, self.size, l, r, &None)
    }
    fn prod_inner(
        &self,
        k: usize,
        l: usize,
        r: usize,
        ql: usize,
        qr: usize,
        carry: &Option<M::F>,
    ) -> M::S {
        if qr <= l || r <= ql {
            return M::id_e();
        }
        if ql <= l && r <= qr {
            return self.value(k, carry);
        }
        let next = self.descend(k, carry);
        let mid = (l + r) / 2;
        M::op(
            &self.prod_inner(2 * k, l, mid, ql, qr, &next),
            &self.prod_inner(2 * k + 1, mid, r, ql, qr, &next),
        )
    }
    pub fn all_prod(&self) -> M::S {
        self.nodes[1].0.clone()
    }
    /// O((r-l) log n)。読み取りで遅延作用をpushしない。
    pub fn get_slice(&self, l: usize, r: usize) -> Vec<M::S> {
        assert!(l <= r && r <= self.n);
        (l..r).map(|p| self.get(p)).collect()
    }
    /// g(id_e())はtrue。区間を右に伸ばしたときtrueからfalseになる単調な述語。
    pub fn max_right<G: Fn(&M::S) -> bool>(&self, l: usize, g: G) -> usize {
        assert!(l <= self.n && g(&M::id_e()));
        self.search(1, 0, self.size, l, self.n, &None, &mut M::id_e(), &g, false)
            .unwrap_or(self.n)
    }
    /// g(id_e())はtrue。区間を左に伸ばしたときtrueからfalseになる単調な述語。
    pub fn min_left<G: Fn(&M::S) -> bool>(&self, r: usize, g: G) -> usize {
        assert!(r <= self.n && g(&M::id_e()));
        self.search(1, 0, self.size, 0, r, &None, &mut M::id_e(), &g, true)
            .unwrap_or(0)
    }
    fn search<G: Fn(&M::S) -> bool>(
        &self,
        k: usize,
        l: usize,
        r: usize,
        ql: usize,
        qr: usize,
        carry: &Option<M::F>,
        acc: &mut M::S,
        g: &G,
        reverse: bool,
    ) -> Option<usize> {
        if qr <= l || r <= ql {
            return None;
        }
        if ql <= l && r <= qr {
            let value = self.value(k, carry);
            let sum = if reverse {
                M::op(&value, acc)
            } else {
                M::op(acc, &value)
            };
            if g(&sum) {
                *acc = sum;
                return None;
            }
            if r - l == 1 {
                return Some(if reverse { r } else { l });
            }
        }
        let next = self.descend(k, carry);
        let mid = (l + r) / 2;
        if reverse {
            self.search(2 * k + 1, mid, r, ql, qr, &next, acc, g, reverse)
                .or_else(|| self.search(2 * k, l, mid, ql, qr, &next, acc, g, reverse))
        } else {
            self.search(2 * k, l, mid, ql, qr, &next, acc, g, reverse)
                .or_else(|| self.search(2 * k + 1, mid, r, ql, qr, &next, acc, g, reverse))
        }
    }
}
