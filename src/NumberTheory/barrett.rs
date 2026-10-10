#[derive(Clone, Copy)]
pub struct Barrett32 {
    m: u64,
    inv: u64,
}
impl Barrett32 {
    pub fn new(m: u32) -> Self {
        assert!(m >= 2);
        Self {
            m: m as u64,
            inv: ((1u128 << 64) / m as u128) as u64,
        }
    }
    pub fn modulus(&self) -> u32 {
        self.m as u32
    }
    #[inline]
    pub fn mul(&self, a: u32, b: u32) -> u32 {
        let x = a as u64 * b as u64;
        let q = ((x as u128 * self.inv as u128) >> 64) as u64;
        let mut r = x - q * self.m;
        if r >= self.m {
            r -= self.m;
        }
        r as u32
    }
    #[inline]
    pub fn pow(&self, a: u32, exp: u64) -> u32 {
        self.mod_pow(a, exp)
    }
    pub fn mod_pow(&self, a: u32, mut exp: u64) -> u32 {
        let mut a = a % self.modulus();
        let mut ans = 1;
        while exp > 0 {
            if exp & 1 != 0 {
                ans = self.mul(ans, a);
            }
            a = self.mul(a, a);
            exp >>= 1;
        }
        ans
    }
    pub fn try_inv(&self, a: u32) -> Option<u32> {
        let (mut r, mut s) = ((a as u64 % self.m) as i128, self.m as i128);
        let (mut x, mut nx) = (1i128, 0i128);
        while s != 0 {
            let q = r / s;
            (r, s) = (s, r - q * s);
            (x, nx) = (nx, x - q * nx);
        }
        (r == 1).then(|| x.rem_euclid(self.m as i128) as u32)
    }
    pub fn inv(&self, a: u32) -> u32 {
        self.try_inv(a).expect("inverse does not exist")
    }
}
