pub fn bit_length(x: usize)->usize{
    64-x.saturating_sub(1).leading_zeros()as usize
}

pub trait SegtreeMonoid{
    type S: Clone+BeatsFail;
    fn identity()->Self::S;
    fn op(a: &Self::S, b: &Self::S)->Self::S;
}

pub trait LazySegtreeMonoid{
    type M: SegtreeMonoid;
    type F: Clone;
    fn id_e()-><Self::M as SegtreeMonoid>::S{<Self::M as SegtreeMonoid>::identity()}
    fn op(a: &<Self::M as SegtreeMonoid>::S, b: &<Self::M as SegtreeMonoid>::S)-><Self::M as SegtreeMonoid>::S{<Self::M>::op(a, b)}
    fn identity()->Self::F;
    fn map(f: &Self::F, x: &<Self::M as SegtreeMonoid>::S)-><Self::M as SegtreeMonoid>::S;
    fn composition(f: &Self::F, g: &Self::F)->Self::F;
}

pub trait BeatsFail{
    fn fail(&self) -> bool;
}

pub struct LazySegtree<F> where F: LazySegtreeMonoid{
    n: usize,
    log: usize,
    data: Vec<<F::M as SegtreeMonoid>::S>,
    lazy: Vec<F::F>,
}

impl<F: LazySegtreeMonoid> LazySegtree<F>{
    pub fn new(n: usize)->Self{
        let n = n.next_power_of_two();
        let log = bit_length(n);
        let lazy = vec![F::identity(); n<<1];
        let data = vec![F::id_e(); n<<1];
        LazySegtree{
            n, log, data, lazy,
        }
    }

    pub fn build(vec: &Vec<<F::M as SegtreeMonoid>::S>)->Self {
        let n = vec.len().next_power_of_two();
        let log = bit_length(n);
        let lazy = vec![F::identity(); n<<1];
        let mut data = vec![F::id_e(); n<<1];
        data[n..(n+vec.len())].clone_from_slice(vec);
        let mut res = LazySegtree{
            n, log, data, lazy,
        };
        for i in (1..n).rev(){
            res.update(i);
        }
        res
    }

    pub fn set(&mut self, mut p: usize, x: <F::M as SegtreeMonoid>::S){
        p += self.n;
        for i in (1..=self.log).rev(){
            self.push(p>>i);
        }
        self.data[p] = x;
        for i in 1..=self.log{
            self.update(p>>i);
        }
    }

    #[inline(always)]
    fn update(&mut self, k: usize){
        self.data[k] = F::op(&self.data[2*k], &self.data[2*k+1]);
    }

    #[inline(always)]
    fn inner_apply(&mut self, k: usize, f: F::F){
        self.data[k] = F::map(&f, &self.data[k]);
        if k < self.n{
            self.lazy[k] = F::composition(&f, &self.lazy[k]);
            if self.data[k].fail(){
                self.push(k); self.update(k);
            }
        }
    }

    #[inline(always)]
    fn push(&mut self, k: usize){
        self.inner_apply(2*k, self.lazy[k].clone());
        self.inner_apply(2*k+1, self.lazy[k].clone());
        self.lazy[k] = F::identity();
    }

    pub fn get(&mut self, mut p: usize)-><F::M as SegtreeMonoid>::S{
        p += self.n;
        for i in (1..self.log).rev(){
            self.push(p>>i);
        }
        self.data[p].clone()
    }

    #[inline]
    pub fn prod(&mut self, mut l: usize, mut r: usize)-><F::M as SegtreeMonoid>::S{
        if r<=l{return F::id_e()}
        l += self.n; r += self.n;
        for i in (1..=self.log).rev(){
            if ((l>>i)<<i) != l{
                self.push(l>>i);
            }
            if ((r>>i)<<i) != r{
                self.push(r>>i);
            }
        }
        let mut acl = F::id_e();
        let mut acr = F::id_e();
        while l < r{
            if l&1 != 0{
                acl = F::op(&acl, &self.data[l]);
                l += 1;
            }
            if r&1 != 0{
                r -= 1;
                acr = F::op(&self.data[r], &acr);
            }
            l >>= 1; r >>= 1;
        }
        F::op(&acl, &acr)
    }

    pub fn all_prod(&mut self)-><F::M as SegtreeMonoid>::S{
        self.update(1);
        self.data[1].clone()
    }

    pub fn apply_range(&mut self, mut l: usize, mut r: usize, f: F::F){
        if l>=r{return;}
        l += self.n; r += self.n;
        for i in (1..=self.log).rev(){
            if ((l>>i)<<i)!=l{
                self.push(l>>i);
            }
            if ((r>>i)<<i)!=r{
                self.push((r-1)>>i);
            }
        }
        let left = l;
        let right = r;
        while l < r{
            if l&1!=0{
                self.inner_apply(l, f.clone());
                l += 1;
            }
            if r&1!=0{
                r -= 1;
                self.inner_apply(r, f.clone());
            }
            l >>= 1; r>>=1;
        }
        for i in 1..=self.log{
            if ((left>>i)<<i)!=left{
                self.update(left>>i);
            }
            if ((right>>i)<<i)!=right{
                self.update((right-1)>>i);
            }
        }
    }

    pub fn max_right<G>(&mut self, mut l: usize, g: G)->usize
    where G: Fn(<F::M as SegtreeMonoid>::S)->bool{
        assert!(g(F::id_e()));
        if l >= self.n{return self.n}
        l += self.n;
        for i in 1..=self.log{
            self.push(l>>i);
        }
        let mut ac = F::id_e();
        while {
            while l%2==0{
                l>>=1;
            }
            if !g(F::op(&ac, &self.data[l])){
                while l < self.n{
                    self.push(l);
                    l *= 2;
                    let res = F::op(&ac, &self.data[l]);
                    if g(res.clone()){
                        ac = res;
                        l += 1;
                    }
                }
                return l-self.n;
            }
            ac = F::op(&ac, &self.data[l]);
            l += 1;
            let left = l as isize;
            (left&-left)!=left
        } {}
        self.n
    }

    pub fn min_left<G>(&mut self, mut r: usize, g: G)->usize
    where G: Fn(<F::M as SegtreeMonoid>::S)->bool{
        assert!(g(F::id_e()));
        if r==0{return 0;}
        r += self.n;
        for i in (1..=self.log).rev(){
            self.push((r-1)>>i);
        }
        let mut ac = F::id_e();
        while {
            r -= 1;
            while r%2 != 0{
                r >>= 1;
            }
            if !g(F::op(&self.data[r], &ac)){
                while r < self.n{
                    self.push(r);
                    r = 2*r+1;
                    let res = F::op(&self.data[r], &ac);
                    if g(res.clone()){
                        ac = res;
                        r -= 1;
                    }
                }
                return r+1-self.n;
            }
            ac = F::op(&self.data[r], &ac);
            let right = r as isize;
            (right&-right)!=right
        } {}
        0
    }

    pub fn get_slice(&mut self, mut l: usize, mut r: usize)->Vec<<F::M as SegtreeMonoid>::S>{
        l += self.n; r += self.n;
        for i in 1..self.n {
            self.push(i)
        }
        (l..r).into_iter().map(|z| self.data[z].clone()).collect()
    }
}

const INF: i64 = 1<<60;

#[derive(Clone, Copy, Debug)]
struct S{
    num: i64,
    ac: i64,
    mx: i64,
    smx: i64,
    mi: i64,
    smi: i64,
    mx_c: i64,
    mi_c: i64,
    fail: bool,
}

impl S{
    #[inline(always)]
    fn new(x: i64, n: i64)->Self{
        S{
            num: n,
            ac: n*x,
            mx: x,
            mi: x,
            smx: -INF,
            smi: INF,
            mx_c: n,
            mi_c: n,
            fail: false,
        }
    }
}

impl BeatsFail for S{
    #[inline(always)]
    fn fail(&self) -> bool {
        self.fail
    }
}

struct M;
impl SegtreeMonoid for M{
    type S = S;

    fn identity() -> Self::S {
        S{
            num: 0,
            ac: 0,
            mx: -INF,
            smx: -INF,
            mi: INF,
            smi: INF,
            mx_c: 0,
            mi_c: 0,
            fail: false,
        }
    }

    #[inline(always)]
    fn op(&a: &Self::S, &b: &Self::S) -> Self::S {
        if a.num==0{return b}
        if b.num==0{return a}
        let (mx, mx_c, smx) = if a.mx > b.mx{
            (a.mx, a.mx_c, a.smx.max(b.mx))
        } else if a.mx == b.mx{
            (a.mx, a.mx_c+b.mx_c, a.smx.max(b.smx))
        } else {
            (b.mx, b.mx_c, a.mx.max(b.smx))
        };
        let (mi, mi_c, smi) = if a.mi < b.mi{
            (a.mi, a.mi_c, a.smi.min(b.mi))
        } else if a.mi == b.mi{
            (a.mi, a.mi_c+b.mi_c, a.smi.min(b.smi))
        } else {
            (b.mi, b.mi_c, a.mi.min(b.smi))
        };
        S{
            num: a.num+b.num,
            ac: a.ac+b.ac,
            mx,
            mx_c,
            smx,
            mi,
            mi_c,
            smi,
            fail: false,
        }
    }
}

struct MM;
impl LazySegtreeMonoid for MM{
    type M = M;
    type F = (i64, i64, i64);

    fn identity() -> Self::F {
        (INF, -INF, 0)
    }

    #[inline(always)]
    fn map(&f: &Self::F, &x: &<Self::M as SegtreeMonoid>::S) -> <Self::M as SegtreeMonoid>::S {
        if x.num == 0 { return x; } else if x.mx == x.mi || f.0 == f.1 || f.0 <= x.mi || f.1 >= x.mx{
            return S::new(x.mi.max(f.1).min(f.0)+f.2, x.num);
        } else if x.smi==x.mx{
            let mut res = x;
            res.mi = x.mi.max(f.1)+f.2;
            res.smx = res.mi;
            res.mx = x.mx.min(f.0)+f.2;
            res.smi = res.mx;
            res.ac = res.mi*res.mi_c+res.mx*res.mx_c;
            return res;
        } else if f.1 < x.smi && f.0 > x.smx{
            let mut res = x;
            let nl = res.mi.max(f.1);
            let nh = res.mx.min(f.0);
            res.ac += (nl-res.mi)*res.mi_c-(res.mx-nh)*res.mx_c+f.2*res.num;
            res.mi = nl+f.2;
            res.mx = nh+f.2;
            res.smx += f.2;
            res.smi += f.2;
            return res;
        }
        let mut res = x;
        res.fail = true;
        res
    }

    #[inline(always)]
    fn composition(&f: &Self::F, &g: &Self::F) -> Self::F {
        let t = (g.1+g.2).min(f.0).max(f.1)-g.2;
        let b = (g.0+g.2).max(f.1).min(f.0)-g.2;
        let add = f.2+g.2;
        (b, t, add)
    }
}
