/// Union by size、経路圧縮なし。find/merge は O(log n)、rollback は O(1)。
/// 失敗した merge も 1 操作として記録する。領域 O(n + 保存中の操作数)。
pub struct RollbackUnionFind {
    parent: Vec<isize>,
    hist: Vec<Option<(usize, usize, isize, isize)>>,
}

impl RollbackUnionFind {
    pub fn new(n: usize) -> Self {
        assert!(n <= isize::MAX as usize);
        Self {
            parent: vec![-1; n],
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
        if u == v {
            self.hist.push(None);
            return false;
        }
        if self.parent[u] > self.parent[v] {
            std::mem::swap(&mut u, &mut v);
        }
        self.hist.push(Some((u, v, self.parent[u], self.parent[v])));
        self.parent[u] += self.parent[v];
        self.parent[v] = u as isize;
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
        if let Some(Some((u, v, pu, pv))) = self.hist.pop() {
            self.parent[u] = pu;
            self.parent[v] = pv;
        }
    }

    /// 元実装と同じく、現在を基準にして過去の履歴を捨てる。
    /// 保存した状態への復帰ではない。O(保存中の操作数) 以内。
    pub fn snapshot(&mut self) {
        self.hist.clear();
    }

    /// 最後の snapshot（なければ初期状態）まで戻す。O(保存中の操作数)。
    pub fn all_back(&mut self) {
        while !self.hist.is_empty() {
            self.rollback();
        }
    }
}
