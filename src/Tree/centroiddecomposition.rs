pub struct CentroidDecomposition {
    pre: Vec<usize>,
    level: Vec<usize>,
}

impl CentroidDecomposition {
    pub fn new(tree: &UnweightedGraph) -> Self {
        let n = tree.n;
        let mut pp = vec![!0; n];
        let mut level = vec![!0; n];
        let mut size = CentroidDecomposition::size_dfs(n, tree);
        let mut stack = VecDeque::from([(0, !0, 0)]);
        for _ in 0..n {
            let (mut p, pre, d) = stack.pop_front().unwrap();
            let mut non = true;
            while non {
                non = false;
                for &nex in &tree[p] {
                    if level[nex] == !0 && size[nex] * 2 > size[p] {
                        size.swap(p, nex);
                        (size[p], p, non) = (size[nex] - size[p], nex, true);
                        break;
                    }
                }
            }
            pp[p] = pre;
            level[p] = d;
            if size[p] > 1 {
                for &nex in &tree[p] {
                    if level[nex] == !0 {
                        stack.push_back((nex, p, d+1));
                    }
                }
            }
        }
        CentroidDecomposition { pre: pp, level }
    }

    #[inline]
    pub fn parent(&self, v: usize) -> usize {
        self.pre[v]
    }

    #[inline]
    pub fn depth(&self, v: usize) -> usize {
        self.level[v]
    }

    fn size_dfs(n: usize, edge: &UnweightedGraph) -> Vec<usize> {
        let mut size = vec![1; n];
        let mut stack = Vec::from([(0, !0)]);
        let mut query = Vec::new();
        while let Some((p, pre)) = stack.pop() {
            for &nex in &edge[p] {
                if pre == nex {continue}
                stack.push((nex, p));
                query.push((nex, p));
            }
        }
        for &(p, pre) in query.iter().rev() {
            size[pre] += size[p];
        }
        size
    }

    #[inline]
    pub fn lca(&self, mut u: usize, mut v: usize) -> usize {
        let (du, dv) = (self.level[u], self.level[v]);
        if du > dv {
            for _ in 0..du - dv {
                u = self.pre[u];
            }
        } else {
            for _ in 0..dv - du {
                v = self.pre[v];
            }
        }
        while u != v {
            (u, v) = (self.pre[u], self.pre[v])
        }
        u
    }

    #[inline]
    pub fn ancestors(&self, v: usize) -> impl Iterator<Item = usize> + '_ {
        std::iter::successors(Some(v), |&v| {
            let p = self.pre[v];
            (p != !0).then_some(p)
        })
    }
}
