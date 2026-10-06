/// op は結合的、identity は両側単位元。可換性・Debug・Default は不要。
pub trait KeyedAvlMonoid {
    type S: Clone;
    fn identity() -> Self::S;
    fn op(a: &Self::S, b: &Self::S) -> Self::S;
}

type AvlLink<K, M> = Option<Box<AvlNode<K, M>>>;

struct AvlNode<K: Ord, M: KeyedAvlMonoid> {
    left: AvlLink<K, M>,
    right: AvlLink<K, M>,
    key: K,
    value: M::S,
    prod: M::S,
    reverse_prod: M::S,
    size: usize,
    height: usize,
}

impl<K: Ord, M: KeyedAvlMonoid> AvlNode<K, M> {
    fn new(key: K, value: M::S) -> Box<Self> {
        Box::new(Self {
            left: None,
            right: None,
            prod: value.clone(),
            reverse_prod: value.clone(),
            key,
            value,
            size: 1,
            height: 1,
        })
    }

    fn size(t: &AvlLink<K, M>) -> usize {
        t.as_ref().map_or(0, |x| x.size)
    }

    fn height(t: &AvlLink<K, M>) -> usize {
        t.as_ref().map_or(0, |x| x.height)
    }

    fn pull(&mut self) {
        self.size = 1 + Self::size(&self.left) + Self::size(&self.right);
        self.height = 1 + Self::height(&self.left).max(Self::height(&self.right));
        let mut prod = self.value.clone();
        if let Some(l) = &self.left {
            prod = M::op(&l.prod, &prod);
        }
        if let Some(r) = &self.right {
            prod = M::op(&prod, &r.prod);
        }
        self.prod = prod;
        let mut prod = self.value.clone();
        if let Some(r) = &self.right {
            prod = M::op(&r.reverse_prod, &prod);
        }
        if let Some(l) = &self.left {
            prod = M::op(&prod, &l.reverse_prod);
        }
        self.reverse_prod = prod;
    }

    fn rotate_left(mut t: Box<Self>) -> Box<Self> {
        let mut r = t.right.take().unwrap();
        t.right = r.left.take();
        t.pull();
        r.left = Some(t);
        r.pull();
        r
    }

    fn rotate_right(mut t: Box<Self>) -> Box<Self> {
        let mut l = t.left.take().unwrap();
        t.left = l.right.take();
        t.pull();
        l.right = Some(t);
        l.pull();
        l
    }

    // 呼び出し時の左右の高さの差は高々2。
    fn balance(mut t: Box<Self>) -> Box<Self> {
        t.pull();
        let lh = Self::height(&t.left);
        let rh = Self::height(&t.right);
        if lh > rh + 1 {
            let l = t.left.as_ref().unwrap();
            if Self::height(&l.left) < Self::height(&l.right) {
                t.left = Some(Self::rotate_left(t.left.take().unwrap()));
            }
            Self::rotate_right(t)
        } else if rh > lh + 1 {
            let r = t.right.as_ref().unwrap();
            if Self::height(&r.right) < Self::height(&r.left) {
                t.right = Some(Self::rotate_right(t.right.take().unwrap()));
            }
            Self::rotate_left(t)
        } else {
            t
        }
    }

    // left + pivot + right。高さが近づくまで高い側を降りる。
    fn join(left: AvlLink<K, M>, mut pivot: Box<Self>, right: AvlLink<K, M>) -> Box<Self> {
        let lh = Self::height(&left);
        let rh = Self::height(&right);
        if lh > rh + 1 {
            let mut l = left.unwrap();
            l.right = Some(Self::join(l.right.take(), pivot, right));
            Self::balance(l)
        } else if rh > lh + 1 {
            let mut r = right.unwrap();
            r.left = Some(Self::join(left, pivot, r.left.take()));
            Self::balance(r)
        } else {
            pivot.left = left;
            pivot.right = right;
            pivot.pull();
            pivot
        }
    }

    fn split(t: AvlLink<K, M>, at: usize) -> (AvlLink<K, M>, AvlLink<K, M>) {
        let Some(mut t) = t else { return (None, None) };
        let n = Self::size(&t.left);
        let left = t.left.take();
        let right = t.right.take();
        if at <= n {
            let (a, b) = Self::split(left, at);
            (a, Some(Self::join(b, t, right)))
        } else {
            let (a, b) = Self::split(right, at - n - 1);
            (Some(Self::join(left, t, a)), b)
        }
    }

    fn pop_last(mut t: Box<Self>) -> (AvlLink<K, M>, Box<Self>) {
        if let Some(r) = t.right.take() {
            let (right, last) = Self::pop_last(r);
            t.right = right;
            (Some(Self::balance(t)), last)
        } else {
            let left = t.left.take();
            (left, t)
        }
    }

    fn concat(left: AvlLink<K, M>, right: AvlLink<K, M>) -> AvlLink<K, M> {
        match (left, right) {
            (None, t) | (t, None) => t,
            (Some(l), Some(r)) => {
                let (left, pivot) = Self::pop_last(l);
                Some(Self::join(left, pivot, Some(r)))
            }
        }
    }

    fn build(it: &mut impl Iterator<Item = (K, M::S)>, n: usize) -> AvlLink<K, M> {
        if n == 0 {
            return None;
        }
        let left = Self::build(it, n / 2);
        let (key, value) = it.next().unwrap();
        let mut t = Self::new(key, value);
        t.left = left;
        t.right = Self::build(it, n - n / 2 - 1);
        t.pull();
        Some(t)
    }

    fn split_key(t: AvlLink<K, M>, key: &K) -> (AvlLink<K, M>, AvlLink<K, M>, AvlLink<K, M>) {
        let Some(mut t) = t else {
            return (None, None, None);
        };
        let left = t.left.take();
        let right = t.right.take();
        match key.cmp(&t.key) {
            std::cmp::Ordering::Less => {
                let (a, equal, b) = Self::split_key(left, key);
                (a, equal, Some(Self::join(b, t, right)))
            }
            std::cmp::Ordering::Greater => {
                let (a, equal, b) = Self::split_key(right, key);
                (Some(Self::join(left, t, a)), equal, b)
            }
            std::cmp::Ordering::Equal => {
                t.pull();
                (left, Some(t), right)
            }
        }
    }

    // 小さい木の根で大きい木をキー分割し、再帰的に統合する。
    fn union(a: AvlLink<K, M>, b: AvlLink<K, M>) -> AvlLink<K, M> {
        let (Some(mut a), Some(mut b)) = (a, b) else {
            unreachable!("union_nonempty only");
        };
        if a.size > b.size {
            std::mem::swap(&mut a, &mut b);
        }
        let (bl, equal, br) = Self::split_key(Some(b), &a.key);
        assert!(equal.is_none(), "merge requires disjoint keys");
        let left = Self::merge(a.left.take(), bl);
        let right = Self::merge(a.right.take(), br);
        Some(Self::join(left, a, right))
    }

    fn merge(a: AvlLink<K, M>, b: AvlLink<K, M>) -> AvlLink<K, M> {
        match (a, b) {
            (None, t) | (t, None) => t,
            (a, b) => Self::union(a, b),
        }
    }

    fn range_prod(&self, l: usize, r: usize, reverse: bool) -> M::S {
        if l == r {
            return M::identity();
        }
        if l == 0 && r == self.size {
            return if reverse {
                self.reverse_prod.clone()
            } else {
                self.prod.clone()
            };
        }
        let n = Self::size(&self.left);
        // 部分区間も、左・要素・右の順序を保って集約する。
        let left = if l < n {
            self.left.as_ref().unwrap().range_prod(l, r.min(n), reverse)
        } else {
            M::identity()
        };
        let mid = if l <= n && n < r {
            self.value.clone()
        } else {
            M::identity()
        };
        let right = if r > n + 1 {
            self.right
                .as_ref()
                .unwrap()
                .range_prod(l.saturating_sub(n + 1), r - n - 1, reverse)
        } else {
            M::identity()
        };
        let prod = if reverse {
            M::op(&M::op(&right, &mid), &left)
        } else {
            M::op(&M::op(&left, &mid), &right)
        };
        prod
    }
}

/// キー昇順の(key, value)を保持する非永続AVL木。集約対象はvalueのみ。
/// キーは木の中で一意。mergeはキーの範囲が重なっていてもよいが、同じキーは不可。
/// splitは昇順で先頭k要素と残り、split_keyはkey未満とkey以上へ分割する。
/// get/insert/remove/prod/split/concat: 最悪O(log n)、from_sorted/to_vec: O(n)。
/// merge: 小さい木の要素数をm、大きい木をnとしてO(m log(n/m+1))。
/// all_prod/all_prod_reverse/len: O(1)。op/Clone/キー比較をO(1)とした計算量。
/// 領域O(n)。unsafe・乱数・外部依存なし。キーはClone不要。
pub struct KeyedAvlTree<K: Ord, M: KeyedAvlMonoid> {
    root: AvlLink<K, M>,
}

impl<K: Ord, M: KeyedAvlMonoid> Default for KeyedAvlTree<K, M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Ord, M: KeyedAvlMonoid> KeyedAvlTree<K, M> {
    pub fn new() -> Self {
        Self { root: None }
    }

    /// 入力のキーは狭義単調増加であること。不正ならpanic。
    pub fn from_sorted(values: Vec<(K, M::S)>) -> Self {
        assert!(
            values.windows(2).all(|w| w[0].0 < w[1].0),
            "keys must be strictly increasing"
        );
        let n = values.len();
        Self {
            root: AvlNode::build(&mut values.into_iter(), n),
        }
    }

    pub fn singleton(key: K, value: M::S) -> Self {
        Self {
            root: Some(AvlNode::new(key, value)),
        }
    }

    pub fn len(&self) -> usize {
        AvlNode::size(&self.root)
    }
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub fn get(&self, key: &K) -> Option<&M::S> {
        let mut t = self.root.as_ref();
        while let Some(x) = t {
            match key.cmp(&x.key) {
                std::cmp::Ordering::Less => t = x.left.as_ref(),
                std::cmp::Ordering::Greater => t = x.right.as_ref(),
                std::cmp::Ordering::Equal => return Some(&x.value),
            }
        }
        None
    }

    /// 昇順でi番目。範囲外ならpanic。
    pub fn get_index(&self, mut i: usize) -> (&K, &M::S) {
        assert!(i < self.len(), "index out of bounds");
        let mut t = self.root.as_ref().unwrap();
        loop {
            let n = AvlNode::size(&t.left);
            if i < n {
                t = t.left.as_ref().unwrap();
            } else if i == n {
                return (&t.key, &t.value);
            } else {
                i -= n + 1;
                t = t.right.as_ref().unwrap();
            }
        }
    }

    /// 既存キーなら値を置き換え、以前の値を返す。
    pub fn insert(&mut self, key: K, value: M::S) -> Option<M::S> {
        let (left, equal, right) = AvlNode::split_key(self.root.take(), &key);
        let old = equal.map(|x| x.value);
        self.root = Some(AvlNode::join(left, AvlNode::new(key, value), right));
        old
    }

    pub fn remove(&mut self, key: &K) -> Option<M::S> {
        let (left, equal, right) = AvlNode::split_key(self.root.take(), key);
        self.root = AvlNode::concat(left, right);
        equal.map(|x| x.value)
    }

    pub fn split(mut self, at: usize) -> (Self, Self) {
        let right = self.split_off(at);
        (self, right)
    }

    pub fn split_off(&mut self, at: usize) -> Self {
        assert!(at <= self.len(), "split index out of bounds");
        let (left, right) = AvlNode::split(self.root.take(), at);
        self.root = left;
        Self { root: right }
    }

    /// key未満とkey以上に分割する。
    pub fn split_key(mut self, key: &K) -> (Self, Self) {
        let (left, equal, right) = AvlNode::split_key(self.root.take(), key);
        let right = match equal {
            Some(pivot) => Some(AvlNode::join(None, pivot, right)),
            None => right,
        };
        (Self { root: left }, Self { root: right })
    }

    /// キー集合の統合。範囲が重なる木も可。同じキーが両方にあればpanic。
    pub fn merge(mut self, mut other: Self) -> Self {
        if !self.is_empty() && !other.is_empty() {
            if self.get_index(self.len() - 1).0 < other.get_index(0).0 {
                self.root = AvlNode::concat(self.root.take(), other.root.take());
                return self;
            }
            if other.get_index(other.len() - 1).0 < self.get_index(0).0 {
                self.root = AvlNode::concat(other.root.take(), self.root.take());
                return self;
            }
        }
        self.root = AvlNode::merge(self.root.take(), other.root.take());
        self
    }

    /// otherの全要素を移す。otherは空になる。キーの条件はmergeと同じ。
    pub fn append(&mut self, other: &mut Self) {
        self.root = AvlNode::merge(self.root.take(), other.root.take());
    }

    /// 全てのself.key < 全てのother.keyである場合の高速な連結。
    pub fn concat(mut self, mut other: Self) -> Self {
        if !self.is_empty() && !other.is_empty() {
            assert!(
                self.get_index(self.len() - 1).0 < other.get_index(0).0,
                "concat requires ordered disjoint key ranges"
            );
        }
        self.root = AvlNode::concat(self.root.take(), other.root.take());
        self
    }

    /// 昇順で[l,r)番目の値の積。
    pub fn prod(&self, l: usize, r: usize) -> M::S {
        self.prod_impl(l, r, false)
    }

    /// 昇順で[l,r)番目の値を、r-1からlへ降順に集約する。
    pub fn prod_reverse(&self, l: usize, r: usize) -> M::S {
        self.prod_impl(l, r, true)
    }

    fn prod_impl(&self, l: usize, r: usize, reverse: bool) -> M::S {
        assert!(l <= r && r <= self.len(), "invalid range");
        self.root
            .as_ref()
            .map_or_else(M::identity, |t| t.range_prod(l, r, reverse))
    }

    pub fn all_prod(&self) -> M::S {
        self.root
            .as_ref()
            .map_or_else(M::identity, |t| t.prod.clone())
    }

    pub fn all_prod_reverse(&self) -> M::S {
        self.root
            .as_ref()
            .map_or_else(M::identity, |t| t.reverse_prod.clone())
    }

    pub fn to_vec(&self) -> Vec<(K, M::S)>
    where
        K: Clone,
    {
        fn visit<K: Ord + Clone, M: KeyedAvlMonoid>(t: &AvlLink<K, M>, out: &mut Vec<(K, M::S)>) {
            if let Some(t) = t {
                visit(&t.left, out);
                out.push((t.key.clone(), t.value.clone()));
                visit(&t.right, out);
            }
        }
        let mut out = Vec::with_capacity(self.len());
        visit(&self.root, &mut out);
        out
    }
}

#[cfg(test)]
impl<K: Ord, M: KeyedAvlMonoid> KeyedAvlTree<K, M> {
    pub(crate) fn check_invariants(&self)
    where
        M::S: PartialEq,
    {
        fn check<'a, K: Ord, M: KeyedAvlMonoid>(
            t: &'a AvlLink<K, M>,
        ) -> (usize, usize, M::S, M::S, Option<&'a K>, Option<&'a K>)
        where
            M::S: PartialEq,
        {
            let Some(t) = t else {
                return (0, 0, M::identity(), M::identity(), None, None);
            };
            let (ls, lh, lp, lr, lmin, lmax) = check(&t.left);
            let (rs, rh, rp, rr, rmin, rmax) = check(&t.right);
            assert!(lh.abs_diff(rh) <= 1);
            assert_eq!(t.size, ls + rs + 1);
            assert_eq!(t.height, 1 + lh.max(rh));
            assert!(lmax.is_none_or(|k| k < &t.key));
            assert!(rmin.is_none_or(|k| k > &t.key));
            let p = M::op(&M::op(&lp, &t.value), &rp);
            let r = M::op(&M::op(&rr, &t.value), &lr);
            assert!(t.prod == p && t.reverse_prod == r);
            (
                t.size,
                t.height,
                p,
                r,
                lmin.or(Some(&t.key)),
                rmax.or(Some(&t.key)),
            )
        }
        check(&self.root);
    }
}
