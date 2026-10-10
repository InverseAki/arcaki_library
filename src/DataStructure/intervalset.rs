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

    pub fn insert_with_data(&mut self, mut l: T, r: T)->impl Iterator<Item = (T, T)> + '_ {
        let mut gap = l;
        if l >= r {
            return IntervalSetAdded::new(None, r, r);
        }
        if let Some((&ll, &lr)) = self.s.range(..=l).next_back() {
            if r <= lr {
                return IntervalSetAdded::new(None, r, r);
            }
            if l <= lr {
                gap = lr;
                l = ll;
            }
        }
        let bounds = (std::ops::Bound::Excluded(l), std::ops::Bound::Included(r));
        let last = self.s.range(bounds).next_back().map(|(_, &nr)| nr);
        let end = last.map_or(r, |nr| r.max(nr));
        self.s.insert(l, end);
        let removed = last.map(|_| self.s.extract_if(bounds, interval_set_extract_all::<T> as fn(&T, &mut T) -> bool));
        IntervalSetAdded::new(removed, gap, r)
    }

    pub fn remove_with_data(&mut self, l: T, r: T)->impl Iterator<Item = (T, T)> + '_ {
        let mut prefix = None;
        if l >= r {
            return IntervalSetRemoved::new(None, prefix, r);
        }
        if let Some((&ll, end)) = self.s.range_mut(..=l).next_back() {
            let lr = *end;
            if l < lr {
                prefix = Some((l, lr.min(r)));
                if ll < l { *end = l; }
                else { self.s.remove(&ll); }
                if r <= lr {
                    if r < lr { self.s.insert(r, lr); }
                    return IntervalSetRemoved::new(None, prefix, r);
                }
            }
        }
        let bounds = (std::ops::Bound::Included(l), std::ops::Bound::Excluded(r));
        let last = self.s.range(bounds).next_back().map(|(_, &nr)| nr);
        if let Some(nr) = last.filter(|&nr| r < nr) { self.s.insert(r, nr); }
        let removed = last.map(|_| self.s.extract_if(bounds, interval_set_extract_all::<T> as fn(&T, &mut T) -> bool));
        IntervalSetRemoved::new(removed, prefix, r)
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

fn interval_set_extract_all<T>(_: &T, _: &mut T) -> bool { true }

type IntervalSetExtract<'a, T> = std::collections::btree_map::ExtractIf<
    'a, T, T, (std::ops::Bound<T>, std::ops::Bound<T>), fn(&T, &mut T) -> bool,
>;

struct IntervalSetAdded<'a, T: Ord + Copy> {
    removed: Option<IntervalSetExtract<'a, T>>,
    gap: T,
    r: T,
    done: bool,
}

impl<'a, T: Ord + Copy> IntervalSetAdded<'a, T> {
    fn new(removed: Option<IntervalSetExtract<'a, T>>, gap: T, r: T) -> Self {
        Self { removed, gap, r, done: false }
    }
}

impl<T: Ord + Copy> Iterator for IntervalSetAdded<'_, T> {
    type Item = (T, T);
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.done { return None; }
        if let Some(removed) = &mut self.removed {
            for (l, r) in removed {
                let a = self.gap;
                let b = l.min(self.r);
                self.gap = self.gap.max(r);
                if a < b { return Some((a, b)); }
            }
            self.removed = None;
        }
        self.done = true;
        if self.gap < self.r { Some((self.gap, self.r)) } else { None }
    }
}

impl<T: Ord + Copy> Drop for IntervalSetAdded<'_, T> {
    fn drop(&mut self) {
        if let Some(removed) = &mut self.removed { for _ in removed {} }
    }
}

struct IntervalSetRemoved<'a, T: Ord + Copy> {
    removed: Option<IntervalSetExtract<'a, T>>,
    prefix: Option<(T, T)>,
    r: T,
}

impl<'a, T: Ord + Copy> IntervalSetRemoved<'a, T> {
    fn new(removed: Option<IntervalSetExtract<'a, T>>, prefix: Option<(T, T)>, r: T) -> Self {
        Self { removed, prefix, r }
    }
}

impl<T: Ord + Copy> Iterator for IntervalSetRemoved<'_, T> {
    type Item = (T, T);
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(change) = self.prefix.take() { return Some(change); }
        if let Some(removed) = &mut self.removed {
            if let Some((l, r)) = removed.next() { return Some((l, r.min(self.r))); }
            self.removed = None;
        }
        None
    }
}

impl<T: Ord + Copy> Drop for IntervalSetRemoved<'_, T> {
    fn drop(&mut self) {
        if let Some(removed) = &mut self.removed { for _ in removed {} }
    }
}
