pub trait DSUonTreeState {
    type Data;
    type Query;
    type Ans;
    fn add(&mut self, idx: usize, data: &Self::Data);
    fn sub(&mut self, idx: usize, data: &Self::Data);
    fn ans(&mut self, query_id: usize, q_data: &Self::Query) -> Self::Ans;
}

pub struct DSUonTreeSolver<M: DSUonTreeState>{
    r: usize,
    data: Vec<M::Data>,
    query: Vec<(usize, M::Query)>,
}

impl<M> DSUonTreeSolver<M> where M: DSUonTreeState{
    pub fn new(r: usize, data: Vec<M::Data>)->Self {
        DSUonTreeSolver { r, data, query: Vec::new() }
    }

    pub fn add_query(&mut self, p: usize, qs: M::Query){
        self.query.push((p, qs));
    }

    #[inline]
    fn size_dfs(p: usize, pre: usize, edge: &mut UnweightedGraph)->usize{
        let mut res = 1;
        let l = (&edge[p]).len();
        let mut mx = 0;
        let mut rv = 0;
        for i in 0..l{
            let mut nex = edge[p][i];
            if nex==pre{edge.edge.edge.swap(edge.edge.ac[p]+i, edge.edge.ac[p+1]-1);nex=edge[p][i];}
            if nex==pre{continue;}
            let nn = Self::size_dfs(nex, p, edge);
            if mx < nn {
                mx = nn;
                rv = i;
            }
            res += nn;
        }
        if rv!=0{edge.edge[p].swap(0, rv)};
        res
    }

    #[inline]
    fn euler_tour(&self, p: usize, edge: &UnweightedGraph, out: &mut Vec<usize>, seq: &mut Vec<usize>){
        seq.push(p);
        let l  = edge[p].len();
        let cs = if self.r==p {&edge[p]}else{&edge[p][..l-1]};
        for &nex in cs{
            self.euler_tour(nex, edge, out, seq);
        }
        out[p] = seq.len();
    }

    pub fn solve(&self, edge: &mut UnweightedGraph, state: &mut M)->Vec<M::Ans>{
        let mut res: Vec<Option<M::Ans>> = (0..self.query.len()).map(|_| None).collect::<Vec<_>>();
        Self::size_dfs(self.r, !0, edge);
        let n = edge.n;
        let mut out = vec![0; n];
        let mut seq = Vec::with_capacity(n);
        self.euler_tour(self.r, edge, &mut out, &mut seq);
        let mut cnt_qs = vec![0; n+1];
        let q = self.query.len();
        for t in self.query.iter(){
            cnt_qs[t.0+1]+=1;
        }
        let mut qs: Vec<usize> = vec![0; q];
        for i in 0..n{cnt_qs[i+1]+=cnt_qs[i];}
        let ac = cnt_qs.clone();
        for (i,(p,_)) in self.query.iter().enumerate(){
            qs[cnt_qs[*p]] = i;
            cnt_qs[*p]+=1;
        }
        for i in (0..n).rev(){
            let c = seq[i];
            let ch = if c == self.r {&edge[c][..]} else {let l = edge[c].len();&edge[c][..l-1]};
            if let Some(&heavy) = ch.first() {
                for t in out[heavy]..out[c]{
                    let x = seq[t];
                    state.add(x, &self.data[x]);
                }
            }
            state.add(c, &self.data[c]);
            for &idx in &qs[ac[c]..ac[c+1]]{
                res[idx] = Some(state.ans(idx, &self.query[idx].1));
            }
            if c!=self.r{
                let &p = *&edge[c].last().unwrap();
                if edge[p][0]!=c{
                    for t in i..out[c]{
                        let x = seq[t];
                        state.sub(x, &self.data[x]);
                    }
                }
            }
        }
        res.into_iter().map(|x| x.unwrap()).collect::<Vec<_>>()
    }

    pub fn solve_all<F>(&self, edge: &mut UnweightedGraph, state: &mut M, mut f: F) where F: FnMut(usize, &mut M){
        Self::size_dfs(self.r, !0, edge);
        let n = edge.n;
        let mut out = vec![0; n];
        let mut seq = Vec::with_capacity(n);
        self.euler_tour(self.r, edge, &mut out, &mut seq);
        for i in (0..n).rev(){
            let c = seq[i];
            let ch = if c == self.r {&edge[c][..]} else {let l = edge[c].len();&edge[c][..l-1]};
            if let Some(&heavy) = ch.first() {
                for t in out[heavy]..out[c]{
                    let x = seq[t];
                    state.add(x, &self.data[x]);
                }
            }
            state.add(c, &self.data[c]);
            f(c, state);
            if c!=self.r{
                let &p = *&edge[c].last().unwrap();
                if edge[p][0]!=c{
                    for t in i..out[c]{
                        let x = seq[t];
                        state.sub(x, &self.data[x]);
                    }
                }
            }
        }
    }

    pub fn solve_all_with_data<F>(&self, edge: &mut UnweightedGraph, state: &mut M, qd: &[M::Query], mut f: F) where F: FnMut(usize, &M::Query, &mut M){
        Self::size_dfs(self.r, !0, edge);
        let n = edge.n;
        let mut out = vec![0; n];
        let mut seq = Vec::with_capacity(n);
        self.euler_tour(self.r, edge, &mut out, &mut seq);
        for i in (0..n).rev(){
            let c = seq[i];
            let ch = if c == self.r {&edge[c][..]} else {let l = edge[c].len();&edge[c][..l-1]};
            if let Some(&heavy) = ch.first() {
                for t in out[heavy]..out[c]{
                    let x = seq[t];
                    state.add(x, &self.data[x]);
                }
            }
            state.add(c, &self.data[c]);
            f(c, &qd[c], state);
            if c!=self.r{
                let &p = edge[c].last().unwrap();
                if edge[p][0]!=c{
                    for t in i..out[c]{
                        let x = seq[t];
                        state.sub(x, &self.data[x]);
                    }
                }
            }
        }
    }
}
