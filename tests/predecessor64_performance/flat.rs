#[derive(Clone, Debug)]
pub struct Flat {
    mx: usize,
    data: Vec<u64>,
    offset: [usize; 11],
    levels: usize,
}

impl Flat {
    pub fn new(mx: usize) -> Self {
        let mut offset = [0; 11];
        let mut levels = 0;
        let mut len = (mx + 63) >> 6;
        let mut total = 0;
        loop {
            offset[levels] = total;
            levels += 1;
            total += len;
            if len <= 1 { break; }
            len = (len + 63) >> 6;
        }
        Self { mx, data: vec![0; total], offset, levels }
    }

    #[inline(always)]
    pub fn insert(&mut self, mut p: usize) {
        for i in 0..self.levels {
            let word = &mut self.data[self.offset[i] + (p >> 6)];
            let old = *word;
            *word |= 1 << (p & 63);
            if old != 0 { return; }
            p >>= 6;
        }
    }

    #[inline(always)]
    pub fn remove(&mut self, mut p: usize) {
        for i in 0..self.levels {
            let word = &mut self.data[self.offset[i] + (p >> 6)];
            let old = *word;
            *word &= !(1 << (p & 63));
            if *word != 0 || old == 0 { return; }
            p >>= 6;
        }
    }

    #[inline(always)]
    pub fn include(&self, p: usize) -> bool { self.data[p >> 6] & (1 << (p & 63)) != 0 }
    #[inline(always)]
    pub fn is_empty(&self) -> bool { self.data[self.offset[self.levels - 1]] == 0 }
    #[inline(always)]
    pub fn min(&self) -> usize { self.extreme::<true>() }
    #[inline(always)]
    pub fn max(&self) -> usize { self.extreme::<false>() }
    #[inline(always)]
    fn extreme<const MIN: bool>(&self) -> usize {
        let leaf_index = if MIN { 0 } else { (self.mx - 1) >> 6 };
        let leaf = self.data[leaf_index];
        if leaf != 0 {
            let bit = if MIN { leaf.trailing_zeros() as usize }
                else { 63 - leaf.leading_zeros() as usize };
            return (leaf_index << 6) | bit;
        }
        let mut word = self.data[self.offset[self.levels - 1]];
        if word == 0 { return !0; }
        let mut p = if MIN { word.trailing_zeros() as usize }
            else { 63 - word.leading_zeros() as usize };
        for i in (0..self.levels - 1).rev() {
            word = self.data[self.offset[i] + p];
            let bit = if MIN { word.trailing_zeros() as usize }
                else { 63 - word.leading_zeros() as usize };
            p = (p << 6) | bit;
        }
        p
    }
    #[inline(always)]
    pub fn prev(&self, p: usize) -> usize { self.search_prev::<false>(p) }
    #[inline(always)]
    pub fn inprev(&self, p: usize) -> usize { self.search_prev::<true>(p) }
    #[inline(always)]
    pub fn next(&self, p: usize) -> usize { self.search_next::<false>(p) }
    #[inline(always)]
    pub fn innext(&self, p: usize) -> usize { self.search_next::<true>(p) }

    #[inline(always)]
    fn search_prev<const INCLUSIVE: bool>(&self, mut p: usize) -> usize {
        for i in 0..self.levels {
            let mask = if i == 0 && INCLUSIVE {
                u64::MAX >> (63 - (p & 63))
            } else { (1u64 << (p & 63)) - 1 };
            let word = self.data[self.offset[i] + (p >> 6)] & mask;
            if word != 0 {
                let mut res = (p & !63) | (63 - word.leading_zeros() as usize);
                for j in (0..i).rev() {
                    res = (res << 6) | (63 - self.data[self.offset[j] + res].leading_zeros() as usize);
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
            } else { (u64::MAX << (p & 63)) << 1 };
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


#[derive(Clone, Debug)]
pub struct FlatShifted(pub Flat);

impl FlatShifted {
    pub fn new(mx: usize) -> Self { Self(Flat::new(mx)) }
    #[inline(always)]
    pub fn insert(&mut self, p: usize) { self.0.insert(p); }
    #[inline(always)]
    pub fn remove(&mut self, p: usize) { self.0.remove(p); }
    #[inline(always)]
    pub fn include(&self, p: usize) -> bool { self.0.include(p) }
    pub fn is_empty(&self) -> bool { self.0.is_empty() }
    pub fn min(&self) -> usize { self.0.min() }
    pub fn max(&self) -> usize { self.0.max() }
    #[inline(always)]
    pub fn prev(&self, p: usize) -> usize { self.search::<false, false>(p) }
    #[inline(always)]
    pub fn inprev(&self, p: usize) -> usize { self.search::<false, true>(p) }
    #[inline(always)]
    pub fn next(&self, p: usize) -> usize { self.search::<true, false>(p) }
    #[inline(always)]
    pub fn innext(&self, p: usize) -> usize { self.search::<true, true>(p) }

    #[inline(always)]
    fn search<const NEXT: bool, const INCLUSIVE: bool>(&self, mut p: usize) -> usize {
        let word = self.0.data[p >> 6];
        let word = if NEXT {
            let w = word >> (p & 63);
            if INCLUSIVE { w } else { w >> 1 }
        } else {
            let w = word << (63 - (p & 63));
            if INCLUSIVE { w } else { w << 1 }
        };
        if word != 0 {
            return if NEXT { p + word.trailing_zeros() as usize + (!INCLUSIVE as usize) }
            else { p - word.leading_zeros() as usize - (!INCLUSIVE as usize) };
        }
        p >>= 6;
        for i in 1..self.0.levels {
            let word = self.0.data[self.0.offset[i] + (p >> 6)];
            let word = if NEXT { (word >> (p & 63)) >> 1 }
            else { (word << (63 - (p & 63))) << 1 };
            if word != 0 {
                let mut res = if NEXT { p + word.trailing_zeros() as usize + 1 }
                else { p - word.leading_zeros() as usize - 1 };
                for j in (0..i).rev() {
                    let word = self.0.data[self.0.offset[j] + res];
                    let bit = if NEXT { word.trailing_zeros() as usize }
                    else { 63 - word.leading_zeros() as usize };
                    res = (res << 6) | bit;
                }
                return res;
            }
            p >>= 6;
        }
        !0
    }
}
