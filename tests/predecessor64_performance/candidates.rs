#[derive(Clone, Debug)]
pub struct Incremental {
    mx: usize,
    d: Vec<Vec<u64>>,
}

impl Incremental {
    pub fn new(mx: usize) -> Self {
        let mut d = Vec::new();
        let mut len = (mx + 63) >> 6;
        d.push(vec![0; len]);
        while len > 1 {
            len = (len + 63) >> 6;
            d.push(vec![0; len]);
        }
        Self { mx, d }
    }

    #[inline(always)]
    pub fn insert(&mut self, mut p: usize) {
        for level in &mut self.d {
            let word = &mut level[p >> 6];
            let old = *word;
            *word |= 1 << (p & 63);
            if old != 0 { return; }
            p >>= 6;
        }
    }

    #[inline(always)]
    pub fn remove(&mut self, mut p: usize) {
        for level in &mut self.d {
            let word = &mut level[p >> 6];
            let old = *word;
            *word &= !(1 << (p & 63));
            if *word != 0 || old == 0 { return; }
            p >>= 6;
        }
    }

    #[inline(always)]
    pub fn include(&self, p: usize) -> bool { self.d[0][p >> 6] & (1 << (p & 63)) != 0 }
    #[inline(always)]
    pub fn is_empty(&self) -> bool { self.d.last().unwrap()[0] == 0 }
    #[inline(always)]
    pub fn min(&self) -> usize { self.innext(0) }
    #[inline(always)]
    pub fn max(&self) -> usize { self.inprev(self.mx - 1) }
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
        for (i, level) in self.d.iter().enumerate() {
            let mask = if i == 0 && INCLUSIVE {
                u64::MAX >> (63 - (p & 63))
            } else { (1u64 << (p & 63)) - 1 };
            let word = level[p >> 6] & mask;
            if word != 0 {
                let mut res = (p & !63) | (63 - word.leading_zeros() as usize);
                for j in (0..i).rev() {
                    res = (res << 6) | (63 - self.d[j][res].leading_zeros() as usize);
                }
                return res;
            }
            p >>= 6;
        }
        !0
    }

    #[inline(always)]
    fn search_next<const INCLUSIVE: bool>(&self, mut p: usize) -> usize {
        for (i, level) in self.d.iter().enumerate() {
            let mask = if i == 0 && INCLUSIVE {
                u64::MAX << (p & 63)
            } else { (u64::MAX << (p & 63)) << 1 };
            let word = level[p >> 6] & mask;
            if word != 0 {
                let mut res = (p & !63) | word.trailing_zeros() as usize;
                for j in (0..i).rev() {
                    res = (res << 6) | self.d[j][res].trailing_zeros() as usize;
                }
                return res;
            }
            p >>= 6;
        }
        !0
    }
}

#[derive(Clone, Debug)]
pub struct LeafFirst(pub Incremental);

impl LeafFirst {
    pub fn new(mx: usize) -> Self { Self(Incremental::new(mx)) }
    #[inline(always)]
    pub fn insert(&mut self, p: usize) { self.0.insert(p); }
    #[inline(always)]
    pub fn remove(&mut self, p: usize) { self.0.remove(p); }
    #[inline(always)]
    pub fn include(&self, p: usize) -> bool { self.0.include(p) }
    pub fn is_empty(&self) -> bool { self.0.is_empty() }
    pub fn min(&self) -> usize { self.innext(0) }
    pub fn max(&self) -> usize { self.inprev(self.0.mx - 1) }
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
        let mask = if NEXT {
            let m = u64::MAX << (p & 63);
            if INCLUSIVE { m } else { m << 1 }
        } else if INCLUSIVE { u64::MAX >> (63 - (p & 63)) }
        else { (1u64 << (p & 63)) - 1 };
        let word = self.0.d[0][p >> 6] & mask;
        if word != 0 {
            let bit = if NEXT { word.trailing_zeros() as usize }
            else { 63 - word.leading_zeros() as usize };
            return (p & !63) | bit;
        }
        p >>= 6;
        for i in 1..self.0.d.len() {
            let mask = if NEXT { (u64::MAX << (p & 63)) << 1 }
            else { (1u64 << (p & 63)) - 1 };
            let word = self.0.d[i][p >> 6] & mask;
            if word != 0 {
                let bit = if NEXT { word.trailing_zeros() as usize }
                else { 63 - word.leading_zeros() as usize };
                let mut res = (p & !63) | bit;
                for j in (0..i).rev() {
                    let word = self.0.d[j][res];
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

#[derive(Clone, Debug)]
pub struct Shifted(pub Incremental);

impl Shifted {
    pub fn new(mx: usize) -> Self { Self(Incremental::new(mx)) }
    #[inline(always)]
    pub fn insert(&mut self, p: usize) { self.0.insert(p); }
    #[inline(always)]
    pub fn remove(&mut self, p: usize) { self.0.remove(p); }
    #[inline(always)]
    pub fn include(&self, p: usize) -> bool { self.0.include(p) }
    pub fn is_empty(&self) -> bool { self.0.is_empty() }
    pub fn min(&self) -> usize { self.innext(0) }
    pub fn max(&self) -> usize { self.inprev(self.0.mx - 1) }
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
        let word = self.0.d[0][p >> 6];
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
        for i in 1..self.0.d.len() {
            let word = self.0.d[i][p >> 6];
            let word = if NEXT { (word >> (p & 63)) >> 1 }
            else { (word << (63 - (p & 63))) << 1 };
            if word != 0 {
                let mut res = if NEXT { p + word.trailing_zeros() as usize + 1 }
                else { p - word.leading_zeros() as usize - 1 };
                for j in (0..i).rev() {
                    let word = self.0.d[j][res];
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
