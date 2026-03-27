// [0, r) の半開区間
pub struct BIT<T>
where
    T: Copy
        + std::ops::Add<Output = T>
        + std::ops::Sub<Output = T>
        + PartialOrd,
{
    n: usize,
    vec: Vec<T>,
    zero: T,
}

impl<T> BIT<T>
where
    T: Copy
        + std::ops::Add<Output = T>
        + std::ops::Sub<Output = T>
        + PartialOrd,
{
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
    pub fn get(&self, p: usize) -> T {
        self.g(p + 1) - self.g(p)
    }

    #[inline]
    pub fn set(&mut self, p: usize, x: T) {
        let pre = self.get(p);
        self.add(p, x - pre);
    }

    // Sum(A[0, r)) < ac となる最大の r を返す
    #[inline]
    pub fn lower_bound(&self, ac: T) -> usize {
        let mut r = 0;
        let mut cur = self.zero;
        let mut k = 1;
        while (k << 1) <= self.n {
            k <<= 1;
        }
        while k > 0 {
            let nx = r + k;
            if nx <= self.n {
                let v = cur + self.vec[nx];
                if v < ac {
                    r = nx;
                    cur = v;
                }
            }
            k >>= 1;
        }
        r
    }
}

pub struct OrderedSet {
    n: usize,
    cnt: Vec<i64>,
    bit: BIT<i64>,
    all: i64,
}

impl OrderedSet {
    pub fn new(n: usize) -> Self {
        OrderedSet {
            n,
            cnt: vec![0; n],
            bit: BIT::new(n, 0),
            all: 0,
        }
    }

    #[inline]
    fn assert_index(&self, p: usize) {
        assert!(p < self.n);
    }

    #[inline]
    pub fn insert(&mut self, p: usize) {
        self.assert_index(p);
        self.cnt[p] += 1;
        self.bit.add(p, 1);
        self.all += 1;
    }

    #[inline]
    pub fn erase_one(&mut self, p: usize) {
        self.assert_index(p);
        if self.cnt[p] == 0 {
            return;
        }
        self.cnt[p] -= 1;
        self.bit.add(p, -1);
        self.all -= 1;
    }

    #[inline]
    pub fn erase_all(&mut self, p: usize) {
        self.assert_index(p);
        if self.cnt[p] == 0 {
            return;
        }
        self.all -= self.cnt[p];
        self.bit.add(p, -self.cnt[p]);
        self.cnt[p] = 0;
    }

    #[inline]
    pub fn one(&mut self, p: usize) {
        self.assert_index(p);
        self.all += 1 - self.cnt[p];
        self.bit.add(p, 1 - self.cnt[p]);
        self.cnt[p] = 1;
    }

    #[inline]
    pub fn zero(&mut self, p: usize) {
        self.assert_index(p);
        self.all -= self.cnt[p];
        self.bit.add(p, -self.cnt[p]);
        self.cnt[p] = 0;
    }

    #[inline]
    pub fn set(&mut self, p: usize, num: usize) {
        self.assert_index(p);
        let num = num as i64;
        self.all += num - self.cnt[p];
        self.bit.add(p, num - self.cnt[p]);
        self.cnt[p] = num;
    }

    #[inline]
    pub fn get(&self, p: usize) -> usize {
        self.assert_index(p);
        self.cnt[p] as usize
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.all as usize
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.all == 0
    }

    #[inline]
    pub fn min(&self) -> Option<usize> {
        if self.all == 0 {
            None
        } else {
            Some(self.bit.lower_bound(1))
        }
    }

    #[inline]
    pub fn max(&self) -> Option<usize> {
        if self.all == 0 {
            None
        } else {
            Some(self.bit.lower_bound(self.all))
        }
    }

    #[inline]
    pub fn kth_min(&self, k: usize) -> Option<usize> {
        if (k as i64) >= self.all {
            None
        } else {
            Some(self.bit.lower_bound(k as i64 + 1))
        }
    }

    #[inline]
    pub fn kth_max(&self, k: usize) -> Option<usize> {
        if (k as i64) >= self.all {
            None
        } else {
            Some(self.bit.lower_bound(self.all - k as i64))
        }
    }

    #[inline]
    pub fn min_pop(&mut self) -> Option<usize> {
        let p = self.min()?;
        self.erase_one(p);
        Some(p)
    }

    #[inline]
    pub fn max_pop(&mut self) -> Option<usize> {
        let p = self.max()?;
        self.erase_one(p);
        Some(p)
    }

    #[inline]
    pub fn kth_pop(&mut self, k: usize) -> Option<usize> {
        let p = self.kth_min(k)?;
        self.erase_one(p);
        Some(p)
    }

    // < x
    #[inline]
    pub fn count_l(&self, x: usize) -> usize {
        self.bit.g(x.min(self.n)) as usize
    }

    // <= x
    #[inline]
    pub fn count_leq(&self, x: usize) -> usize {
        self.bit.g((x + 1).min(self.n)) as usize
    }

    // > x
    #[inline]
    pub fn count_r(&self, x: usize) -> usize {
        self.len() - self.count_leq(x)
    }

    // >= x
    #[inline]
    pub fn count_req(&self, x: usize) -> usize {
        self.len() - self.count_l(x)
    }

    #[inline]
    pub fn next(&self, x: usize) -> Option<usize> {
        if x + 1 >= self.n {
            return None;
        }
        let num = self.bit.g(x + 1);
        let p = self.bit.lower_bound(num + 1);
        if p < self.n {
            Some(p)
        } else {
            None
        }
    }

    #[inline]
    pub fn prev(&self, x: usize) -> Option<usize> {
        let num = self.bit.g(x.min(self.n));
        if num == 0 {
            None
        } else {
            Some(self.bit.lower_bound(num))
        }
    }

    #[inline]
    pub fn innext(&self, x: usize) -> Option<usize> {
        if x >= self.n {
            return None;
        }
        let num = self.bit.g(x);
        let p = self.bit.lower_bound(num + 1);
        if p < self.n {
            Some(p)
        } else {
            None
        }
    }

    #[inline]
    pub fn inprev(&self, x: usize) -> Option<usize> {
        let num = self.bit.g((x + 1).min(self.n));
        if num == 0 {
            None
        } else {
            Some(self.bit.lower_bound(num))
        }
    }

    #[inline]
    pub fn from_x_kth_min(&self, x: usize, k: usize) -> Option<usize> {
        self.kth_min(k + self.count_l(x))
    }

    #[inline]
    pub fn from_x_kth_max(&self, x: usize, k: usize) -> Option<usize> {
        self.kth_max(k + self.count_r(x))
    }
}