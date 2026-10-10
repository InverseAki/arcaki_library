pub struct BITConcatenated<T>
where
    T: Copy
        + std::ops::Add<Output = T>
        + std::ops::Sub<Output = T>
        + PartialOrd,
{
    n: usize,
    ac: Vec<usize>,
    data: Vec<T>,
    zero: T,
}

impl<T> BITConcatenated<T>
where
    T: Copy
        + std::ops::Add<Output = T>
        + std::ops::Sub<Output = T>
        + PartialOrd,
{
    pub fn new(num: Vec<usize>, zero: T) -> Self {
        let n = num.len();
        let mut ac = vec![0; n + 1];
        for (i, &l) in num.iter().enumerate() {ac[i + 1] = ac[i] + l;}
        let l = ac[n];
        Self {
            n,
            ac,
            data: vec![zero; l + 1],
            zero,
        }
    }

    pub fn build(num: Vec<usize>, data: Vec<T>, zero: T) -> Self {
        let n = num.len();
        let mut ac = vec![0usize; n + 1];
        for (i, &x) in num.iter().enumerate() {
            ac[i + 1] = ac[i] + x;
        }
        let l = ac[n];
        assert_eq!(l, data.len());
        let mut bit_data = vec![zero; l + 1];
        for i in 0..l {
            bit_data[i + 1] = data[i];
        }
        for k in 0..n {
            let base = ac[k];
            let len = ac[k + 1] - base;
            for i in 1..=len {
                let j = i + (i & (!i + 1));
                if j <= len {
                    bit_data[base + j] = bit_data[base + j] + bit_data[base + i];
                }
            }
        }
        Self {
            n,ac,data: bit_data,zero,
        }
    }

    #[inline]
    pub fn len(&self, k: usize) -> usize {
        debug_assert!(k < self.n);
        self.ac[k + 1] - self.ac[k]
    }

    #[inline]
    pub fn add(&mut self, k: usize, mut idx: usize, x: T) {
        debug_assert!(k < self.n);
        let base = self.ac[k];
        let len = self.ac[k + 1] - base;
        debug_assert!(idx < len);
        idx += 1;
        while idx <= len {
            self.data[base + idx] = self.data[base + idx] + x;
            idx += idx & (!idx + 1);
        }
    }

    #[inline]
    pub fn g(&self, k: usize, mut r: usize) -> T {
        debug_assert!(k < self.n);
        let base = self.ac[k];
        let len = self.ac[k + 1] - base;
        debug_assert!(r <= len);
        let mut res = self.zero;
        while r > 0 {
            res = res + self.data[base + r];
            r -= r & (!r + 1);
        }

        res
    }

    #[inline]
    pub fn prod(&self, k: usize, l: usize, r: usize) -> T {
        self.g(k, r) - self.g(k, l)
    }

    #[inline]
    pub fn get(&self, k: usize, p: usize) -> T {
        self.g(k, p + 1) - self.g(k, p)
    }

    #[inline]
    pub fn set(&mut self, k: usize, p: usize, x: T) {
        let pre = self.get(k, p);
        self.add(k, p, x - pre);
    }

    #[inline]
    pub fn lower_bound(&self, k: usize, x: T) -> usize {
        debug_assert!(k < self.n);
        let base = self.ac[k];
        let len = self.ac[k + 1] - base;
        if len == 0 {
            return 0;
        }
        let mut r = 0;
        let mut cur = self.zero;
        let mut step = 1usize << len.ilog2();
        while step > 0 {
            let nx = r + step;
            if nx <= len {
                let v = cur + self.data[base + nx];
                if v < x {
                    r = nx;
                    cur = v;
                }
            }
            step >>= 1;
        }
        r
    }
}
