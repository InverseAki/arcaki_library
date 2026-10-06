pub struct BIT2D {
    h: usize,
    xs: Vec<i32>,
    flat: Vec<i32>,
    xp: Vec<usize>,
    data: Vec<i64>,
}

impl BIT2D {
    pub fn new(add: &[(i32, i32)])->Self{
        let mut xs = Vec::new();
        for &(x, _) in add{xs.push(x);}
        xs.sort_unstable();xs.dedup();
        let h = xs.len();
        let mut ys: Vec<Vec<i32>> = vec![Vec::new(); h+1];
        let mut flat = Vec::new();
        let mut xp = vec![0,0];
        for &(x,y)in add{ys[xs.partition_point(|&z| z<x)+1].push(y);}
        for i in 1..=h {
            ys[i].sort_unstable();ys[i].dedup();
            let j = i+(i&(!i+1));
            if j <= h{
                let z = ys[i].clone();
                ys[j].extend(z);
            }
            flat.extend(take(&mut ys[i]));
            xp.push(flat.len());
        }
        let data = vec![0;flat.len()+1];
        BIT2D { h, xs, flat, xp, data }
    }

    #[inline(always)]
    fn y_add(&mut self, p: usize, v: i32, w: i64){
        let l = self.xp[p];
        let n = self.xp[p+1]-l;
        let mut p = self.flat[l..l+n].partition_point(|&y| y<v)+1;
        while p <= n {
            self.data[p+l] += w;
            p += p&(!p+1);
        }
    }

    #[inline]
    pub fn add(&mut self, u: i32, v: i32, w: i64) {
        let mut u = self.xs.partition_point(|&y| y<u)+1;
        while u <= self.h {
            self.y_add(u, v, w);
            u += u & (!u + 1);
        }
    }

    #[inline(always)]
    fn y_g(&self, p: usize, v: i32, res: &mut i64){
        let l = self.xp[p];
        let n = self.xp[p+1]-l;
        let mut p = self.flat[l..l+n].partition_point(|&y| y<v);
        while p > 0 {
            *res += self.data[p+l];
            p -= p&(!p+1);
        }
    }

    #[inline(always)]
    fn g(&self, u: i32, v: i32)->i64 {
        let mut res = 0;
        let mut u = self.xs.partition_point(|&y| y<u);
        while u > 0 {
            self.y_g(u, v, &mut res);
            u -= u&(!u+1);
        }
        res
    }

    #[inline]
    pub fn prod(&self, lx: i32, ly: i32, rx: i32, ry: i32)->i64 {
        self.g(rx, ry)+self.g(lx, ly)-self.g(lx, ry)-self.g(rx, ly)
    }
}

/// x は 0..h の配列添字、y は Ord な座標の二次元 BIT。重みは i64。
/// 更新する全座標を new に事前登録する（重複可）。
/// 座標を所有するため T に Clone / Copy は不要。
///
/// ```ignore
/// let mut bit = BIT2DArray::new(5, [(0, "apple"), (3, "pear")]);
/// bit.add(3, &"pear", 7);
/// assert_eq!(bit.prod(1, &"banana", 5, &"z"), 7);
/// ```
/// 登録点数 m とすると構築 O(m log m + m log(h+1) + h)、
/// 空間 O(h + m log(h+1))。更新・検索は O(log(m+1) log(h+1))。
/// 比較を O(1) とした計算量。
pub struct BIT2DArray<T: Ord> {
    h: usize,
    ys: Vec<T>,
    flat: Vec<usize>,
    xp: Vec<usize>,
    data: Vec<i64>,
}

impl<T: Ord> BIT2DArray<T> {
    /// h: x 方向の配列長。add: 将来更新する (x, y) の全座標。
    /// Vec・配列・所有権を渡す iterator を受け取れる。
    pub fn new(h: usize, add: impl IntoIterator<Item = (usize, T)>) -> Self {
        let mut points: Vec<_> = add.into_iter().collect();
        assert!(points.iter().all(|(x, _)| *x < h), "x must be in 0..h");
        points.sort_unstable_by(|a, b| a.1.cmp(&b.1));
        let mut ys = Vec::new();
        let mut nodes: Vec<Vec<usize>> = (0..=h).map(|_| Vec::new()).collect();
        for (x, y) in points {
            if ys.last() != Some(&y) {
                ys.push(y);
            }
            let id = ys.len() - 1;
            let mut p = x + 1;
            while p <= h {
                // y 順に登録するため各節点も整列済み。重複だけ除く。
                if nodes[p].last() != Some(&id) {
                    nodes[p].push(id);
                }
                p += p & p.wrapping_neg();
            }
        }
        let mut flat = Vec::new();
        let mut xp = vec![0, 0];
        for node in nodes.into_iter().skip(1) {
            flat.extend(node);
            xp.push(flat.len());
        }
        let data = vec![0; flat.len() + 1];
        Self { h, ys, flat, xp, data }
    }

    /// 事前登録した点 (x, y) に w を加える。負の重みも可。
    /// x は 0..h。未登録点への更新は不可。
    pub fn add(&mut self, x: usize, y: &T, w: i64) {
        assert!(x < self.h, "x must be in 0..h");
        let id = self.ys.binary_search(y).expect("y must be registered");
        let mut u = x + 1;
        while u <= self.h {
            let l = self.xp[u];
            let n = self.xp[u + 1] - l;
            let mut p = self.flat[l..l + n]
                .binary_search(&id)
                .expect("point must be registered") + 1;
            while p <= n {
                self.data[l + p] += w;
                p += p & p.wrapping_neg();
            }
            u += u & u.wrapping_neg();
        }
    }

    fn prefix_rank(&self, mut x: usize, y: usize) -> i64 {
        let mut res = 0;
        while x > 0 {
            let l = self.xp[x];
            let r = self.xp[x + 1];
            let mut p = self.flat[l..r].partition_point(|&id| id < y);
            while p > 0 {
                res += self.data[l + p];
                p -= p & p.wrapping_neg();
            }
            x -= x & x.wrapping_neg();
        }
        res
    }

    /// [0, x) × { v | v < y } の和。x は 0..=h、y は未登録でも可。
    pub fn prefix(&self, x: usize, y: &T) -> i64 {
        assert!(x <= self.h, "x must be in 0..=h");
        self.prefix_rank(x, self.ys.partition_point(|v| v < y))
    }

    /// [lx, rx) × [ly, ry) の和。0 <= lx <= rx <= h、ly <= ry。
    /// y の端点は未登録でも可。空区間の和は 0。
    pub fn prod(&self, lx: usize, ly: &T, rx: usize, ry: &T) -> i64 {
        assert!(lx <= rx && rx <= self.h, "invalid x range");
        assert!(ly <= ry, "invalid y range");
        let l = self.ys.partition_point(|v| v < ly);
        let r = self.ys.partition_point(|v| v < ry);
        self.prefix_rank(rx, r) + self.prefix_rank(lx, l)
            - self.prefix_rank(lx, r) - self.prefix_rank(rx, l)
    }
}
