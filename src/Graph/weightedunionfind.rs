pub trait UFMonoid{
    type S: Clone+PartialEq;
    fn identity()->Self::S;
    fn op(a: &Self::S, b: &Self::S)->Self::S;
    fn inv(x: &Self::S)->Self::S;
}

pub struct WeightedUnionFind<M> where M:UFMonoid{
    parent: Vec<i32>,
    data: Vec<M::S>,
}

impl<M> WeightedUnionFind<M> where M:UFMonoid{
    pub fn new(n: usize)->Self{
        WeightedUnionFind{
            parent: vec![-1; n],
            data: vec![M::identity(); n],
        }
    }

    #[inline]
    pub fn dist(&mut self, p: usize)->M::S{
        self.compress(p);
        self.data[p].clone()
    }

    #[inline(always)]
    pub fn find(&self, mut p: usize) -> (usize, M::S){
        let mut w = self.data[p].clone();
        while self.parent[p] >= 0{
            p = self.parent[p] as usize;
            w = M::op(&self.data[p], &w);
        }
        (p, w)
    }

    #[inline]
    pub fn compress(&mut self, p: usize) -> usize{
        if self.parent[p] < 0{
            return p;
        }
        let pre = self.parent[p]as usize;
        let res = self.compress(pre);
        self.data[p] = M::op(&self.data[pre], &self.data[p]);
        self.parent[p] = res as i32;
        res
    }

    #[inline(always)]
    pub fn union(&mut self, u: usize, v: usize, w: M::S)->bool{
        let ((mut pu, wu), (mut pv, wv)) = (self.find(u), self.find(v));
        if pu==pv{
            return wv==M::op(&wu, &w)
        }
        let mut nex = M::op(&M::op(&wu, &w), &M::inv(&wv));
        if self.parent[pu] > self.parent[pv]{
            swap(&mut pu, &mut pv);
            nex = M::inv(&nex);
        }
        self.parent[pu] += self.parent[pv];
        self.parent[pv] = pu as i32;
        self.data[pv] = nex;
        true
    }

    #[inline(always)]
    pub fn union_c(&mut self, u: usize, v: usize, w: M::S)->bool{
        let (mut pu, mut pv) = (self.compress(u), self.compress(v));
        if pu==pv{
            return self.data[v]==M::op(&self.data[u], &w)
        }
        if self.parent[pu] > self.parent[pv] {
            swap(&mut pu, &mut pv);
        }
        let mut nex = M::op(&M::op(&self.dist(u), &w), &M::inv(&self.dist(v)));
        if self.parent[pu] > self.parent[pv]{
            swap(&mut pu, &mut pv);
            nex = M::inv(&nex);
        }
        self.parent[pu] += self.parent[pv];
        self.parent[pv] = pu as i32;
        self.data[pv] = nex;
        true
    }

    #[inline(always)]
    pub fn same_c(&mut self, u: usize, v: usize)->bool{
        self.compress(u)==self.compress(v)
    }

    #[inline(always)]
    pub fn same(&self, u: usize, v: usize)->bool{
        self.find(u).0==self.find(v).0
    }

    pub fn size_c(&mut self, p: usize)->usize{
        let pre = self.compress(p);
        (-self.parent[pre]) as usize
    }

    pub fn size(&self, p: usize)->usize{
        (-self.parent[self.find(p).0]) as usize
    }

    #[inline(always)]
    pub fn diff(&mut self, u: usize, v: usize)->Option<M::S>{
        let (pu, wu) = self.find(u);
        let (pv, wv) = self.find(v);
        if pu==pv{
            Some(M::op(&M::inv(&wu), &wv))
        } else {
            None
        }
    }
}
