#[derive(Clone, Debug)]
pub struct SparseTableMI<T: Ord + Copy> {
    table: Vec<Vec<T>>,
}

impl<T: Ord + Copy> SparseTableMI<T> {
    pub fn build(a: &Vec<T>) -> Self {
        let n = a.len();
        let mut table: Vec<Vec<T>> = Vec::new();
        table.push(a.clone());
        let mut k = 1;
        while 1<<k <= n {
            let prev = &table[k-1];
            let len = n-(1<<k)+1;
            let mut cur = Vec::with_capacity(len);
            let w = 1<<(k-1);
            for i in 0..len {
                cur.push(prev[i].min(prev[i+w]));
            }
            table.push(cur);
            k += 1;
        }
        Self { table }
    }

    pub fn query(&self, l: usize, r: usize) -> T {
        let s = r-l;
        let k = (usize::BITS-1-s.leading_zeros())as usize;
        let w = 1<<k;
        self.table[k][l].min(self.table[k][r - w])
    }
}

#[derive(Clone,Debug)]
pub struct STLCA{
    int: Vec<usize>,
    data: SparseTableMI<(usize, usize)>,
    dist: Vec<usize>
}

impl STLCA {
    pub fn new(n: usize, p: usize, edge: &Vec<Vec<usize>>) -> Self{
        let mut int = vec![0; n];
        let mut data = Vec::with_capacity(2*n);
        let mut dist = vec![0; n];
        fn lca_dfs(p: usize, pre: usize, d: usize, edge: &Vec<Vec<usize>>, data: &mut Vec<(usize, usize)>, int: &mut Vec<usize>, dist: &mut Vec<usize>){
            int[p] = data.len();
            data.push((d, p));
            for &nex in &edge[p]{
                if nex==pre{continue;}
                dist[nex] = dist[p]+1;
                lca_dfs(nex, p, d+1, edge, data, int, dist);
                data.push((d, p));
            }
        }
        lca_dfs(p, !0, 0, edge, &mut data, &mut int, &mut dist);
        let data = SparseTableMI::build(&data);
        STLCA { int, data, dist}
    }

    pub fn query(&self, mut u: usize, mut v: usize)->usize{
        if self.int[u] > self.int[v]{swap(&mut u, &mut v);}
        self.data.query(self.int[u], self.int[v]+1).1
    }

    pub fn distance(&self, u: usize, v: usize)->usize{
        let p = self.query(u, v);
        self.dist[u]+self.dist[v]-2*self.dist[p]
    }
}
