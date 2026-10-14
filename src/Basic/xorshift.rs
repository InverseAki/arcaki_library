// Source: https://github.com/InverseAki/arcaki_library/blob/main/src/Basic/xorshift.rs
pub struct XorShiftStar{
    state: u64,
}

impl XorShiftStar{
    pub fn new(seed: u64)->Self{
        XorShiftStar{state: seed}
    }

    pub fn next(&mut self)->u64{
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }

    pub fn get(&mut self, l: usize, r: usize) -> usize {
        let range = r - l + 1;
        let num = (self.next() as usize) % (range) + l;
        num
    }

    pub fn shuffle<T>(&mut self, a: &mut Vec<T>){
        let n = a.len();
        for i in 0..n{
            let j = self.get(0, i);
            a.swap(i, j);
        }
    }

    pub fn choose<T>(&mut self, a: &Vec<T>)->T where T: Copy{
        let n = a.len();
        let p = self.get(0, n-1);
        a[p]
    }
}
