// Source: https://github.com/InverseAki/arcaki_library/blob/main/src/nonavailable/SparseSegmentTree2D.rs
pub trait SegmentTreeMonoid{
    type S: Copy+Debug;
    fn identity()->Self::S;
    fn op(a: Self::S, b: Self::S)->Self::S;
}

#[derive(Copy, Clone, Debug)]
pub struct SegmentTree2DNode<M> where M: SegmentTreeMonoid{
    left: usize,
    right: usize,
    pre: usize,
    data: M::S,
}

impl<M> SegmentTree2DNode<M> where M: SegmentTreeMonoid{
    pub fn new()->Self{
        SegmentTree2DNode{
            left: !0,
            right: !0,
            pre: !0,
            data: M::identity(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SparseSegmentTree2D<M> where M: SegmentTreeMonoid{
    ln: usize,
    lm: usize,
    data: Vec<SegmentTree2DNode<M>>,
}

impl<M> SparseSegmentTree2D<M> where M: SegmentTreeMonoid{
    pub fn new(n: usize, m: usize)->Self{
        let (n, m) = (n.next_power_of_two(), m.next_power_of_two());
        let mut data = vec![SegmentTree2DNode::<M>::new()];
        data.reserve(1<<24);
        SparseSegmentTree2D{
            ln:63-n.leading_zeros()as usize,lm:63-m.leading_zeros()as usize,
            data: vec![SegmentTree2DNode::new()],
        }
    }

    pub fn set(&mut self, x: usize, y: usize, v: M::S){
        let mut p = 0;
        for i in (0..self.ln).rev(){
            if x&1<<i==0{
                let mut np = self.data[p].left;
                if np==!0{
                    self.data[p].left = self.data.len();
                    np = self.data.len();
                    self.data.push(SegmentTree2DNode::<M>::new());
                    self.data[np].pre = p;
                }
                p = np;
            } else {
                let mut np = self.data[p].right;
                if np==!0{
                    self.data[p].right = self.data.len();
                    np = self.data.len();
                    self.data.push(SegmentTree2DNode::<M>::new());
                    self.data[np].pre = p;
                }
                p = np;
            }
        }
        for i in (0..self.lm).rev(){
            if y&1<<i==0{
                let mut np = self.data[p].left;
                if np==!0{
                    self.data[p].left = self.data.len();
                    np = self.data.len();
                    self.data.push(SegmentTree2DNode::<M>::new());
                    self.data[np].pre = p;
                }
                p = np;
            } else {
                let mut np = self.data[p].right;
                if np==!0{
                    self.data[p].right = self.data.len();
                    np = self.data.len();
                    self.data.push(SegmentTree2DNode::<M>::new());
                    self.data[np].pre = p;
                }
                p = np;
            }
        }
        self.data[p].data = v.clone();
        while self.data[p].pre != !0{
            p = self.data[p].pre;
            let lc = self.data[p].left;
            let lx = if lc==!0{M::identity()}else{self.data[lc].data.clone()};
            let rc = self.data[p].right;
            let rx = if rc==!0{M::identity()}else{self.data[rc].data.clone()};
            self.data[p].data = M::op(lx, rx);
        }
    }

    pub fn push(&mut self, x: usize, y: usize, v: M::S){
        let mut p = 0;
        for i in (0..self.ln).rev(){
            if x&1<<i==0{
                let mut np = self.data[p].left;
                if np==!0{
                    self.data[p].left = self.data.len();
                    np = self.data.len();
                    self.data.push(SegmentTree2DNode::<M>::new());
                    self.data[np].pre = p;
                }
                p = np;
            } else {
                let mut np = self.data[p].right;
                if np==!0{
                    self.data[p].right = self.data.len();
                    np = self.data.len();
                    self.data.push(SegmentTree2DNode::<M>::new());
                    self.data[np].pre = p;
                }
                p = np;
            }
        }
        for i in (0..self.lm).rev(){
            if y&1<<i==0{
                let mut np = self.data[p].left;
                if np==!0{
                    self.data[p].left = self.data.len();
                    np = self.data.len();
                    self.data.push(SegmentTree2DNode::<M>::new());
                    self.data[np].pre = p;
                }
                p = np;
            } else {
                let mut np = self.data[p].right;
                if np==!0{
                    self.data[p].right = self.data.len();
                    np = self.data.len();
                    self.data.push(SegmentTree2DNode::<M>::new());
                    self.data[np].pre = p;
                }
                p = np;
            }
        }
        self.data[p].data = M::op(self.data[p].data, v);
        while self.data[p].pre != !0{
            p = self.data[p].pre;
            let lc = self.data[p].left;
            let lx = if lc==!0{M::identity()}else{self.data[lc].data.clone()};
            let rc = self.data[p].right;
            let rx = if rc==!0{M::identity()}else{self.data[rc].data.clone()};
            self.data[p].data = M::op(lx, rx);
        }
    }

    pub fn get(&self, p: usize, q: usize,)->M::S{
        let mut res = M::identity();
        self.prod_dfs(0, &mut res, 0, 1<<self.ln, 0, 1<<self.lm, p, p+1, q, q+1);
        res
    }

    pub fn prod(&self, l: usize, r: usize, d: usize, u: usize)->M::S{
        let mut res = M::identity();
        self.prod_dfs(0, &mut res, 0, 1<<self.ln, 0, 1<<self.lm, l, r, d, u);
        res
    }

    fn prod_dfs(&self, p: usize, res: &mut M::S,
                    pl: usize, pr: usize, pd: usize, pu: usize,
                    l: usize, r: usize, d: usize, u: usize){
        if p==!0||r<=pl||pr<=l||u<=pd||pu<=d{return;}
        if l<=pl&&pr<=r&&d<=pd&&pu<=u{*res = M::op(*res, self.data[p].data);return;}
        if pl+1==pr{
            let m = (pd+pu)/2;
            if !(u<=pd||m<=d){
                self.prod_dfs(self.data[p].left, res, pl, pr, pd, m, l, r, d, u);
            }
            if !(u<=m||pu<=d){
                self.prod_dfs(self.data[p].right, res, pl, pr, m, pu, l, r, d, u);
            }
        } else {
            let m = (pl+pr)/2;
            if !(r<=pl||m<=l){
                self.prod_dfs(self.data[p].left, res, pl, m, pd, pu, l, r, d, u);
            }
            if !(r<=m||pr<=l){
                self.prod_dfs(self.data[p].right, res, m, pr, pd, pu, l, r, d, u);
            }
        }
    }
}