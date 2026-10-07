pub trait RerootingMonoid{
    type V: Clone;
    type E: Clone;
    fn identity(&self) -> Self::E;
    fn merge(&self, a: &Self::E, b: &Self::E) -> Self::E;
    fn pub_edge(&self, v: &Self::V, idx: usize) -> Self::E;
    fn pub_vertex(&self, e: &Self::E, idx: usize) -> Self::V;
}

pub struct Rerooting<M> where M: RerootingMonoid{
    monoid: M,
    n: usize,
    root: usize,
    edge_id: usize,
    es: Vec<(usize, usize, usize)>,
    st: Vec<usize>,
    out: Vec<M::E>,
}

impl<M> Rerooting<M> where M: RerootingMonoid{
    pub fn new(n: usize, monoid: M)->Self{
        Rerooting{
            monoid,
            n,
            root: 0,
            edge_id: 0,
            es: Vec::with_capacity(2*n-2),
            st: Vec::with_capacity(2*n-2),
            out: Vec::with_capacity(n),
        }
    }

    #[inline]
    pub fn add_edge(&mut self, u: usize, v: usize, idx: usize, r_idx: usize){
        self.st.push(u);
        self.es.push((v, idx, r_idx));
        self.edge_id += 1;
        self.st.push(v);
        self.es.push((u, r_idx, idx));
        self.edge_id += 1;
    }

    pub fn set_root(&mut self, r: usize){
        self.root = r;
    }

    pub fn build(&mut self){
        let mut nes = vec![(0, 0, 0); 2*self.n-2];
        let mut ns = vec![0; self.n+2];
        for &v in &self.st{ ns[v+2] += 1; }
        for i in 0..self.n{ ns[i+1] += ns[i]; }
        for (&v, &x) in self.st.iter().zip(&self.es){
            let p = ns[v+1];
            ns[v+1] += 1;
            nes[p] = x;
        }
        self.es = nes;
        self.st = ns;


        let mut subdp = vec![M::pub_vertex(&self.monoid, &M::identity(&self.monoid), 0); self.n];
        self.out = vec![M::identity(&self.monoid); self.n];
        let mut place = vec![0; self.n+1];
        for i in 0..self.n{
            place[i+1] = self.st[i+1]-self.st[i]-1;
        }
        place[self.root+1] += 1;
        for i in 0..self.n{place[i+1] += place[i];}
        #[inline]
        fn dfs<M: RerootingMonoid>(dp: &mut Rerooting<M>, p: usize, pre: usize, subdp: &mut Vec<M::V>, place: &mut Vec<usize>){
            let mut res = M::identity(&dp.monoid);
            for i in dp.st[p]..dp.st[p+1]{
                if dp.es[i].0 == pre{
                    dp.es.swap(dp.st[p+1]-1, i);
                }
                if dp.es[i].0 == pre{continue;}
                dfs(dp, dp.es[i].0, p, subdp, place);
                let nv = M::pub_edge(&dp.monoid, &subdp[dp.es[i].0], dp.es[i].1);
                dp.out[place[p]] = nv.clone();place[p] += 1;
                res = M::merge(&dp.monoid, &res, &nv);
            }
            subdp[p] = M::pub_vertex(&dp.monoid, &res, p);
        }
        dfs(self, self.root, !0, &mut subdp, &mut place);
    }

    pub fn reroot(&mut self) -> Vec<M::V>{
        let mut rev_edge = vec![M::identity(&self.monoid); self.n];
        let mut res = vec![M::pub_vertex(&self.monoid, &M::identity(&self.monoid), 0); self.n];
        #[inline]
        fn dfs<M: RerootingMonoid>(dp: &mut Rerooting<M>, p: usize, res: &mut Vec<M::V>, rev_edge: &mut Vec<M::E>){
            let lp = dp.calc_start_place(p);
            let rp = dp.calc_start_place(p+1);
            let num = rp-lp;
            let mut ret = vec![M::identity(&dp.monoid); num+1];
            for i in (0..num).rev(){
                ret[i] = M::merge(&dp.monoid, &dp.out[lp+i], &ret[i+1]);
            }
            res[p] = M::pub_vertex(&dp.monoid, &M::merge(&dp.monoid, &ret[0], &rev_edge[p]), p);
            let mut left = M::identity(&dp.monoid);
            for i in 0..num{
                let (nex, _, rdx) = dp.es[dp.st[p]+i];
                let r = M::pub_vertex(&dp.monoid, &M::merge(&dp.monoid, &M::merge(&dp.monoid, &left, &ret[i+1]), &rev_edge[p]), p);
                rev_edge[nex] = M::pub_edge(&dp.monoid, &r, rdx);
                left = M::merge(&dp.monoid, &left, &dp.out[lp+i]);
                dfs(dp, nex, res, rev_edge);
            }
        }
        dfs(self, self.root, &mut res, &mut rev_edge);
        res
    }

    #[inline]
    fn calc_start_place(&self, p: usize)->usize{
        let mut res = self.st[p]-p;
        if self.root < p{
            res += 1;
        }
        return res;
    }
}

struct M{
    vec: Vec<usize>,
}

impl RerootingMonoid for M{
    type V = (usize, usize);
    type E = (usize, usize);

    fn identity(&self) -> Self::E {
        (0, 0)
    }

    #[inline]
    fn merge(&self, &a: &Self::E, &b: &Self::E) -> Self::E {
        (a.0+b.0, a.1+b.1)
    }

    #[inline]
    fn pub_edge(&self, v: &Self::V, _idx: usize) -> Self::E {
        (v.0, v.0+v.1)
    }

    #[inline]
    fn pub_vertex(&self, e: &Self::E, idx: usize) -> Self::V {
        (e.0+self.vec[idx], e.1)
    }
}
