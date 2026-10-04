// 可変長列のSplay木。位置・区間は0-indexed、区間は[l,r)。
// 列のみ: SplayTree<NilMonoid<T>>。区間積のみ: SplayTree<ProdMonoid<M>>。
// 区間積+遅延更新: 従来どおりSplayTree<F>（F: SplayLazyMonoid）。
// 各操作は償却O(log n)、from_vecはO(n)。空区間の積は単位元。
// map(f, op(a,b)) = op(map(f,a), map(f,b))を満たす作用を用意する。
// 区間和ならSに要素数も含める。composition(f,g)はgの後にfを適用する順。
// 作用は反転と可換であること。reverse_prodは集約値の要素順だけを反転する。
// Boxでノードのアドレスを固定し、所有権はdata/_p_nilが持つ。
// UnsafeCell内のノードをraw pointerで操作し、所有用Boxから&mut Nodeは作らない。
pub trait SplayMonoid {
    type S: Clone + std::fmt::Debug;
    fn identity() -> Self::S;
    fn op(a: &Self::S, b: &Self::S) -> Self::S;
    fn reverse_prod(x: &mut Self::S);
}

pub trait SplayLazyMonoid {
    type M: SplayMonoid;
    type F: Clone + std::fmt::Debug;
    fn id_e() -> <Self::M as SplayMonoid>::S {
        <Self::M as SplayMonoid>::identity()
    }
    fn op(
        a: &<Self::M as SplayMonoid>::S,
        b: &<Self::M as SplayMonoid>::S,
    ) -> <Self::M as SplayMonoid>::S {
        <Self::M>::op(a, b)
    }
    fn reverse_prod(x: &mut <Self::M as SplayMonoid>::S) {
        <Self::M>::reverse_prod(x)
    }
    fn identity() -> Self::F;
    fn map(f: &Self::F, x: &<Self::M as SplayMonoid>::S) -> <Self::M as SplayMonoid>::S;
    fn composition(f: &Self::F, g: &Self::F) -> Self::F;
}

/// 区間積・遅延更新を持たない可変長列用。TにDefaultや単位元は不要。
/// `SplayTree::<NilMonoid<T>>::new()` の形で使う。
#[derive(Clone, Debug)]
pub struct NilMonoid<T>(std::marker::PhantomData<fn() -> T>);

/// 区間積と反転を持ち、遅延更新を持たない用途のアダプタ。
/// Mは通常のSplayMonoid。`SplayTree::<ProdMonoid<M>>::new()` の形で使う。
#[derive(Clone, Debug)]
pub struct ProdMonoid<M>(std::marker::PhantomData<fn() -> M>);

// 保存型を分離する内部アダプタ。既存のSplayLazyMonoid実装は自動で対応する。
// 通常の利用者はこのtraitを実装せず、上の2型またはSplayLazyMonoidを使う。
#[doc(hidden)]
pub trait SplaySpec {
    type Value: Clone + std::fmt::Debug;
    type ValueStorage: Clone + std::fmt::Debug;
    type Product: Clone + std::fmt::Debug;
    type Action: Clone + std::fmt::Debug;
    const HAS_PROD: bool;
    const HAS_LAZY: bool;
    fn empty_value() -> Self::ValueStorage;
    fn store_value(x: Self::Value) -> Self::ValueStorage;
    fn value(x: &Self::ValueStorage) -> &Self::Value;
    fn into_value(x: Self::ValueStorage) -> Self::Value;
    fn empty_product() -> Self::Product;
    fn single_product(x: &Self::Value) -> Self::Product;
    fn aggregate(
        l: &Self::Product,
        x: &Self::Value,
        r: &Self::Product,
        has_l: bool,
        has_r: bool,
    ) -> Self::Product;
    fn reverse_product(x: &mut Self::Product);
    fn empty_action() -> Self::Action;
    fn map_value(f: &Self::Action, x: &Self::Value) -> Self::Value;
    fn map_product(f: &Self::Action, x: &Self::Product) -> Self::Product;
    fn compose(f: &Self::Action, g: &Self::Action) -> Self::Action;
}

#[inline(always)]
fn splay_aggregate<M: SplayMonoid>(l: &M::S, x: &M::S, r: &M::S, has_l: bool, has_r: bool) -> M::S {
    if !has_l {
        if !has_r {
            x.clone()
        } else {
            M::op(x, r)
        }
    } else if !has_r {
        M::op(l, x)
    } else {
        M::op(&M::op(l, x), r)
    }
}

impl<F: SplayLazyMonoid> SplaySpec for F {
    type Value = <F::M as SplayMonoid>::S;
    type ValueStorage = Self::Value;
    type Product = Self::Value;
    type Action = F::F;
    const HAS_PROD: bool = true;
    const HAS_LAZY: bool = true;
    #[inline(always)]
    fn empty_value() -> Self::Value {
        F::id_e()
    }
    #[inline(always)]
    fn store_value(x: Self::Value) -> Self::Value {
        x
    }
    #[inline(always)]
    fn value(x: &Self::Value) -> &Self::Value {
        x
    }
    #[inline(always)]
    fn into_value(x: Self::Value) -> Self::Value {
        x
    }
    #[inline(always)]
    fn empty_product() -> Self::Product {
        F::id_e()
    }
    #[inline(always)]
    fn single_product(x: &Self::Value) -> Self::Product {
        x.clone()
    }
    #[inline(always)]
    fn aggregate(
        l: &Self::Product,
        x: &Self::Value,
        r: &Self::Product,
        has_l: bool,
        has_r: bool,
    ) -> Self::Product {
        // SplayLazyMonoidでのopの上書きも従来どおり尊重する。
        if !has_l {
            if !has_r {
                x.clone()
            } else {
                F::op(x, r)
            }
        } else if !has_r {
            F::op(l, x)
        } else {
            F::op(&F::op(l, x), r)
        }
    }
    #[inline(always)]
    fn reverse_product(x: &mut Self::Product) {
        F::reverse_prod(x)
    }
    #[inline(always)]
    fn empty_action() -> Self::Action {
        F::identity()
    }
    #[inline(always)]
    fn map_value(f: &Self::Action, x: &Self::Value) -> Self::Value {
        F::map(f, x)
    }
    #[inline(always)]
    fn map_product(f: &Self::Action, x: &Self::Product) -> Self::Product {
        F::map(f, x)
    }
    #[inline(always)]
    fn compose(f: &Self::Action, g: &Self::Action) -> Self::Action {
        F::composition(f, g)
    }
}

impl<T: Clone + std::fmt::Debug> SplaySpec for NilMonoid<T> {
    type Value = T;
    // 番兵だけNone。単位元を持たない型も保存でき、区間積を複製しない。
    type ValueStorage = Option<T>;
    type Product = ();
    type Action = ();
    const HAS_PROD: bool = false;
    const HAS_LAZY: bool = false;
    #[inline(always)]
    fn empty_value() -> Option<T> {
        None
    }
    #[inline(always)]
    fn store_value(x: T) -> Option<T> {
        Some(x)
    }
    #[inline(always)]
    fn value(x: &Option<T>) -> &T {
        x.as_ref().expect("sentinel has no value")
    }
    #[inline(always)]
    fn into_value(x: Option<T>) -> T {
        x.expect("sentinel has no value")
    }
    #[inline(always)]
    fn empty_product() {}
    #[inline(always)]
    fn single_product(_: &T) {}
    #[inline(always)]
    fn aggregate(_: &(), _: &T, _: &(), _: bool, _: bool) {}
    #[inline(always)]
    fn reverse_product(_: &mut ()) {}
    #[inline(always)]
    fn empty_action() {}
    #[inline(always)]
    fn map_value(_: &(), x: &T) -> T {
        x.clone()
    }
    #[inline(always)]
    fn map_product(_: &(), _: &()) {}
    #[inline(always)]
    fn compose(_: &(), _: &()) {}
}

impl<M: SplayMonoid> SplaySpec for ProdMonoid<M> {
    type Value = M::S;
    type ValueStorage = M::S;
    type Product = M::S;
    type Action = ();
    const HAS_PROD: bool = true;
    const HAS_LAZY: bool = false;
    #[inline(always)]
    fn empty_value() -> Self::Value {
        M::identity()
    }
    #[inline(always)]
    fn store_value(x: Self::Value) -> Self::Value {
        x
    }
    #[inline(always)]
    fn value(x: &Self::Value) -> &Self::Value {
        x
    }
    #[inline(always)]
    fn into_value(x: Self::Value) -> Self::Value {
        x
    }
    #[inline(always)]
    fn empty_product() -> Self::Product {
        M::identity()
    }
    #[inline(always)]
    fn single_product(x: &Self::Value) -> Self::Product {
        x.clone()
    }
    #[inline(always)]
    fn aggregate(
        l: &Self::Product,
        x: &Self::Value,
        r: &Self::Product,
        has_l: bool,
        has_r: bool,
    ) -> Self::Product {
        splay_aggregate::<M>(l, x, r, has_l, has_r)
    }
    #[inline(always)]
    fn reverse_product(x: &mut Self::Product) {
        M::reverse_prod(x)
    }
    #[inline(always)]
    fn empty_action() {}
    #[inline(always)]
    fn map_value(_: &(), x: &Self::Value) -> Self::Value {
        x.clone()
    }
    #[inline(always)]
    fn map_product(_: &(), x: &Self::Product) -> Self::Product {
        x.clone()
    }
    #[inline(always)]
    fn compose(_: &(), _: &()) {}
}

#[derive(Clone, Debug)]
pub struct Node<F>
where
    F: SplaySpec,
{
    l: *mut Node<F>,
    r: *mut Node<F>,
    p: *mut Node<F>,
    data: F::ValueStorage,
    prod: F::Product,
    lazy: F::Action,
    idx: usize,
    ac: usize,
    rev: bool,
    has_lazy: bool,
}

impl<F> Node<F>
where
    F: SplaySpec,
{
    pub fn new_nil() -> Self {
        Node {
            l: std::ptr::null_mut(),
            r: std::ptr::null_mut(),
            p: std::ptr::null_mut(),
            data: F::empty_value(),
            prod: F::empty_product(),
            lazy: F::empty_action(),
            idx: !0,
            ac: 0,
            rev: false,
            has_lazy: false,
        }
    }

    pub fn new(x: F::Value, idx: usize, nil: *mut Node<F>) -> Self {
        let prod = F::single_product(&x);
        Node {
            l: nil,
            r: nil,
            p: nil,
            data: F::store_value(x),
            prod,
            lazy: F::empty_action(),
            idx,
            ac: 1,
            rev: false,
            has_lazy: false,
        }
    }
}

pub struct SplayTree<F>
where
    F: SplaySpec,
{
    _p_nil: Box<std::cell::UnsafeCell<Node<F>>>,
    nil: *mut Node<F>,
    data: Vec<Box<std::cell::UnsafeCell<Node<F>>>>,
    r: *mut Node<F>,
}

impl<F> SplayTree<F>
where
    F: SplaySpec,
{
    pub fn new() -> Self {
        let _p_nil = Box::new(std::cell::UnsafeCell::new(Node::<F>::new_nil()));
        let ptr = _p_nil.get();
        unsafe {
            (*ptr).l = ptr;
            (*ptr).r = ptr;
            (*ptr).p = ptr;
        }
        SplayTree {
            _p_nil,
            nil: ptr,
            data: Vec::new(),
            r: ptr,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        // rはselfが所有するノードまたはnilを指す。
        unsafe { (*self.r).ac }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.r == self.nil
    }

    /// 平衡形で一括構築する。O(n)時間、構築直後の木の高さはO(log n)。
    pub fn from_vec(values: Vec<F::Value>) -> Self {
        let mut tree = Self::new();
        tree.data.reserve(values.len());
        for (idx, value) in values.into_iter().enumerate() {
            tree.data
                .push(Box::new(std::cell::UnsafeCell::new(Node::new(
                    value, idx, tree.nil,
                ))));
        }
        let n = tree.data.len();
        tree.r = tree.build_balanced(0, n, tree.nil);
        tree
    }

    fn build_balanced(&mut self, l: usize, r: usize, parent: *mut Node<F>) -> *mut Node<F> {
        if l == r {
            return self.nil;
        }
        let m = l + (r - l) / 2;
        let c = self.data[m].get();
        unsafe {
            (*c).p = parent;
            (*c).l = self.build_balanced(l, m, c);
            (*c).r = self.build_balanced(m + 1, r, c);
        }
        self.upprod(c);
        c
    }

    #[inline(always)]
    fn apply_node(&mut self, c: *mut Node<F>, f: &F::Action) {
        unsafe {
            (*c).data = F::store_value(F::map_value(f, F::value(&(*c).data)));
            (*c).prod = F::map_product(f, &(*c).prod);
            (*c).lazy = if F::HAS_LAZY && (*c).has_lazy {
                F::compose(f, &(*c).lazy)
            } else {
                f.clone()
            };
            (*c).has_lazy = true;
        }
    }

    #[inline(always)]
    fn reverse_node(&mut self, c: *mut Node<F>) {
        unsafe {
            (*c).rev ^= true;
            if F::HAS_PROD {
                F::reverse_product(&mut (*c).prod);
            }
        }
    }

    #[inline(always)]
    fn apply_down(&mut self, c: *mut Node<F>) {
        unsafe {
            if F::HAS_LAZY && (*c).has_lazy {
                if (*c).l != self.nil {
                    self.apply_node((*c).l, &(*c).lazy);
                }
                if (*c).r != self.nil {
                    self.apply_node((*c).r, &(*c).lazy);
                }
                (*c).has_lazy = false;
                // lazyの値は次のapplyで上書きする。identityの再生成は不要。
            }
            if (*c).rev {
                std::mem::swap(&mut (*c).l, &mut (*c).r);
                if (*c).l != self.nil {
                    self.reverse_node((*c).l);
                }
                if (*c).r != self.nil {
                    self.reverse_node((*c).r);
                }
                (*c).rev = false;
            }
        }
    }

    #[inline(always)]
    fn upprod(&mut self, c: *mut Node<F>) {
        unsafe {
            let (l, r) = ((*c).l, (*c).r);
            (*c).ac = (*l).ac + (*r).ac + 1;
            if F::HAS_PROD {
                (*c).prod = F::aggregate(
                    &(*l).prod,
                    F::value(&(*c).data),
                    &(*r).prod,
                    l != self.nil,
                    r != self.nil,
                );
            }
        }
    }

    #[inline(always)]
    fn pc(&mut self, p: *mut Node<F>) -> *mut *mut Node<F> {
        unsafe {
            if (*p).p == self.nil {
                &mut self.r
            } else if (*(*p).p).l == p {
                &mut (*(*p).p).l
            } else {
                &mut (*(*p).p).r
            }
        }
    }

    #[inline(always)]
    fn rotleft(&mut self, c: *mut Node<F>) {
        unsafe {
            let p = (*c).p;
            *self.pc(p) = c;
            (*c).p = (*p).p;
            (*p).p = c;
            if (*c).l != self.nil {
                (*(*c).l).p = p
            }
            (*p).r = (*c).l;
            (*c).l = p;
        }
    }

    #[inline(always)]
    fn rotright(&mut self, c: *mut Node<F>) {
        unsafe {
            let p = (*c).p;
            *self.pc(p) = c;
            (*c).p = (*p).p;
            (*p).p = c;
            if (*c).r != self.nil {
                (*(*c).r).p = p
            }
            (*p).l = (*c).r;
            (*c).r = p;
        }
    }

    #[inline(always)]
    fn splay(&mut self, c: *mut Node<F>) {
        unsafe {
            // kthが根からcまで伝播済み。回転中に再伝播する必要はない。
            while (*c).p != self.nil {
                let p = (*c).p;
                let pp = (*p).p;
                if (*p).l == c {
                    if pp == self.nil {
                        self.rotright(c);
                    } else if (*pp).l == p {
                        self.rotright(p);
                        self.rotright(c)
                    } else if (*pp).r == p {
                        self.rotright(c);
                        self.rotleft(c)
                    }
                } else {
                    if pp == self.nil {
                        self.rotleft(c)
                    } else if (*pp).r == p {
                        self.rotleft(p);
                        self.rotleft(c)
                    } else if (*pp).l == p {
                        self.rotleft(c);
                        self.rotright(c)
                    }
                }
                if pp != self.nil {
                    self.upprod(pp)
                }
                if p != self.nil {
                    self.upprod(p)
                }
            }
            self.upprod(c);
        }
    }

    // 0-indexed
    #[inline(always)]
    fn kth(&mut self, mut k: usize) -> *mut Node<F> {
        unsafe {
            let mut c = self.r;
            loop {
                self.apply_down(c);
                if (*(*c).l).ac == k {
                    break;
                }
                if (*(*c).l).ac > k {
                    c = (*c).l;
                    continue;
                }
                k -= (*(*c).l).ac + 1;
                c = (*c).r;
            }
            self.splay(c);
            c
        }
    }

    /// k番目の要素を返す。償却O(log n)。
    #[inline]
    pub fn get(&mut self, k: usize) -> F::Value {
        assert!(k < self.len(), "get index out of range");
        let c = self.kth(k);
        unsafe { F::value(&(*c).data).clone() }
    }

    /// k番目の要素を置き換える。償却O(log n)。
    #[inline]
    pub fn set(&mut self, k: usize, x: F::Value) {
        assert!(k < self.len(), "set index out of range");
        let c = self.kth(k);
        unsafe {
            (*c).data = F::store_value(x);
        }
        if F::HAS_PROD {
            self.upprod(c);
        }
    }

    #[inline]
    pub fn push(&mut self, x: F::Value) {
        self.insert(self.len(), x);
    }

    /// 値をcloneせず取り出す。償却O(log n)。
    #[inline]
    pub fn remove(&mut self, k: usize) -> F::Value {
        let node = (*self.erase_node(k)).into_inner();
        F::into_value(node.data)
    }

    #[inline]
    pub fn pop(&mut self) -> Option<F::Value> {
        if self.is_empty() {
            None
        } else {
            Some(self.remove(self.len() - 1))
        }
    }

    /// 全要素を順番どおり複製する。O(n)時間（値のcloneの費用は別）。
    pub fn to_vec(&mut self) -> Vec<F::Value> {
        let mut result = Vec::with_capacity(self.len());
        let mut stack = Vec::new();
        let mut c = self.r;
        unsafe {
            while c != self.nil || !stack.is_empty() {
                while c != self.nil {
                    self.apply_down(c);
                    stack.push(c);
                    c = (*c).l;
                }
                c = stack.pop().unwrap();
                result.push(F::value(&(*c).data).clone());
                c = (*c).r;
            }
        }
        result
    }

    #[inline]
    pub fn insert(&mut self, k: usize, x: F::Value) {
        assert!(k <= self.len(), "insert index out of range");
        let idx = self.data.len();
        let node = Box::new(std::cell::UnsafeCell::new(Node::new(x, idx, self.nil)));
        let c = node.get();
        self.data.push(node);
        unsafe {
            if k == 0 {
                (*c).r = self.r;
                if self.r != self.nil {
                    (*self.r).p = c;
                }
                self.r = c;
                self.upprod(c);
                return;
            } else if k == (*self.r).ac {
                (*c).l = self.r;
                if self.r != self.nil {
                    (*self.r).p = c;
                }
                self.r = c;
                self.upprod(c);
                return;
            }
            let p = self.kth(k);
            (*c).l = (*p).l;
            (*c).r = p;
            self.r = c;
            if (*p).l != self.nil {
                (*(*p).l).p = c;
            }
            (*p).p = c;
            (*p).l = self.nil;
            self.upprod(p);
            self.upprod(c);
        }
    }

    #[inline]
    pub fn erase(&mut self, k: usize) {
        drop(self.erase_node(k));
    }

    #[inline]
    fn erase_node(&mut self, k: usize) -> Box<std::cell::UnsafeCell<Node<F>>> {
        assert!(k < self.len(), "erase index out of range");
        unsafe {
            let p = self.kth(k);
            if k == 0 {
                self.r = (*p).r;
                if self.r != self.nil {
                    (*self.r).p = self.nil;
                }
            } else if k == (*self.r).ac - 1 {
                self.r = (*p).l;
                if self.r != self.nil {
                    (*self.r).p = self.nil;
                }
            } else {
                let l = (*p).l;
                let mut r = (*p).r;
                (*r).p = self.nil;
                self.r = r;
                self.kth(0);
                r = self.r;
                (*r).l = l;
                (*l).p = r;
                self.upprod(r);
            }
            let idx = (*p).idx;
            let removed = self.data.swap_remove(idx);
            if idx < self.data.len() {
                (*self.data[idx].get()).idx = idx;
            }
            removed
        }
    }

    fn sec(&mut self, l: usize, r: usize) -> *mut Node<F> {
        unsafe {
            if l == 0 && r == (*self.r).ac {
                return self.r;
            } else if l == 0 {
                return (*self.kth(r)).l;
            } else if r == (*self.r).ac {
                return (*self.kth(l - 1)).r;
            }
            let rp = self.kth(r);
            let mut lp = (*rp).l;
            self.r = lp;
            (*lp).p = self.nil;
            lp = self.kth(l - 1);
            self.r = rp;
            (*rp).l = lp;
            (*lp).p = rp;
            self.upprod(rp);
            (*lp).r
        }
    }

    #[inline]
    fn check_range(&self, l: usize, r: usize) {
        assert!(l <= r && r <= self.len(), "range out of bounds");
    }

    // secの返す区間ノードには境界の祖先が高々2つある。
    // 区間全体を根へ回転する代わりに、その祖先の集約値だけを更新する。
    #[inline(always)]
    fn update_boundaries(&mut self, c: *mut Node<F>) {
        unsafe {
            let p = (*c).p;
            if p != self.nil {
                self.upprod(p);
                let pp = (*p).p;
                if pp != self.nil {
                    self.upprod(pp);
                }
            }
        }
    }

    #[inline]
    pub fn reverse(&mut self, l: usize, r: usize) {
        self.check_range(l, r);
        if l == r {
            return;
        }
        let c = self.sec(l, r);
        self.reverse_node(c);
        if F::HAS_PROD {
            self.update_boundaries(c);
        }
    }

    #[inline]
    pub fn apply(&mut self, l: usize, r: usize, f: F::Action) {
        self.check_range(l, r);
        if !F::HAS_LAZY || l == r {
            return;
        }
        let c = self.sec(l, r);
        self.apply_node(c, &f);
        self.update_boundaries(c);
    }

    #[inline]
    pub fn prod(&mut self, l: usize, r: usize) -> F::Product {
        self.check_range(l, r);
        if !F::HAS_PROD || l == r {
            return F::empty_product();
        }
        unsafe { (*self.sec(l, r)).prod.clone() }
    }
}

#[derive(Debug, Clone)]
struct M;
impl SplayMonoid for M {
    type S = (i64, usize);

    #[inline(always)]
    fn identity() -> Self::S {
        (0, 0)
    }

    #[inline(always)]
    fn op(&a: &Self::S, &b: &Self::S) -> Self::S {
        (a.0 + b.0, a.1 + b.1)
    }

    #[inline(always)]
    fn reverse_prod(_x: &mut Self::S) {}
}

#[derive(Debug, Clone)]
struct MM;
impl SplayLazyMonoid for MM {
    type M = M;
    type F = i64;

    #[inline(always)]
    fn identity() -> Self::F {
        0
    }

    #[inline(always)]
    fn map(&f: &Self::F, &x: &<Self::M as SplayMonoid>::S) -> <Self::M as SplayMonoid>::S {
        (x.0 + f * x.1 as i64, x.1)
    }

    #[inline(always)]
    fn composition(&f: &Self::F, &g: &Self::F) -> Self::F {
        f + g
    }
}
