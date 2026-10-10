pub trait SqrtDecomposition{
    type S: Clone;
    fn identity()->Self::S;
    fn op(a: &Self::S, b: &Self::S)->Self::S;
    fn inv(a: &Self::S)->Self::S;
}

pub struct SqrtDecompositionData<M> where M: SqrtDecomposition{
    b: usize,
    data: Vec<M::S>,
    block: Vec<M::S>,
}

impl<M> SqrtDecompositionData<M> where M: SqrtDecomposition{
    pub fn new(n: usize, q: usize)->Self{
        let b = ((n as f64/(q as f64).sqrt()).ceil() as usize).max(1);
        SqrtDecompositionData{
            b,
            data: vec![M::identity(); n],
            block: vec![M::identity(); (n+b-1)/b],
        }
    }

    pub fn from(a: &Vec<M::S>, q: usize)->Self{
        let b = ((a.len() as f64/(q as f64).sqrt()).ceil() as usize).max(1);
        let data = a.clone();
        let mut block = vec![M::identity(); (a.len()+b-1)/b];
        for (i, v) in a.iter().enumerate(){
            block[i/b] = M::op(&block[i/b], v);
        }
        SqrtDecompositionData{
            b, data, block
        }
    }

    pub fn set(&mut self, p: usize, x: &M::S){
        self.block[p/self.b] = M::op(&self.block[p/self.b], &M::inv(&self.data[p]));
        self.data[p] = x.clone();
        self.block[p/self.b] = M::op(&self.block[p/self.b], &x);
    }

    pub fn set_both(&mut self, p: usize, x: &M::S, b: &M::S){
        self.data[p] = x.clone();
        self.block[p/self.b] = b.clone();
    }

    pub fn get(&self, p: usize)->M::S{
        self.data[p].clone()
    }

    pub fn get_b(&self, p: usize)->M::S{
        self.block[p/self.b].clone()
    }

    pub fn prod(&self, l: usize, r: usize)->M::S{
        let (bl, br) = ((l + self.b - 1) / self.b, r / self.b);
        if bl >= br{
            let mut res = M::identity();
            for i in l..r{
                res = M::op(&res, &self.data[i]);
            }
            res
        } else {
            let mut res = M::identity();
            for i in l..bl * self.b {
                res = M::op(&res, &self.data[i]);
            }
            for i in bl..br {
                res = M::op(&res, &self.block[i]);
            }
            for i in self.b * br..r {
                res = M::op(&res, &self.data[i]);
            }
            res
        }
    }
}
