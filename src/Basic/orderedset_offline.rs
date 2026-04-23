// [l, r)の半開区間で設定します。
pub struct BIT<T> where T: Copy + std::ops::Add<Output = T> + std::ops::Sub<Output = T>+PartialOrd{
    n: usize,
    vec: Vec<T>,
    zero: T,
}

impl<T> BIT<T> where T: Copy + std::ops::Add<Output = T> + std::ops::Sub<Output = T>+PartialOrd{
    pub fn new(n: usize, zero: T) -> Self {
        let k = n.max(1);
        let base = vec![zero; k + 1];
        BIT { n: k, vec: base, zero }
    }

    #[inline]
    pub fn add(&mut self, mut idx: usize, x: T) {
        idx += 1;
        while idx <= self.n {
            self.vec[idx] = self.vec[idx] + x;
            idx += idx & (!idx + 1);
        }
    }

    #[inline]
    pub fn g(&self, mut r: usize) -> T {
        let mut res = self.zero;
        while r > 0 {
            res = res + self.vec[r];
            r -= r & (!r + 1);
        }
        res
    }

    #[inline]
    pub fn prod(&self, l: usize, r: usize) -> T {
        self.g(r) - self.g(l)
    }

    #[inline]
    pub fn get(&self, p: usize)->T{
        self.g(p+1)-self.g(p)
    }

    #[inline]
    pub fn set(&mut self, p: usize, x: T){
        let pre = self.get(p);
        self.add(p, x-pre);
    }

    // Sum(A[0, r))がac < Tとなる最大のrを返す
    #[inline]
    pub fn lower_bound(&self, ac: T)->usize{
        let mut r = 0;
        let mut cur = self.zero;
        let mut k = self.n;
        while k > 0{
            let nx = r+k;
            if nx <= self.n{
                let v = cur+self.vec[nx];
                if v < ac{
                    r = nx;
                    cur = v;
                } 
            }
            k >>= 1;
        }
        r
    }
}

pub struct OrderedSet<T> where T: Copy+Ord+Hash{
    n: usize,
    a: Vec<T>,
    cnt: Vec<i64>,
    map: FxMap<T, usize>,
    bit: BIT<i64>,
    all: i64,
}

impl<T> OrderedSet<T> where T: Copy+Ord+Hash{
    pub fn new(mut xs: Vec<T>)->Self {
        xs.sort_unstable();
        xs.dedup();
        let n = xs.len();
        let mut map: HashMap<T, usize, BuildHasherDefault<FxHasher>> = FxMap::default();
        for (i, &v) in xs.iter().enumerate(){
            map.insert(v, i);
        }
        let cnt = vec![0; n];
        let bit = BIT::new(n, 0);
        OrderedSet { n, a: xs, cnt, map, bit, all: 0}
    }

    pub fn raw(xs: Vec<T>, map: FxMap<T, usize>)->Self {
        let n = xs.len();
        let cnt = vec![0; n];
        let bit = BIT::new(n, 0);
        OrderedSet { n, a: xs, cnt, map, bit, all: 0}
    }

    #[inline]
    pub fn insert(&mut self, p: T){
        let p= self.map[&p];
        self.cnt[p] += 1;
        self.bit.add(p, 1);
        self.all += 1;
    }

    #[inline]
    pub fn erase_one(&mut self, p: T){
        if let Some(&p)= self.map.get(&p){
            if self.cnt[p]==0{return;}
            self.all -= 1;
            self.cnt[p] -= 1;
            self.bit.add(p, -1);
        }
    }

    #[inline]
    pub fn erase_all(&mut self, p: T){
        if let Some(&p) = self.map.get(&p){
            self.all -= self.cnt[p];
            self.bit.add(p, -self.cnt[p]);
            self.cnt[p] = 0;
        }
    }

    #[inline]
    pub fn one(&mut self, p: T){
        let p = self.map[&p];
        self.all += 1-self.cnt[p];
        self.bit.add(p, 1-self.cnt[p]);
        self.cnt[p] = 1;
    }

    #[inline]
    pub fn zero(&mut self, p: T){
        if let Some(&p) = self.map.get(&p){
            self.all -= self.cnt[p];
            self.bit.add(p, -self.cnt[p]);
            self.cnt[p] = 0;
        }
    }

    #[inline]
    pub fn set(&mut self, p: T, num: usize){
        let p = self.map[&p];
        let num = num as i64;
        self.all += num-self.cnt[p];
        self.bit.add(p, num-self.cnt[p]);
        self.cnt[p] = num;
    }

    #[inline]
    pub fn min(&self)->Option<T>{
        if self.all==0{return None;}
        let r = self.bit.lower_bound(1);
        Some(self.a[r])
    }

    #[inline]
    pub fn max(&self)->Option<T>{
        if self.all == 0{
            None
        } else {
            let r = self.bit.lower_bound(self.all);
            Some(self.a[r])
        }
    }

    #[inline]
    pub fn kth_min(&self, k: usize)->Option<T>{
        if self.all==0{return None;}
        let r = self.bit.lower_bound(k as i64+1);
        if r < self.n {
            Some(self.a[r])
        } else {
            None
        }
    }

    #[inline]
    pub fn kth_max(&self, k: usize)->Option<T>{
        if (k as i64) < self.all{
            let r = self.bit.lower_bound(self.all-k as i64);
            Some(self.a[r])
        } else {
            None
        }
    }

    #[inline]
    pub fn get(&self, p: T)->usize{
        if let Some(&p) = self.map.get(&p){
            self.cnt[p]as usize
        } else {
            0
        }
    }

    #[inline]
    pub fn min_pop(&mut self)->Option<T>{
        if self.all==0{return None;}
        let r = self.bit.lower_bound(1);
        self.bit.add(r, -1);
        self.cnt[r] -= 1;
        self.all -= 1;
        Some(self.a[r])
    }

    #[inline]
    pub fn max_pop(&mut self)->Option<T>{
        if self.all==0{return None;}
        let r = self.bit.lower_bound(self.all);
        self.bit.add(r, -1);
        self.cnt[r] -= 1;
        self.all -= 1;
        Some(self.a[r])
    }

    #[inline]
    pub fn kth_pop(&mut self, k: usize)->Option<T>{
        let r = self.bit.lower_bound(k as i64+1);
        if r >= self.n{
            None
        } else {
            self.bit.add(r, -1);
            self.cnt[r] -= 1;
            self.all -= 1;
            Some(self.a[r])
        }
    }

    // x)
    #[inline]
    pub fn count_l(&self, x: T)->usize{
        let l = self.a.partition_point(|&v| v<x);
        self.bit.g(l)as usize
    }

    #[inline]
    pub fn count_leq(&self, x: T)->usize{
        let l = self.a.partition_point(|&v| v<=x);
        self.bit.g(l)as usize
    }

    #[inline]
    pub fn count_r(&self, x: T)->usize{
        self.all as usize-self.count_leq(x)
    }

    #[inline]
    pub fn count_req(&self, x: T)->usize{
        self.all as usize-self.count_l(x)
    }

    #[inline]
    pub fn next(&self, x: T)->Option<T>{
        let l = self.a.partition_point(|&v| v<=x);
        let num = self.bit.g(l);
        let p = self.bit.lower_bound(num+1);
        if p < self.n{
            Some(self.a[p])
        } else {
            None
        }
    }

    #[inline]
    pub fn prev(&self, x: T) -> Option<T> {
        let l = self.a.partition_point(|&v| v < x);
        let num = self.bit.g(l);
        if num == 0 {
            None
        } else {
            let p = self.bit.lower_bound(num);
            Some(self.a[p])
        }
    }

    #[inline]
    pub fn innext(&self, x: T)->Option<T>{
        let l = self.a.partition_point(|&v| v<x);
        let num = self.bit.g(l);
        let p = self.bit.lower_bound(num+1);
        if p < self.n{
            Some(self.a[p])
        } else {
            None
        }
    }

    #[inline]
    pub fn inprev(&self, x: T) -> Option<T> {
        let l = self.a.partition_point(|&v| v <= x);
        let num = self.bit.g(l);
        if num == 0 {
            None
        } else {
            let p = self.bit.lower_bound(num);
            Some(self.a[p])
        }
    }

    #[inline]
    pub fn from_x_kth_min(&self, x: T, k: usize,)->Option<T>{
        self.kth_min(k+self.count_l(x))
    }

    #[inline]
    pub fn from_x_kth_max(&self, x: T, k: usize)->Option<T>{
        self.kth_max(k+self.count_r(x))
    }
}
