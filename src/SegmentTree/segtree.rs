pub trait SegtreeMonoid{
    type S: Clone;
    fn identity() -> Self::S;
    fn op(a: &Self::S, b: &Self::S) -> Self::S;
}

#[derive(Clone)]
pub struct Segtree<M: SegtreeMonoid> {
    n: usize,
    data: Vec<M::S>,
}

impl<M: SegtreeMonoid> Segtree<M> {
    pub fn new(n: usize) -> Self {
        let n = n.next_power_of_two();
        let data = vec![M::identity(); 2 * n];
        Segtree{ n, data }
    }

    pub fn set(&mut self, i: usize, x: M::S) {
        let mut p = i + self.n;
        self.data[p] = x;
        while p > 0 {
            p /= 2;
            self.data[p] = M::op(&self.data[p << 1], &self.data[(p << 1) | 1]);
        }
    }

    pub fn from(a: Vec<M::S>) -> Self{
        let n = a.len().next_power_of_two();
        let mut data = vec![M::identity(); 2*n];
        for (i, v) in a.iter().enumerate(){
            data[i+n] = v.clone();
        }
        for i in (1..n).rev(){
            data[i] = M::op(&data[2*i], &data[2*i+1]);
        }
        Segtree{
            n, data,
        }
    }

    pub fn get(&self, p: usize)->M::S{
        self.data[self.n+p].clone()
    }

    pub fn push(&mut self, i: usize, x: M::S) {
        let mut p = i + self.n;
        self.data[p] = M::op(&self.data[p], &x);
        while p > 0 {
            p /= 2;
            self.data[p] = M::op(&self.data[p << 1], &self.data[(p << 1) | 1]);
        }
    }

    pub fn prod(&self, l: usize, r: usize) -> M::S {
        let mut p_l = l + self.n;
        let mut p_r = r + self.n;
        let mut res_l = M::identity();
        let mut res_r = M::identity();
        while p_l < p_r {
            if p_l & 1 == 1 {
                res_l = M::op(&res_l, &self.data[p_l]);
                p_l += 1;
            }
            if p_r & 1 == 1 {
                p_r -= 1;
                res_r = M::op(&self.data[p_r], &res_r);
            }
            p_l >>= 1;
            p_r >>= 1;
        }
        M::op(&res_l, &res_r)
    }

    pub fn all_prod(&self)-> M::S {
        self.data[1].clone()
    }

    pub fn max_right<F>(&self, mut l: usize, f: F) -> usize where F: Fn(&M::S)->bool {
        assert!(f(&M::identity())); // これはバグってくれないと多分デバックが悲惨
        if l == self.n {
            return self.n 
        }
        l += self.n; 
        let mut ac = M::identity();
        while {
            while l % 2 == 0 {
                l >>= 1;
            }
            if !f(&M::op(&ac, &self.data[l])) {
                while l < self.n {
                    l <<= 1;
                    let res = M::op(&ac, &self.data[l]);
                    if f(&res) {
                        ac = res;
                        l += 1;
                    }
                }
                return l - self.n;
            }
            ac = M::op(&ac, &self.data[l]);
            l += 1;
            let z = l as isize;
            (z & -z) != z
        } {}
        self.n
    }

    pub fn min_left<F>(&self, mut r: usize, f: F) -> usize where F: Fn(&M::S) -> bool {
        assert!(f(&M::identity()));
        if r == 0 {return 0}
        r += self.n;
        let mut ac = M::identity();
        while {
            r -= 1;
            while r > 1 && r % 2 == 1 {
                r >>= 1;
            }
            if !f(&M::op(&self.data[r], &ac)) {
                while r < self.n{
                    r = 2 * r + 1;
                    let res = M::op(&self.data[r], &ac);
                    if f(&res) {
                        ac = res;
                        r -= 1;
                    }
                }
                return r + 1 - self.n;
            }
            ac = M::op(&self.data[r], &ac);
            let z = r as isize;
            z & -z != z
        } {}
        0
    }
}
