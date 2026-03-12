pub trait MatrixMonoid {
    type S: Clone;
    fn sum(a: &Self::S, b: &Self::S)->Self::S;
    fn one()->Self::S;
    fn mul(a: &Self::S, b: &Self::S)->Self::S;
    fn zero()->Self::S;
}

#[derive(Debug)]
pub struct DoublingMatrix<M> where M: MatrixMonoid{
    n: usize,
    g: Vec<M::S>,
}
impl<M> Clone for DoublingMatrix<M>
where
    M: MatrixMonoid,
    M::S: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self { n: self.n, g: self.g.clone() }
    }
}
impl<M> DoublingMatrix<M> where M: MatrixMonoid{
    #[inline]
    pub fn new(n: usize, a: &Vec<M::S>)->Self{
        DoublingMatrix {n, g: a.clone() }
    }

    #[inline]
    pub fn zeros(n: usize)->Self{
        DoublingMatrix { n, g: vec![M::zero(); n*n] }
    }

    #[inline]
    pub fn e(n: usize)->Self{
        let mut res = Self::zeros(n);
        for i in 0..n{
            res.set(i, i, M::one());
        }
        res
    }

    #[inline]
    pub fn get(&self, i: usize, j: usize)-> &M::S{
        &self.g[i*self.n+j]
    }

    #[inline]
    pub fn set(&mut self, i: usize, j: usize, v: M::S){
        self.g[i*self.n+j] = v;
    }

    #[inline(always)]
    pub fn prod(&self, rhs: &Self)->Self{
        let n = self.n;
        let mut res = vec![M::zero(); n*n];
        for i in 0..n {
            let a_row = &self.g[i * n .. (i + 1) * n];
            let out_row = &mut res[i * n .. (i + 1) * n];
            for k in 0..n {
                let a = &a_row[k];
                let b_row = &rhs.g[k * n .. (k + 1) * n];
                for j in 0..n {
                    let addend = M::mul(a, &b_row[j]);
                    out_row[j] = M::sum(&out_row[j], &addend);
                }
            }
        }
        Self {n, g:res}
    }

    #[inline]
    pub fn pow(&self, mut k: usize)->Self {
        let n = self.n;
        let mut res = Self::e(n);
        let mut r = (*self).clone();
        while k > 0 {
            if (k & 1) == 1 {
                res = res.prod(&r);
            }
            k >>= 1;
            if k > 0 {
                r = r.prod(&r);
            }
        }
        res
    }
}

pub struct AddMulMonoid;
impl MatrixMonoid for AddMulMonoid{
    type S = i64;

    fn zero()->Self::S {
        0
    }

    fn one()->Self::S {
        1
    }

    fn sum(&a: &Self::S, &b: &Self::S)->Self::S {
        if a+b < MOD{a+b}else{a+b-MOD}
    }

    fn mul(a: &Self::S, b: &Self::S)->Self::S {
        a*b%MOD
    }
}

pub struct MinPlusMonoid;
impl MatrixMonoid for MinPlusMonoid{
    type S = i64;

    fn zero()->Self::S {
        INF
    }

    fn one()->Self::S {
        0
    }

    fn sum(&a: &Self::S, &b: &Self::S)->Self::S {
        a.min(b)
    }

    fn mul(&a: &Self::S, &b: &Self::S)->Self::S {
        a+b
    }
}
