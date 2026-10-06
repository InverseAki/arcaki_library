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

/// 各行をBitSetで保持する0/1行列。bitset.rsと同じスコープで使う。
/// mul_and/or/xorは C[i][j] = sum_k (A[i][k] op B[k][j]) を返す。
/// 加算は整数の和。OR-ANDの論理行列積やGF(2)の積とは異なる。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BitMatrix {
    width: usize,
    rows: Vec<BitSet>,
}

impl BitMatrix {
    pub fn new(height: usize, width: usize) -> Self {
        Self { width, rows: vec![BitSet::new(width); height] }
    }

    /// widthを明示するので0行の行列も表せる。不揃いな行はpanic。
    pub fn from_rows(width: usize, rows: Vec<BitSet>) -> Self {
        assert!(rows.iter().all(|row| row.len() == width), "matrix row length mismatch");
        Self { width, rows }
    }

    pub fn height(&self) -> usize { self.rows.len() }
    pub fn width(&self) -> usize { self.width }
    pub fn rows(&self) -> &[BitSet] { &self.rows }
    pub fn row(&self, i: usize) -> &BitSet { &self.rows[i] }

    pub fn get(&self, i: usize, j: usize) -> bool {
        assert!(j < self.width, "matrix column out of range");
        self.rows[i].get(j)
    }

    pub fn set(&mut self, i: usize, j: usize, value: bool) {
        assert!(j < self.width, "matrix column out of range");
        self.rows[i].set(j, value);
    }

    /// O(height * ceil(width/64) + 1の個数)。末尾の未使用ビットは常に0。
    pub fn transpose(&self) -> Self {
        let mut out = Self::new(self.width, self.height());
        for (i, row) in self.rows.iter().enumerate() {
            for (block, &word) in row.data.iter().enumerate() {
                let mut bits = word;
                while bits != 0 {
                    let j = block * BITSET_BLOCK_BITS + bits.trailing_zeros() as usize;
                    out.rows[j].insert(i);
                    bits &= bits - 1;
                }
            }
        }
        out
    }

    /// C[i][j] = sum_k (A[i][k] & B[k][j])。共通する1の個数。
    pub fn mul_and(&self, rhs: &Self) -> Vec<Vec<usize>> {
        self.count_product::<0>(rhs)
    }

    /// C[i][j] = sum_k (A[i][k] | B[k][j])。どちらかが1の個数。
    pub fn mul_or(&self, rhs: &Self) -> Vec<Vec<usize>> {
        self.count_product::<1>(rhs)
    }

    /// C[i][j] = sum_k (A[i][k] ^ B[k][j])。異なるビットの個数。
    pub fn mul_xor(&self, rhs: &Self) -> Vec<Vec<usize>> {
        self.count_product::<2>(rhs)
    }

    /// AND-OR積: C[i][j] = AND_k (A[i][k] | B[k][j])。
    /// 内側次元0の空のANDはtrue。mul_andの整数集計とは異なる。
    pub fn prod_and(&self, rhs: &Self) -> Self {
        self.logical_product::<0>(rhs)
    }

    /// OR-AND積: C[i][j] = OR_k (A[i][k] & B[k][j])。
    pub fn prod_or(&self, rhs: &Self) -> Self {
        self.logical_product::<1>(rhs)
    }

    /// XOR-AND積: C[i][j] = XOR_k (A[i][k] & B[k][j])。GF(2)の積。
    pub fn prod_xor(&self, rhs: &Self) -> Self {
        self.logical_product::<2>(rhs)
    }

    /// AND-OR積による累乗。0乗は対角false・非対角true。
    pub fn pow_and(&self, exponent: u64) -> Self {
        self.logical_power::<0>(exponent)
    }

    /// OR-AND積による累乗。0乗は対角true・非対角false。
    pub fn pow_or(&self, exponent: u64) -> Self {
        self.logical_power::<1>(exponent)
    }

    /// GF(2)の累乗。0乗は対角true・非対角false。
    pub fn pow_xor(&self, exponent: u64) -> Self {
        self.logical_power::<2>(exponent)
    }

    fn logical_product<const OP: u8>(&self, rhs: &Self) -> Self {
        assert_eq!(self.width, rhs.height(), "matrix product shape mismatch");
        let transposed = rhs.transpose();
        let mut out = Self::new(self.height(), rhs.width);
        for (i, a) in self.rows.iter().enumerate() {
            for (j, b) in transposed.rows.iter().enumerate() {
                let bit = match OP {
                    0 => a.or_count_ones(b) == self.width,
                    1 => a.and_is_nonempty(b),
                    2 => a.and_count_ones(b) & 1 != 0,
                    _ => unreachable!(),
                };
                if bit { out.rows[i].insert(j); }
            }
        }
        out
    }

    fn logical_power<const OP: u8>(&self, mut exponent: u64) -> Self {
        assert_eq!(self.height(), self.width, "matrix power requires a square matrix");
        let mut result = Self::new(self.width, self.width);
        for (i, row) in result.rows.iter_mut().enumerate() {
            if OP == 0 { row.fill(); }
            row.set(i, OP != 0);
        }
        if exponent == 0 { return result; }
        let mut base = self.clone();
        while exponent > 0 {
            if exponent & 1 != 0 {
                result = result.logical_product::<OP>(&base);
            }
            exponent >>= 1;
            if exponent > 0 {
                base = base.logical_product::<OP>(&base);
            }
        }
        result
    }

    // OPはconstなので内側ループには演算選択の分岐が残らない。
    // 積O(height * rhs.width * ceil(width/64)) + 右辺転置・出力初期化。
    fn count_product<const OP: u8>(&self, rhs: &Self) -> Vec<Vec<usize>> {
        assert_eq!(self.width, rhs.height(), "matrix product shape mismatch");
        let transposed = rhs.transpose();
        self.rows.iter().map(|a| {
            transposed.rows.iter().map(|b| {
                match OP {
                    0 => a.and_count_ones(b),
                    1 => a.or_count_ones(b),
                    2 => a.xor_count_ones(b),
                    _ => unreachable!(),
                }
            }).collect()
        }).collect()
    }
}
