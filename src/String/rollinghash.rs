const RM: u64 = (1u64<<61)-1;
const RB: u64 = 1001399;
#[inline]
fn mod_add(a: u64, b: u64)->u64{
    let mut x = a+b;
    if x >= RM{x -= RM;}
    x
}
#[inline]
fn mod_sub(a:u64,b:u64)->u64{
    if a>=b{a-b}else{RM+a-b}
}
#[inline]
fn mod_mul(a:u64,b:u64)->u64{
    let t = a as u128*b as u128;
    let mut x = (t>>61)as u64+(t as u64&RM);
    x=(x>>61)+(x&RM);
    if x >= RM{x -=RM;}
    x
}
pub struct RollingHash {
    hash: Vec<u64>,
    pow: Vec<u64>,
}

impl RollingHash {
    pub fn char(s: &Vec<char>) -> Self {
        let (mut pow, mut hash) = (Vec::from([1]), Vec::from([0]));
        let (mut p, mut h) = (1, 0);
        for i in 0..s.len() {
            p = mod_mul(p, RB);
            h = mod_add(mod_mul(h, RB), s[i]as u64+1);
            pow.push(p);
            hash.push(h);
        }
        RollingHash {
            hash,
            pow,
        }
    }

    pub fn num_seq(a: &Vec<usize>) -> Self {
        let (mut pow, mut hash) = (Vec::from([1]), Vec::from([0]));
        let (mut p, mut h) = (1, 0);
        for i in 0..a.len() {
            p = mod_mul(p, RB);
            h = mod_add(mod_mul(h, RB), a[i]as u64+1);
            pow.push(p);
            hash.push(h);
        }
        RollingHash {
            hash,
            pow,
        }
    }

    // mxは種類数上限。あと種類はちゃん全部列挙しないとなので配列が複数あるなら全部まとめてから座圧せよ。
    pub fn num_set(a: &Vec<usize>, mx: usize) -> Self {
        let (mut pow, mut hash) = (Vec::from([1]), Vec::from([0]));
        let (mut p, mut h) = (1, 0);
        let mut used = vec![false; mx];
        for _ in 0..mx {
            p = mod_mul(p, RB);
            pow.push(p);
        }
        for i in 0..a.len() {
            if !used[a[i]] {
                used[a[i]] = true;
                h = mod_add(h, pow[a[i]]);
            }
            hash.push(h);
        }
        RollingHash {
            hash,
            pow,
        }
    }

    pub fn num_map(a: &Vec<usize>, mx: usize) -> Self {
        let (mut pow, mut hash) = (Vec::from([1]), Vec::from([0]));
        let (mut p, mut h) = (1, 0);
        for _ in 0..mx {
            p = mod_mul(p, RB);
            pow.push(p);
        }
        for i in 0..a.len() {
            h = mod_add(h, pow[a[i]]);
            hash.push(h);
        }
        RollingHash {
            hash,
            pow,
        }
    }

    pub fn get(&self, l: usize, r: usize) -> u64 {
        mod_sub(self.hash[r], mod_mul(self.hash[l], self.pow[r-l]))
    }

    pub fn len_hash(&self, l: usize)->u64{
        self.pow[l]
    }

    pub fn map_get(&self, l: usize, r: usize) -> u64 {
        mod_sub(self.hash[r], self.hash[l])
    }

    pub fn same(&self, l1: usize, r1: usize, l2: usize, r2: usize) -> bool {
        self.get(l1, r1) == self.get(l2, r2)
    }
}
