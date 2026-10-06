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
    /// 更新はイテレータの作成・消費に伴って進む。Drop でも更新を完了する。
    /// 記録を保存したい場合は collect::<Vec<_>>() を使う。
    pub fn insert_with_data(&mut self, mut l: T, r: T)->impl Iterator<Item = (T, T, bool)> + '_ {
        let mut prefix = [None; 3];
        if l >= r {
            return IntervalSetChanges::new(None, prefix, None);
        }
        if let Some((&ll, &lr)) = self.s.range(..=l).next_back() {
            if r <= lr {
                return IntervalSetChanges::new(None, prefix, None);
            }
            if l <= lr {
                prefix[0] = Some((ll, lr, false));
                l = ll;
            }
        }
        let bounds = (std::ops::Bound::Excluded(l), std::ops::Bound::Included(r));
        // 既存区間は隣接もしないため、r を越えて伸びるのは最後の一つだけ。
        let last = self.s.range(bounds).next_back().map(|(_, &nr)| nr);
        let end = last.map_or(r, |nr| r.max(nr));
        // 新しい区間の左端は抽出範囲から除外される。
        self.s.insert(l, end);
        let removed = last.map(|_| self.s.extract_if(bounds, interval_set_extract_all::<T> as fn(&T, &mut T) -> bool));
        IntervalSetChanges::new(removed, prefix, Some((l, end, true)))
    }

    /// 変更記録 (左端, 右端, 追加なら true) を順に返す。
    /// 更新はイテレータの作成・消費に伴って進む。Drop でも更新を完了する。
    /// 記録を保存したい場合は collect::<Vec<_>>() を使う。
    pub fn remove_with_data(&mut self, l: T, r: T)->impl Iterator<Item = (T, T, bool)> + '_ {
        let mut prefix = [None; 3];
        if l >= r {
            return IntervalSetChanges::new(None, prefix, None);
        }
        if let Some((&ll, end)) = self.s.range_mut(..=l).next_back() {
            let lr = *end;
            if l < lr {
                prefix[0] = Some((ll, lr, false));
                let mut count = 1;
                if ll < l {
                    *end = l;
                    prefix[count] = Some((ll, l, true));
                    count += 1;
                } else {
                    self.s.remove(&ll);
                }
                if r <= lr {
                    if r < lr {
                        self.s.insert(r, lr);
                        prefix[count] = Some((r, lr, true));
                    }
                    return IntervalSetChanges::new(None, prefix, None);
                }
            }
        }
        let bounds = (std::ops::Bound::Included(l), std::ops::Bound::Excluded(r));
        let last = self.s.range(bounds).next_back().map(|(_, &nr)| nr);
        let suffix = last.filter(|&nr| r < nr).map(|nr| {
            self.s.insert(r, nr);
            (r, nr, true)
        });
        let removed = last.map(|_| self.s.extract_if(bounds, interval_set_extract_all::<T> as fn(&T, &mut T) -> bool));
        IntervalSetChanges::new(removed, prefix, suffix)
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

// Rust 1.91 以降の BTreeMap::extract_if を使用する。
fn interval_set_extract_all<T>(_: &T, _: &mut T) -> bool { true }

type IntervalSetExtract<'a, T> = std::collections::btree_map::ExtractIf<
    'a, T, T, (std::ops::Bound<T>, std::ops::Bound<T>), fn(&T, &mut T) -> bool,
>;

// 境界の分割記録は最大三つ。抽出中は木の走査位置を extract_if が保持する。
struct IntervalSetChanges<'a, T: Ord + Copy> {
    removed: Option<IntervalSetExtract<'a, T>>,
    prefix: [Option<(T, T, bool)>; 3],
    prefix_index: usize,
    suffix: Option<(T, T, bool)>,
}

impl<'a, T: Ord + Copy> IntervalSetChanges<'a, T> {
    fn new(removed: Option<IntervalSetExtract<'a, T>>, prefix: [Option<(T, T, bool)>; 3], suffix: Option<(T, T, bool)>) -> Self {
        Self { removed, prefix, prefix_index: 0, suffix }
    }
}

impl<T: Ord + Copy> Iterator for IntervalSetChanges<'_, T> {
    type Item = (T, T, bool);

    fn next(&mut self) -> Option<Self::Item> {
        if self.prefix_index < self.prefix.len() {
            if let Some(change) = self.prefix[self.prefix_index].take() {
                self.prefix_index += 1;
                return Some(change);
            }
            self.prefix_index = self.prefix.len();
        }
        if let Some(removed) = &mut self.removed {
            if let Some((l, r)) = removed.next() {
                return Some((l, r, false));
            }
            self.removed = None;
        }
        self.suffix.take()
    }
}

impl<T: Ord + Copy> Drop for IntervalSetChanges<'_, T> {
    fn drop(&mut self) {
        if let Some(removed) = &mut self.removed {
            for _ in removed {}
        }
    }
}
