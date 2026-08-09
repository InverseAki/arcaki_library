pub struct MintCombination{
    fact: Vec<MI>,
    inv_fact: Vec<MI>,
    inv: Vec<MI>,
}

impl MintCombination{
    pub fn new(n: usize)->Self {
        let mut fact = vec![MI::new(1); n+1];
        let mut inv_fact = vec![MI::new(0); n+1];
        let mut inv = vec![MI::new(1); n+1];
        for i in 0..n{
            fact[i+1] = fact[i]*(i+1);
        }
        inv_fact[n] = MI::new(1)/fact[n];
        for i in (0..n).rev(){
            inv_fact[i] = inv_fact[i+1]*(i+1);
            inv[i+1] = fact[i]*inv_fact[i+1];
        }
        MintCombination { fact, inv_fact, inv }
    }

    #[inline]
    pub fn inv(&self, x: usize)->MI{
        self.inv[x]
    }

    #[inline]
    pub fn f(&self, x: usize)->MI{
        self.fact[x]
    }

    #[inline]
    pub fn fi(&self, x: usize)->MI{
        self.inv_fact[x]
    }

    #[inline]
    pub fn p(&self, x: usize, y: usize)->MI{
        if x < y{MI::new(0)}
        else {self.fact[x]*self.inv_fact[x-y]}
    }

    #[inline]
    pub fn c(&self, x: usize, y: usize)->MI{
        if x < y{return MI::new(0);}
        self.fact[x]*self.inv_fact[y]*self.inv_fact[x-y]
    }
}

pub fn convolution_merge(vs: &mut Vec<Vec<MI>>)->Vec<MI>{
    let n = vs.len();
    let mut heap = BinaryHeap::new();
    for i in 0..n{
        heap.push((Reverse(vs[i].len()), i));
    }
    for _ in 0..n-1{
        let (_, idx1) = heap.pop().unwrap();
        let (_, idx2) = heap.pop().unwrap();
        let v1 = take(&mut vs[idx1]);
        let v2 = take(&mut vs[idx2]);
        let nx = idx1.min(idx2);
        let v = convolution(&v1, &v2);
        heap.push((Reverse(v.len()), nx));
        vs[nx] = v;
    }
    take(&mut vs[0])
}
