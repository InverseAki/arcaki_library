#[derive(Clone)]
pub struct IntervalSetV<T, V> where T: Ord + Copy, V: Eq + Copy{
    s: BTreeMap<T, (T, V)>,
}

impl<T, V> IntervalSetV<T, V> where T: Ord + Copy, V: Eq + Copy{
    pub fn new() -> Self {
        Self { s: BTreeMap::new() }
    }

    pub fn insert(&mut self, l: T, r: T, v: V){
        drop(self.insert_with_data(l, r, v));
    }

    pub fn remove(&mut self, l: T, r: T){
        drop(self.remove_with_data(l, r));
    }

    pub fn insert_with_data(&mut self, mut l: T, mut r: T, v: V) -> impl Iterator<Item = (T, T, V, bool)> + '_ {
        let mut prefix = [None; 3];
        let mut suffix = [None; 2];
        if l >= r {
            return IntervalSetVChanges::new(None, prefix, suffix);
        }
        if let Some((&ll, end)) = self.s.range_mut(..=l).next_back() {
            let (lr, lv) = *end;
            if lv == v && l <= lr {
                prefix[0] = Some((ll, lr, lv, false));
                l = ll;
                r = r.max(lr);
            } else if lv != v && l < lr {
                prefix[0] = Some((ll, lr, lv, false));
                let mut count = 1;
                if ll < l {
                    end.0 = l;
                    prefix[count] = Some((ll, l, lv, true));
                    count += 1;
                } else {
                    self.s.remove(&ll);
                }
                if r < lr {
                    self.s.insert(r, (lr, lv));
                    prefix[count] = Some((r, lr, lv, true));
                }
            }
        }
        let last = self.s.range((std::ops::Bound::Excluded(l), std::ops::Bound::Included(r)))
            .next_back().map(|(&nl, &(nr, nv))| (nl, nr, nv));
        let mut end = r;
        if let Some((nl, nr, nv)) = last {
            if nv == v {
                end = r.max(nr);
            } else if nl < r && r < nr {
                self.s.insert(r, (nr, nv));
                suffix[0] = Some((r, nr, nv, true));
            }
        }
        if r < end {
            for (&nl, &(nr, nv)) in self.s.range(end..) {
                if nl == end && nv == v { end = nr; }
                else { break; }
            }
        }
        self.s.insert(l, (end, v));
        let final_change = Some((l, end, v, true));
        if suffix[0].is_some() { suffix[1] = final_change; }
        else { suffix[0] = final_change; }
        let bounds = (std::ops::Bound::Excluded(l), std::ops::Bound::Excluded(end));
        let has_middle = last.is_some_and(|(nl, _, _)| nl < r) || r < end
            || (last.is_some_and(|(nl, _, _)| nl == r) && self.s.range(bounds).next().is_some());
        let removed = has_middle.then(|| self.s.extract_if(bounds, interval_set_v_extract_all::<T, V> as fn(&T, &mut (T, V)) -> bool));
        IntervalSetVChanges::new(removed, prefix, suffix)
    }

    pub fn remove_with_data(&mut self, l: T, r: T) -> impl Iterator<Item = (T, T, V, bool)> + '_ {
        let mut prefix = [None; 3];
        let mut suffix = [None; 2];
        if l >= r {
            return IntervalSetVChanges::new(None, prefix, suffix);
        }
        if let Some((&ll, end)) = self.s.range_mut(..=l).next_back() {
            let (lr, lv) = *end;
            if l < lr {
                prefix[0] = Some((ll, lr, lv, false));
                let mut count = 1;
                if ll < l {
                    end.0 = l;
                    prefix[count] = Some((ll, l, lv, true));
                    count += 1;
                } else {
                    self.s.remove(&ll);
                }
                if r <= lr {
                    if r < lr {
                        self.s.insert(r, (lr, lv));
                        prefix[count] = Some((r, lr, lv, true));
                    }
                    return IntervalSetVChanges::new(None, prefix, suffix);
                }
            }
        }
        let bounds = (std::ops::Bound::Included(l), std::ops::Bound::Excluded(r));
        let last = self.s.range(bounds).next_back().map(|(_, &(nr, nv))| (nr, nv));
        if let Some((nr, nv)) = last.filter(|&(nr, _)| r < nr) {
            self.s.insert(r, (nr, nv));
            suffix[0] = Some((r, nr, nv, true));
        }
        let removed = last.map(|_| self.s.extract_if(bounds, interval_set_v_extract_all::<T, V> as fn(&T, &mut (T, V)) -> bool));
        IntervalSetVChanges::new(removed, prefix, suffix)
    }

    pub fn contains(&self, p: T) -> bool {
        if let Some((&_l, &(r, _v))) = self.s.range(..=p).next_back() {
            p < r
        } else {
            false
        }
    }

    pub fn get(&self, p: T) -> Option<V> {
        if let Some((&_l, &(r, v))) = self.s.range(..=p).next_back() {
            if p < r {
                Some(v)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn section(&self, p: T) -> Option<(T, T, V)> {
        if let Some((&l, &(r, v))) = self.s.range(..=p).next_back() {
            if p < r {
                Some((l, r, v))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (T, T, V)> + '_ {
        self.s.iter().map(|(&l, &(r, v))| (l, r, v))
    }

    pub fn is_empty(&self) -> bool {
        self.s.is_empty()
    }

    pub fn clear(&mut self) {
        self.s.clear();
    }

    pub fn range(&self, l: T, r: T) -> Vec<(T, T, V)> {
        let mut res = Vec::new();
        if l >= r {return res;}
        if let Some((_, &(lr, lv))) = self.s.range(..=l).next_back() {
            if l < lr {
                let a = l;
                let b = if lr < r { lr } else { r };
                if a < b {
                    res.push((a, b, lv));
                }
            }
        }
        for (&nl, &(nr, nv)) in self.s.range(l..) {
            if nl >= r {
                break;
            }
            let a = nl;
            let b = if nr < r { nr } else { r };
            if a < b {
                res.push((a, b, nv));
            }
        }
        res
    }
}

fn interval_set_v_extract_all<T, V>(_: &T, _: &mut (T, V)) -> bool { true }

type IntervalSetVExtract<'a, T, V> = std::collections::btree_map::ExtractIf<
    'a, T, (T, V), (std::ops::Bound<T>, std::ops::Bound<T>), fn(&T, &mut (T, V)) -> bool,
>;

struct IntervalSetVChanges<'a, T: Ord + Copy, V: Eq + Copy> {
    removed: Option<IntervalSetVExtract<'a, T, V>>,
    prefix: [Option<(T, T, V, bool)>; 3],
    prefix_index: usize,
    suffix: [Option<(T, T, V, bool)>; 2],
    suffix_index: usize,
}

impl<'a, T: Ord + Copy, V: Eq + Copy> IntervalSetVChanges<'a, T, V> {
    fn new(removed: Option<IntervalSetVExtract<'a, T, V>>, prefix: [Option<(T, T, V, bool)>; 3], suffix: [Option<(T, T, V, bool)>; 2]) -> Self {
        Self { removed, prefix, prefix_index: 0, suffix, suffix_index: 0 }
    }
}

impl<T: Ord + Copy, V: Eq + Copy> Iterator for IntervalSetVChanges<'_, T, V> {
    type Item = (T, T, V, bool);
    fn next(&mut self) -> Option<Self::Item> {
        if self.prefix_index < self.prefix.len() {
            if let Some(change) = self.prefix[self.prefix_index].take() {
                self.prefix_index += 1;
                return Some(change);
            }
            self.prefix_index = self.prefix.len();
        }
        if let Some(removed) = &mut self.removed {
            if let Some((l, (r, v))) = removed.next() {
                return Some((l, r, v, false));
            }
            self.removed = None;
        }
        if self.suffix_index < self.suffix.len() {
            let change = self.suffix[self.suffix_index].take();
            self.suffix_index += 1;
            return change;
        }
        None
    }
}

impl<T: Ord + Copy, V: Eq + Copy> Drop for IntervalSetVChanges<'_, T, V> {
    fn drop(&mut self) {
        if let Some(removed) = &mut self.removed {
            for _ in removed {}
        }
    }
}
