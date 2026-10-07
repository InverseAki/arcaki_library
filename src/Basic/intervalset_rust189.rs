#[derive(Clone)]
pub struct IntervalSet<T: Ord+Copy>{
    s: BTreeMap<T, T>,
}

impl<T> IntervalSet<T> where T: Ord+Copy {
    pub fn new()->Self {
        IntervalSet { s:BTreeMap::new(), }
    }

    pub fn insert(&mut self, l: T, r: T){
        drop(self.insert_with_data(l, r));
    }

    pub fn remove(&mut self, l: T, r: T){
        drop(self.remove_with_data(l, r));
    }

    pub fn insert_with_data(&mut self, l: T, r: T)->impl Iterator<Item = (T, T)> + '_ {
        IntervalSetAdded { s: &mut self.s, l, r, scan: l, gap: l, started: false, done: l >= r }
    }

    pub fn remove_with_data(&mut self, l: T, r: T)->impl Iterator<Item = (T, T)> + '_ {
        IntervalSetRemoved { s: &mut self.s, l, r, scan: l, started: false, done: l >= r }
    }

    pub fn contains(&self, p: T)->bool{
        if let Some((&_, &r)) = self.s.range(..=p).next_back(){
            p < r
        } else {
            false
        }
    }

    pub fn section(&self, p: T)->Option<(T, T)>{
        if let Some((&l, &r)) = self.s.range(..=p).next_back(){
            Some((l, r))
        } else {
            None
        }
    }
}

struct IntervalSetAdded<'a, T: Ord + Copy> {
    s: &'a mut BTreeMap<T, T>,
    l: T,
    r: T,
    scan: T,
    gap: T,
    started: bool,
    done: bool,
}

impl<T: Ord + Copy> Iterator for IntervalSetAdded<'_, T> {
    type Item = (T, T);
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.done { return None; }
        if !self.started {
            self.started = true;
            if let Some((&ll, &lr)) = self.s.range(..=self.l).next_back() {
                if self.r <= lr {
                    self.done = true;
                    return None;
                }
                if self.l <= lr {
                    self.l = ll;
                    self.scan = lr;
                    self.gap = lr;
                }
            }
        }
        while let Some((&nl, &nr)) = self.s.range(self.scan..).next() {
            if self.r < nl { break; }
            let a = self.gap;
            let b = nl.min(self.r);
            self.s.remove(&nl);
            self.scan = nr;
            self.gap = nr;
            self.r = self.r.max(nr);
            if a < b { return Some((a, b)); }
        }
        self.s.insert(self.l, self.r);
        self.done = true;
        if self.gap < self.r { Some((self.gap, self.r)) } else { None }
    }
}

impl<T: Ord + Copy> Drop for IntervalSetAdded<'_, T> {
    fn drop(&mut self) {
        if !self.started && !self.done { let _ = self.next(); }
        if self.done { return; }
        while let Some((&nl, &nr)) = self.s.range(self.scan..).next() {
            if self.r < nl { break; }
            self.s.remove(&nl);
            self.scan = nr;
            self.r = self.r.max(nr);
        }
        self.s.insert(self.l, self.r);
    }
}

struct IntervalSetRemoved<'a, T: Ord + Copy> {
    s: &'a mut BTreeMap<T, T>,
    l: T,
    r: T,
    scan: T,
    started: bool,
    done: bool,
}

impl<T: Ord + Copy> Iterator for IntervalSetRemoved<'_, T> {
    type Item = (T, T);
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.done { return None; }
        if !self.started {
            self.started = true;
            if let Some((&ll, end)) = self.s.range_mut(..=self.l).next_back() {
                let lr = *end;
                if self.l < lr {
                    if ll < self.l { *end = self.l; }
                    else { self.s.remove(&ll); }
                    self.scan = lr;
                    if self.r <= lr {
                        if self.r < lr { self.s.insert(self.r, lr); }
                        self.done = true;
                    }
                    return Some((self.l, lr.min(self.r)));
                }
            }
        }
        if let Some((&nl, &nr)) = self.s.range(self.scan..).next() {
            if nl < self.r {
                self.s.remove(&nl);
                self.scan = nr;
                if self.r < nr {
                    self.s.insert(self.r, nr);
                    self.done = true;
                }
                return Some((nl, nr.min(self.r)));
            }
        }
        self.done = true;
        None
    }
}

impl<T: Ord + Copy> Drop for IntervalSetRemoved<'_, T> {
    fn drop(&mut self) {
        if !self.started && !self.done { let _ = self.next(); }
        if self.done { return; }
        while let Some((&nl, &nr)) = self.s.range(self.scan..).next() {
            if self.r <= nl { break; }
            self.s.remove(&nl);
            self.scan = nr;
            if self.r < nr {
                self.s.insert(self.r, nr);
                break;
            }
        }
    }
}
