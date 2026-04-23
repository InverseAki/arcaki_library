pub trait RangeSetSegtreeMonoid {
    type S: Clone;
    fn identity()->Self::S;
    fn op(a: &Self::S, b: &Self::S)->Self::S;
}

pub struct RangeSetSegtree<M> where M: RangeSetSegtreeMonoid {
    n: usize,
    log: usize,
    data: Vec<M::S>,
    lazy: Vec<M::S>,
    point: Vec<usize>,
}

impl<M> RangeSetSegtree<M> where M: RangeSetSegtreeMonoid{
    pub fn new(n: usize)->Self{
        let n = n.next_power_of_two();
        let log = n.trailing_zeros()as usize;
        let mut data = vec![M::identity(); 2*n];
        for i in (1..n).rev(){
            data[i] = M::op(&data[i<<1], &data[(i<<1)|1]);
        }
        let point = vec![!0; 2*n];
        RangeSetSegtree { n, log, data, lazy: Vec::new(), point}
    }

    pub fn build(vec: Vec<M::S>)->Self{
        let n = vec.len().next_power_of_two();
        let log = n.trailing_zeros()as usize;
        let mut data = vec![M::identity(); 2*n];
        for (i, x) in vec.iter().enumerate(){
            data[i+n] = x.clone();
        }
        for i in (1..n).rev(){
            data[i] = M::op(&data[i<<1], &data[(i<<1)|1]);
        }
        let point = vec![!0; 2*n];
        RangeSetSegtree { n, log, data, lazy: Vec::new(), point}
    }

    #[inline]
    fn update(&mut self, p: usize){
        self.data[p] = M::op(&self.data[p<<1], &self.data[(p<<1)|1]);
    }

    #[inline]
    fn inner_apply(&mut self, p: usize, f: usize){
        if f < self.lazy.len(){
            self.data[p] = self.lazy[f].clone();
        } else {return;}
        if p < self.n{
            self.point[p] = f-1;
        }
    }

    #[inline]
    fn push(&mut self, p: usize){
        if self.point[p]==!0{return;}
        self.inner_apply(p<<1, self.point[p]);
        self.inner_apply((p<<1)|1, self.point[p]);
        self.point[p] = !0;
    }

    pub fn get(&mut self, mut p: usize)-> M::S{
        p += self.n;
        for i in (1..=self.log).rev(){
            self.push(p>>i);
        }
        self.data[p].clone()
    }

    pub fn rangeset(&mut self, mut l: usize, mut r: usize, mut x: M::S){
        let mut p = self.lazy.len();
        self.lazy.push(x.clone());
        l += self.n;
        r += self.n;
        for b in (1..=self.log).rev(){
            if ((l>>b)<<b) != l{
                self.push(l>>b);
            }
            if ((r>>b)<<b) != r{
                self.push((r-1)>>b);
            }
        }
        let left = l;
        let right = r;
        while l < r{
            if l&1==1 {
                self.inner_apply(l, p);
                l += 1;
            }
            if r&1==1 {
                r -= 1;
                self.inner_apply(r, p);
            }
            x = M::op(&x, &x);
            self.lazy.push(x.clone());
            p += 1;
            l >>= 1;
            r >>= 1;
        }
        l = left;
        r = right;
        for i in 1..=self.log {
            if ((l >> i) << i) != l {
                self.update(l>>i);
            }
            if ((r >> i) << i) != r {
                self.update((r-1)>>i);
            }
        }
    }

    pub fn prod(&mut self, mut l: usize, mut r: usize)->M::S{
        l += self.n; r += self.n;
        for b in (1..=self.log).rev(){
            if ((l>>b)<<b) != l{
                self.push(l>>b);
            }
            if ((r>>b)<<b) != r{
                self.push((r-1)>>b);
            }
        }
        let (mut res_l, mut res_r) = (M::identity(), M::identity());
        while l < r{
            if l&1 == 1 {
                res_l = M::op(&res_l, &self.data[l]);
                l += 1;
            }
            if r&1 == 1{
                r -= 1;
                res_r = M::op(&self.data[r], &res_r);
            }
            l >>= 1;
            r >>= 1;
        }
        M::op(&res_l, &res_r)
    }
}
