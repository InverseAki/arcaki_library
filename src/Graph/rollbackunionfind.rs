#[path = "../DataStructure/rollbackvector.rs"]
mod rollback_uf_vector;

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

    pub fn rollback(&mut self) {
        if let Some(len) = self.hist.pop() {
            self.parent.rollback(len);
        }
    }

    pub fn snapshot(&mut self) {
        self.hist.clear();
        self.parent.clear_history();
    }

    pub fn all_back(&mut self) {
        while !self.hist.is_empty() {
            self.rollback();
        }
    }
}
