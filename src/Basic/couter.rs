/// 個数0のキーは保持しない。len=種類数、total=総個数。
/// 総個数がusizeを超える追加、_exの前提違反はreleaseでもpanic。
#[derive(Debug, Clone)]
pub struct Counter<T> {
    c: usize,
    map: std::collections::BTreeMap<T, usize>,
}
impl<T: Ord> Default for Counter<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord> Counter<T> {
    pub fn new() -> Self {
        Self {
            c: 0,
            map: std::collections::BTreeMap::new(),
        }
    }
    pub fn range<R: std::ops::RangeBounds<T>>(
        &self,
        range: R,
    ) -> std::collections::btree_map::Range<'_, T, usize> {
        self.map.range(range)
    }
    pub fn mi(&self) -> Option<T>
    where
        T: Copy,
    {
        self.map.first_key_value().map(|(&k, _)| k)
    }
    pub fn mx(&self) -> Option<T>
    where
        T: Copy,
    {
        self.map.last_key_value().map(|(&k, _)| k)
    }

    #[inline]
    pub fn one_add(&mut self, x: T) {
        self.add(x, 1);
    }
    #[inline]
    pub fn one_sub(&mut self, x: T) {
        self.sub(x, 1);
    }
    /// 従来どおり「xを最大1個減らし、yを1個追加」。xがなくてもyを追加する。
    pub fn one_update(&mut self, x: T, y: T) {
        self.one_sub(x);
        self.one_add(y);
    }
    #[inline]
    pub fn add(&mut self, x: T, count: usize) {
        if count == 0 {
            return;
        }
        let total = self.c.checked_add(count).expect("total count overflow");
        *self.map.entry(x).or_insert(0) += count;
        self.c = total;
    }
    /// xが存在することが前提。
    #[inline]
    pub fn add_ex(&mut self, x: T, count: usize) {
        let total = self.c.checked_add(count).expect("total count overflow");
        let v = self.map.get_mut(&x).expect("missing key");
        *v += count;
        self.c = total;
    }
    /// 存在する個数だけ減らす。存在しないキーには何もしない。
    #[inline]
    pub fn sub(&mut self, x: T, count: usize) {
        if count == 0 {
            return;
        }
        if let std::collections::btree_map::Entry::Occupied(mut e) = self.map.entry(x) {
            let removed = count.min(*e.get());
            if removed == *e.get() {
                e.remove();
            } else {
                *e.get_mut() -= removed;
            }
            self.c -= removed;
        }
    }
    /// xが存在し、その個数がcount以上であることが前提。
    #[inline]
    pub fn sub_ex(&mut self, x: T, count: usize) {
        match self.map.entry(x) {
            std::collections::btree_map::Entry::Occupied(mut e) => {
                assert!(count <= *e.get(), "not enough occurrences");
                if count == *e.get() {
                    e.remove();
                } else {
                    *e.get_mut() -= count;
                }
                self.c -= count;
            }
            std::collections::btree_map::Entry::Vacant(_) => panic!("missing key"),
        }
    }
    #[inline]
    pub fn del(&mut self, x: T) {
        if let Some(v) = self.map.remove(&x) {
            self.c -= v;
        }
    }
    #[inline]
    pub fn include(&self, x: T) -> bool {
        self.map.contains_key(&x)
    }
    #[inline]
    pub fn cnt(&self, x: T) -> usize {
        self.map.get(&x).copied().unwrap_or(0)
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
    #[inline]
    pub fn len(&self) -> usize {
        self.map.len()
    }
    #[inline]
    pub fn total(&self) -> usize {
        self.c
    }
    pub fn clear(&mut self) {
        self.map.clear();
        self.c = 0;
    }
    /// 小さいmapを大きいmapへ移す。rhsは空になる。
    pub fn merge(&mut self, rhs: &mut Self) {
        let total = self.c.checked_add(rhs.c).expect("total count overflow");
        if self.len() < rhs.len() {
            std::mem::swap(self, rhs);
        }
        for (k, v) in std::mem::take(&mut rhs.map) {
            *self.map.entry(k).or_insert(0) += v;
        }
        self.c = total;
        rhs.c = 0;
    }
}
