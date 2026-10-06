// Rust 1.89.0 対応。隣接をマージするタイプ。
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

    /// 変更記録 (左端, 右端, 追加なら true) を順に返す。
    /// 更新は遅延実行される。途中で破棄した場合も Drop で更新を完了する。
    /// 記録を保存したい場合は collect::<Vec<_>>() を使う。
    pub fn insert_with_data(&mut self, l: T, r: T)->impl Iterator<Item = (T, T, bool)> + '_ {
        IntervalSetChanges::new(&mut self.s, l, r, true)
    }

    /// 変更記録 (左端, 右端, 追加なら true) を順に返す。
    /// 更新は遅延実行される。途中で破棄した場合も Drop で更新を完了する。
    /// 記録を保存したい場合は collect::<Vec<_>>() を使う。
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

// 分割で発生する追加記録は最大二つなので、固定長バッファで保持する。
struct IntervalSetChanges<'a, T: Ord + Copy> {
    s: &'a mut BTreeMap<T, T>,
    l: T,
    r: T,
    // 木の位置ではなく、次回の根からの検索に使う下限。
    scan: T,
    inserting: bool,
    started: bool,
    done: bool,
    pending: [Option<(T, T, bool)>; 2],
}

impl<'a, T: Ord + Copy> IntervalSetChanges<'a, T> {
    fn new(s: &'a mut BTreeMap<T, T>, l: T, r: T, inserting: bool) -> Self {
        Self { s, l, r, scan: l, inserting, started: false, done: l >= r, pending: [None; 2] }
    }
}

impl<T: Ord + Copy> Iterator for IntervalSetChanges<'_, T> {
    type Item = (T, T, bool);

    #[inline]
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
                if self.l <= lr { self.scan = lr; }
                // 左端を維持できる短縮は、検索で得た値をその場で書き換える。
                if !self.inserting && ll < self.l && self.l < lr {
                    *end = self.l;
                }
                if self.inserting {
                    if self.r <= lr {
                        self.done = true;
                        return None;
                    }
                    if self.l <= lr {
                        // 左端のキーは維持し、最後に右端だけ上書きする。
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
        if let Some((&nl, &nr)) = self.s.range(self.scan..).next() {
            if self.inserting {
                if nl <= self.r {
                    self.s.remove(&nl);
                    self.scan = nr;
                    if self.r < nr {
                        self.r = nr;
                    }
                    return Some((nl, nr, false));
                }
            } else if nl < self.r {
                self.s.remove(&nl);
                self.scan = nr;
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
        if !self.started && !self.done { let _ = self.next(); }
        if self.done { return; }
        // 記録が不要な残りは、next の状態遷移を区間ごとに通さず処理する。
        while let Some((&nl, &nr)) = self.s.range(self.scan..).next() {
            if self.inserting {
                if self.r < nl { break; }
                self.s.remove(&nl);
                self.scan = nr;
                self.r = self.r.max(nr);
            } else {
                if self.r <= nl { break; }
                self.s.remove(&nl);
                self.scan = nr;
                if self.r < nr {
                    self.s.insert(self.r, nr);
                    break;
                }
            }
        }
        if self.inserting { self.s.insert(self.l, self.r); }
    }
}
