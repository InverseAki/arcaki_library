pub trait UFMonoid{
    type S: Clone+PartialEq;
    fn identity(&self, )->Self::S;
    fn op(&self, a: &Self::S, b: &Self::S)->Self::S;
    fn inv(&self, x: &Self::S)->Self::S;
}

pub struct RollbackWeightedUnionFind<M: UFMonoid> {
    monoid: M,
    parent: Vec<i32>,
    data: Vec<M::S>,
    hist: Vec<(usize, usize, i32, i32, M::S, M::S)>,
}

impl<M> RollbackWeightedUnionFind<M> where M: UFMonoid {
    pub fn new(n: usize, monoid: M)->Self {
        let parent = vec![-1;n];
        let data = (0..n).into_iter().map(|_| monoid.identity()).collect::<Vec<_>>();
        RollbackWeightedUnionFind { monoid, parent, data, hist: Vec::new(), }
    }

    #[inline]
    pub fn find(&self, mut p: usize)->(usize, M::S) {
        let mut w = self.data[p].clone();
        while self.parent[p]>=0{
            p=self.parent[p]as usize;
            w=self.monoid.op(&self.data[p],&w);
        }
        (p,w)
    }

    #[inline]
    pub fn dist(&self, p: usize)->M::S {
        self.find(p).1
    }

    #[inline]
    pub fn leader(&self, p: usize)->usize{
        self.find(p).0
    }

    #[inline]
    pub fn union(&mut self, u: usize, v: usize, w: M::S)->bool {
        let ((mut pu,wu), (mut pv, wv)) = (self.find(u),self.find(v));
        if pu==pv {
            self.hist.push((pu,pv,self.parent[pu],self.parent[pv],self.data[pu].clone(),self.data[pv].clone()));
            return wv==self.monoid.op(&wu, &w);
        } 
        let mut nex = self.monoid.op(&self.monoid.op(&wu,&w),&self.monoid.inv(&wv));
        if self.parent[pu]>self.parent[pv]{
            swap(&mut pu, &mut pv);nex = self.monoid.inv(&nex);
        }
        self.hist.push((pu, pv,self.parent[pu],self.parent[pv],self.data[pu].clone(),self.data[pv].clone()));
        self.parent[pu]+=self.parent[pv];
        self.parent[pv]=pu as i32;
        self.data[pv]=nex;
        true
    }

    #[inline]
    pub fn size(&self, p: usize)->usize {
        let p = self.find(p).0;
        (-self.parent[p])as usize
    } 

    #[inline]
    pub fn same(&self, u: usize, v: usize)->bool {
        self.find(u).0==self.find(v).0
    } 

    #[inline]
    pub fn diff(&self, u: usize, v: usize)->Option<M::S> {
        let (pu, wu) = self.find(u);
        let (pv, wv) = self.find(v);
        if pu==pv{Some(self.monoid.op(&self.monoid.inv(&wu),&wv))}
        else {None}
    }

    pub fn rollback(&mut self){
        if let Some((u, v, a, b, c, d)) = self.hist.pop(){
            self.parent[u]=a;self.parent[v]=b;
            self.data[u]=c;self.data[v]=d;
        }
    }
}
