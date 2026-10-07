pub struct BarrettCombination {
    barrett: Barrett32,
    fact: Vec<u32>,
    inv_fact: Vec<u32>,
}
impl BarrettCombination {
    pub fn new(n: usize, modulus: u32) -> Self {
        Self::try_new(n, modulus).expect("requires modulus >= 2 and invertible n! modulo modulus")
    }
    pub fn try_new(n: usize, modulus: u32) -> Option<Self> {
        if modulus < 2 || n as u128 >= modulus as u128 {
            return None;
        }
        Self::try_with_barrett(n, Barrett32::new(modulus))
    }
    pub fn with_barrett(n: usize, barrett: Barrett32) -> Self {
        Self::try_with_barrett(n, barrett).expect("n! must be invertible modulo modulus")
    }
    pub fn try_with_barrett(n: usize, barrett: Barrett32) -> Option<Self> {
        if n as u128 >= barrett.modulus() as u128 {
            return None;
        }
        let mut fact = vec![1; n + 1];
        for i in 1..=n {
            fact[i] = barrett.mul(fact[i - 1], i as u32);
        }
        let last = barrett.try_inv(fact[n])?;
        let mut inv_fact = vec![1; n + 1];
        inv_fact[n] = last;
        for i in (1..=n).rev() {
            inv_fact[i - 1] = barrett.mul(inv_fact[i], i as u32);
        }
        Some(Self {
            barrett,
            fact,
            inv_fact,
        })
    }
    pub fn modulus(&self) -> u32 {
        self.barrett.modulus()
    }
    pub fn max_n(&self) -> usize {
        self.fact.len() - 1
    }
    #[inline]
    pub fn f(&self, x: usize) -> u32 {
        self.fact[x]
    }
    #[inline]
    pub fn fi(&self, x: usize) -> u32 {
        self.inv_fact[x]
    }
    #[inline]
    pub fn inv(&self, x: usize) -> u32 {
        assert!(x > 0);
        self.barrett.mul(self.fact[x - 1], self.inv_fact[x])
    }
    #[inline]
    pub fn p(&self, n: usize, k: usize) -> u32 {
        if k > n {
            return 0;
        }
        self.barrett.mul(self.fact[n], self.inv_fact[n - k])
    }
    #[inline]
    pub fn c(&self, n: usize, k: usize) -> u32 {
        if k > n {
            return 0;
        }
        self.barrett.mul(
            self.barrett.mul(self.fact[n], self.inv_fact[k]),
            self.inv_fact[n - k],
        )
    }
    pub fn h(&self, n: usize, k: usize) -> u32 {
        if k == 0 {
            return 1;
        }
        if n == 0 {
            return 0;
        }
        self.c(n.checked_add(k - 1).expect("n+k-1 overflow"), k)
    }
}
