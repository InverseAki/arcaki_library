/// 異なる種類の上位2候補を保持する。候補は `(値, 種類キー)`。
///
/// 値が大きい方を優先し、同値ならキーが小さい方を優先する。
/// 同じキーの候補は最大値だけを採用する。最小値用には
/// `TopTwo<std::cmp::Reverse<V>, K>` を使う。
///
/// 空状態が merge の単位元で、merge は結合的・可換・冪等。
/// 比較が O(1) なら push / merge / best_excluding は O(1)、空間も O(1)。
/// 保持する候補は2個だけなので、削除や値の引き下げには対応しない。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TopTwo<V = i64, K = usize> {
    a1: Option<(V, K)>,
    a2: Option<(V, K)>,
}

impl<V, K> TopTwo<V, K> {
    #[inline]
    pub const fn new() -> Self {
        Self { a1: None, a2: None }
    }

    /// 保持中の候補数（0〜2）。これまでに追加した種類数ではない。
    #[inline]
    pub fn len(&self) -> usize {
        usize::from(self.a1.is_some()) + usize::from(self.a2.is_some())
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.a1.is_none()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.a1 = None;
        self.a2 = None;
    }

    #[inline]
    pub fn first(&self) -> Option<&(V, K)> {
        self.a1.as_ref()
    }

    #[inline]
    pub fn second(&self) -> Option<&(V, K)> {
        self.a2.as_ref()
    }

    /// 優先順に候補を借用する。
    pub fn iter(&self) -> impl Iterator<Item = &(V, K)> {
        self.a1.iter().chain(self.a2.iter())
    }
}

impl<V, K: Eq> TopTwo<V, K> {
    /// 指定した種類を除く最良候補。候補がなければ None。
    #[inline]
    pub fn best_excluding(&self, key: &K) -> Option<&(V, K)> {
        if self.a1.as_ref().map_or(false, |first| &first.1 == key) {
            self.second()
        } else {
            self.first()
        }
    }

    /// 旧メソッド名の入口。返り値は番兵付きの値から Option に変更。
    #[inline]
    pub fn best_rv(&self, key: K) -> Option<&(V, K)> {
        self.best_excluding(&key)
    }
}

impl<V: Ord, K: Ord> TopTwo<V, K> {
    #[inline]
    fn better(lhs: &(V, K), rhs: &(V, K)) -> bool {
        lhs.0 > rhs.0 || (lhs.0 == rhs.0 && lhs.1 < rhs.1)
    }

    /// 候補を追加する。同じ種類の低い値は無視する（値の上書きではない）。
    /// V / K に Clone / Copy は不要。
    #[inline]
    pub fn push(&mut self, rhs: (V, K)) {
        if let Some(first) = &self.a1 {
            if first.1 == rhs.1 {
                if rhs.0 > first.0 {
                    self.a1 = Some(rhs);
                }
                return;
            }
        }
        if self
            .a1
            .as_ref()
            .map_or(true, |first| Self::better(&rhs, first))
        {
            self.a2 = self.a1.replace(rhs);
        } else if self
            .a2
            .as_ref()
            .map_or(true, |second| Self::better(&rhs, second))
        {
            self.a2 = Some(rhs);
        }
    }

    /// rhs を消費して合併する。候補を Clone する必要はない。
    #[inline]
    pub fn merge(&mut self, rhs: Self) {
        self.extend(rhs);
    }

    /// 両辺を残した合併結果。セグメント木などの集約演算用。
    #[inline]
    pub fn merged(&self, rhs: &Self) -> Self
    where
        V: Clone,
        K: Clone,
    {
        let mut result = self.clone();
        result.extend(rhs.iter().cloned());
        result
    }
}

impl<V, K> Default for TopTwo<V, K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: Ord, K: Ord> Extend<(V, K)> for TopTwo<V, K> {
    fn extend<I: IntoIterator<Item = (V, K)>>(&mut self, iter: I) {
        for item in iter {
            self.push(item);
        }
    }
}

impl<V: Ord, K: Ord> std::iter::FromIterator<(V, K)> for TopTwo<V, K> {
    fn from_iter<I: IntoIterator<Item = (V, K)>>(iter: I) -> Self {
        let mut result = Self::new();
        result.extend(iter);
        result
    }
}

impl<V, K> IntoIterator for TopTwo<V, K> {
    type Item = (V, K);
    type IntoIter = std::iter::Flatten<std::array::IntoIter<Option<(V, K)>, 2>>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIterator::into_iter([self.a1, self.a2]).flatten()
    }
}
