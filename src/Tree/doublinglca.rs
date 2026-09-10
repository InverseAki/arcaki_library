// 現在は演算機能がついていないが今後前計算で計算も一緒に行うようにする予定

pub struct LCA {
    n: usize,
    log: usize,
    edge: Vec<Vec<usize>>,
    depth: Vec<usize>,
    dp: Vec<Vec<usize>>,
}

impl LCA{
    pub fn new(n: usize, edge: &Vec<Vec<usize>>) -> Self {
        let log = (n as f64).log2().ceil() as usize + 1;
        let parent = vec![vec![!0usize; log]; n];
        let depth = vec![0; n];
        let edge = edge.clone();
        LCA {n, log, depth, edge, dp: parent}
    }

    fn _bfs(&mut self, v: usize) {
        use std::collections::VecDeque;
        let mut stack = VecDeque::from([(v, !0, 0)]);
        while let Some((p, pre, d)) = stack.pop_front() {
            self.dp[p][0] = pre;
            self.depth[p] = d;
            for &nex in &self.edge[p] {
                if nex == pre{continue}
                stack.push_back((nex, p, d + 1));
            }
        }
    }

    pub fn build(&mut self, root: usize) {
        self._bfs(root);
        for i in 1..self.log {
            for v in 0..self.n {
                if self.dp[v][i - 1] != !0 {
                    let p = self.dp[v][i - 1];
                    self.dp[v][i] = self.dp[p][i - 1];
                }
            }
        }
    }

    pub fn lca(&self, mut u: usize, mut v: usize) -> usize {
        use std::mem::swap;
        if self.depth[u] < self.depth[v] {
            swap(&mut u, &mut v);
        }
        for i in (0..self.log).rev() {
            if self.depth[u] >= self.depth[v] + (1 << i) {
                u = self.dp[u][i];
            }
        }
        if u == v {
            return u;
        }
        for i in (0..self.log).rev() {
            if self.dp[u][i] != self.dp[v][i] {
                u = self.dp[u][i];
                v = self.dp[v][i];
            }
        }
        self.dp[u][0]
    }

    pub fn parent(&self, u: usize, k: usize) -> usize {
        let mut x = k;
        let mut p = u;
        let mut z = 0;
        while x > 0 && p < self.n + 1 {
            if x % 2 == 1 {
                p = self.dp[p][z];
            }
            z += 1;
            x /= 2;
        }
        p
    }

    pub fn depth(&self, p: usize) -> usize {
        self.depth[p]
    }

    pub fn dist(&self, u: usize, v: usize) -> usize {
        let p = self.lca(u, v);
        self.depth[u]+self.depth[v]-2*self.depth[p]
    }
}
