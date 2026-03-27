#[derive(Clone, Copy, Debug)]
pub struct TopTwo{
    a1: (i64, usize),
    a2: (i64, usize),
}

impl TopTwo{
    #[inline]
    pub fn new()->Self {
        TopTwo { a1: (-INF, !0), a2: (-INF, !0) }
    }

    #[inline]
    pub fn first(&self)->(i64, usize){
        self.a1
    }

    #[inline]
    pub fn second(&self)->(i64, usize){
        self.a2
    }

    #[inline]
    pub fn push(&mut self, rhs: (i64, usize)){
        if self.a1.0 < rhs.0 {
            if self.a1.1==rhs.1{
                self.a1 = rhs;
            } else {
                self.a2 = self.a1;
                self.a1 = rhs;
            }
        } else if self.a1.1 != rhs.1 && self.a2.0 < rhs.0 {
            self.a2 = rhs;
        }
    }

    #[inline]
    pub fn merge(&mut self, rhs: Self){
        self.push(rhs.a1);
        self.push(rhs.a2);
    }

    #[inline]
    pub fn best_rv(&self, id: usize)->(i64, usize){
        if self.a1.1==id {
            self.a2
        } else {
            self.a1
        }
    }
}
