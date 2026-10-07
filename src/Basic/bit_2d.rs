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

pub struct BIT2DArray<T: Ord> {
    h: usize,
    ys: Vec<T>,
    flat: Vec<usize>,
    xp: Vec<usize>,
    data: Vec<i64>,
}

impl<T: Ord> BIT2DArray<T> {
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

    pub fn prefix(&self, x: usize, y: &T) -> i64 {
        assert!(x <= self.h, "x must be in 0..=h");
        self.prefix_rank(x, self.ys.partition_point(|v| v < y))
    }

    pub fn prod(&self, lx: usize, ly: &T, rx: usize, ry: &T) -> i64 {
        assert!(lx <= rx && rx <= self.h, "invalid x range");
        assert!(ly <= ry, "invalid y range");
        let l = self.ys.partition_point(|v| v < ly);
        let r = self.ys.partition_point(|v| v < ry);
        self.prefix_rank(rx, r) + self.prefix_rank(lx, l)
            - self.prefix_rank(lx, r) - self.prefix_rank(rx, l)
    }
}
