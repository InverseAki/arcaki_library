#[derive(Clone, Debug)]
pub struct Predecessor64{
    n: usize,
    d: Vec<Vec<u64>>,
}

impl Predecessor64 {
    // ~2e5: 3, ~1e7: 4, ~1e9: 5
    pub fn new(n: usize)->Self{
        let d = (0..n).into_iter().map(|k| vec![0; 1<<(6*(n-k-1))]).collect::<Vec<Vec<u64>>>();
        Predecessor64{
            n, d
        }
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.d[self.n-1][0]==0
    }

    #[inline(always)]
    pub fn include(&self, p: usize) -> bool {
        self.d[0][p>>6]&1<<(p&63)!=0
    }

    #[inline(always)]
    pub fn insert(&mut self, p: usize){
        for i in 0..self.n{
            if self.d[i][p>>(6*(i+1))]&1<<((p>>(6*i))&63)==0{
                self.d[i][p>>(6*(i+1))] |= 1<<((p>>(6*i))&63);
            } else {
                return;
            }
        }
    }

    #[inline(always)]
    pub fn remove(&mut self, p: usize){
        if self.d[0][p>>6]&1<<(p&63)==0{return;}
        for i in 0..self.n{
            self.d[i][p>>(6*(i+1))] ^= 1<<((p>>(6*i))&63);
            if self.d[i][p>>(6*(i+1))]!=0{
                return;
            } 
        }
    }

    #[inline(always)]
    fn ml(r: usize)->u64{
        (1<<r)-1
    }

    #[inline(always)]
    fn mr(l: usize)->u64{
        if l==63{return 0;}
        !((1<<(l+1))-1)
    }

    #[inline(always)]
    fn msb(bit: u64)->usize{
        63-bit.leading_zeros()as usize
    }

    #[inline(always)]
    fn lsb(bit: u64)->usize{
        bit.trailing_zeros()as usize
    }

    //存在しないは!0
    #[inline(always)]
    pub fn prev(&self, mut p: usize)->usize{
        for i in 0..self.n{
            if Self::ml(p&63)&self.d[i][p>>6]!=0{
                let mut res = ((p>>6)<<6)|Self::msb(self.d[i][p>>6]&Self::ml(p&63));
                for j in (0..i).rev(){
                    res = (res<<6)|Self::msb(self.d[j][res]);
                }
                return res;
            }
            p >>= 6;
        }
        !0
    }

    #[inline(always)]
    pub fn next(&self, mut p: usize)->usize{
        for i in 0..self.n{
            if Self::mr(p&63)&self.d[i][p>>6]!=0{
                let mut res = ((p>>6)<<6)|Self::lsb(self.d[i][p>>6]&Self::mr(p&63));
                for j in (0..i).rev(){
                    res = (res<<6)|Self::lsb(self.d[j][res]);
                }
                return res;
            }
            p >>= 6;
        }
        !0
    }

    #[inline(always)]
    pub fn inprev(&self, p: usize)->usize{
        if self.include(p){p}
        else {self.prev(p)}
    }

    #[inline(always)]
    pub fn innext(&self, p: usize)->usize{
        if self.include(p){p}
        else {self.next(p)}
    }

    #[inline(always)]
    pub fn min(&self)->usize{
        self.innext(0)
    }

    #[inline(always)]
    pub fn max(&self)->usize{
        self.inprev((1<<(6*self.n))-1)
    }
}
