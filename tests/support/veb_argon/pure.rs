#[derive(Clone, Debug)]
pub struct VebTree { mx: usize, root: VebNode }
impl VebTree {
    pub fn new(mx: usize) -> Self { Self { mx: mx.max(1), root: VebNode::new(mx.max(1)) } }
    pub fn from_vec_u64(data: Vec<u64>) -> Self {
        if data.is_empty() { return Self::new(1); }
        let mut result = Self::new(data.len().checked_mul(64).expect("universe size overflow"));
        for (i, mut word) in data.into_iter().enumerate() {
            while word != 0 { result.root.insert((i << 6) | word.trailing_zeros() as usize); word &= word - 1; }
        }
        result
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


    #[inline(always)] pub fn insert(&mut self,p:usize) { assert!(p < self.mx); self.root.insert(p); }
    #[inline(always)] pub fn remove(&mut self,p:usize) { assert!(p < self.mx); if self.root.include(p) { self.root.remove(p); } }
    #[inline(always)] pub fn include(&self,p:usize)->bool { p < self.mx && self.root.include(p) }
    #[inline(always)] pub fn min(&self)->usize { self.root.min }
    #[inline(always)] pub fn max(&self)->usize { self.root.max }
    #[inline(always)] pub fn is_empty(&self)->bool { self.root.min == usize::MAX }
    #[inline(always)] pub fn next(&self,p:usize)->usize { self.root.next(p) }
    #[inline(always)] pub fn prev(&self,p:usize)->usize { self.root.prev(p) }
    #[inline(always)] pub fn innext(&self,p:usize)->usize { if p == 0 { self.min() } else { self.next(p-1) } }
    #[inline(always)] pub fn inprev(&self,p:usize)->usize { self.prev(p.saturating_add(1)) }
}
#[derive(Clone, Debug)]
struct VebNode {
    min: usize,
    max: usize,
    bits: u64,
    shift: u32,
    leaves: Vec<u64>,
    clusters: Vec<VebNode>,
    summary: Option<Box<VebNode>>,
}

impl VebNode {
    #[inline]
    fn include(&self, p: usize) -> bool {
        if self.min == usize::MAX || p < self.min || p > self.max { return false; }
        if p == self.min || p == self.max { return true; }
        if self.shift == 0 { return self.bits & (1 << p) != 0; }
        if !self.leaves.is_empty() { return self.leaves[p >> 6] & (1 << (p & 63)) != 0; }
        self.clusters[p >> self.shift].include(p & ((1usize << self.shift) - 1))
    }

    fn new(n: usize) -> Self {
        let mut node = Self {
            min: usize::MAX,
            max: usize::MAX,
            bits: 0,
            shift: 0,
            leaves: Vec::new(),
            clusters: Vec::new(),
            summary: None,
        };
        if n > 64 && n <= 4096 {
            node.leaves = vec![0; n.div_ceil(64)];
            node.shift = 6;
        } else if n > 4096 {
            let width = usize::BITS - (n - 1).leading_zeros();
            node.shift = width.div_ceil(6).div_ceil(2) * 6;
            let size = 1usize << node.shift;
            let count = n.div_ceil(size);
            node.clusters = (0..count)
                .map(|i| Self::new((n - i * size).min(size)))
                .collect();
            node.summary = Some(Box::new(Self::new(count)));
        }
        node
    }

    #[inline(always)]
    fn insert(&mut self, p: usize) {
        if !self.leaves.is_empty() {
            self.leaves[p >> 6] |= 1 << (p & 63);
            self.bits |= 1 << (p >> 6);
            if self.min == usize::MAX { self.min = p; self.max = p; }
            else { self.min = self.min.min(p); self.max = self.max.max(p); }
            return;
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
        self.insert_inner(p);
    }

    fn insert_inner(&mut self, mut p: usize) {
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
            self.summary.as_mut().unwrap().insert(hi);
        }
        self.clusters[hi].insert(lo);
    }

    #[inline(always)]
    fn remove(&mut self, p: usize) {
        if !self.leaves.is_empty() {
            let word = &mut self.leaves[p >> 6];
            *word &= !(1 << (p & 63));
            if *word == 0 { self.bits &= !(1 << (p >> 6)); }
            if self.bits == 0 { self.min = usize::MAX; self.max = usize::MAX; return; }
            if self.min == p {
                let i = self.bits.trailing_zeros() as usize;
                self.min = (i << 6) | self.leaves[i].trailing_zeros() as usize;
            }
            if self.max == p {
                let i = 63 - self.bits.leading_zeros() as usize;
                self.max = (i << 6) | (63 - self.leaves[i].leading_zeros() as usize);
            }
            return;
        }
        if self.shift == 0 {
            self.bits &= !(1 << p);
            if self.bits == 0 {
                self.min = usize::MAX;
                self.max = usize::MAX;
            } else {
                self.min = self.bits.trailing_zeros() as usize;
                self.max = 63 - self.bits.leading_zeros() as usize;
            }
            return;
        }
        if self.min == self.max {
            if p == self.min {
                self.min = usize::MAX;
                self.max = usize::MAX;
            }
            return;
        }
        self.remove_inner(p);
    }

    fn remove_inner(&mut self, mut p: usize) {
        if p == self.min {
            let hi = self.summary.as_ref().unwrap().min;
            p = (hi << self.shift) | self.clusters[hi].min;
            self.min = p;
        }
        let hi = p >> self.shift;
        self.clusters[hi].remove(p & ((1usize << self.shift) - 1));
        if self.clusters[hi].min == usize::MAX {
            self.summary.as_mut().unwrap().remove(hi);
        }
        if p == self.max {
            let hi = self.summary.as_ref().unwrap().max;
            self.max = if hi == usize::MAX {
                self.min
            } else {
                (hi << self.shift) | self.clusters[hi].max
            };
        }
    }

    #[inline(always)]
    fn next(&self, p: usize) -> usize {
        if self.min == usize::MAX || p >= self.max {
            return usize::MAX;
        }
        if p < self.min {
            return self.min;
        }
        if self.shift == 0 {
            return (self.bits & ((u64::MAX << p) << 1)).trailing_zeros() as usize;
        }
        if !self.leaves.is_empty() {
            let word = self.leaves[p >> 6] & (!1u64 << (p & 63));
            if word != 0 { return (p & !63) | word.trailing_zeros() as usize; }
            let i = (self.bits & (!1u64 << (p >> 6))).trailing_zeros() as usize;
            return (i << 6) | self.leaves[i].trailing_zeros() as usize;
        }
        self.next_inner(p)
    }

    fn next_inner(&self, p: usize) -> usize {
        let hi = p >> self.shift;
        let lo = p & ((1usize << self.shift) - 1);
        let child = &self.clusters[hi];
        if child.min != usize::MAX && lo < child.max {
            return (hi << self.shift) | child.next(lo);
        }
        let hi = self.summary.as_ref().unwrap().next(hi);
        (hi << self.shift) | self.clusters[hi].min
    }

    #[inline(always)]
    fn prev(&self, p: usize) -> usize {
        if self.min == usize::MAX || p <= self.min {
            return usize::MAX;
        }
        if p > self.max {
            return self.max;
        }
        if self.shift == 0 {
            return 63 - (self.bits & ((1u64 << p) - 1)).leading_zeros() as usize;
        }
        if !self.leaves.is_empty() {
            let word = self.leaves[p >> 6] & ((1u64 << (p & 63)) - 1);
            if word != 0 { return (p & !63) | (63 - word.leading_zeros() as usize); }
            let i = 63 - (self.bits & ((1u64 << (p >> 6)) - 1)).leading_zeros() as usize;
            return (i << 6) | (63 - self.leaves[i].leading_zeros() as usize);
        }
        self.prev_inner(p)
    }

    fn prev_inner(&self, p: usize) -> usize {
        let hi = p >> self.shift;
        let lo = p & ((1usize << self.shift) - 1);
        let child = &self.clusters[hi];
        if lo > child.min {
            return (hi << self.shift) | child.prev(lo);
        }
        let hi = self.summary.as_ref().unwrap().prev(hi);
        if hi == usize::MAX {
            self.min
        } else {
            (hi << self.shift) | self.clusters[hi].max
        }
    }
}
