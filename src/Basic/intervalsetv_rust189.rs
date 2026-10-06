// Rust 1.89.0 対応。extract_if は使用しない。
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

    /// (左端, 右端, 値, 追加なら true) を順に返す。Vec は確保しない。
    /// 更新は消費時に進み、途中で破棄した場合も Drop で完了する。
    pub fn insert_with_data(&mut self, l: T, r: T, v: V) -> impl Iterator<Item = (T, T, V, bool)> + '_ {
        IntervalSetVChanges::new(&mut self.s, l, r, Some(v))
    }

    /// (左端, 右端, 値, 追加なら true) を順に返す。Vec は確保しない。
    /// 更新は消費時に進み、途中で破棄した場合も Drop で完了する。
    pub fn remove_with_data(&mut self, l: T, r: T) -> impl Iterator<Item = (T, T, V, bool)> + '_ {
        IntervalSetVChanges::new(&mut self.s, l, r, None)
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

// 0: 左境界、1: 中間区間の走査、2: 挿入の完了、3: 終了。
struct IntervalSetVChanges<'a, T: Ord + Copy, V: Eq + Copy> {
    s: &'a mut BTreeMap<T, (T, V)>,
    l: T,
    r: T,
    // 木の位置ではなく、次回の根からの検索に使う下限。
    scan: T,
    value: Option<V>,
    phase: u8,
    pending: [Option<(T, T, V, bool)>; 2],
}

impl<'a, T: Ord + Copy, V: Eq + Copy> IntervalSetVChanges<'a, T, V> {
    fn new(s: &'a mut BTreeMap<T, (T, V)>, l: T, r: T, value: Option<V>) -> Self {
        Self { s, l, r, scan: l, value, phase: if l >= r { 3 } else { 0 }, pending: [None; 2] }
    }
}

impl<T: Ord + Copy, V: Eq + Copy> Iterator for IntervalSetVChanges<'_, T, V> {
    type Item = (T, T, V, bool);
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(change) = self.pending[0].take() {
            self.pending[0] = self.pending[1].take();
            return Some(change);
        }
        if self.phase == 3 { return None; }
        if self.phase == 0 {
            self.phase = 1;
            if let Some((&ll, end)) = self.s.range_mut(..=self.l).next_back() {
                let (lr, lv) = *end;
                if self.l <= lr { self.scan = lr; }
                if self.value == Some(lv) && self.l <= lr {
                    // キーを維持し、最後に右端を上書きする。
                    self.l = ll;
                    self.r = self.r.max(lr);
                    return Some((ll, lr, lv, false));
                } else if self.l < lr {
                    let mut count = 0;
                    if ll < self.l {
                        end.0 = self.l;
                        self.pending[count] = Some((ll, self.l, lv, true));
                        count += 1;
                    } else {
                        self.s.remove(&ll);
                    }
                    if self.r <= lr {
                        if self.r < lr {
                            self.s.insert(self.r, (lr, lv));
                            self.pending[count] = Some((self.r, lr, lv, true));
                        }
                        self.phase = if self.value.is_none() { 3 }
                            else if self.r < lr { 2 } else { 1 };
                    }
                    return Some((ll, lr, lv, false));
                }
            }
        }
        if self.phase == 1 {
            if let Some((&nl, &(nr, nv))) = self.s.range(self.scan..).next() {
                if (self.value == Some(nv) && nl <= self.r) || nl < self.r {
                    self.s.remove(&nl);
                    self.scan = nr;
                    if self.value == Some(nv) {
                        self.r = self.r.max(nr);
                    } else if self.r < nr {
                        self.s.insert(self.r, (nr, nv));
                        self.pending[0] = Some((self.r, nr, nv, true));
                        self.phase = if self.value.is_some() { 2 } else { 3 };
                    }
                    return Some((nl, nr, nv, false));
                }
            }
            self.phase = if self.value.is_some() { 2 } else { 3 };
        }
        if self.phase == 2 {
            self.phase = 3;
            let v = self.value.unwrap();
            self.s.insert(self.l, (self.r, v));
            return Some((self.l, self.r, v, true));
        }
        None
    }
}

impl<T: Ord + Copy, V: Eq + Copy> Drop for IntervalSetVChanges<'_, T, V> {
    fn drop(&mut self) {
        if self.phase == 0 { let _ = self.next(); }
        // すでに適用した pending の記録は捨て、残りの更新だけを進める。
        if self.phase == 1 {
            if let Some(v) = self.value {
                while let Some((&nl, &(nr, nv))) = self.s.range(self.scan..).next() {
                    if nv == v && nl <= self.r {
                        self.s.remove(&nl);
                        self.scan = nr;
                        self.r = self.r.max(nr);
                    } else if nl < self.r {
                        self.s.remove(&nl);
                        self.scan = nr;
                        if self.r < nr {
                            self.s.insert(self.r, (nr, nv));
                            break;
                        }
                    } else { break; }
                }
                self.phase = 2;
            } else {
                while let Some((&nl, &(nr, nv))) = self.s.range(self.scan..).next() {
                    if self.r <= nl { break; }
                    self.s.remove(&nl);
                    self.scan = nr;
                    if self.r < nr {
                        self.s.insert(self.r, (nr, nv));
                        break;
                    }
                }
                self.phase = 3;
            }
        }
        if self.phase == 2 {
            self.s.insert(self.l, (self.r, self.value.unwrap()));
        }
    }
}
