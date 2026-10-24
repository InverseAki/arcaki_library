// Source: https://github.com/InverseAki/arcaki_library/blob/main/src/DataStructure/predecessor64.rs
#[derive(Clone, Debug)]
pub struct Predecessor64 {
    mx: usize,
    data: Vec<u64>,
    offset: [usize; 11],
    levels: usize,
}

impl Predecessor64 {
    pub fn new(mx: usize) -> Self {
        let (offset, levels, total) = Self::layout(mx.max(1).div_ceil(64));
        Self {
            mx: mx.max(1),
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
        let mx = base.len().checked_mul(64).expect("universe size overflow");
        let (offset, levels, total) = Self::layout(base.len());
        base.reserve_exact(total - base.len());
        base.resize(total, 0);
        for i in 1..levels {
            let (lower, upper) = base.split_at_mut(offset[i]);
            let lower = &lower[offset[i - 1]..];
            let mut chunks = lower.chunks_exact(64);
            let mut words = upper.iter_mut();
            for (chunk, word) in (&mut chunks).zip(&mut words) {
                let mut bits = 0;
                for j in 0..64 {
                    bits |= ((chunk[j] != 0) as u64) << j;
                }
                *word = bits;
            }
            if let Some(word) = words.next() {
                let mut bits = 0;
                for (j, &v) in chunks.remainder().iter().enumerate() {
                    bits |= ((v != 0) as u64) << j;
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
        let mut base = Vec::with_capacity(s.len().div_ceil(64));
        for chunk in s.as_bytes().chunks(64) {
            let mut word = 0;
            let mut groups = chunk.chunks_exact(8);
            for (i, group) in (&mut groups).enumerate() {
                let mut bytes = [0; 8];
                for j in 0..8 {
                    bytes[j] = (group[j] == b'1') as u8;
                }
                let bits = u64::from_le_bytes(bytes).wrapping_mul(0x0102_0408_1020_4080) >> 56;
                word |= bits << (i * 8);
            }
            let start = chunk.len() & !7;
            for (j, &c) in groups.remainder().iter().enumerate() {
                word |= ((c == b'1') as u64) << (start + j);
            }
            base.push(word);
        }
        Self::from_vec_u64(base)
    }

    #[inline(always)]
    fn word(&self, index: usize) -> u64 {
        // SAFETY: callers use layout indices; public positions are checked against mx.
        unsafe { *self.data.get_unchecked(index) }
    }

    #[inline(always)]
    fn word_mut(&mut self, index: usize) -> &mut u64 {
        // SAFETY: checked leaf positions and nonempty/empty transitions stay within layout.
        unsafe { self.data.get_unchecked_mut(index) }
    }

    #[inline(always)]
    pub fn insert(&mut self, mut p: usize) {
        assert!(p < self.mx);
        let word = self.word_mut(p >> 6);
        let old = *word;
        *word |= 1 << (p & 63);
        if old != 0 {
            return;
        }
        for i in 1..self.levels {
            p >>= 6;
            let word = self.word_mut(self.offset[i] + (p >> 6));
            let old = *word;
            *word |= 1 << (p & 63);
            if old != 0 {
                return;
            }
        }
    }

    #[inline(always)]
    pub fn remove(&mut self, mut p: usize) {
        assert!(p < self.mx);
        let word = self.word_mut(p >> 6);
        let old = *word;
        *word &= !(1 << (p & 63));
        if *word != 0 || old == 0 {
            return;
        }
        for i in 1..self.levels {
            p >>= 6;
            let word = self.word_mut(self.offset[i] + (p >> 6));
            let old = *word;
            *word &= !(1 << (p & 63));
            if *word != 0 || old == 0 {
                return;
            }
        }
    }

    #[inline(always)]
    pub fn include(&self, p: usize) -> bool {
        if p >= self.mx {
            return false;
        }
        self.word(p >> 6) & (1 << (p & 63)) != 0
    }
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.word(self.offset[self.levels - 1]) == 0
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
        let leaf = self.word(leaf_index);
        if leaf != 0 {
            let bit = if MIN {
                leaf.trailing_zeros() as usize
            } else {
                63 - leaf.leading_zeros() as usize
            };
            return (leaf_index << 6) | bit;
        }
        let mut word = self.word(self.offset[self.levels - 1]);
        if word == 0 {
            return !0;
        }
        let mut p = if MIN {
            word.trailing_zeros() as usize
        } else {
            63 - word.leading_zeros() as usize
        };
        for i in (0..self.levels - 1).rev() {
            word = self.word(self.offset[i] + p);
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
        if p >= self.mx {
            return self.max();
        }
        let mask = if INCLUSIVE {
            u64::MAX >> (63 - (p & 63))
        } else {
            (1u64 << (p & 63)) - 1
        };
        let word = self.word(p >> 6) & mask;
        if word != 0 {
            return (p & !63) | (63 - word.leading_zeros() as usize);
        }
        for i in 1..self.levels {
            p >>= 6;
            let word = self.word(self.offset[i] + (p >> 6)) & ((1u64 << (p & 63)) - 1);
            if word != 0 {
                let mut res = (p & !63) | (63 - word.leading_zeros() as usize);
                for j in (0..i).rev() {
                    res = (res << 6)
                        | (63 - self.word(self.offset[j] + res).leading_zeros() as usize);
                }
                return res;
            }
        }
        !0
    }

    #[inline(always)]
    fn search_next<const INCLUSIVE: bool>(&self, mut p: usize) -> usize {
        if p >= self.mx {
            return !0;
        }
        let mask = if INCLUSIVE {
            u64::MAX << (p & 63)
        } else {
            (u64::MAX << (p & 63)) << 1
        };
        let word = self.word(p >> 6) & mask;
        if word != 0 {
            return (p & !63) | (word.trailing_zeros() as usize);
        }
        for i in 1..self.levels {
            p >>= 6;
            let word = self.word(self.offset[i] + (p >> 6)) & ((u64::MAX << (p & 63)) << 1);
            if word != 0 {
                let mut res = (p & !63) | (word.trailing_zeros() as usize);
                for j in (0..i).rev() {
                    res = (res << 6) | (self.word(self.offset[j] + res).trailing_zeros() as usize);
                }
                return res;
            }
        }
        !0
    }
}
