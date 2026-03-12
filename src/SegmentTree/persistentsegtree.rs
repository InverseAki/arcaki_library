pub trait SegtreeMonoid {
    type S: Clone;
    fn identity() -> Self::S;
    fn op(a: &Self::S, b: &Self::S) -> Self::S;
}

#[derive(Clone, Debug)]
pub struct SegtreeNode<S: Clone>{
    val: S,
    left: u32,
    right: u32,
}

#[derive(Debug)]
pub struct PersistentSegtree<M: SegtreeMonoid>{
    n: usize,
    data: Vec<SegtreeNode<M::S>>,
    root: Vec<u32>,
}

impl<M: SegtreeMonoid> PersistentSegtree<M> {
    pub fn new(mut n: usize) -> Self {
        n = n.next_power_of_two();
        let data = Vec::with_capacity(2*n);
        let mut sg = Self {
            n, data, root: Vec::new(),
        };
        let r = sg.init(0, n);
        sg.root.push(r as u32);
        sg
    }

        pub fn new_with_q(mut n: usize, q: usize) -> Self {
        n = n.next_power_of_two();
        let data = Vec::with_capacity(2*n+q*20);
        let mut sg = Self {
            n, data, root: Vec::new(),
        };
        let r = sg.init(0, n);
        sg.root.push(r as u32);
        sg
    }

    pub fn build(a: &[M::S]) -> Self {
        let n = a.len().next_power_of_two();
        let data = Vec::with_capacity(2*n);
        let mut sg = Self {
            n, data, root: Vec::new(),
        };
        let r = sg.init_s(a, 0, n);
        sg.root.push(r as u32);
        sg
    }

    #[inline(always)]
    fn push_node(&mut self, node: SegtreeNode<M::S>)->usize{
        let r = self.data.len();
        self.data.push(node);
        r
    }

    #[inline(always)]
    fn init(&mut self, l: usize, r: usize)->usize{
        if l+1==r{
            return self.push_node(SegtreeNode { val: M::identity(), left: !0, right: !0 });
        }
        let m = (l+r)>>1;
        let left = self.init(l, m);
        let right = self.init(m, r);
        let val = M::op(&self.data[left].val, &self.data[right].val);
        self.push_node(SegtreeNode { val, left: left as u32, right: right as u32 })
    }

    #[inline(always)]
    fn init_s(&mut self, a: &[M::S], l: usize, r: usize)->usize{
        if l+1==r{
            return self.push_node(SegtreeNode { val: if l < a.len(){a[l].clone()}else{M::identity()}, left: !0, right: !0 });
        }
        let m = (l+r)>>1;
        let left = self.init_s(a, l, m);
        let right = self.init_s(a, m, r);
        let val = M::op(&self.data[left].val, &self.data[right].val);
        self.push_node(SegtreeNode { val, left: left as u32, right: right as u32 })
    }

    #[inline]
    pub fn versions(&self) -> usize {
        self.root.len()
    }

    #[inline]
    pub fn update(&mut self, t: usize, p: usize, x: M::S){
        let nr = self.update_dfs(self.root[t] as usize, 0, self.n, p, &x);
        self.root.push(nr as u32);
    }

    #[inline(always)]
    fn update_dfs(&mut self, cur: usize, l: usize, r: usize, p: usize, x: &M::S)->usize{
        if l+1==r{
            return self.push_node(SegtreeNode { val: x.clone(), left: !0, right: !0 });
        }
        let m = (l+r)>>1;
        let pre = &self.data[cur];
        let (cl, cr) = (pre.left, pre.right);
        let (nl, nr) = if p < m{
            let nl = self.update_dfs(cl as usize, l, m, p, x) as u32;
            (nl, cr)
        } else {
            let nr = self.update_dfs(cr as usize, m, r, p, x)as u32;
            (cl, nr)
        };
        self.push_node(SegtreeNode { val: M::op(&self.data[nl as usize].val, &self.data[nr as usize].val), left: nl, right: nr })
    }

    #[inline]
    pub fn prod(&self, t: usize, l: usize, r: usize) -> M::S {
        self.prod_dfs(self.root[t]as usize, 0, self.n, l, r)
    }

    #[inline(always)]
    fn prod_dfs(&self, cur: usize, cl: usize, cr: usize, l: usize, r: usize) -> M::S {
        if r <= cl || cr <= l{
            return M::identity();
        } else if l <= cl && cr <= r {
            return self.data[cur].val.clone();
        }
        let m = (cl+cr)/2;
        let node = &self.data[cur];
        let ln = self.prod_dfs(node.left as usize, cl, m, l, r);
        let rn = self.prod_dfs(node.right as usize, m, cr, l, r);
        M::op(&ln, &rn)
    }

    #[inline]
    pub fn min_left<F>(&self, t: usize, r: usize, f: F) -> usize where F: Fn(&M::S)->bool{
        assert!(f(&M::identity()));
        if r==0{return 0;}
        let mut ac = M::identity();
        self.min_left_dfs(self.root[t] as usize, 0, self.n, r, &mut ac, &f)
    }

    #[inline]
    pub fn max_right<F>(&self, t: usize, l: usize, f: F) -> usize where F: Fn(&M::S)->bool{
        assert!(f(&M::identity()));
        if l==self.n{return self.n;}
        let mut ac = M::identity();
        self.max_right_dfs(self.root[t] as usize, 0, self.n, l, &mut ac, &f)
    }

    #[inline(always)]
    fn min_left_dfs<F>(&self, cur: usize, l: usize, r: usize, x: usize, ac: &mut M::S, f: &F) -> usize where F: Fn(&M::S)->bool{
        if x <= l {return l;}
        if r <= x{
            let m = M::op(&self.data[cur].val, ac);
            if f(&m){
                *ac = m;
                return l;
            } else if r-l==1{
                return r;
            }
        }
        let m = (l+r)>>1;
        let node = &self.data[cur];
        let ret = self.min_left_dfs(node.right as usize, m, r, x, ac, f);
        if ret > m{
            return ret;
        }
        self.min_left_dfs(node.left as usize, l, m, x, ac, f)
    }

    #[inline(always)]
    fn max_right_dfs<F>(&self, cur: usize, l: usize, r: usize, x: usize, ac: &mut M::S, f: &F) -> usize where F: Fn(&M::S)->bool{
        if r <= x{return x;}
        if x <= l{
            let m = M::op(ac, &self.data[cur].val);
            if f(&m){
                *ac = m;
                return r;
            }
            if l+1==r{
                return l;
            }
        }
        let m = (l+r)>>1;
        let node = &self.data[cur];
        let (ln, rn) = (node.left, node.right);
        let ret = self.max_right_dfs(ln as usize, l, m, x, ac, f);
        if ret < m{
            return ret;
        }
        self.max_right_dfs(rn as usize, m, r, x, ac, f)
    }

    #[inline(always)]
    pub fn get(&self, t: usize, p: usize) -> M::S {
        self.prod(t, p, p+1)
    }
}
