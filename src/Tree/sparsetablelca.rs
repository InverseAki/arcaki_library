#[derive(Clone,Debug)]
pub struct STLCA{
    int: Vec<usize>,
    data: SparseTableMI<usize>,
    dist: Box<[usize]>
}

impl STLCA {
    pub fn new(p: usize, edge: &UnweightedGraph) -> Self{
        let n = edge.n;
        let mut int = vec![0; n];
        let mut data = Vec::with_capacity(2*n);
        let mut dist = vec![0; n].into_boxed_slice();
        fn lca_dfs(p: usize, pre: usize, edge: &UnweightedGraph, data: &mut Vec<usize>, int: &mut [usize], dist: &mut [usize]){
            int[p] = data.len();
            data.push(p);
            for &nex in &edge[p]{
                if nex==pre{continue;}
                dist[nex] = dist[p]+1;
                lca_dfs(nex, p, edge, data, int, dist);
                data.push(p);
            }
        }
        lca_dfs(p, !0, edge, &mut data, &mut int, &mut dist);
        let data = SparseTableMI::build_by_key(&data, |v| dist[v]);
        STLCA { int, data, dist}
    }
    #[inline]
    pub fn lca(&self, mut u: usize, mut v: usize)->usize{
        if self.int[u] > self.int[v]{std::mem::swap(&mut u, &mut v);}
        self.data.query_by_key(self.int[u], self.int[v]+1, |v| self.dist[v])
    }
    #[inline]
    pub fn distance(&self, u: usize, v: usize)->usize{
        let p = self.lca(u, v);
        self.dist[u]+self.dist[v]-2*self.dist[p]
    }
}
