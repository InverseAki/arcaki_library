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
}

#[derive(Clone, Copy, Debug)]
pub enum PointAddRectangleSumQuery {
    Add{
        x: i32, y: i32, w: i64,
    }, 
    Query{
        lx: i32, ly: i32, rx: i32, ry: i32,
    }
}

#[derive(Clone)]
pub struct PointAddRectangleSum {
    yn: usize, 
    data: Vec<PointAddRectangleSumQuery>,
}

impl PointAddRectangleSum {
    pub fn new()->Self{
        PointAddRectangleSum { yn: 0, data: Vec::new() }
    }

    #[inline]
    pub fn push_add(&mut self, x: i32, y: i32, w: i64){
        self.data.push(PointAddRectangleSumQuery::Add{x, y, w});
        self.yn+=1;
    }

    // [lx, rx)×[ly, ry) なので注意
    #[inline]
    pub fn push_query(&mut self, lx: i32, ly: i32, rx: i32, ry: i32,){
        self.data.push(PointAddRectangleSumQuery::Query {lx, ly, rx, ry});
        self.yn+=2;
    }

    pub fn solve(&mut self)->Vec<i64>{
        let q = self.data.len();
        let mut ans = vec![0; q];
        let mut ys = Vec::with_capacity(self.yn);
        for x in self.data.iter(){
            match *x {
                PointAddRectangleSumQuery::Add { y, .. } => {
                    ys.push(y);
                }
                PointAddRectangleSumQuery::Query { ly, ry, .. } => {
                    ys.push(ly); ys.push(ry);
                }
            }
        }
        ys.sort_unstable();ys.dedup();
        for x in self.data.iter_mut(){
            match x {
                PointAddRectangleSumQuery::Add { y, .. } => {
                    *y = ys.binary_search(&*y).unwrap()as i32;
                }
                PointAddRectangleSumQuery::Query { ly, ry, .. } => {
                    *ly = ys.binary_search(&*ly).unwrap()as i32;
                    *ry = ys.binary_search(&*ry).unwrap()as i32;
                }
            }
        }
        let mut bit = BIT::new(ys.len(), 0);
        let mut seq = Vec::new();
        self.dfs(0, q, &mut bit, &mut seq, &mut ans);
        let mut res = Vec::new();
        for (i,&x) in self.data.iter().enumerate(){
            if matches!(x, PointAddRectangleSumQuery::Query{..}){
                res.push(ans[i]);
            }
        }
        self.data.clear();
        res
    }

    #[inline]
    fn dfs(&self, l: usize, r: usize, bit: &mut BIT<i64>, seq: &mut Vec<(i32, i32, i32, i64)>, ans: &mut Vec<i64>){
        if l+1==r {return;}
        let m = (l+r)>>1;
        for i in l..m {
            if let PointAddRectangleSumQuery::Add { x, y, w} = self.data[i] {
                seq.push((x, I, y, w));
            }
        }
        for i in m..r {
            if let PointAddRectangleSumQuery::Query { lx, ly, rx, ry } = self.data[i]{
                seq.push((lx, ly, ry, i as i64));
                seq.push((rx, ly, ry, -(i as i64+1)));
            }
        }
        seq.sort_unstable_by_key(|w| (w.0,w.1));
        for &(_, l, r, w) in seq.iter(){
            if l==I {
                bit.add(r as usize, w);
            } else if w < 0{
                ans[(-w-1) as usize] += bit.prod(l as usize, r as usize);
            } else {
                ans[w as usize] -= bit.prod(l as usize, r as usize);
            }
        }
        for &(_, l, r, w) in seq.iter(){
            if l==I {
                bit.add(r as usize, -w);
            }
        }
        seq.clear();
        self.dfs(l, m, bit, seq, ans);
        self.dfs(m, r, bit, seq, ans);
    }
}
