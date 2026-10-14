// Source: https://github.com/InverseAki/arcaki_library/blob/main/src/SegmentTree/commutativedualsegmenttree.rs
pub trait CommutativeDualSegmentTreeMonoid{
    type S: Clone;
    type F: Clone;
    fn id_s()->Self::S;
    fn id_f()->Self::F;
    fn map(f: &Self::F, x: &Self::S)->Self::S;
    fn composition(f: &Self::F, g: &Self::F)->Self::F;
}

pub struct CommutativeDualSegmentTree<M> where M: CommutativeDualSegmentTreeMonoid{
    n: usize,
    lazy: Vec<M::F>,
    data: Vec<M::S>,
}

impl<M> CommutativeDualSegmentTree<M> where M: CommutativeDualSegmentTreeMonoid{
    pub fn new(n: usize)->Self{
        let n = n.next_power_of_two();
        CommutativeDualSegmentTree{
            n,
            data: vec![M::id_s(); n],
            lazy: vec![M::id_f(); 2*n]
        }
    }

    pub fn from(mut data: Vec<M::S>)->Self{
        let n = data.len().next_power_of_two();
        while data.len() < n{
            data.push(M::id_s());
        }
        CommutativeDualSegmentTree{
            n,
            data,
            lazy: vec![M::id_f(); 2*n],
        }
    }

    pub fn set(&mut self, p: usize, x: M::S){
        self.data[p] = x.clone();
    }

    pub fn apply(&mut self, mut l: usize, mut r: usize, f: M::F){
        l += self.n; r += self.n;
        while l < r{
            if l&1==1{
                self.lazy[l] = M::composition(&f, &self.lazy[l]);
                l += 1;
            }
            if r&1==1{
                r -= 1;
                self.lazy[r] = M::composition(&f, &self.lazy[r]);
            }
            l >>= 1;
            r >>= 1;
        }
    }

    pub fn get(&self, mut p: usize)->M::S{
        let mut res = self.data[p].clone();
        p += self.n;
        while p > 0{
            res = M::map(&self.lazy[p], &res);
            p >>= 1;
        }
        res
    }
}
