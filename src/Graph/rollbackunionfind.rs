#[path = "../Basic/rollbackvector.rs"]
mod rollback_uf_vector;

/// Union by size、経路圧縮なし。find/merge は O(log n)、rollback は O(1)。
/// 失敗した merge も 1 操作として記録する。領域 O(n + 保存中の操作数)。
pub struct RollbackUnionFind {
    parent: rollback_uf_vector::RollbackVector<isize>,
    hist: Vec<usize>,
}

impl RollbackUnionFind {
    pub fn new(n: usize) -> Self {
        assert!(n <= isize::MAX as usize);
        Self {
            parent: rollback_uf_vector::RollbackVector::new(n, -1),
            hist: Vec::new(),
        }
    }

    pub fn find(&self, mut u: usize) -> usize {
        while self.parent[u] >= 0 {
            u = self.parent[u] as usize;
        }
        u
    }

    pub fn merge(&mut self, u: usize, v: usize) -> bool {
        let (mut u, mut v) = (self.find(u), self.find(v));
        self.hist.push(self.parent.history_len());
        if u == v {
            return false;
        }
        if self.parent[u] > self.parent[v] {
            std::mem::swap(&mut u, &mut v);
        }
        self.parent.set(u, self.parent[u] + self.parent[v]);
        self.parent.set(v, u as isize);
        true
    }

    pub fn same(&self, u: usize, v: usize) -> bool {
        self.find(u) == self.find(v)
    }

    pub fn size(&self, u: usize) -> usize {
        (-self.parent[self.find(u)]) as usize
    }

    /// 直前の merge を取り消す。履歴がなければ何もしない。
    pub fn rollback(&mut self) {
        if let Some(len) = self.hist.pop() {
            self.parent.rollback(len);
        }
    }

    /// 元実装と同じく、現在を基準にして過去の履歴を捨てる。
    /// 保存した状態への復帰ではない。O(保存中の操作数) 以内。
    pub fn snapshot(&mut self) {
        self.hist.clear();
        self.parent.clear_history();
    }

    /// 最後の snapshot（なければ初期状態）まで戻す。O(保存中の操作数)。
    pub fn all_back(&mut self) {
        while !self.hist.is_empty() {
            self.rollback();
        }
    }
}
