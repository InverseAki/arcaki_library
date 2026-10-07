#[derive(Clone)]
pub struct IntervalSet<T: Ord+Copy>{
    s: BTreeMap<T, T>,
}

impl<T> IntervalSet<T> where T: Ord+Copy {
    pub fn new()->Self {
        IntervalSet { s:BTreeMap::new(), }
    }

    pub fn insert(&mut self, mut l: T, mut r: T){
        if l >= r{return;}
        if let Some((&ll, &lr)) = self.s.range(..=l).next_back(){
            if r <= lr{
                return;
            } else if l <= lr{
                self.s.remove(&ll);
                l = ll;
            }
        }
        while let Some((&nl, &nr)) = self.s.range(l..).next(){
            if nr <= r{
                self.s.remove(&nl);
            } else if nl <= r{
                self.s.remove(&nl);
                r = nr;
                continue;
            } else {
                break;
            }
        }
        if l < r{
            self.s.insert(l, r);
        }
    }

    pub fn remove(&mut self, l: T, r: T){
        if l >= r{return;}
        if let Some((&ll, end)) = self.s.range_mut(..=l).next_back(){
            let lr = *end;
            if l < lr{
                if ll < l{
                    *end = l;
                } else {
                    self.s.remove(&ll);
                }
                if r <= lr{
                    if r < lr{
                        self.s.insert(r, lr);
                    }
                    return;
                }
            }
        }
        while let Some((&nl, &nr)) = self.s.range(l..).next(){
            if nr <= r{
                self.s.remove(&nl);
            } else if nl < r{
                self.s.remove(&nl);
                self.s.insert(r, nr);
                continue;
            } else {
                break;
            }
        } 
    }

    pub fn insert_with_data(&mut self, l: T, r: T)->impl Iterator<Item = (T, T, bool)> + '_ {
        IntervalSetChanges::new(&mut self.s, l, r, true)
    }

    pub fn remove_with_data(&mut self, l: T, r: T)->impl Iterator<Item = (T, T, bool)> + '_ {
        IntervalSetChanges::new(&mut self.s, l, r, false)
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

struct IntervalSetChanges<'a, T: Ord + Copy> {
    s: &'a mut BTreeMap<T, T>,
    l: T,
    r: T,
    inserting: bool,
    started: bool,
    done: bool,
    pending: [Option<(T, T, bool)>; 2],
}

impl<'a, T: Ord + Copy> IntervalSetChanges<'a, T> {
    fn new(s: &'a mut BTreeMap<T, T>, l: T, r: T, inserting: bool) -> Self {
        Self { s, l, r, inserting, started: false, done: l >= r, pending: [None; 2] }
    }
}

impl<T: Ord + Copy> Iterator for IntervalSetChanges<'_, T> {
    type Item = (T, T, bool);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(change) = self.pending[0].take() {
            self.pending[0] = self.pending[1].take();
            return Some(change);
        }
        if self.done {
            return None;
        }
        if !self.started {
            self.started = true;
            if let Some((&ll, end)) = self.s.range_mut(..=self.l).next_back() {
                let lr = *end;
                if !self.inserting && ll < self.l && self.l < lr {
                    *end = self.l;
                }
                if self.inserting {
                    if self.r <= lr {
                        self.done = true;
                        return None;
                    }
                    if self.l <= lr {
                        self.s.remove(&ll);
                        self.l = ll;
                        return Some((ll, lr, false));
                    }
                } else if self.l < lr {
                    let mut count = 0;
                    if ll < self.l {
                        self.pending[count] = Some((ll, self.l, true));
                        count += 1;
                    } else {
                        self.s.remove(&ll);
                    }
                    if self.r <= lr {
                        if self.r < lr {
                            self.s.insert(self.r, lr);
                            self.pending[count] = Some((self.r, lr, true));
                        }
                        self.done = true;
                    }
                    return Some((ll, lr, false));
                }
            }
        }
        if let Some((&nl, &nr)) = self.s.range(self.l..).next() {
            if self.inserting {
                if nl <= self.r {
                    self.s.remove(&nl);
                    if self.r < nr {
                        self.r = nr;
                    }
                    return Some((nl, nr, false));
                }
            } else if nl < self.r {
                self.s.remove(&nl);
                if self.r < nr {
                    self.s.insert(self.r, nr);
                    self.pending[0] = Some((self.r, nr, true));
                    self.done = true;
                }
                return Some((nl, nr, false));
            }
        }
        self.done = true;
        if self.inserting {
            self.s.insert(self.l, self.r);
            Some((self.l, self.r, true))
        } else {
            None
        }
    }
}

impl<T: Ord + Copy> Drop for IntervalSetChanges<'_, T> {
    fn drop(&mut self) {
        while self.next().is_some() {}
    }
}
