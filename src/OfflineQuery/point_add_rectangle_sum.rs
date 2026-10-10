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
        let k = n.max(1);
        let base = vec![zero; k + 1];
        BIT {
            n: k,
            vec: base,
            zero,
        }
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
pub enum PointAddRectangleSumQuery<X = i32, Y = X> {
    Add { x: X, y: Y, w: i64 },
    Query { lx: X, ly: Y, rx: X, ry: Y },
}

#[derive(Clone)]
pub struct PointAddRectangleSum<X = i32, Y = X> {
    initial: Vec<(X, Y, i64)>,
    data: Vec<PointAddRectangleSumQuery<X, Y>>,
    adds: usize,
    queries: usize,
}

enum PointAddRectangleSumRanked<X> {
    Add {
        x: X,
        y: usize,
        w: i64,
    },
    Query {
        lx: X,
        ly: usize,
        rx: X,
        ry: usize,
        answer: usize,
    },
}

#[derive(Clone, Copy)]
struct PointAddRectangleSumEvent<'a, X> {
    x: &'a X,
    lo: usize,
    hi: usize,
    payload: i64,
}

impl<X: Ord, Y: Ord + Clone> PointAddRectangleSum<X, Y> {
    pub fn new() -> Self {
        Self {
            initial: Vec::new(),
            data: Vec::new(),
            adds: 0,
            queries: 0,
        }
    }

    pub fn build(points: impl IntoIterator<Item = (X, Y, i64)>) -> Self {
        let mut initial: Vec<_> = points.into_iter().collect();
        initial.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        Self {
            initial,
            data: Vec::new(),
            adds: 0,
            queries: 0,
        }
    }

    #[inline]
    pub fn push_add(&mut self, x: X, y: Y, w: i64) {
        self.data.push(PointAddRectangleSumQuery::Add { x, y, w });
        self.adds += 1;
    }

    #[inline]
    pub fn push_query(&mut self, lx: X, ly: Y, rx: X, ry: Y) {
        assert!(lx <= rx && ly <= ry, "invalid rectangle");
        self.data
            .push(PointAddRectangleSumQuery::Query { lx, ly, rx, ry });
        self.queries += 1;
    }

    pub fn solve(&mut self) -> Vec<i64> {
        let mut ans = vec![0; self.queries];
        if self.queries == 0 || (self.initial.is_empty() && self.adds == 0) {
            self.initial.clear();
            self.data.clear();
            self.adds = 0;
            self.queries = 0;
            return ans;
        }
        let dynamic_adds = self.adds;
        let mut ys = Vec::with_capacity(self.initial.len() + dynamic_adds);
        ys.extend(self.initial.iter().map(|p| p.1.clone()));
        for op in &self.data {
            if let PointAddRectangleSumQuery::Add { y, .. } = op {
                ys.push(y.clone());
            }
        }
        ys.sort_unstable();
        ys.dedup();
        let initial: Vec<_> = std::mem::take(&mut self.initial)
            .into_iter()
            .map(|(x, y, w)| {
                let y = ys.binary_search(&y).unwrap();
                (x, y, w)
            })
            .collect();
        let mut answer = 0;
        let data: Vec<_> = std::mem::take(&mut self.data)
            .into_iter()
            .map(|op| match op {
                PointAddRectangleSumQuery::Add { x, y, w } => PointAddRectangleSumRanked::Add {
                    x,
                    y: ys.binary_search(&y).unwrap(),
                    w,
                },
                PointAddRectangleSumQuery::Query { lx, ly, rx, ry } => {
                    let ly = ys.partition_point(|y| y < &ly);
                    let ry = ys.partition_point(|y| y < &ry);
                    let op = PointAddRectangleSumRanked::Query {
                        lx,
                        ly,
                        rx,
                        ry,
                        answer,
                    };
                    answer += 1;
                    op
                }
            })
            .collect();
        self.adds = 0;
        self.queries = 0;
        let yn = ys.len();
        drop(ys);
        let mut bit = BIT::new(yn, 0i64);
        let mut seq = Vec::new();
        if !initial.is_empty() {
            for op in &data {
                if let PointAddRectangleSumRanked::Query {
                    lx,
                    ly,
                    rx,
                    ry,
                    answer,
                } = op
                {
                    if lx < rx && ly < ry {
                        seq.push(PointAddRectangleSumEvent {
                            x: lx,
                            lo: *ly,
                            hi: *ry,
                            payload: *answer as i64,
                        });
                        seq.push(PointAddRectangleSumEvent {
                            x: rx,
                            lo: *ly,
                            hi: *ry,
                            payload: -(*answer as i64) - 1,
                        });
                    }
                }
            }
            seq.sort_unstable_by(|a, b| a.x.cmp(b.x));
            let mut p = 0;
            for e in &seq {
                while p < initial.len() && &initial[p].0 < e.x {
                    bit.add(initial[p].1, initial[p].2);
                    p += 1;
                }
                Self::answer_event(e, &bit, &mut ans);
            }
            bit.vec.fill(0);
            seq.clear();
        }
        drop(initial);
        if dynamic_adds != 0 {
            Self::dfs(&data, 0, data.len(), &mut bit, &mut seq, &mut ans);
        }
        ans
    }

    #[inline]
    fn answer_event(e: &PointAddRectangleSumEvent<'_, X>, bit: &BIT<i64>, ans: &mut [i64]) {
        let sum = bit.prod(e.lo, e.hi);
        if e.payload >= 0 {
            ans[e.payload as usize] -= sum;
        } else {
            ans[(-e.payload - 1) as usize] += sum;
        }
    }

    fn dfs<'a>(
        data: &'a [PointAddRectangleSumRanked<X>],
        l: usize,
        r: usize,
        bit: &mut BIT<i64>,
        seq: &mut Vec<PointAddRectangleSumEvent<'a, X>>,
        ans: &mut [i64],
    ) {
        if r - l <= 1 {
            return;
        }
        let m = (l + r) / 2;
        let mut left_add = false;
        for op in &data[l..m] {
            if let PointAddRectangleSumRanked::Add { x, y, w } = op {
                seq.push(PointAddRectangleSumEvent {
                    x,
                    lo: usize::MAX,
                    hi: *y,
                    payload: *w,
                });
                left_add = true;
            }
        }
        let mut right_query = false;
        for op in &data[m..r] {
            if let PointAddRectangleSumRanked::Query {
                lx,
                ly,
                rx,
                ry,
                answer,
            } = op
            {
                if lx < rx && ly < ry {
                    if left_add {
                        seq.push(PointAddRectangleSumEvent {
                            x: lx,
                            lo: *ly,
                            hi: *ry,
                            payload: *answer as i64,
                        });
                        seq.push(PointAddRectangleSumEvent {
                            x: rx,
                            lo: *ly,
                            hi: *ry,
                            payload: -(*answer as i64) - 1,
                        });
                    }
                    right_query = true;
                }
            }
        }
        if left_add && right_query {
            seq.sort_unstable_by(|a, b| {
                a.x.cmp(b.x)
                    .then_with(|| (a.lo == usize::MAX).cmp(&(b.lo == usize::MAX)))
            });
            for e in seq.iter() {
                if e.lo == usize::MAX {
                    bit.add(e.hi, e.payload);
                } else {
                    Self::answer_event(e, bit, ans);
                }
            }
            for e in seq.iter() {
                if e.lo == usize::MAX {
                    bit.add(e.hi, -e.payload);
                }
            }
        }
        seq.clear();
        if left_add {
            Self::dfs(data, l, m, bit, seq, ans);
        }
        if right_query {
            Self::dfs(data, m, r, bit, seq, ans);
        }
    }
}

impl<X: Ord, Y: Ord + Clone> Default for PointAddRectangleSum<X, Y> {
    fn default() -> Self {
        Self::new()
    }
}
