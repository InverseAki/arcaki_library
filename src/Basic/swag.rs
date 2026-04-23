pub trait SWAGMonoid{
    type S: Clone;
    fn identity()->Self::S;
    fn op(a: &Self::S, b: &Self::S)->Self::S;
}

pub struct FoldableDeque<M: SWAGMonoid>{
    front: Vec<M::S>,
    back: Vec<M::S>,
    frontfold: Vec<M::S>,
    backfold: Vec<M::S>
}

impl<M> FoldableDeque<M> where M: SWAGMonoid{
    pub fn new()->Self{
        FoldableDeque { front: Vec::new(), back: Vec::new(), frontfold: vec![M::identity()], backfold: vec![M::identity()]}
    }

    #[inline]
    pub fn push_front(&mut self, x: M::S){
        self.front.push(x.clone());
        self.frontfold.push(M::op(&x, self.frontfold.last().unwrap()));
    }

    #[inline]
    pub fn push_back(&mut self, x: M::S){
        self.back.push(x.clone());
        self.backfold.push(M::op(&self.backfold.last().unwrap(),&x));
    }

    #[inline]
    fn _pop_front(&mut self)->M::S{
        self.frontfold.pop();
        self.front.pop().unwrap()
    }

    #[inline]
    fn _pop_back(&mut self)->M::S{
        self.backfold.pop();
        self.back.pop().unwrap()
    }

    #[inline]
    pub fn pop_front(&mut self)->M::S{
        if self.front.is_empty(){
            if self.back.is_empty(){return M::identity();}
            let n = self.back.len();
            let mut s = Vec::with_capacity(n);
            for _ in 0..n{
                s.push(self._pop_back());
            }
            let m = n>>1;
            for i in (0..m).rev(){
                self.push_back(s[i].clone());
            }
            for i in m..n{
                self.push_front(s[i].clone());
            }
        }
        if self.front.is_empty(){return M::identity()}
        self._pop_front()
    }

    #[inline]
    pub fn pop_back(&mut self)->M::S{
        if self.back.is_empty(){
            if self.front.is_empty(){return M::identity();}
            let n = self.front.len();
            let mut s = Vec::with_capacity(n);
            for _ in 0..n{
                s.push(self._pop_front());
            }
            let m = n>>1;
            for i in (0..m).rev(){
                self.push_front(s[i].clone());
            }
            for i in m..n{
                self.push_back(s[i].clone());
            }
        }
        if self.back.is_empty(){return M::identity()}
        self._pop_back()
    }

    #[inline]
    pub fn fold(&self)->M::S{
        M::op(self.frontfold.last().unwrap(), self.backfold.last().unwrap())
    }

    pub fn get_all(&self)->Vec<M::S>{
        let mut res = Vec::with_capacity(self.front.len()+self.back.len());
        for v in self.front.iter().rev(){
            res.push(v.clone());
        }
        for v in self.back.iter(){
            res.push(v.clone());
        }
        res
    }

    pub fn len(&self)->usize{
        self.front.len()+self.back.len()
    }

    pub fn is_empty(&self)->bool{
        self.front.is_empty()&&self.back.is_empty()
    }
}

struct M;
impl SWAGMonoid for M{
    type S = (MI, MI);

    fn identity()->Self::S {
        (MI::new(1), MI::new(0))
    }

    fn op(&a: &Self::S, &b: &Self::S)->Self::S {
        (a.0*b.0, a.1*b.0+b.1)
    }
}
