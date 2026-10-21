// Source: https://github.com/InverseAki/arcaki_library/blob/main/src/DataStructure/bitset.rs
const BITSET_BLOCK_BITS: usize = 64;
const BITSET_SHIFT: usize = 6;
const BITSET_MASK: usize = 63;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BitSet {
    n: usize,
    data: Vec<u64>,
}

impl BitSet {
    pub fn from_binary(bytes: &[u8]) -> Self {
        let mut result = Self::new(bytes.len());
        for (i, chunk) in bytes.chunks(8).enumerate() {
            let mut packed = [b'0'; 8];
            packed[..chunk.len()].copy_from_slice(chunk);
            assert!(chunk.iter().all(|&c| c == b'0' || c == b'1'), "invalid binary digit");
            let word = u64::from_le_bytes(packed) & 0x0101_0101_0101_0101;
            let byte = word.wrapping_mul(0x0102_0408_1020_4080) >> 56;
            result.data[i >> 3] |= byte << ((i & 7) * 8);
        }
        result
    }

    pub fn write_binary(&self, bytes: &mut [u8]) {
        assert_eq!(bytes.len(), self.n, "binary output length mismatch");
        for (i, chunk) in bytes.chunks_mut(8).enumerate() {
            let byte = (self.data[i >> 3] >> ((i & 7) * 8)) & 255;
            let spread = ((byte & 127).wrapping_mul(0x0002_0408_1020_4081) & 0x0101_0101_0101_0101) | ((byte >> 7) << 56);
            chunk.copy_from_slice(&(spread | 0x3030_3030_3030_3030).to_le_bytes()[..chunk.len()]);
        }
    }

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BitMatrix {
    width: usize,
    rows: Vec<BitSet>,
}

impl BitMatrix {
    pub fn new(height: usize, width: usize) -> Self {
        Self { width, rows: vec![BitSet::new(width); height] }
    }

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

    pub fn mul_and(&self, rhs: &Self) -> Vec<Vec<usize>> {
        self.count_product::<0>(rhs)
    }

    pub fn mul_or(&self, rhs: &Self) -> Vec<Vec<usize>> {
        self.count_product::<1>(rhs)
    }

    pub fn mul_xor(&self, rhs: &Self) -> Vec<Vec<usize>> {
        self.count_product::<2>(rhs)
    }

    pub fn prod_and(&self, rhs: &Self) -> Self {
        self.logical_product::<0>(rhs)
    }

    pub fn prod_or(&self, rhs: &Self) -> Self {
        self.logical_product::<1>(rhs)
    }

    pub fn prod_xor(&self, rhs: &Self) -> Self {
        assert_eq!(self.width, rhs.height(), "matrix product shape mismatch");
        if self.height() == 0 || self.width == 0 || rhs.width == 0 {
            return Self::new(self.height(), rhs.width);
        }
        let ones: usize = self.rows.iter().flat_map(|row| &row.data).map(|w| w.count_ones() as usize).sum();
        let cells = self.height().saturating_mul(self.width);
        let complement = ones >= cells - cells / 32;
        if self.height() <= 4 || ones <= cells / 32 || complement {
            let mut initial = BitSet::new(rhs.width);
            if complement {
                for row in &rhs.rows {
                    for (c, &b) in initial.data.iter_mut().zip(&row.data) { *c ^= b; }
                }
            }
            let mut out = Self::from_rows(rhs.width, vec![initial; self.height()]);
            for (a, c) in self.rows.iter().zip(out.rows.iter_mut()) {
                for (block, &word) in a.data.iter().enumerate() {
                    let mut word = if complement { !word } else { word };
                    let remaining = self.width - block * 64;
                    if remaining < 64 { word &= (1u64 << remaining) - 1; }
                    while word != 0 {
                        let k = block * 64 + word.trailing_zeros() as usize;
                        for (c, &b) in c.data.iter_mut().zip(&rhs.rows[k].data) { *c ^= b; }
                        word &= word - 1;
                    }
                }
            }
            return out;
        }
        if rhs.width <= 64 && self.width >= 64 {
            let transposed = rhs.transpose();
            let mut out = Self::new(self.height(), rhs.width);
            for (a, c) in self.rows.iter().zip(out.rows.iter_mut()) {
                for (j, b) in transposed.rows.iter().enumerate() {
                    let mut parity = 0;
                    for (&x, &y) in a.data.iter().zip(&b.data) { parity ^= x & y; }
                    c.data[0] |= ((parity.count_ones() & 1) as u64) << j;
                }
            }
            return out;
        }
        macro_rules! product {
            ($p:literal) => {
                if self.height() <= 64 || self.width <= 32 { self.gf2_table_product::<4, 4, $p>(rhs) }
                else if self.height() <= 512 { self.gf2_table_product::<6, 4, $p>(rhs) }
                else { self.gf2_table_product::<8, 8, $p>(rhs) }
            };
        }
        match rhs.rows[0].data.len() {
            1 => product!(1),
            2 => product!(2),
            3..=4 => product!(4),
            5..=8 => product!(8),
            9..=16 => product!(16),
            17..=32 => product!(32),
            _ => product!(64),
        }
    }

    fn gf2_table_product<const B: usize, const G: usize, const P: usize>(&self, rhs: &Self) -> Self {
        let mut out = Self::new(self.height(), rhs.width);
        if self.height() == 0 || self.width == 0 || rhs.width == 0 { return out; }
        let words = rhs.rows[0].data.len();
        let size = 1usize << B;
        let mut table = vec![0u64; G * size * P];
        for col in (0..words).step_by(P) {
            let len = P.min(words - col);
            for base in (0..self.width).step_by(B * G) {
                for g in 0..G {
                    let t = &mut table[g * size * P..(g + 1) * size * P];
                    t[..len].fill(0);
                    for bit in 0..B.min(self.width.saturating_sub(base + g * B)) {
                        let row = base + g * B + bit;
                        let (lo, hi) = t.split_at_mut((1 << bit) * P);
                        let b = &rhs.rows[row].data[col..col + len];
                        for mask in 0..1 << bit {
                            let a = &lo[mask * P..mask * P + len];
                            let c = &mut hi[mask * P..mask * P + len];
                            for w in 0..len { c[w] = a[w] ^ b[w]; }
                        }
                    }
                }
                for (a, c) in self.rows.iter().zip(out.rows.iter_mut()) {
                    let offsets: [usize; G] = std::array::from_fn(|g| {
                        let bit = base + g * B;
                        let mask = if bit < self.width {
                            let mut word = a.data[bit >> 6] >> (bit & 63);
                            if (bit & 63) + B > 64 && (bit >> 6) + 1 < a.data.len() {
                                word |= a.data[(bit >> 6) + 1] << (64 - (bit & 63));
                            }
                            word as usize & (size - 1)
                        } else { 0 };
                        (g * size + mask) * P
                    });
                    let c = &mut c.data[col..col + len];
                    match G {
                        4 => {
                            let a = &table[offsets[0]..offsets[0] + len];
                            let b = &table[offsets[1]..offsets[1] + len];
                            let d = &table[offsets[2]..offsets[2] + len];
                            let e = &table[offsets[3]..offsets[3] + len];
                            for w in 0..len { c[w] ^= a[w] ^ b[w] ^ d[w] ^ e[w]; }
                        }
                        8 => {
                            let a = &table[offsets[0]..offsets[0] + len];
                            let b = &table[offsets[1]..offsets[1] + len];
                            let d = &table[offsets[2]..offsets[2] + len];
                            let e = &table[offsets[3]..offsets[3] + len];
                            let f = &table[offsets[4]..offsets[4] + len];
                            let h = &table[offsets[5]..offsets[5] + len];
                            let i = &table[offsets[6]..offsets[6] + len];
                            let j = &table[offsets[7]..offsets[7] + len];
                            for w in 0..len { c[w] ^= a[w] ^ b[w] ^ d[w] ^ e[w] ^ f[w] ^ h[w] ^ i[w] ^ j[w]; }
                        }
                        _ => {
                            for offset in offsets {
                                let a = &table[offset..offset + len];
                                for w in 0..len { c[w] ^= a[w]; }
                            }
                        }
                    }
                }
            }
        }
        out
    }

    pub fn pow_and(&self, exponent: u64) -> Self {
        self.logical_power::<0>(exponent)
    }

    pub fn pow_or(&self, exponent: u64) -> Self {
        self.logical_power::<1>(exponent)
    }

    pub fn pow_xor(&self, exponent: u64) -> Self {
        self.logical_power::<2>(exponent)
    }

    fn logical_product<const OP: u8>(&self, rhs: &Self) -> Self {
        if OP == 2 { return self.prod_xor(rhs); }
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
