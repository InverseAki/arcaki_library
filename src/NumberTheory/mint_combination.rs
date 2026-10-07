pub struct MintCombination {
    fact: Vec<MI>,
    inv_fact: Vec<MI>,
    inv: Vec<MI>,
}

impl MintCombination {
    pub fn new(n: usize) -> Self {
        let mut fact = vec![MI::new(1); n + 1];
        let mut inv_fact = vec![MI::new(0); n + 1];
        let mut inv = vec![MI::new(1); n + 1];
        for i in 0..n {
            fact[i + 1] = fact[i] * (i + 1);
        }
        inv_fact[n] = MI::new(1) / fact[n];
        for i in (0..n).rev() {
            inv_fact[i] = inv_fact[i + 1] * (i + 1);
            inv[i + 1] = fact[i] * inv_fact[i + 1];
        }
        MintCombination {
            fact,
            inv_fact,
            inv,
        }
    }

    #[inline]
    pub fn inv(&self, x: usize) -> MI {
        assert!(x > 0, "zero has no inverse");
        self.inv[x]
    }

    #[inline]
    pub fn f(&self, x: usize) -> MI {
        self.fact[x]
    }

    #[inline]
    pub fn fi(&self, x: usize) -> MI {
        self.inv_fact[x]
    }

    #[inline]
    pub fn p(&self, x: usize, y: usize) -> MI {
        if x < y {
            MI::new(0)
        } else {
            self.fact[x] * self.inv_fact[x - y]
        }
    }

    #[inline]
    pub fn c(&self, x: usize, y: usize) -> MI {
        if x < y {
            return MI::new(0);
        }
        self.fact[x] * self.inv_fact[y] * self.inv_fact[x - y]
    }
}

pub fn convolution_merge(vs: &mut Vec<Vec<MI>>) -> Vec<MI> {
    convolution_merge_with_mx(vs, usize::MAX)
}

pub fn convolution_merge_with_mx(vs: &mut Vec<Vec<MI>>, mx: usize) -> Vec<MI> {
    if mx == 0 || vs.iter().any(Vec::is_empty) {
        for v in vs.iter_mut() {
            v.clear();
        }
        return vec![];
    }
    if vs.is_empty() {
        return vec![MI::new(1)];
    }
    let mut heap = std::collections::BinaryHeap::new();
    for (i, v) in vs.iter_mut().enumerate() {
        v.truncate(mx);
        heap.push((std::cmp::Reverse(v.len()), i));
    }
    while heap.len() > 1 {
        let (_, i) = heap.pop().unwrap();
        let (_, j) = heap.pop().unwrap();
        let a = std::mem::take(&mut vs[i]);
        let b = std::mem::take(&mut vs[j]);
        let mut c = convolution(&a, &b);
        c.truncate(mx);
        vs[i] = c;
        heap.push((std::cmp::Reverse(vs[i].len()), i));
    }
    std::mem::take(&mut vs[heap.pop().unwrap().1])
}
