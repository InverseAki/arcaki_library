// [0, r) の半開区間
pub struct BITInplace<T>
where
    T: Copy
        + std::ops::Add<Output = T>
        + std::ops::Sub<Output = T>
        + PartialOrd,
{
    n: usize,
    r: usize,
    vec: Vec<T>,
    zero: T,
}

impl<T> BITInplace<T>
where
    T: Copy
        + std::ops::Add<Output = T>
        + std::ops::Sub<Output = T>
        + PartialOrd,{
    pub fn new(n: usize, zero: T) -> Self {
        let k = n.max(1);
        let base = vec![zero; k + 1];
        BITInplace { n: k, r: k, vec: base, zero }
    }

    pub fn from_vec(a: Vec<T>, zero: T) -> Self {
        let n = a.len();
        let mut vec = vec![zero; n+1];
        for i in 0..n {vec[i+1] = a[i];}
        for i in 1..=n {
            let j = i+(i&(!i+1));
            if j <= n {vec[j] = vec[j]+vec[i];}
        }
        BITInplace { n,r:n,vec,zero}
    }

    #[inline]
    pub fn reset(&mut self) {
        self.vec[1..=self.r].fill(self.zero);
    }

    #[inline]
    pub fn set_r(&mut self, r: usize){
        self.r = r;
    }

    #[inline]
    pub fn init(&mut self, r: usize) {
        self.reset();
        self.set_r(r);
    }

    #[inline]
    pub fn add(&mut self, mut idx: usize, x: T) {
        idx += 1;
        while idx <= self.r {
            self.vec[idx] = self.vec[idx] + x;
            idx += idx & (!idx + 1);
        }
    }

    #[inline]
    pub fn g(&self, mut r: usize) -> T {
        let mut res = self.zero;
        while r > 0 {
            res = res + self.vec[r];
            r -= r & (!r + 1);
        }
        res
    }

    #[inline]
    pub fn prod(&self, l: usize, r: usize) -> T {
        self.g(r) - self.g(l)
    }

    #[inline]
    pub fn get(&self, p: usize) -> T {
        self.g(p + 1) - self.g(p)
    }

    #[inline]
    pub fn set(&mut self, p: usize, x: T) {
        let pre = self.get(p);
        self.add(p, x - pre);
    }

    // Sum(A[0, r)) < ac となる最大の r を返す
    #[inline]
    pub fn lower_bound(&self, ac: T) -> usize {
        if self.n==0{return 0;}
        let mut r = 0;
        let mut cur = self.zero;
        let mut k = 1<<self.n.ilog2();
        while k > 0 {
            let nx = r + k;
            if nx <= self.n {
                let v = cur + self.vec[nx];
                if v < ac {
                    r = nx;
                    cur = v;
                }
            }
            k >>= 1;
        }
        r
    }
}
