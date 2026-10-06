#[path = "../Basic/rollbackvector.rs"]
mod rollback_weighted_uf_vector;

/// 群上の重み。opは群の積、identityは両側単位元、invは両側逆元。
/// 可換性は不要。
pub trait UFMonoid {
    type S: Clone + PartialEq;
    fn identity(&self) -> Self::S;
    fn op(&self, a: &Self::S, b: &Self::S) -> Self::S;
    fn inv(&self, x: &Self::S) -> Self::S;
}

pub struct RollbackWeightedUnionFind<M: UFMonoid> {
    monoid: M,
    parent: rollback_weighted_uf_vector::RollbackVector<isize>,
    data: rollback_weighted_uf_vector::RollbackVector<M::S>,
    hist: Vec<(usize, usize)>,
}

impl<M> RollbackWeightedUnionFind<M>
where
    M: UFMonoid,
{
    pub fn new(n: usize, monoid: M) -> Self {
        assert!(n <= isize::MAX as usize);
        let parent = rollback_weighted_uf_vector::RollbackVector::new(n, -1);
        let data = (0..n)
            .into_iter()
            .map(|_| monoid.identity())
            .collect::<Vec<_>>();
        let data = rollback_weighted_uf_vector::RollbackVector::from_vec(data);
        RollbackWeightedUnionFind {
            monoid,
            parent,
            data,
            hist: Vec::new(),
        }
    }

    #[inline]
    pub fn find(&self, mut p: usize) -> (usize, M::S) {
        let mut w = self.data[p].clone();
        while self.parent[p] >= 0 {
            p = self.parent[p] as usize;
            w = self.monoid.op(&self.data[p], &w);
        }
        (p, w)
    }

    #[inline]
    pub fn dist(&self, p: usize) -> M::S {
        self.find(p).1
    }

    #[inline]
    pub fn leader(&self, p: usize) -> usize {
        self.find(p).0
    }

    /// diff(u, v) = wという制約を追加する。
    /// 既存の制約と矛盾する場合は状態を変えずfalseを返す。
    /// 矛盾は内部に蓄積しない。どの場合も1操作として履歴に記録する。
    #[inline]
    pub fn union(&mut self, u: usize, v: usize, w: M::S) -> bool {
        let ((mut pu, wu), (mut pv, wv)) = (self.find(u), self.find(v));
        self.hist
            .push((self.parent.history_len(), self.data.history_len()));
        if pu == pv {
            return wv == self.monoid.op(&wu, &w);
        }
        let mut nex = self
            .monoid
            .op(&self.monoid.op(&wu, &w), &self.monoid.inv(&wv));
        if self.parent[pu] > self.parent[pv] {
            std::mem::swap(&mut pu, &mut pv);
            nex = self.monoid.inv(&nex);
        }
        self.parent.set(pu, self.parent[pu] + self.parent[pv]);
        self.parent.set(pv, pu as isize);
        self.data.set(pv, nex);
        true
    }

    #[inline]
    pub fn size(&self, p: usize) -> usize {
        let p = self.find(p).0;
        (-self.parent[p]) as usize
    }

    #[inline]
    pub fn same(&self, u: usize, v: usize) -> bool {
        self.find(u).0 == self.find(v).0
    }

    #[inline]
    pub fn diff(&self, u: usize, v: usize) -> Option<M::S> {
        let (pu, wu) = self.find(u);
        let (pv, wv) = self.find(v);
        if pu == pv {
            Some(self.monoid.op(&self.monoid.inv(&wu), &wv))
        } else {
            None
        }
    }

    /// 直前のunionを取り消す。整合済み・矛盾した制約も1操作として扱う。
    pub fn rollback(&mut self) {
        if let Some((parent, data)) = self.hist.pop() {
            self.parent.rollback(parent);
            self.data.rollback(data);
        }
    }
}

impl<M: UFMonoid> RollbackWeightedUnionFind<M> {
    /// 現在の状態を基準にして履歴を捨てる。
    pub fn snapshot(&mut self) {
        self.hist.clear();
        self.parent.clear_history();
        self.data.clear_history();
    }
    pub fn all_back(&mut self) {
        while !self.hist.is_empty() {
            self.rollback();
        }
    }
}
