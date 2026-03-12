pub trait SegtreeMonoid{
    type S: Clone;
    fn identity() -> Self::S;
    fn op(&self, a: &Self::S, b: &Self::S) -> Self::S;
}

pub struct Segtree<M: SegtreeMonoid> {
    n: usize,
    data: Vec<M::S>,
    m: M,
}

impl<M: SegtreeMonoid> Segtree<M> {
    pub fn new(n: usize, m: M) -> Self {
        let n = n.next_power_of_two();
        let e = m.identity();
        let data = vec![e; 2 * n];
        Segtree { n, data, m }
    }

    pub fn from(a: Vec<M::S>, m: M) -> Self {
        let n = a.len().next_power_of_two();
        let e = m.identity();
        let mut data = vec![e; 2 * n];

        for (i, v) in a.into_iter().enumerate() {
            data[i + n] = v;
        }
        for i in (1..n).rev() {
            data[i] = m.op(&data[2 * i], &data[2 * i + 1]);
        }
        Segtree { n, data, m }
    }

    pub fn set(&mut self, i: usize, x: M::S) {
        let mut p = i + self.n;
        self.data[p] = x;
        while p > 1 {
            p >>= 1;
            self.data[p] = self.m.op(&self.data[p << 1], &self.data[(p << 1) | 1]);
        }
    }

    pub fn get(&self, p: usize) -> M::S {
        self.data[self.n + p].clone()
    }

    pub fn push(&mut self, i: usize, x: M::S) {
        let mut p = i + self.n;
        self.data[p] = self.m.op(&self.data[p], &x);
        while p > 1 {
            p >>= 1;
            self.data[p] = self.m.op(&self.data[p << 1], &self.data[(p << 1) | 1]);
        }
    }

    pub fn prod(&self, l: usize, r: usize) -> M::S {
        let mut p_l = l + self.n;
        let mut p_r = r + self.n;
        let mut res_l = self.m.identity();
        let mut res_r = self.m.identity();
        while p_l < p_r {
            if p_l & 1 == 1 {
                res_l = self.m.op(&res_l, &self.data[p_l]);
                p_l += 1;
            }
            if p_r & 1 == 1 {
                p_r -= 1;
                res_r = self.m.op(&self.data[p_r], &res_r);
            }
            p_l >>= 1;
            p_r >>= 1;
        }
        self.m.op(&res_l, &res_r)
    }

    pub fn all_prod(&self) -> M::S {
        self.data[1].clone()
    }

    pub fn max_right<F>(&self, mut l: usize, f: F) -> usize
    where
        F: Fn(&M::S) -> bool,
    {
        let e = self.m.identity();
        assert!(f(&e));
        if l == self.n {
            return self.n;
        }
        l += self.n;
        let mut ac = self.m.identity();
        while {
            while l % 2 == 0 {
                l >>= 1;
            }
            if !f(&self.m.op(&ac, &self.data[l])) {
                while l < self.n {
                    l <<= 1;
                    let res = self.m.op(&ac, &self.data[l]);
                    if f(&res) {
                        ac = res;
                        l += 1;
                    }
                }
                return l - self.n;
            }
            ac = self.m.op(&ac, &self.data[l]);
            l += 1;
            let z = l as isize;
            (z & -z) != z
        } {}
        self.n
    }

    pub fn min_left<F>(&self, mut r: usize, f: F) -> usize
    where
        F: Fn(&M::S) -> bool,
    {
        let e = self.m.identity();
        assert!(f(&e));
        if r == 0 {
            return 0;
        }
        r += self.n;
        let mut ac = self.m.identity();
        while {
            r -= 1;
            while r > 1 && r % 2 == 1 {
                r >>= 1;
            }
            if !f(&self.m.op(&self.data[r], &ac)) {
                while r < self.n {
                    r = 2 * r + 1;
                    let res = self.m.op(&self.data[r], &ac);
                    if f(&res) {
                        ac = res;
                        r -= 1;
                    }
                }
                return r + 1 - self.n;
            }
            ac = self.m.op(&self.data[r], &ac);
            let z = r as isize;
            z & -z != z
        } {}
        0
    }
}
