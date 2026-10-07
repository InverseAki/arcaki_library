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

#[derive(Clone, Debug)]
pub struct NilMonoid<T>(std::marker::PhantomData<fn() -> T>);

#[derive(Clone, Debug)]
pub struct ProdMonoid<M>(std::marker::PhantomData<fn() -> M>);

#[doc(hidden)]
pub trait SplaySpec {
    type Value;
    type ValueStorage;
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

impl<T> SplaySpec for NilMonoid<T> {
    type Value = T;
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
    fn map_value(_: &(), _: &T) -> T {
        unreachable!("NilMonoid has no range action")
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

impl<F: SplaySpec> Clone for Node<F>
where
    F::ValueStorage: Clone,
{
    fn clone(&self) -> Self {
        Self {
            l: self.l,
            r: self.r,
            p: self.p,
            data: self.data.clone(),
            prod: self.prod.clone(),
            lazy: self.lazy.clone(),
            idx: self.idx,
            ac: self.ac,
            rev: self.rev,
            has_lazy: self.has_lazy,
        }
    }
}
impl<F: SplaySpec> std::fmt::Debug for Node<F>
where
    F::ValueStorage: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("l", &self.l)
            .field("r", &self.r)
            .field("p", &self.p)
            .field("data", &self.data)
            .field("prod", &self.prod)
            .field("lazy", &self.lazy)
            .field("idx", &self.idx)
            .field("ac", &self.ac)
            .field("rev", &self.rev)
            .field("has_lazy", &self.has_lazy)
            .finish()
    }
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
        unsafe { (*self.r).ac }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.r == self.nil
    }

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

    #[inline]
    pub fn get(&mut self, k: usize) -> F::Value
    where
        F::Value: Clone,
    {
        assert!(k < self.len(), "get index out of range");
        let c = self.kth(k);
        unsafe { F::value(&(*c).data).clone() }
    }

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

    pub fn to_vec(&mut self) -> Vec<F::Value>
    where
        F::Value: Clone,
    {
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

pub struct SplayVector<T> {
    tree: SplayTree<NilMonoid<T>>,
}

impl<T> SplayVector<T> {
    pub fn new() -> Self {
        Self {
            tree: SplayTree::new(),
        }
    }
    pub fn with_capacity(capacity: usize) -> Self {
        let mut v = Self::new();
        v.reserve_exact(capacity);
        v
    }
    pub fn from_vec(values: Vec<T>) -> Self {
        Self {
            tree: SplayTree::from_vec(values),
        }
    }
    #[inline]
    pub fn len(&self) -> usize {
        self.tree.len()
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.tree.is_empty()
    }
    pub fn capacity(&self) -> usize {
        self.tree.data.capacity()
    }
    pub fn reserve(&mut self, additional: usize) {
        self.tree.data.reserve(additional);
    }
    pub fn reserve_exact(&mut self, additional: usize) {
        self.tree.data.reserve_exact(additional);
    }
    pub fn shrink_to_fit(&mut self) {
        self.tree.data.shrink_to_fit();
    }
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.tree.data.shrink_to(min_capacity);
    }

    fn node_at(&self, mut k: usize) -> Option<*mut Node<NilMonoid<T>>> {
        if k >= self.len() {
            return None;
        }
        let mut c = self.tree.r;
        let mut flipped = false;
        unsafe {
            loop {
                flipped ^= (*c).rev;
                let (l, r) = if flipped {
                    ((*c).r, (*c).l)
                } else {
                    ((*c).l, (*c).r)
                };
                let left_len = (*l).ac;
                if k == left_len {
                    return Some(c);
                }
                if k < left_len {
                    c = l;
                } else {
                    k -= left_len + 1;
                    c = r;
                }
            }
        }
    }
    #[inline]
    pub fn get(&self, k: usize) -> Option<&T> {
        self.node_at(k)
            .map(|c| unsafe { (*c).data.as_ref().unwrap() })
    }
    pub fn get_splayed(&mut self, k: usize) -> Option<&T> {
        if k >= self.len() {
            return None;
        }
        let c = self.tree.kth(k);
        unsafe { (*c).data.as_ref() }
    }
    pub fn get_mut(&mut self, k: usize) -> Option<&mut T> {
        if k >= self.len() {
            return None;
        }
        let c = self.tree.kth(k);
        unsafe { (*c).data.as_mut() }
    }
    #[inline]
    pub fn first(&self) -> Option<&T> {
        self.get(0)
    }
    #[inline]
    pub fn last(&self) -> Option<&T> {
        self.len().checked_sub(1).and_then(|k| self.get(k))
    }
    #[inline]
    pub fn first_mut(&mut self) -> Option<&mut T> {
        self.get_mut(0)
    }
    #[inline]
    pub fn last_mut(&mut self) -> Option<&mut T> {
        let k = self.len().checked_sub(1)?;
        self.get_mut(k)
    }
    #[inline]
    pub fn push(&mut self, x: T) {
        self.tree.push(x);
    }
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        self.tree.pop()
    }
    #[inline]
    pub fn insert(&mut self, k: usize, x: T) {
        self.tree.insert(k, x);
    }
    #[inline]
    pub fn remove(&mut self, k: usize) -> T {
        self.tree.remove(k)
    }
    #[inline]
    pub fn set(&mut self, k: usize, x: T) {
        self.tree.set(k, x);
    }
    pub fn swap(&mut self, a: usize, b: usize) {
        assert!(a < self.len() && b < self.len(), "swap index out of range");
        if a == b {
            return;
        }
        let ca = self.tree.kth(a);
        let cb = self.tree.kth(b);
        unsafe {
            std::mem::swap(&mut (*ca).data, &mut (*cb).data);
        }
    }
    pub fn swap_remove(&mut self, k: usize) -> T {
        assert!(k < self.len(), "swap_remove index out of range");
        self.swap(k, self.len() - 1);
        self.pop().unwrap()
    }
    pub fn truncate(&mut self, len: usize) {
        while self.len() > len {
            self.pop();
        }
    }
    pub fn clear(&mut self) {
        self.tree.r = self.tree.nil;
        self.tree.data.clear();
    }
    pub fn resize_with<F: FnMut() -> T>(&mut self, len: usize, mut f: F) {
        self.truncate(len);
        self.reserve(len - self.len());
        while self.len() < len {
            self.push(f());
        }
    }
    pub fn resize(&mut self, len: usize, value: T)
    where
        T: Clone,
    {
        self.resize_with(len, || value.clone());
    }
    pub fn extend_from_slice(&mut self, values: &[T])
    where
        T: Clone,
    {
        self.extend(values.iter().cloned());
    }
    pub fn append(&mut self, other: &mut Self) {
        self.extend(other.take_all());
    }
    fn take_all(&mut self) -> Vec<T> {
        let mut walk = SplayVectorWalk::new(self);
        let mut values = Vec::with_capacity(self.len());
        while let Some(c) = walk.take(false) {
            unsafe {
                values.push((*c).data.take().unwrap());
            }
        }
        self.clear();
        values
    }
    pub fn split_off(&mut self, at: usize) -> Self {
        assert!(at <= self.len(), "split index out of range");
        let count = self.len() - at;
        let mut tail = Vec::with_capacity(count);
        for _ in 0..count {
            tail.push(self.remove(at));
        }
        Self::from_vec(tail)
    }
    pub fn reverse(&mut self) {
        self.tree.reverse(0, self.len());
    }
    pub fn reverse_range<R: std::ops::RangeBounds<usize>>(&mut self, range: R) {
        let (l, r) = self.bounds(range);
        self.tree.reverse(l, r);
    }
    fn bounds<R: std::ops::RangeBounds<usize>>(&self, range: R) -> (usize, usize) {
        use std::ops::Bound;
        let l = match range.start_bound() {
            Bound::Included(&k) => k,
            Bound::Excluded(&k) => k.checked_add(1).expect("range overflow"),
            Bound::Unbounded => 0,
        };
        let r = match range.end_bound() {
            Bound::Included(&k) => k.checked_add(1).expect("range overflow"),
            Bound::Excluded(&k) => k,
            Bound::Unbounded => self.len(),
        };
        assert!(l <= r && r <= self.len(), "range out of bounds");
        (l, r)
    }
    pub fn drain<R: std::ops::RangeBounds<usize>>(&mut self, range: R) -> SplayVectorDrain<'_, T> {
        let (l, r) = self.bounds(range);
        let mut values = Vec::with_capacity(r - l);
        for _ in l..r {
            values.push(self.remove(l));
        }
        SplayVectorDrain {
            values: values.into_iter(),
            marker: std::marker::PhantomData,
        }
    }
    pub fn retain<F: FnMut(&T) -> bool>(&mut self, mut f: F) {
        self.retain_mut(|x| f(x));
    }
    pub fn retain_mut<F: FnMut(&mut T) -> bool>(&mut self, mut f: F) {
        let mut k = 0;
        while k < self.len() {
            if f(self.get_mut(k).unwrap()) {
                k += 1;
            } else {
                self.tree.erase(k);
            }
        }
    }
    pub fn iter(&self) -> SplayVectorIter<'_, T> {
        SplayVectorIter {
            walk: SplayVectorWalk::new(self),
            marker: std::marker::PhantomData,
        }
    }
    pub fn iter_mut(&mut self) -> SplayVectorIterMut<'_, T> {
        SplayVectorIterMut {
            walk: SplayVectorWalk::new(self),
            marker: std::marker::PhantomData,
        }
    }
    pub fn to_vec(&self) -> Vec<T>
    where
        T: Clone,
    {
        self.iter().cloned().collect()
    }
    pub fn into_vec(self) -> Vec<T> {
        self.into_iter().collect()
    }
}

impl<T> Default for SplayVector<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T> From<Vec<T>> for SplayVector<T> {
    fn from(x: Vec<T>) -> Self {
        Self::from_vec(x)
    }
}
impl<T> From<SplayVector<T>> for Vec<T> {
    fn from(x: SplayVector<T>) -> Self {
        x.into_vec()
    }
}
impl<T, const N: usize> From<[T; N]> for SplayVector<T> {
    fn from(x: [T; N]) -> Self {
        x.into_iter().collect()
    }
}
impl<T> std::iter::FromIterator<T> for SplayVector<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::from_vec(iter.into_iter().collect())
    }
}
impl<T> Extend<T> for SplayVector<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let iter = iter.into_iter();
        self.reserve(iter.size_hint().0);
        for x in iter {
            self.push(x);
        }
    }
}
impl<'a, T: Clone + 'a> Extend<&'a T> for SplayVector<T> {
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.extend(iter.into_iter().cloned());
    }
}
impl<T: Clone> Clone for SplayVector<T> {
    fn clone(&self) -> Self {
        Self::from_vec(self.to_vec())
    }
}
impl<T: std::fmt::Debug> std::fmt::Debug for SplayVector<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}
impl<T, U> PartialEq<SplayVector<U>> for SplayVector<T>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &SplayVector<U>) -> bool {
        self.iter().eq(other.iter())
    }
}
impl<T: Eq> Eq for SplayVector<T> {}
impl<T> std::ops::Index<usize> for SplayVector<T> {
    type Output = T;
    fn index(&self, k: usize) -> &T {
        self.get(k).expect("index out of range")
    }
}
impl<T> std::ops::IndexMut<usize> for SplayVector<T> {
    fn index_mut(&mut self, k: usize) -> &mut T {
        self.get_mut(k).expect("index out of range")
    }
}

struct SplayVectorWalk<T> {
    nil: *mut Node<NilMonoid<T>>,
    front: Vec<(*mut Node<NilMonoid<T>>, bool)>,
    back: Vec<(*mut Node<NilMonoid<T>>, bool)>,
    remaining: usize,
}
impl<T> SplayVectorWalk<T> {
    fn new(v: &SplayVector<T>) -> Self {
        let mut walk = Self {
            nil: v.tree.nil,
            front: Vec::new(),
            back: Vec::new(),
            remaining: v.len(),
        };
        Self::path(&mut walk.front, v.tree.r, false, walk.nil, false);
        Self::path(&mut walk.back, v.tree.r, false, walk.nil, true);
        walk
    }
    fn path(
        stack: &mut Vec<(*mut Node<NilMonoid<T>>, bool)>,
        mut c: *mut Node<NilMonoid<T>>,
        mut flipped: bool,
        nil: *mut Node<NilMonoid<T>>,
        backward: bool,
    ) {
        unsafe {
            while c != nil {
                flipped ^= (*c).rev;
                stack.push((c, flipped));
                c = if flipped ^ backward { (*c).r } else { (*c).l };
            }
        }
    }
    fn take(&mut self, backward: bool) -> Option<*mut Node<NilMonoid<T>>> {
        if self.remaining == 0 {
            return None;
        }
        let stack = if backward {
            &mut self.back
        } else {
            &mut self.front
        };
        let (c, flipped) = stack.pop().unwrap();
        let child = unsafe {
            if flipped ^ backward {
                (*c).l
            } else {
                (*c).r
            }
        };
        Self::path(stack, child, flipped, self.nil, backward);
        self.remaining -= 1;
        Some(c)
    }
}

pub struct SplayVectorIter<'a, T> {
    walk: SplayVectorWalk<T>,
    marker: std::marker::PhantomData<&'a T>,
}
impl<'a, T> Iterator for SplayVectorIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        self.walk
            .take(false)
            .map(|c| unsafe { (*c).data.as_ref().unwrap() })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.walk.remaining, Some(self.walk.remaining))
    }
}
impl<T> DoubleEndedIterator for SplayVectorIter<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.walk
            .take(true)
            .map(|c| unsafe { (*c).data.as_ref().unwrap() })
    }
}
impl<T> ExactSizeIterator for SplayVectorIter<'_, T> {}
impl<T> std::iter::FusedIterator for SplayVectorIter<'_, T> {}

pub struct SplayVectorIterMut<'a, T> {
    walk: SplayVectorWalk<T>,
    marker: std::marker::PhantomData<&'a mut T>,
}
impl<'a, T> Iterator for SplayVectorIterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        self.walk
            .take(false)
            .map(|c| unsafe { (*c).data.as_mut().unwrap() })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.walk.remaining, Some(self.walk.remaining))
    }
}
impl<T> DoubleEndedIterator for SplayVectorIterMut<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.walk
            .take(true)
            .map(|c| unsafe { (*c).data.as_mut().unwrap() })
    }
}
impl<T> ExactSizeIterator for SplayVectorIterMut<'_, T> {}
impl<T> std::iter::FusedIterator for SplayVectorIterMut<'_, T> {}

pub struct SplayVectorIntoIter<T> {
    walk: SplayVectorWalk<T>,
    _owner: SplayVector<T>,
}
impl<T> Iterator for SplayVectorIntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.walk
            .take(false)
            .map(|c| unsafe { (*c).data.take().unwrap() })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.walk.remaining, Some(self.walk.remaining))
    }
}
impl<T> DoubleEndedIterator for SplayVectorIntoIter<T> {
    fn next_back(&mut self) -> Option<T> {
        self.walk
            .take(true)
            .map(|c| unsafe { (*c).data.take().unwrap() })
    }
}
impl<T> ExactSizeIterator for SplayVectorIntoIter<T> {}
impl<T> std::iter::FusedIterator for SplayVectorIntoIter<T> {}
impl<T> IntoIterator for SplayVector<T> {
    type Item = T;
    type IntoIter = SplayVectorIntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        SplayVectorIntoIter {
            walk: SplayVectorWalk::new(&self),
            _owner: self,
        }
    }
}
impl<'a, T> IntoIterator for &'a SplayVector<T> {
    type Item = &'a T;
    type IntoIter = SplayVectorIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<'a, T> IntoIterator for &'a mut SplayVector<T> {
    type Item = &'a mut T;
    type IntoIter = SplayVectorIterMut<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

pub struct SplayVectorDrain<'a, T> {
    values: std::vec::IntoIter<T>,
    marker: std::marker::PhantomData<&'a mut SplayVector<T>>,
}
impl<T> Iterator for SplayVectorDrain<'_, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.values.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.values.size_hint()
    }
}
impl<T> DoubleEndedIterator for SplayVectorDrain<'_, T> {
    fn next_back(&mut self) -> Option<T> {
        self.values.next_back()
    }
}
impl<T> ExactSizeIterator for SplayVectorDrain<'_, T> {}
impl<T> std::iter::FusedIterator for SplayVectorDrain<'_, T> {}

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
