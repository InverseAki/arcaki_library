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
    pub fn new(n: usize, p: usize, edge: &UnweightedGraph) -> Self{
        let mut int = vec![0; n];
        let mut data = Vec::with_capacity(2*n);
        let mut dist = vec![0; n];
        fn lca_dfs(p: usize, pre: usize, d: usize, edge: &UnweightedGraph, data: &mut Vec<(usize, usize)>, int: &mut Vec<usize>, dist: &mut Vec<usize>){
            int[p] = data.len();
            data.push((d, p));
            for &nex in &edge[p]{
                if nex==pre{continue;}
                dist[nex] = dist[p]+1;
                lca_dfs(nex, p, d+1, edge, data, int, dist);
                data.push((d, p));
            }
        }
        lca_dfs(p, !0, 0, &edge, &mut data, &mut int, &mut dist);
        let data = SparseTableMI::build(&data);
        STLCA { int, data, dist}
    }

    pub fn lca(&self, mut u: usize, mut v: usize)->usize{
        if self.int[u] > self.int[v]{swap(&mut u, &mut v);}
        self.data.query(self.int[u], self.int[v]+1).1
    }

    pub fn distance(&self, u: usize, v: usize)->usize{
        let p = self.lca(u, v);
        self.dist[u]+self.dist[v]-2*self.dist[p]
    }
}

pub trait MoState {
    type Data;
    type Query;
    type Ans;
    fn add(&mut self, idx: usize, data: &Self::Data);
    fn sub(&mut self, idx: usize, data: &Self::Data);
    fn ans(&mut self, query_id: usize, q_data: &Self::Query) -> Self::Ans;
}

pub struct MoQuery {
    pub l: usize,
    pub r: usize,
    pub id: usize,
}

pub struct MoVertexSolver<M: MoState>{
    n: usize,
    data: Vec<M::Data>,
    query: Vec<(MoQuery, M::Query)>,
}

impl<M> MoVertexSolver<M> where M: MoState{
    pub fn new(data: Vec<M::Data>)->Self {
        let n = data.len();
        MoVertexSolver { n, data, query: Vec::new() }
    }

    #[inline]
    pub fn add_query(&mut self, u: usize, v: usize, qd: M::Query){
        let id = self.query.len();
        self.query.push((MoQuery{l:u,r:v,id}, qd));
    }

    #[inline]
    fn mo_dfs(&self, p: usize, pre: usize, edge: &UnweightedGraph, int: &mut [usize], seq: &mut Vec<usize>){
        int[p] = seq.len();
        seq.push(p);
        for &nex in &edge[p]{
            if nex==pre{continue;}
            self.mo_dfs(nex, p, edge, int, seq);
        }
        seq.push(p);
    }

    const ROT_DELTA: [u32; 4] = [3, 0, 0, 1];
    #[inline]
    fn hilbert_order(x: u32, y: u32, pow: u32, rot: u32) -> u64 {
        if pow == 0 { return 0; }
        let h: u32 = 1u32 << (pow - 1);
        let mut seg: u32 = if x < h { if y < h { 0 } else { 3 } } else { if y < h { 1 } else { 2 } };
        seg = (seg + rot) & 3;
        let nrot = (rot + MoVertexSolver::<M>::ROT_DELTA[seg as usize]) & 3;
        let nx = x & (h - 1);
        let ny = y & (h - 1);
        let sub: u64 = 1u64 << (2 * pow - 2);
        let mut ord = (seg as u64) * sub;
        let add = MoVertexSolver::<M>::hilbert_order(nx, ny, pow - 1, nrot);
        ord += if seg == 1 || seg == 2 { add } else { sub - 1 - add };
        ord
    }

    fn get_ord(&self) -> Vec<usize> {
        let mx: usize = self.query.iter().map(|(q, _)| q.l.max(q.r)).max().unwrap_or(0);
        let pow = if mx == 0 {0} else {(usize::BITS - mx.leading_zeros()) as u32};
        let mut ord = (0..self.query.len()).map(|i| {
            let q = &self.query[i].0;
            (Self::hilbert_order(q.l as u32,q.r as u32,pow,0),i,)
        }).collect::<Vec<_>>();
        ord.sort_unstable_by_key(|&(key, _)| key);
        ord.into_iter().map(|(_, i)| i).collect()
    }
    
    pub fn solve(&mut self, edge: &UnweightedGraph, state: &mut M)->Vec<<M as MoState>::Ans>{
        let n = self.n;
        let lca = STLCA::new(n, 0, &edge);
        let mut int = vec![0; n];
        let mut seq = Vec::with_capacity(2*n);
        self.mo_dfs(0, !0, edge, &mut int, &mut seq);
        let mut ls = vec![0; self.query.len()];
        for i in 0..self.query.len(){
            let qd = &self.query[i].0;
            let (mut u, mut v) = (qd.l, qd.r);
            if int[u] > int[v]{swap(&mut u, &mut v);}
            ls[i] = lca.lca(u, v);
            self.query[i].0.l = int[u]+1;
            self.query[i].0.r = int[v]+1;
        }
        self.run(state, &ls, &seq)
    }

    #[inline]
    fn toggle(&self, p: usize, f: &mut [bool],state: &mut M){
        if f[p] {
            state.sub(p, &self.data[p]);
        } else {
            state.add(p, &self.data[p]);
        }
        f[p] = !f[p];
    }

    fn run(&mut self, state: &mut M, lca: &[usize], seq: &[usize]) -> Vec<M::Ans> {
        let q = self.query.len();
        let mut res: Vec<Option<M::Ans>> = (0..q).map(|_| None).collect();
        let ord = self.get_ord();
        let mut f = vec![false; self.n];
        let mut l = 0;
        let mut r = 0;
        for qi in ord {
            let (query, qd) = &self.query[qi];
            while l > query.l {
                l -= 1;
                self.toggle(seq[l], &mut f, state);
            }
            while r < query.r {
                self.toggle(seq[r], &mut f, state);
                r += 1;
            }
            while l < query.l {
                self.toggle(seq[l], &mut f, state);
                l += 1;
            }
            while r > query.r {
                r -= 1;
                self.toggle(seq[r], &mut f, state);
            }
            let p = lca[qi];
            self.toggle(p, &mut f, state);
            res[query.id] = Some(state.ans(query.id, qd));
            self.toggle(p, &mut f, state);
        }
        res.into_iter().map(|x| x.unwrap()).collect()
    }
}
