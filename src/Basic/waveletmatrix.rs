#[derive(Clone, Debug)]
struct BitVector {
    n: usize,
    bits: Vec<u64>,
    acc: Vec<usize>,
}

impl BitVector {
    #[inline]
    fn blocks(n: usize) -> usize {
        (n + 63) >> 6
    }

    #[inline]
    fn index(i: usize) -> (usize, usize) {
        (i >> 6, i & 63)
    }

    fn new(n: usize) -> Self {
        let m = Self::blocks(n);
        Self {
            n,
            bits: vec![0; m],
            acc: vec![0; m + 1],
        }
    }

    #[inline]
    fn set(&mut self, i: usize) {
        let (b, off) = Self::index(i);
        self.bits[b] |= 1u64 << off;
    }

    fn build(&mut self) {
        let m = self.bits.len();
        self.acc[0] = 0;
        for i in 0..m {
            let valid = if (i + 1) << 6 <= self.n {
                64
            } else {
                self.n.saturating_sub(i << 6)
            };
            let mask = if valid == 64 {
                !0u64
            } else if valid == 0 {
                0
            } else {
                (1u64 << valid) - 1
            };
            self.acc[i + 1] = self.acc[i] + ((self.bits[i] & mask).count_ones() as usize);
        }
    }

    #[inline]
    fn access(&self, i: usize) -> bool {
        let (b, off) = Self::index(i);
        ((self.bits[b] >> off) & 1) != 0
    }

    #[inline]
    fn rank1(&self, k: usize) -> usize {
        let k = k.min(self.n);
        let (b, off) = Self::index(k);
        if off == 0 {
            self.acc[b]
        } else {
            self.acc[b] + ((self.bits[b] & ((1u64 << off) - 1)).count_ones() as usize)
        }
    }

    #[inline]
    fn rank0(&self, k: usize) -> usize {
        k.min(self.n) - self.rank1(k)
    }

    #[inline]
    fn rank0_range(&self, l: usize, r: usize) -> usize {
        self.rank0(r) - self.rank0(l)
    }
}

#[derive(Clone, Debug)]
pub struct WaveletMatrix {
    n: usize,
    max_log: usize,
    mids: Vec<usize>,
    bitvecs: Vec<BitVector>,
}

impl WaveletMatrix {
    pub fn new(a: &[usize]) -> Self {
        let n = a.len();
        let max_val = a.iter().copied().max().unwrap_or(0);
        let max_log = if n == 0 || max_val == 0 {
            1
        } else {
            (usize::BITS - max_val.leading_zeros()) as usize
        };

        let mut cur = a.to_vec();
        let mut bitvecs = Vec::with_capacity(max_log);
        let mut mids = Vec::with_capacity(max_log);

        for level_rev in 0..max_log {
            let level = max_log - 1 - level_rev;
            let mut bv = BitVector::new(n);
            let mut zero = Vec::with_capacity(n);
            let mut one = Vec::with_capacity(n);

            for (i, &x) in cur.iter().enumerate() {
                if ((x >> level) & 1) == 1 {
                    bv.set(i);
                    one.push(x);
                } else {
                    zero.push(x);
                }
            }

            bv.build();
            mids.push(zero.len());
            zero.extend(one);
            cur = zero;
            bitvecs.push(bv);
        }

        Self {
            n,
            max_log,
            mids,
            bitvecs,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.n
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.n == 0
    }

    pub fn access(&self, mut i: usize) -> usize {
        let mut x = 0usize;
        for d in 0..self.max_log {
            let bit = self.bitvecs[d].access(i);
            if bit {
                x |= 1usize << (self.max_log - 1 - d);
                i = self.mids[d] + self.bitvecs[d].rank1(i);
            } else {
                i = self.bitvecs[d].rank0(i);
            }
        }
        x
    }

    pub fn rank(&self, x: usize, mut r: usize) -> usize {
        if self.max_log < usize::BITS as usize && x >= (1usize << self.max_log) {
            return 0;
        }
        for d in 0..self.max_log {
            let bit = (x >> (self.max_log - 1 - d)) & 1;
            if bit == 0 {
                r = self.bitvecs[d].rank0(r);
            } else {
                r = self.mids[d] + self.bitvecs[d].rank1(r);
            }
        }
        r
    }

    pub fn rank_range(&self, x: usize, l: usize, r: usize) -> usize {
        self.rank(x, r) - self.rank(x, l)
    }

    pub fn kth_smallest(&self, mut l: usize, mut r: usize, mut k: usize) -> usize {
        assert!(l <= r && r <= self.n);
        assert!(k < r - l);

        let mut ans = 0usize;
        for d in 0..self.max_log {
            let zeros = self.bitvecs[d].rank0_range(l, r);
            if k < zeros {
                l = self.bitvecs[d].rank0(l);
                r = self.bitvecs[d].rank0(r);
            } else {
                ans |= 1usize << (self.max_log - 1 - d);
                k -= zeros;
                l = self.mids[d] + self.bitvecs[d].rank1(l);
                r = self.mids[d] + self.bitvecs[d].rank1(r);
            }
        }
        ans
    }

    pub fn kth_largest(&self, l: usize, r: usize, k: usize) -> usize {
        assert!(k < r - l);
        self.kth_smallest(l, r, r - l - 1 - k)
    }

    pub fn quantile(&self, l: usize, r: usize, k: usize) -> usize {
        self.kth_smallest(l, r, k)
    }

    pub fn range_freq(&self, mut l: usize, mut r: usize, upper: usize) -> usize {
        if l >= r || upper == 0 {
            return 0;
        }

        let mut cnt = 0usize;
        for d in 0..self.max_log {
            let bit = (upper >> (self.max_log - 1 - d)) & 1;
            if bit == 1 {
                cnt += self.bitvecs[d].rank0_range(l, r);
                l = self.mids[d] + self.bitvecs[d].rank1(l);
                r = self.mids[d] + self.bitvecs[d].rank1(r);
            } else {
                l = self.bitvecs[d].rank0(l);
                r = self.bitvecs[d].rank0(r);
            }
        }
        cnt
    }

    pub fn range_freq_between(&self, l: usize, r: usize, lower: usize, upper: usize) -> usize {
        if lower >= upper || l >= r {
            0
        } else {
            self.range_freq(l, r, upper) - self.range_freq(l, r, lower)
        }
    }

    pub fn prev_value(&self, l: usize, r: usize, upper: usize) -> Option<usize> {
        let cnt = self.range_freq(l, r, upper);
        if cnt == 0 {
            None
        } else {
            Some(self.kth_smallest(l, r, cnt - 1))
        }
    }

    pub fn next_value(&self, l: usize, r: usize, lower: usize) -> Option<usize> {
        let cnt = self.range_freq(l, r, lower);
        if cnt == r - l {
            None
        } else {
            Some(self.kth_smallest(l, r, cnt))
        }
    }
}
