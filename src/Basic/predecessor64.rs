#[derive(Clone, Debug)]
pub struct Predecessor64 {
    mx: usize,
    data: Vec<u64>,
    offset: [usize; 11],
    levels: usize,
}

impl Predecessor64 {
    pub fn new(mx: usize) -> Self {
        let (offset, levels, total) = Self::layout(mx.div_ceil(64));
        Self {
            mx,
            data: vec![0; total],
            offset,
            levels,
        }
    }

    fn layout(mut len: usize) -> ([usize; 11], usize, usize) {
        let mut offset = [0; 11];
        let mut levels = 0;
        let mut total = 0;
        loop {
            offset[levels] = total;
            levels += 1;
            total += len;
            if len <= 1 {
                break;
            }
            len = len.div_ceil(64);
        }
        (offset, levels, total)
    }

    pub fn from_vec_u64(mut base: Vec<u64>) -> Self {
        if base.is_empty() {
            return Self::new(1);
        }
        let mx = base.len() << 6;
        let (offset, levels, total) = Self::layout(base.len());
        base.reserve_exact(total - base.len());
        base.resize(total, 0);
        for i in 1..levels {
            let (lower, upper) = base.split_at_mut(offset[i]);
            let lower = &lower[offset[i - 1]..];
            for (chunk, word) in lower.chunks(64).zip(upper.iter_mut()) {
                let mut bits = 0;
                for (j, &v) in chunk.iter().enumerate() {
                    if v != 0 {
                        bits |= 1 << j;
                    }
                }
                *word = bits;
            }
        }
        Self {
            mx,
            data: base,
            offset,
            levels,
        }
    }

    pub fn from_vec_usize(base: Vec<usize>) -> Self {
        Self::from_vec_u64(base.into_iter().map(|v| v as u64).collect())
    }

    pub fn from_01_string(s: String) -> Self {
        let mut base = vec![0; s.len().div_ceil(64)];
        for (i, c) in s.bytes().enumerate() {
            if c == b'1' {
                base[i >> 6] |= 1 << (i & 63);
            }
        }
        Self::from_vec_u64(base)
    }

    #[inline(always)]
    pub fn insert(&mut self, mut p: usize) {
        for i in 0..self.levels {
            let word = &mut self.data[self.offset[i] + (p >> 6)];
            let old = *word;
            *word |= 1 << (p & 63);
            if old != 0 {
                return;
            }
            p >>= 6;
        }
    }

    #[inline(always)]
    pub fn remove(&mut self, mut p: usize) {
        for i in 0..self.levels {
            let word = &mut self.data[self.offset[i] + (p >> 6)];
            let old = *word;
            *word &= !(1 << (p & 63));
            if *word != 0 || old == 0 {
                return;
            }
            p >>= 6;
        }
    }

    #[inline(always)]
    pub fn include(&self, p: usize) -> bool {
        self.data[p >> 6] & (1 << (p & 63)) != 0
    }
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.data[self.offset[self.levels - 1]] == 0
    }
    #[inline(always)]
    pub fn min(&self) -> usize {
        self.extreme::<true>()
    }
    #[inline(always)]
    pub fn max(&self) -> usize {
        self.extreme::<false>()
    }
    #[inline(always)]
    fn extreme<const MIN: bool>(&self) -> usize {
        let leaf_index = if MIN { 0 } else { (self.mx - 1) >> 6 };
        let leaf = self.data[leaf_index];
        if leaf != 0 {
            let bit = if MIN {
                leaf.trailing_zeros() as usize
            } else {
                63 - leaf.leading_zeros() as usize
            };
            return (leaf_index << 6) | bit;
        }
        let mut word = self.data[self.offset[self.levels - 1]];
        if word == 0 {
            return !0;
        }
        let mut p = if MIN {
            word.trailing_zeros() as usize
        } else {
            63 - word.leading_zeros() as usize
        };
        for i in (0..self.levels - 1).rev() {
            word = self.data[self.offset[i] + p];
            let bit = if MIN {
                word.trailing_zeros() as usize
            } else {
                63 - word.leading_zeros() as usize
            };
            p = (p << 6) | bit;
        }
        p
    }

    #[inline(always)]
    pub fn prev(&self, p: usize) -> usize {
        self.search_prev::<false>(p)
    }
    #[inline(always)]
    pub fn inprev(&self, p: usize) -> usize {
        self.search_prev::<true>(p)
    }
    #[inline(always)]
    pub fn next(&self, p: usize) -> usize {
        self.search_next::<false>(p)
    }
    #[inline(always)]
    pub fn innext(&self, p: usize) -> usize {
        self.search_next::<true>(p)
    }

    #[inline(always)]
    fn search_prev<const INCLUSIVE: bool>(&self, mut p: usize) -> usize {
        for i in 0..self.levels {
            let mask = if i == 0 && INCLUSIVE {
                u64::MAX >> (63 - (p & 63))
            } else {
                (1u64 << (p & 63)) - 1
            };
            let word = self.data[self.offset[i] + (p >> 6)] & mask;
            if word != 0 {
                let mut res = (p & !63) | (63 - word.leading_zeros() as usize);
                for j in (0..i).rev() {
                    res = (res << 6)
                        | (63 - self.data[self.offset[j] + res].leading_zeros() as usize);
                }
                return res;
            }
            p >>= 6;
        }
        !0
    }

    #[inline(always)]
    fn search_next<const INCLUSIVE: bool>(&self, mut p: usize) -> usize {
        for i in 0..self.levels {
            let mask = if i == 0 && INCLUSIVE {
                u64::MAX << (p & 63)
            } else {
                (u64::MAX << (p & 63)) << 1
            };
            let word = self.data[self.offset[i] + (p >> 6)] & mask;
            if word != 0 {
                let mut res = (p & !63) | word.trailing_zeros() as usize;
                for j in (0..i).rev() {
                    res = (res << 6) | self.data[self.offset[j] + res].trailing_zeros() as usize;
                }
                return res;
            }
            p >>= 6;
        }
        !0
    }
}
