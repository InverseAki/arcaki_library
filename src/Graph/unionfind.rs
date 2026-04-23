#[derive(Clone)]
pub struct UnionFind{
    parent: Vec<i32>,
}

impl UnionFind{
    pub fn new(n: usize)->Self{
        UnionFind{
            parent: vec![-1; n],
        }
    }

    pub fn find(&mut self, r: usize)->usize{
        if self.parent[r] < 0{return r;}
        let p = self.find(self.parent[r] as usize);
        self.parent[r] = p as i32;
        p
    }

    pub fn union(&mut self, u: usize, v: usize){
        let (mut pu, mut pv) = (self.find(u), self.find(v));
        if pu==pv{return;}
        if self.parent[pu] > self.parent[pv]{
            swap(&mut pu, &mut pv);
        }
        self.parent[pu] += self.parent[pv];
        self.parent[pv] = pu as i32;
    }

    pub fn same(&mut self, u: usize, v: usize)->bool{
        self.find(u)==self.find(v)
    }

    pub fn size(&mut self, p: usize)->usize{
        let r = self.find(p);
        (-self.parent[r]) as usize
    }
}
