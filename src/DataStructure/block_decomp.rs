pub struct FixBlock{
    n: usize,
    b: usize,
}

impl FixBlock{
    pub fn new(n: usize, b: usize)->Self{
        Self{n,b}
    }

    pub fn decompose(&self, l: usize, r: usize)->Vec<(usize, usize, bool)>{
        if l==r{return vec![];}
        let lb = l/self.b;
        let rb = (r-1)/self.b;
        if lb==rb{return vec![(l, r, false)];}
        let le = ((lb+1)*self.b).min(self.n);
        let re = rb*self.b;
        let mut res = Vec::new();
        if l < le{
            res.push((l, le, false));
        }
        if lb+1 < rb{
            res.push((lb+1, rb, true));
        }
        if re < r{
            res.push((re, r, false));
        }
        res
    }
}