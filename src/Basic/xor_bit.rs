// [0, r) の半開区間
pub struct BIT<T>
where
    T: Copy
        + std::ops::BitXor<Output = T>
        + std::ops::BitXorAssign
{
    n: usize,
    vec: Vec<T>,
    zero: T,
}

impl<T> BIT<T>
where
    T: Copy
        + std::ops::BitXor<Output = T>
        + std::ops::BitXorAssign
{
    pub fn new(n: usize, zero: T) -> Self {
        let base = vec![zero; n + 1];
        BIT { n, vec: base, zero }
    }

    #[inline]
    pub fn add(&mut self, mut idx: usize, x: T) {
        idx += 1;
        while idx <= self.n {
            self.vec[idx] = self.vec[idx] ^ x;
            idx += idx & (!idx + 1);
        }
    }

    #[inline]
    pub fn g(&self, mut r: usize) -> T {
        let mut res = self.zero;
        while r > 0 {
            res = res ^ self.vec[r];
            r -= r & (!r + 1);
        }
        res
    }

    #[inline]
    pub fn prod(&self, l: usize, r: usize) -> T {
        self.g(r) ^ self.g(l)
    }

    #[inline]
    pub fn get(&self, p: usize) -> T {
        self.g(p + 1) ^ self.g(p)
    }

    #[inline]
    pub fn set(&mut self, p: usize, x: T) {
        let pre = self.get(p);
        self.add(p, x ^ pre);
    }
}
