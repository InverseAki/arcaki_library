const BITSET_BLOCK_BITS: usize = 64;
const BITSET_SHIFT: usize = 6;
const BITSET_MASK: usize = 63;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BitSet {
    n: usize,
    data: Vec<u64>,
}

impl BitSet {
    #[inline]
    pub fn new(n: usize) -> Self {
        Self {
            n, data: vec![0; (n + BITSET_MASK) >> BITSET_SHIFT],
        }
    }

    #[inline]
    pub fn build(n: usize, data: Vec<u64>) -> Self {
        debug_assert_eq!(
            data.len(),
            (n + BITSET_MASK) >> BITSET_SHIFT
        );

        let mut res = Self { n, data };
        res.mask_last();
        res
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.n
    }

    #[inline(always)]
    pub fn blocks(&self) -> usize {
        self.data.len()
    }

    #[inline(always)]
    pub fn is_empty_len(&self) -> bool {
        self.n == 0
    }

    #[inline(always)]
    fn mask_last(&mut self) {
        let r = self.n & BITSET_MASK;

        if r != 0 {
            if let Some(last) = self.data.last_mut() {
                *last &= (1u64 << r) - 1;
            }
        }
    }

    #[inline(always)]
    pub fn set(&mut self, p: usize, f: bool) {
        debug_assert!(p < self.n);

        if f {
            self.data[p >> BITSET_SHIFT] |=
                1u64 << (p & BITSET_MASK);
        } else {
            self.data[p >> BITSET_SHIFT] &=
                !(1u64 << (p & BITSET_MASK));
        }
    }

    #[inline(always)]
    pub fn insert(&mut self, p: usize) {
        debug_assert!(p < self.n);

        self.data[p >> BITSET_SHIFT] |=
            1u64 << (p & BITSET_MASK);
    }

    #[inline(always)]
    pub fn remove(&mut self, p: usize) {
        debug_assert!(p < self.n);

        self.data[p >> BITSET_SHIFT] &=
            !(1u64 << (p & BITSET_MASK));
    }

    #[inline(always)]
    pub fn flip(&mut self, p: usize) {
        debug_assert!(p < self.n);

        self.data[p >> BITSET_SHIFT] ^=
            1u64 << (p & BITSET_MASK);
    }

    #[inline(always)]
    pub fn get(&self, p: usize) -> bool {
        debug_assert!(p < self.n);

        self.data[p >> BITSET_SHIFT]
            & (1u64 << (p & BITSET_MASK))
            != 0
    }

    #[inline]
    pub fn clear(&mut self) {
        self.data.fill(0);
    }

    #[inline]
    pub fn fill(&mut self) {
        self.data.fill(!0u64);
        self.mask_last();
    }

    #[inline]
    pub fn flip_all(&mut self) {
        for x in &mut self.data {
            *x = !*x;
        }

        self.mask_last();
    }

    #[inline]
    pub fn count_ones(&self) -> usize {
        self.data
            .iter()
            .map(|x| x.count_ones() as usize)
            .sum()
    }

    #[inline]
    pub fn count_zeros(&self) -> usize {
        self.n - self.count_ones()
    }

    #[inline]
    pub fn none(&self) -> bool {
        self.data.iter().all(|&x| x == 0)
    }

    #[inline]
    pub fn any(&self) -> bool {
        self.data.iter().any(|&x| x != 0)
    }

    #[inline]
    pub fn all(&self) -> bool {
        self.count_ones() == self.n
    }

    #[inline]
    pub fn and_count_ones(&self, rhs: &Self) -> usize {
        self.assert_same_len(rhs);

        self.data
            .iter()
            .zip(rhs.data.iter())
            .map(|(&x, &y)| (x & y).count_ones() as usize)
            .sum()
    }

    #[inline]
    pub fn and_count_zeros(&self, rhs: &Self) -> usize {
        self.n - self.and_count_ones(rhs)
    }

    #[inline]
    pub fn and_is_empty(&self, rhs: &Self) -> bool {
        self.assert_same_len(rhs);

        self.data
            .iter()
            .zip(rhs.data.iter())
            .all(|(&x, &y)| (x & y) == 0)
    }

    #[inline]
    pub fn and_is_nonempty(&self, rhs: &Self) -> bool {
        self.assert_same_len(rhs);

        self.data
            .iter()
            .zip(rhs.data.iter())
            .any(|(&x, &y)| (x & y) != 0)
    }

    #[inline]
    pub fn or_count_ones(&self, rhs: &Self) -> usize {
        self.assert_same_len(rhs);

        self.data
            .iter()
            .zip(rhs.data.iter())
            .map(|(&x, &y)| (x | y).count_ones() as usize)
            .sum()
    }

    #[inline]
    pub fn or_count_zeros(&self, rhs: &Self) -> usize {
        self.n - self.or_count_ones(rhs)
    }

    #[inline]
    pub fn or_is_empty(&self, rhs: &Self) -> bool {
        self.assert_same_len(rhs);

        self.data
            .iter()
            .zip(rhs.data.iter())
            .all(|(&x, &y)| (x | y) == 0)
    }

    #[inline]
    pub fn or_is_nonempty(&self, rhs: &Self) -> bool {
        self.assert_same_len(rhs);

        self.data
            .iter()
            .zip(rhs.data.iter())
            .any(|(&x, &y)| (x | y) != 0)
    }

    #[inline]
    pub fn xor_count_ones(&self, rhs: &Self) -> usize {
        self.assert_same_len(rhs);

        self.data
            .iter()
            .zip(rhs.data.iter())
            .map(|(&x, &y)| (x ^ y).count_ones() as usize)
            .sum()
    }

    #[inline]
    pub fn xor_count_zeros(&self, rhs: &Self) -> usize {
        self.n - self.xor_count_ones(rhs)
    }

    #[inline]
    pub fn xor_is_empty(&self, rhs: &Self) -> bool {
        self.assert_same_len(rhs);

        self.data
            .iter()
            .zip(rhs.data.iter())
            .all(|(&x, &y)| (x ^ y) == 0)
    }

    #[inline]
    pub fn xor_is_nonempty(&self, rhs: &Self) -> bool {
        self.assert_same_len(rhs);

        self.data
            .iter()
            .zip(rhs.data.iter())
            .any(|(&x, &y)| (x ^ y) != 0)
    }

    #[inline]
    pub fn disjoint(&self, rhs: &Self) -> bool {
        self.and_is_empty(rhs)
    }

    #[inline]
    pub fn is_subset(&self, rhs: &Self) -> bool {
        self.assert_same_len(rhs);

        self.data
            .iter()
            .zip(rhs.data.iter())
            .all(|(&x, &y)| x & !y == 0)
    }

    #[inline]
    pub fn is_superset(&self, rhs: &Self) -> bool {
        rhs.is_subset(self)
    }

    #[inline]
    pub fn get_shift_left(&self, k: usize) -> Self {
        if k >= self.n {
            return Self::new(self.n);
        }

        let block = k >> BITSET_SHIFT;
        let rem = k & BITSET_MASK;
        let m = self.data.len();

        let mut res = vec![0u64; m];

        for i in 0..m {
            let j = i + block;

            if j >= m {
                break;
            }

            res[j] |= self.data[i] << rem;

            if rem != 0 && j + 1 < m {
                res[j + 1] |=
                    self.data[i] >> (BITSET_BLOCK_BITS - rem);
            }
        }

        Self::build(self.n, res)
    }

    #[inline]
    pub fn get_shift_right(&self, k: usize) -> Self {
        if k >= self.n {
            return Self::new(self.n);
        }

        let block = k >> BITSET_SHIFT;
        let rem = k & BITSET_MASK;
        let m = self.data.len();

        let mut res = vec![0u64; m];

        for i in block..m {
            res[i - block] |= self.data[i] >> rem;

            if rem != 0 && i >= block + 1 {
                res[i - block - 1] |=
                    self.data[i] << (BITSET_BLOCK_BITS - rem);
            }
        }

        Self::build(self.n, res)
    }

    #[inline(always)]
    fn assert_same_len(&self, rhs: &Self) {
        debug_assert_eq!(self.n, rhs.n);
    }
}

impl BitAndAssign<&BitSet> for BitSet {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: &BitSet) {
        self.assert_same_len(rhs);

        for (x, &y) in self.data.iter_mut().zip(rhs.data.iter()) {
            *x &= y;
        }
    }
}

impl BitOrAssign<&BitSet> for BitSet {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: &BitSet) {
        self.assert_same_len(rhs);

        for (x, &y) in self.data.iter_mut().zip(rhs.data.iter()) {
            *x |= y;
        }
    }
}

impl BitXorAssign<&BitSet> for BitSet {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: &BitSet) {
        self.assert_same_len(rhs);

        for (x, &y) in self.data.iter_mut().zip(rhs.data.iter()) {
            *x ^= y;
        }
    }
}

impl<'a, 'b> BitAnd<&'b BitSet> for &'a BitSet {
    type Output = BitSet;

    #[inline]
    fn bitand(self, rhs: &'b BitSet) -> BitSet {
        self.assert_same_len(rhs);

        let mut res = self.clone();
        res &= rhs;
        res
    }
}

impl<'a, 'b> BitOr<&'b BitSet> for &'a BitSet {
    type Output = BitSet;

    #[inline]
    fn bitor(self, rhs: &'b BitSet) -> BitSet {
        self.assert_same_len(rhs);

        let mut res = self.clone();
        res |= rhs;
        res
    }
}

impl<'a, 'b> BitXor<&'b BitSet> for &'a BitSet {
    type Output = BitSet;

    #[inline]
    fn bitxor(self, rhs: &'b BitSet) -> BitSet {
        self.assert_same_len(rhs);

        let mut res = self.clone();
        res ^= rhs;
        res
    }
}