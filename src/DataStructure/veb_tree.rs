// Source: https://ocw.mit.edu/courses/6-851-advanced-data-structures-spring-2012/5714d393dbd597b3f070ca4be17341ec_MIT6_851S12_L11.pdf
#[derive(Clone, Debug)]
pub struct VebTree {
    mx: usize,
    data: Vec<u64>,
    middle: usize,
    summary: VebNode,
}

impl VebTree {
    pub fn new(mx: usize) -> Self {
        let mx = mx.max(1);
        let middle = mx.div_ceil(64);
        let mut total = middle + mx.div_ceil(4096);
        let summary = VebNode::new(mx.div_ceil(4096), &mut total);
        Self {
            mx,
            data: vec![0; total],
            middle,
            summary,
        }
    }

    pub fn from_vec_u64(mut data: Vec<u64>) -> Self {
        if data.is_empty() {
            return Self::new(1);
        }
        let mx = data.len().checked_mul(64).expect("universe size overflow");
        let middle = data.len();
        let count = middle.div_ceil(64);
        let mut total = middle + count;
        let mut summary = VebNode::new(count, &mut total);
        data.reserve_exact(total - middle);
        data.resize(total, 0);
        let (base, upper) = data.split_at_mut(middle);
        let mut chunks = base.chunks_exact(64);
        let mut words = upper[..count].iter_mut();
        for (chunk, word) in (&mut chunks).zip(&mut words) {
            let mut bits = 0;
            for j in 0..64 {
                bits |= ((chunk[j] != 0) as u64) << j;
            }
            *word = bits;
        }
        if let Some(word) = words.next() {
            for (j, &v) in chunks.remainder().iter().enumerate() {
                *word |= ((v != 0) as u64) << j;
            }
        }
        summary.build(middle, count, &mut data);
        Self {
            mx,
            data,
            middle,
            summary,
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
                let raw = u64::from_le_bytes(group.try_into().unwrap());
                let x = raw ^ 0x3131_3131_3131_3131;
                let low = 0x7f7f_7f7f_7f7f_7f7f;
                let eq = !(((x & low).wrapping_add(low)) | x | low) >> 7;
                let bits = (eq & 0x0101_0101_0101_0101).wrapping_mul(0x0102_0408_1020_4080) >> 56;
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
    fn word(&self, i: usize) -> u64 {
        // SAFETY: checked positions and nonempty summaries select allocated leaf/middle words.
        unsafe { *self.data.get_unchecked(i) }
    }

    #[inline(always)]
    fn word_mut(&mut self, i: usize) -> &mut u64 {
        // SAFETY: p < mx bounds both p / 64 and the middle offset plus p / 4096.
        unsafe { self.data.get_unchecked_mut(i) }
    }

    #[inline(always)]
    pub fn insert(&mut self, p: usize) {
        assert!(p < self.mx);
        let word = self.word_mut(p >> 6);
        let old = *word;
        *word |= 1 << (p & 63);
        if old == 0 {
            let word = self.word_mut(self.middle + (p >> 12));
            let old = *word;
            *word |= 1 << ((p >> 6) & 63);
            if old == 0 {
                self.summary.insert(p >> 12, &mut self.data);
            }
        }
    }

    #[inline(always)]
    pub fn remove(&mut self, p: usize) {
        assert!(p < self.mx);
        let word = self.word_mut(p >> 6);
        let old = *word;
        *word &= !(1 << (p & 63));
        if *word == 0 && old != 0 {
            let word = self.word_mut(self.middle + (p >> 12));
            *word &= !(1 << ((p >> 6) & 63));
            if *word == 0 {
                self.summary.remove(p >> 12, &mut self.data);
            }
        }
    }

    #[inline(always)]
    pub fn include(&self, p: usize) -> bool {
        p < self.mx && self.word(p >> 6) & (1 << (p & 63)) != 0
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.summary.min == usize::MAX
    }

    #[inline(always)]
    pub fn min(&self) -> usize {
        let word = self.word(0);
        if word != 0 {
            return word.trailing_zeros() as usize;
        }
        let p = self.summary.min;
        if p == usize::MAX {
            p
        } else {
            let p = (p << 6) | self.word(self.middle + (p)).trailing_zeros() as usize;
            (p << 6) | self.word(p).trailing_zeros() as usize
        }
    }

    #[inline(always)]
    pub fn max(&self) -> usize {
        let i = (self.mx - 1) >> 6;
        let word = self.word(i);
        if word != 0 {
            return (i << 6) | (63 - word.leading_zeros() as usize);
        }
        let p = self.summary.max;
        if self.summary.min == usize::MAX {
            usize::MAX
        } else {
            let p = (p << 6) | (63 - self.word(self.middle + (p)).leading_zeros() as usize);
            (p << 6) | (63 - self.word(p).leading_zeros() as usize)
        }
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
    fn search_prev<const INCLUSIVE: bool>(&self, p: usize) -> usize {
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
        let word = self.word(self.middle + (p >> 12)) & ((1u64 << ((p >> 6) & 63)) - 1);
        if word != 0 {
            let i = ((p >> 6) & !63) | (63 - word.leading_zeros() as usize);
            return (i << 6) | (63 - self.word(i).leading_zeros() as usize);
        }
        let i = self.summary.prev(p >> 12, &self.data);
        if i == usize::MAX {
            i
        } else {
            let i = (i << 6) | (63 - self.word(self.middle + (i)).leading_zeros() as usize);
            (i << 6) | (63 - self.word(i).leading_zeros() as usize)
        }
    }

    #[inline(always)]
    fn search_next<const INCLUSIVE: bool>(&self, p: usize) -> usize {
        if p >= self.mx {
            return usize::MAX;
        }
        let mask = if INCLUSIVE {
            u64::MAX << (p & 63)
        } else {
            (u64::MAX << (p & 63)) << 1
        };
        let word = self.word(p >> 6) & mask;
        if word != 0 {
            return (p & !63) | word.trailing_zeros() as usize;
        }
        let word = self.word(self.middle + (p >> 12)) & ((u64::MAX << ((p >> 6) & 63)) << 1);
        if word != 0 {
            let i = ((p >> 6) & !63) | word.trailing_zeros() as usize;
            return (i << 6) | self.word(i).trailing_zeros() as usize;
        }
        let i = self.summary.next(p >> 12, &self.data);
        if i == usize::MAX {
            i
        } else {
            let i = (i << 6) | self.word(self.middle + (i)).trailing_zeros() as usize;
            (i << 6) | self.word(i).trailing_zeros() as usize
        }
    }
}

#[derive(Clone, Debug)]
struct VebNode {
    min: usize,
    max: usize,
    bits: u64,
    shift: u32,
    leaf: usize,
    upper: usize,
    clusters: Vec<VebNode>,
    summary: Option<Box<VebNode>>,
}

impl VebNode {
    fn build(&mut self, first: usize, count: usize, arena: &mut [u64]) {
        if self.shift == 0 {
            for i in 0..count {
                self.bits |= ((arena[first + i] != 0) as u64) << i;
            }
            if self.bits != 0 {
                self.min = self.bits.trailing_zeros() as usize;
                self.max = 63 - self.bits.leading_zeros() as usize;
            }
        } else if self.shift == 6 {
            for j in 0..count.div_ceil(64) {
                let mut bits = 0;
                for i in 0..64.min(count - j * 64) {
                    bits |= ((arena[first + j * 64 + i] != 0) as u64) << i;
                }
                arena[self.leaf + j] = bits;
                self.bits |= ((bits != 0) as u64) << j;
            }
            if self.bits != 0 {
                let lo = self.bits.trailing_zeros() as usize;
                let hi = 63 - self.bits.leading_zeros() as usize;
                self.min = (lo << 6) | arena[self.leaf + lo].trailing_zeros() as usize;
                self.max = (hi << 6) | (63 - arena[self.leaf + hi].leading_zeros() as usize);
                arena[self.leaf + lo] &= !(1 << (self.min & 63));
                if arena[self.leaf + lo] == 0 {
                    self.bits &= !(1 << lo);
                }
            }
        } else {
            for i in 0..count {
                if arena[first + i] != 0 {
                    self.insert(i, arena);
                }
            }
        }
    }

    fn new(n: usize, total: &mut usize) -> Self {
        let mut node = Self {
            min: usize::MAX,
            max: 0,
            bits: 0,
            shift: 0,
            leaf: 0,
            upper: 0,
            clusters: Vec::new(),
            summary: None,
        };
        if n > 64 && n <= 4096 {
            node.leaf = *total;
            *total += n.div_ceil(64);
            node.shift = 6;
        } else if n > 4096 && n <= 262144 {
            node.leaf = *total;
            *total += n.div_ceil(64);
            node.upper = *total;
            *total += n.div_ceil(4096);
            node.shift = 12;
        } else if n > 262144 {
            let width = usize::BITS - (n - 1).leading_zeros();
            node.shift = (width.div_ceil(6).div_ceil(2) * 6).max(18);
            let size = 1usize << node.shift;
            let count = n.div_ceil(size);
            node.clusters = (0..count)
                .map(|i| Self::new((n - i * size).min(size), total))
                .collect();
            node.summary = Some(Box::new(Self::new(count, total)));
        }
        node
    }

    #[inline(always)]
    fn insert(&mut self, p: usize, arena: &mut [u64]) {
        if self.shift == 6 {
            return self.insert_packed::<false>(p, arena);
        }
        if self.shift == 12 {
            return self.insert_packed::<true>(p, arena);
        }
        if self.shift == 0 {
            self.bits |= 1 << p;
            self.min = self.bits.trailing_zeros() as usize;
            self.max = 63 - self.bits.leading_zeros() as usize;
            return;
        }
        if self.min == usize::MAX {
            self.min = p;
            self.max = p;
            return;
        }
        self.insert_inner(p, arena);
    }

    #[inline(always)]
    fn insert_packed<const THREE: bool>(&mut self, mut p: usize, arena: &mut [u64]) {
        if self.min == usize::MAX {
            self.min = p;
            self.max = p;
            return;
        }
        if p < self.min {
            std::mem::swap(&mut p, &mut self.min);
        }
        arena[self.leaf + (p >> 6)] |= 1 << (p & 63);
        if !THREE {
            self.bits |= 1 << (p >> 6);
        } else {
            arena[self.upper + (p >> 12)] |= 1 << ((p >> 6) & 63);
            self.bits |= 1 << (p >> 12);
        }
        self.max = self.max.max(p);
        return;
    }

    fn insert_inner(&mut self, mut p: usize, arena: &mut [u64]) {
        if p == self.min || p == self.max {
            return;
        }
        if p < self.min {
            std::mem::swap(&mut p, &mut self.min);
        }
        self.max = self.max.max(p);
        let hi = p >> self.shift;
        let lo = p & ((1usize << self.shift) - 1);
        if self.clusters[hi].min == usize::MAX {
            self.summary.as_mut().unwrap().insert(hi, arena);
        }
        self.clusters[hi].insert(lo, arena);
    }

    #[inline(always)]
    fn remove(&mut self, p: usize, arena: &mut [u64]) {
        if self.shift == 6 {
            return self.remove_packed::<false>(p, arena);
        }
        if self.shift == 12 {
            return self.remove_packed::<true>(p, arena);
        }
        if self.shift == 0 {
            self.bits &= !(1 << p);
            if self.bits == 0 {
                self.min = usize::MAX;
                self.max = 0;
            } else {
                self.min = self.bits.trailing_zeros() as usize;
                self.max = 63 - self.bits.leading_zeros() as usize;
            }
            return;
        }
        if self.min == usize::MAX || self.min == self.max {
            if p == self.min {
                self.min = usize::MAX;
                self.max = 0;
            }
            return;
        }
        self.remove_inner(p, arena);
    }

    #[inline(always)]
    fn remove_packed<const THREE: bool>(&mut self, mut p: usize, arena: &mut [u64]) {
        if self.min == self.max {
            self.min = usize::MAX;
            self.max = 0;
            return;
        }
        if p == self.min {
            let mut i = self.bits.trailing_zeros() as usize;
            if THREE {
                i = (i << 6) | arena[self.upper + i].trailing_zeros() as usize;
            }
            p = (i << 6) | arena[self.leaf + i].trailing_zeros() as usize;
            self.min = p;
        }
        let word = &mut arena[self.leaf + (p >> 6)];
        *word &= !(1 << (p & 63));
        if *word == 0 {
            if !THREE {
                self.bits &= !(1 << (p >> 6));
            } else {
                let upper = &mut arena[self.upper + (p >> 12)];
                *upper &= !(1 << ((p >> 6) & 63));
                if *upper == 0 {
                    self.bits &= !(1 << (p >> 12));
                }
            }
        }
        if self.bits == 0 {
            self.max = self.min;
            return;
        }
        if self.max == p {
            let mut i = 63 - self.bits.leading_zeros() as usize;
            if THREE {
                i = (i << 6) | (63 - arena[self.upper + i].leading_zeros() as usize);
            }
            self.max = (i << 6) | (63 - arena[self.leaf + (i)].leading_zeros() as usize);
        }
        return;
    }

    fn remove_inner(&mut self, mut p: usize, arena: &mut [u64]) {
        if p == self.min {
            let hi = self.summary.as_ref().unwrap().min;
            p = (hi << self.shift) | self.clusters[hi].min;
            self.min = p;
        }
        let hi = p >> self.shift;
        self.clusters[hi].remove(p & ((1usize << self.shift) - 1), arena);
        if self.clusters[hi].min == usize::MAX {
            self.summary.as_mut().unwrap().remove(hi, arena);
        }
        if p == self.max {
            let hi = self.summary.as_ref().unwrap().max;
            self.max = if self.summary.as_ref().unwrap().min == usize::MAX {
                self.min
            } else {
                (hi << self.shift) | self.clusters[hi].max
            };
        }
    }

    #[inline(always)]
    fn next(&self, p: usize, arena: &[u64]) -> usize {
        if p >= self.max {
            return usize::MAX;
        }
        if p < self.min {
            return self.min;
        }
        if self.shift == 0 {
            return (self.bits & ((u64::MAX << p) << 1)).trailing_zeros() as usize;
        }
        if self.shift == 6 {
            return self.next_packed::<false>(p, arena);
        }
        if self.shift == 12 {
            return self.next_packed::<true>(p, arena);
        }
        self.next_inner(p, arena)
    }

    #[inline(always)]
    fn next_packed<const THREE: bool>(&self, p: usize, arena: &[u64]) -> usize {
        let word = arena[self.leaf + (p >> 6)] & (!1u64 << (p & 63));
        if word != 0 {
            return (p & !63) | word.trailing_zeros() as usize;
        }
        let i = if !THREE {
            (self.bits & (!1u64 << (p >> 6))).trailing_zeros() as usize
        } else {
            let word = arena[self.upper + (p >> 12)] & (!1u64 << ((p >> 6) & 63));
            if word != 0 {
                ((p >> 6) & !63) | word.trailing_zeros() as usize
            } else {
                let j = (self.bits & (!1u64 << (p >> 12))).trailing_zeros() as usize;
                (j << 6) | arena[self.upper + j].trailing_zeros() as usize
            }
        };
        return (i << 6) | arena[self.leaf + (i)].trailing_zeros() as usize;
    }

    fn next_inner(&self, p: usize, arena: &[u64]) -> usize {
        let hi = p >> self.shift;
        let lo = p & ((1usize << self.shift) - 1);
        let child = &self.clusters[hi];
        if lo < child.max {
            return (hi << self.shift) | child.next(lo, arena);
        }
        let hi = self.summary.as_ref().unwrap().next(hi, arena);
        (hi << self.shift) | self.clusters[hi].min
    }

    #[inline(always)]
    fn prev(&self, p: usize, arena: &[u64]) -> usize {
        if p <= self.min {
            return usize::MAX;
        }
        if p > self.max {
            return self.max;
        }
        if self.shift == 0 {
            return 63 - (self.bits & ((1u64 << p) - 1)).leading_zeros() as usize;
        }
        if self.shift == 6 {
            return self.prev_packed::<false>(p, arena);
        }
        if self.shift == 12 {
            return self.prev_packed::<true>(p, arena);
        }
        self.prev_inner(p, arena)
    }

    #[inline(always)]
    fn prev_packed<const THREE: bool>(&self, p: usize, arena: &[u64]) -> usize {
        let word = arena[self.leaf + (p >> 6)] & ((1u64 << (p & 63)) - 1);
        if word != 0 {
            return (p & !63) | (63 - word.leading_zeros() as usize);
        }
        let i = if !THREE {
            let bits = self.bits & ((1u64 << (p >> 6)) - 1);
            if bits == 0 {
                return self.min;
            }
            63 - bits.leading_zeros() as usize
        } else {
            let word = arena[self.upper + (p >> 12)] & ((1u64 << ((p >> 6) & 63)) - 1);
            if word != 0 {
                ((p >> 6) & !63) | (63 - word.leading_zeros() as usize)
            } else {
                let bits = self.bits & ((1u64 << (p >> 12)) - 1);
                if bits == 0 {
                    return self.min;
                }
                let j = 63 - bits.leading_zeros() as usize;
                (j << 6) | (63 - arena[self.upper + j].leading_zeros() as usize)
            }
        };
        return (i << 6) | (63 - arena[self.leaf + (i)].leading_zeros() as usize);
    }

    fn prev_inner(&self, p: usize, arena: &[u64]) -> usize {
        let hi = p >> self.shift;
        let lo = p & ((1usize << self.shift) - 1);
        let child = &self.clusters[hi];
        if lo > child.min {
            return (hi << self.shift) | child.prev(lo, arena);
        }
        let hi = self.summary.as_ref().unwrap().prev(hi, arena);
        if hi == usize::MAX {
            self.min
        } else {
            (hi << self.shift) | self.clusters[hi].max
        }
    }
}
