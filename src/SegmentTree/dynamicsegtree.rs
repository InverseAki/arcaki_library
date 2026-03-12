pub trait SegtreeMonoid {
    type S: Copy;
    fn identity() -> Self::S;
    fn op(a: &Self::S, b: &Self::S) -> Self::S;
}

pub struct DynamicSegtree<M> where M: SegtreeMonoid {
    n: usize,
    data: Vec<(M::S, Option<usize>, Option<usize>)>,
}

impl<M> DynamicSegtree<M> where M: SegtreeMonoid {
    pub fn new(n: usize) -> Self {
        let n = n.next_power_of_two();
        DynamicSegtree {
            n, data: vec![(M::identity(), None, None)]
        }
    }

    pub fn set(&mut self, p: usize, x: M::S) {
        self.update_dfs(0, 0, self.n, p, x, false);
    }

    pub fn push(&mut self, p: usize, x: M::S) {
        self.update_dfs(0, 0, self.n, p, x, true);
    }

    pub fn get(&self, p: usize)->M::S{
        self.prod(p, p+1)
    }

    fn update_dfs(&mut self, p: usize, l: usize, r: usize, idx: usize, x: M::S, f: bool) -> M::S {
        if l + 1 == r {
            let (pre, left, right) = self.data[p];
            if f {
                self.data[p] = (M::op(&pre, &x), left, right);
            } else {
                self.data[p] = (x, left, right);
            }
            return self.data[p].0;
        }
        let (_, mut left, mut right) = self.data[p];
        let m = (l + r) / 2;
        let res = if idx < m {
            let res_l = if let Some(ln) = left {
                self.update_dfs(ln, l, m, idx, x, f)
            } else {
                let nex = self.data.len();
                left = Some(nex);
                self.data.push((M::identity(), None, None));
                self.update_dfs(nex, l, m, idx, x, f)
            };
            let res_r = if let Some(rn) = right {
                self.data[rn].0
            } else {
                M::identity()
            };
            M::op(&res_l, &res_r)
        } else {
            let res_l = if let Some(ln) = left {
                self.data[ln].0
            } else {
                M::identity()
            };
            let res_r = if let Some(rn) = right {
                self.update_dfs(rn, m, r, idx, x, f)
            } else {
                let nex = self.data.len();
                right = Some(nex);
                self.data.push((M::identity(), None, None));
                self.update_dfs(nex, m, r, idx, x, f)
            };
            M::op(&res_l, &res_r)
        };
        self.data[p] = (res, left, right);
        res
    }

    pub fn prod(&self, l: usize, r: usize) -> M::S {
        self.prod_dfs(0, 0, self.n, l, r)
    }

    fn prod_dfs(&self, p: usize, l: usize, r: usize, x: usize, y: usize) -> M::S {
        if r <= x || y <= l { return M::identity(); }
        let (z, left, right) =self.data[p];
        if x <= l && r <= y {
            return z;
        }
        let m = (l + r) / 2;
        let res_l = if let Some(ln) = left {
            self.prod_dfs(ln, l, m, x, y)
        } else {
            M::identity()
        };
        let res_r = if let Some(rn) = right {
            self.prod_dfs(rn, m, r, x, y)
        } else {
            M::identity()
        };
        M::op(&res_l, &res_r)
    }

    pub fn max_right<F>(&self, l: usize, f: F) -> usize where F: Fn(&M::S) -> bool {
        assert!(f(&M::identity()));
        if l == self.n { return self.n }
        let mut ac = M::identity();
        self.max_right_dfs(0, 0, self.n, l, &mut ac, &f)
    }

    fn max_right_dfs<F>(&self, p: usize, l: usize, r: usize, x: usize, ac: &mut M::S, f: &F) -> usize where F: Fn(&M::S) -> bool {
        if r <= x {
            return x
        }
        if l >= x {
            let res = M::op(ac, &self.data[p].0);
            if f(&res) {
                *ac = res;
                return r;
            } else if r - l == 1 {
                return l;
            }
        }
        let m = (l + r) / 2;
        let (_, left, right) = self.data[p];
        let ret = if let Some(ln) = left {
            self.max_right_dfs(ln, l, m, x, ac, f)
        } else {
            let rr = M::op(ac, &M::identity());
            if f(&rr){
                *ac = rr;
                m
            } else {
                l.max(x)
            }
        };
        if ret < m {
            return ret;
        } else if let Some(rn) = right {
            self.max_right_dfs(rn, m, r, x, ac, f)
        } else {
            r
        }
    }

    pub fn min_left<F>(&self, r: usize, f: F) -> usize where F: Fn(&M::S) -> bool {
        assert!(f(&M::identity()));
        if r == 0 { return 0; }
        let mut ac = M::identity();
        self.min_left_dfs(0, 0, self.n, r, &mut ac, &f)
    }

    fn min_left_dfs<F>(&self, p: usize, l: usize, r: usize, x: usize, ac: &mut M::S, f: &F) -> usize where F: Fn(&M::S) -> bool {
        if x <= l {
            return l;
        } else if r <= x {
            let res = M::op(&self.data[p].0, ac);
            if f(&res) {
                *ac = res;
                return l;
            } else if l + 1 == r {
                return r;
            }
        }
        let m = (l + r) / 2;
        let (_, left, right) = self.data[p];
        let ret = if let Some(rn) = right {
            self.min_left_dfs(rn, m, r, x, ac, f)
        } else {
            let ll = M::op(&ac, &M::identity());
            if f(&ll){
                *ac = ll;
                m
            } else {
                r.min(x)
            }
        };
        if ret > m {
            ret
        } else if let Some(ln) = left {
            self.min_left_dfs(ln, l, m, x, ac, f)
        } else {
            l
        }
    }

    pub fn all_prod(&self) -> M::S {
        self.data[0].0
    }
}
