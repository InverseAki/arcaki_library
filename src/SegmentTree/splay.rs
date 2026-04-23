pub trait SplayMonoid {
    type S: Clone+Debug;
    fn identity() -> Self::S;
    fn op(a: &Self::S, b: &Self::S) -> Self::S;
    fn reverse_prod(x: &mut Self::S);
}

pub trait SplayLazyMonoid{
    type M: SplayMonoid;
    type F: Clone+Debug;
    fn id_e()-><Self::M as SplayMonoid>::S{<Self::M as SplayMonoid>::identity()}
    fn op(a: &<Self::M as SplayMonoid>::S, b: &<Self::M as SplayMonoid>::S)-><Self::M as SplayMonoid>::S{<Self::M>::op(a, b)}
    fn reverse_prod(x: &mut <Self::M as SplayMonoid>::S) {<Self::M>::reverse_prod(x)}
    fn identity()->Self::F;
    fn map(f: &Self::F, x: &<Self::M as SplayMonoid>::S)-><Self::M as SplayMonoid>::S;
    fn composition(f: &Self::F, g: &Self::F)->Self::F;
}

#[derive(Clone, Debug)]
pub struct Node<F> where F: SplayLazyMonoid{
    l: *mut Node<F>,
    r: *mut Node<F>,
    p: *mut Node<F>,
    data: <F::M as SplayMonoid>::S,
    prod: <F::M as SplayMonoid>::S,
    lazy: F::F,
    idx: usize,
    _w: usize,
    ac: usize,
    rev: bool,
}

impl<F> Node<F> where F: SplayLazyMonoid{
    pub fn new_nil() -> Self {
        Node {
            l: null_mut(),
            r: null_mut(),
            p: null_mut(),
            data: F::id_e(),
            prod: F::id_e(),
            lazy: F::identity(),
            idx: !0,
            _w: 0,
            ac: 0,
            rev: false,
        }
    }

    pub fn new(x: <F::M as SplayMonoid>::S, idx: usize, nil: *mut Node<F>) -> Self {
        Node {
            l: nil,
            r: nil,
            p: nil,
            data: x.clone(),
            prod: x.clone(),
            lazy: F::identity(),
            idx,
            _w: 1,
            ac: 1,
            rev: false,
        }
    }
}

pub struct SplayTree<F>  where F: SplayLazyMonoid{
    _p_nil: Box<Node<F>>,
    nil: *mut Node<F>,
    data: Vec<Box<Node<F>>>,
    r: *mut Node<F>,
}

impl<F> SplayTree<F> where F: SplayLazyMonoid {
    pub fn new() -> Self {
        let mut _p_nil = Box::new(Node::<F>::new_nil());
        let ptr: *mut Node<F> = &mut *_p_nil;
        (*_p_nil).l = ptr;
        (*_p_nil).r = ptr;
        (*_p_nil).p = ptr;
        SplayTree {
            _p_nil,
            nil: ptr,
            data: Vec::new(),
            r: ptr,
        }
    }

    #[inline(always)]
    fn apply_down(&mut self, c: *mut Node<F>) {
        unsafe{
            if (*c).l != self.nil {
            (*(*c).l).data = F::map(&(*c).lazy, &(*(*c).l).data);
            (*(*c).l).prod = F::map(&(*c).lazy, &(*(*c).l).prod);
            (*(*c).l).lazy = F::composition(&(*c).lazy, &(*(*c).l).lazy);
        }
        if (*c).r != self.nil {
            (*(*c).r).data = F::map(&(*c).lazy, &(*(*c).r).data);
            (*(*c).r).prod = F::map(&(*c).lazy, &(*(*c).r).prod);
            (*(*c).r).lazy = F::composition(&(*c).lazy, &(*(*c).r).lazy);
        }
        if (*c).rev {
            swap(&mut (*c).l, &mut (*c).r);
            if (*c).l != self.nil {
                (*(*c).l).rev ^= true;
                F::reverse_prod(&mut (*(*c).l).prod);
            }
            if (*c).r != self.nil {
                (*(*c).r).rev ^= true;
                F::reverse_prod(&mut (*(*c).r).prod);
            }
            (*c).rev = false;
        }
        (*c).lazy = F::identity();
        }
    }

    #[inline(always)]
    fn upprod(&mut self, c: *mut Node<F>) {
        unsafe{
        (*c).ac = (*(*c).l).ac + (*(*c).r).ac+1;
        (*c).prod = F::op(&F::op(&(*(*c).l).prod, &(*c).data), &(*(*c).r).prod);
        }
    }

    #[inline(always)]
    fn pc(&mut self, p: *mut Node<F>) -> *mut *mut Node<F> {
        unsafe{
        if (*p).p == self.nil {&mut self.r}
        else if (*(*p).p).l==p {&mut (*(*p).p).l}
        else {&mut (*(*p).p).r}
        }
    }

    #[inline(always)]
    fn rotleft(&mut self, c: *mut Node<F>) {
        unsafe{
        let p = (*c).p;
        *self.pc(p) = c;
        (*c).p = (*p).p;
        (*p).p = c;
        if (*c).l != self.nil {(*(*c).l).p = p}
        (*p).r = (*c).l;
        (*c).l = p;
        }
    }

    #[inline(always)]
    fn rotright(&mut self, c: *mut Node<F>) {
        unsafe{
        let p = (*c).p;
        *self.pc(p) = c;
        (*c).p = (*p).p;
        (*p).p = c;
        if (*c).r != self.nil {(*(*c).r).p = p}
        (*p).l = (*c).r;
        (*c).r = p;
        }
    }

    #[inline(always)]
    fn splay(&mut self, c: *mut Node<F>) {
        unsafe{
        self.apply_down(c);
        while (*c).p != self.nil {
            let p = (*c).p;
            let pp = (*p).p;
            if pp != self.nil {
                self.apply_down(pp);
            }
            if p != self.nil {
                self.apply_down(p);
            }
            self.apply_down(c);
            if (*p).l == c {
                if pp == self.nil {self.rotright(c);}
                else if (*pp).l == p{self.rotright(p); self.rotright(c)}
                else if (*pp).r == p{self.rotright(c); self.rotleft(c)}
            } else {
                if pp == self.nil {self.rotleft(c)}
                else if (*pp).r == p {self.rotleft(p); self.rotleft(c)}
                else if (*pp).l == p {self.rotleft(c); self.rotright(c)}
            }
            if pp != self.nil {self.upprod(pp)}
            if p != self.nil {self.upprod(p)}
            self.upprod(c);
        }
        self.upprod(c);
        }
    }

    // 0-indexed
    #[inline(always)]
    fn kth(&mut self, mut k: usize) -> *mut Node<F> {
        unsafe{
        let mut c = self.r;
        loop {
            self.apply_down(c);
            if (*(*c).l).ac == k{break;}
            if (*(*c).l).ac > k{c = (*c).l; continue;}
            k -= (*(*c).l).ac+1;
            c = (*c).r;
        }
        self.apply_down(c);
        self.splay(c);
        c
        }
    }

    #[inline]
    pub fn insert(&mut self, k: usize, x: <F::M as SplayMonoid>::S){
        unsafe{let idx = self.data.len();
        let x = Box::new(Node::new(x, idx, self.nil));
        let c = Box::leak(x);
        self.data.push(Box::from_raw(c));
        if k==0 {
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
        (*(*p).l).p = c;
        (*p).p = c;
        (*p).l = self.nil;
        self.upprod(p);
        self.upprod(c);
        self.splay(c);}
    }

    #[inline]
    pub fn erase(&mut self, k: usize) {
        unsafe{let p = self.kth(k);
        if k == 0{
            self.r = (*p).r;
            if self.r != self.nil {
                (*self.r).p = self.nil;
            }
        } else if k == (*self.r).ac-1{
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
        let z = self.data.len()-1;
        let x = &mut *self.data[z];
        let id1 = (*p).idx;
        let id2 = (*x).idx;
        swap(&mut (*p).idx, &mut (*x).idx);
        self.data.swap(id1, id2);
        self.data.pop();}
    }

    fn sec(&mut self, l: usize, r: usize) -> *mut Node<F>{
        unsafe{
        if l == 0 && r == (*self.r).ac{
            return self.r;
        } else if l==0{
            return (*self.kth(r)).l;
        } else if r==(*self.r).ac {
            return (*self.kth(l-1)).r;
        }
        let rp = self.kth(r);
        let mut lp = (*rp).l;
        self.r = lp;
        (*lp).p = self.nil;
        lp = self.kth(l-1);
        self.r = rp;
        (*rp).l = lp;
        (*lp).p = rp;
        self.upprod(rp);
        (*lp).r
        }
    }

    #[inline]
    pub fn reverse(&mut self, l: usize, r: usize){
        if l >= r{return;}
        unsafe{let c = self.sec(l, r);
        (*c).rev ^= true;
        F::reverse_prod(&mut (*c).prod);
        self.splay(c);}
    }

    #[inline]
    pub fn apply(&mut self, l: usize, r: usize, f: F::F) {
        unsafe{let c = self.sec(l, r);
        (*c).data = F::map(&f, &(*c).data);
        (*c).prod = F::map(&f, &(*c).prod);
        (*c).lazy = F::composition(&f, &(*c).lazy);
        self.splay(c);
    }
    }

    #[inline]
    pub fn prod(&mut self, l: usize, r: usize) -> <F::M as SplayMonoid>::S {
        unsafe {
            (*self.sec(l, r)).prod.clone()
        }
    }
}

#[derive(Debug, Clone)]
struct M;
impl SplayMonoid for M{
    type S = i64;

    #[inline(always)]
    fn identity() -> Self::S {
        0
    }

    #[inline(always)]
    fn op(&a: &Self::S, &b: &Self::S) -> Self::S {
        a+b
    }

    #[inline(always)]
    fn reverse_prod(_x: &mut Self::S) {}
}

#[derive(Debug, Clone)]
struct MM;
impl SplayLazyMonoid for MM{
    type M = M;
    type F = i64;

    #[inline(always)]
    fn identity() -> Self::F {
        0
    }

    #[inline(always)]
    fn map(&f: &Self::F, &x: &<Self::M as SplayMonoid>::S) -> <Self::M as SplayMonoid>::S {
        f+x
    }

    #[inline(always)]
    fn composition(&f: &Self::F, &g: &Self::F) -> Self::F {
        f+g
    }
}
