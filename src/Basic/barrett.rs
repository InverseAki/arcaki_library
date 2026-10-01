#[derive(Clone, Copy)]
pub struct Barrett32 {
    m: u64,
    inv: u64,
}

impl Barrett32 {
    #[inline]
    pub fn new(m: u32) -> Self {
        assert!(m >= 2);
        let m = m as u64;
        let inv = ((1u128 << 64) / m as u128) as u64;
        Self { m, inv }
    }

    #[inline(always)]
    pub fn mul(&self, a: u32, b: u32) -> u32 {
        let x = a as u64 * b as u64;
        let q = ((x as u128 * self.inv as u128) >> 64) as u64;
        let mut r = x - q * self.m;
        if r >= self.m {
            r -= self.m;
        }
        r as u32
    }
}
