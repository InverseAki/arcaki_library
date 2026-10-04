/// 加算・減算による Fenwick tree。区間は 0-indexed の [l, r)。
/// T の加算は可換、zero は加法単位元。演算結果は T に収まること。
/// 構築 O(n)、更新・取得・探索 O(log n)、領域 O(n)。
pub struct BIT<T>
where
    T: Copy + std::ops::Add<Output = T> + std::ops::Sub<Output = T> + PartialOrd,
{
    n: usize,
    vec: Vec<T>,
    zero: T,
}

impl<T> BIT<T>
where
    T: Copy + std::ops::Add<Output = T> + std::ops::Sub<Output = T> + PartialOrd,
{
    pub fn new(n: usize, zero: T) -> Self {
        let base = vec![zero; n + 1];
        BIT { n, vec: base, zero }
    }

    pub fn from_vec(a: Vec<T>, zero: T) -> Self {
        let n = a.len();
        let mut vec = vec![zero; n + 1];
        for i in 0..n {
            vec[i + 1] = a[i];
        }
        for i in 1..=n {
            let j = i + (i & (!i + 1));
            if j <= n {
                vec[j] = vec[j] + vec[i];
            }
        }
        BIT { n, vec, zero }
    }

    pub fn len(&self) -> usize {
        self.n
    }

    pub fn is_empty(&self) -> bool {
        self.n == 0
    }

    #[inline]
    pub fn add(&mut self, mut idx: usize, x: T) {
        assert!(idx < self.n);
        idx += 1;
        while idx <= self.n {
            self.vec[idx] = self.vec[idx] + x;
            idx += idx & (!idx + 1);
        }
    }

    #[inline]
    pub fn g(&self, mut r: usize) -> T {
        assert!(r <= self.n);
        let mut res = self.zero;
        while r > 0 {
            res = res + self.vec[r];
            r -= r & (!r + 1);
        }
        res
    }

    #[inline]
    pub fn prod(&self, l: usize, r: usize) -> T {
        assert!(l <= r && r <= self.n);
        self.g(r) - self.g(l)
    }

    #[inline]
    pub fn get(&self, p: usize) -> T {
        assert!(p < self.n);
        self.g(p + 1) - self.g(p)
    }

    #[inline]
    pub fn set(&mut self, p: usize, x: T) {
        let pre = self.get(p);
        self.add(p, x - pre);
    }

    /// 全要素が非負であること（累積和が単調非減少）。
    /// ac > zero のとき、sum(A[0..r]) < ac を満たす最大の r を返す。
    /// つまり累積和が ac 以上になる最初の要素の 0-indexed 添字。
    /// 到達しなければ n、ac <= zero または空配列なら 0。
    #[inline]
    pub fn lower_bound(&self, ac: T) -> usize {
        if self.n == 0 || ac <= self.zero {
            return 0;
        }
        let mut r = 0;
        let mut cur = self.zero;
        let mut k = 1 << self.n.ilog2();
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
