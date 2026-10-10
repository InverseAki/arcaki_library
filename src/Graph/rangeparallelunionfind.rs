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

    pub fn union(&mut self, u: usize, v: usize)->(usize, usize){
        let (mut pu, mut pv) = (self.find(u), self.find(v));
        if pu==pv{return (!0,!0);}
        if self.parent[pu] > self.parent[pv]{
            swap(&mut pu, &mut pv);
        }
        self.parent[pu] += self.parent[pv];
        self.parent[pv] = pu as i32;
        (pu,pv)
    }

    pub fn same(&mut self, u: usize, v: usize)->bool{
        self.find(u)==self.find(v)
    }

    pub fn size(&mut self, p: usize)->usize{
        let r = self.find(p);
        (-self.parent[r]) as usize
    }
}

pub struct RangeParallelUnionFind{
    _n: usize,
    uf: Vec<UnionFind>,
}

impl RangeParallelUnionFind {
    pub fn new(n: usize)->Self{
        let k = (n+1).next_power_of_two().trailing_zeros()as usize;
        RangeParallelUnionFind { _n: n, uf: vec![UnionFind::new(n);k]}
    }

    pub fn size(&mut self, p: usize)->usize{
        self.uf[0].size(p)
    }

    pub fn same(&mut self, u: usize, v: usize)->bool{
        self.uf[0].same(u, v)
    }

    pub fn union<F>(&mut self, k: usize, mut a: usize, mut b: usize, mut f: F) where F: FnMut(usize, usize){
        if k==0||a==b{return;}
        if a > b{swap(&mut a, &mut b);}
        let mut stack = Vec::with_capacity(20);
        let x = (k+1).next_power_of_two().trailing_zeros()as usize-1;
        stack.push((x, a, b));
        stack.push((x, a+k-(1<<x), b+k-(1<<x)));
        while let Some((mut x, u, v)) = stack.pop(){
            let (d, e) = self.uf[x].union(u, v);
            if d != !0{
                if x==0 {
                    f(d, e);
                } else {
                    x -= 1;
                    stack.push((x, u, v));
                    stack.push((x, u+(1<<x), v+(1<<x)));
                }
            }
        }
    }
}
