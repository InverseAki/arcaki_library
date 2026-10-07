#[derive(Clone, Copy, Debug)]
pub enum RectangleAddPointGetQuery<X = i32, Y = X> {
    Add { lx: X, ly: Y, rx: X, ry: Y, w: i64 },
    Query { x: X, y: Y },
}

#[derive(Clone)]
pub struct RectangleAddPointGet<X = i32, Y = X> {
    initial: Vec<(X, Y, X, Y, i64)>,
    data: Vec<RectangleAddPointGetQuery<X, Y>>,
    adds: usize,
    queries: usize,
}

enum RectangleAddPointGetRanked<X> {
    Add {
        lx: X,
        ly: usize,
        rx: X,
        ry: usize,
        w: i64,
    },
    Query {
        x: X,
        y: usize,
        answer: usize,
    },
}

struct RectangleAddPointGetEvent<'a, X> {
    x: &'a X,
    lo: usize,
    hi: usize,
    payload: i64,
}

struct RectangleAddPointGetBit {
    data: Vec<i64>,
}

impl RectangleAddPointGetBit {
    fn new(n: usize) -> Self {
        Self {
            data: vec![0; n + 1],
        }
    }

    #[inline]
    fn add(&mut self, mut p: usize, w: i64) {
        p += 1;
        while p < self.data.len() {
            self.data[p] += w;
            p += p & p.wrapping_neg();
        }
    }

    #[inline]
    fn prefix(&self, mut r: usize) -> i64 {
        let mut res = 0;
        while r != 0 {
            res += self.data[r];
            r &= r - 1;
        }
        res
    }
}

impl<X: Ord, Y: Ord + Clone> RectangleAddPointGet<X, Y> {
    pub fn new() -> Self {
        Self {
            initial: Vec::new(),
            data: Vec::new(),
            adds: 0,
            queries: 0,
        }
    }

    pub fn build(rectangles: impl IntoIterator<Item = (X, Y, X, Y, i64)>) -> Self {
        let initial = rectangles
            .into_iter()
            .filter(|(lx, ly, rx, ry, _)| {
                assert!(lx <= rx && ly <= ry, "invalid rectangle");
                lx < rx && ly < ry
            })
            .collect();
        Self {
            initial,
            data: Vec::new(),
            adds: 0,
            queries: 0,
        }
    }

    #[inline]
    pub fn push_add(&mut self, lx: X, ly: Y, rx: X, ry: Y, w: i64) {
        assert!(lx <= rx && ly <= ry, "invalid rectangle");
        if lx == rx || ly == ry {
            return;
        }
        self.data
            .push(RectangleAddPointGetQuery::Add { lx, ly, rx, ry, w });
        self.adds += 1;
    }

    #[inline]
    pub fn push_query(&mut self, x: X, y: Y) {
        self.data.push(RectangleAddPointGetQuery::Query { x, y });
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
        let mut ys = Vec::with_capacity(self.queries);
        for op in &self.data {
            if let RectangleAddPointGetQuery::Query { y, .. } = op {
                ys.push(y.clone());
            }
        }
        ys.sort_unstable();
        ys.dedup();
        let rank = |y: &Y| ys.partition_point(|v| v < y);
        let initial: Vec<_> = std::mem::take(&mut self.initial)
            .into_iter()
            .map(|(lx, ly, rx, ry, w)| (lx, rank(&ly), rx, rank(&ry), w))
            .collect();
        let dynamic_adds = self.adds;
        let mut answer = 0;
        let data: Vec<_> = std::mem::take(&mut self.data)
            .into_iter()
            .map(|op| match op {
                RectangleAddPointGetQuery::Add { lx, ly, rx, ry, w } => {
                    RectangleAddPointGetRanked::Add {
                        lx,
                        ly: rank(&ly),
                        rx,
                        ry: rank(&ry),
                        w,
                    }
                }
                RectangleAddPointGetQuery::Query { x, y } => {
                    let op = RectangleAddPointGetRanked::Query {
                        x,
                        y: rank(&y) + 1,
                        answer,
                    };
                    answer += 1;
                    op
                }
            })
            .collect();
        self.adds = 0;
        self.queries = 0;
        let mut bit = RectangleAddPointGetBit::new(ys.len());
        drop(ys);
        let mut seq = Vec::new();
        if !initial.is_empty() {
            let mut edges = Vec::new();
            for (i, (lx, ly, rx, ry, _)) in initial.iter().enumerate() {
                if ly < ry {
                    edges.push((lx, i << 1));
                    edges.push((rx, (i << 1) | 1));
                }
            }
            for op in &data {
                if let RectangleAddPointGetRanked::Query { x, y, answer } = op {
                    seq.push(RectangleAddPointGetEvent {
                        x,
                        lo: *y,
                        hi: usize::MAX,
                        payload: *answer as i64,
                    });
                }
            }
            edges.sort_unstable_by_key(|e| e.0);
            seq.sort_unstable_by_key(|e| e.x);
            let mut p = 0;
            for e in &seq {
                while p < edges.len() && edges[p].0 <= e.x {
                    let (_, ly, _, ry, w) = &initial[edges[p].1 >> 1];
                    let w = if edges[p].1 & 1 == 0 { *w } else { -*w };
                    bit.add(*ly, w);
                    bit.add(*ry, -w);
                    p += 1;
                }
                ans[e.payload as usize] += bit.prefix(e.lo);
            }
            bit.data.fill(0);
            seq.clear();
        }
        drop(initial);
        if dynamic_adds != 0 {
            Self::dfs(&data, 0, data.len(), &mut bit, &mut seq, &mut ans);
        }
        ans
    }

    fn rectangle_events<'a>(
        lx: &'a X,
        ly: usize,
        rx: &'a X,
        ry: usize,
        w: i64,
        seq: &mut Vec<RectangleAddPointGetEvent<'a, X>>,
    ) {
        if ly < ry {
            seq.push(RectangleAddPointGetEvent {
                x: lx,
                lo: ly,
                hi: ry,
                payload: w,
            });
            seq.push(RectangleAddPointGetEvent {
                x: rx,
                lo: ly,
                hi: ry,
                payload: -w,
            });
        }
    }

    fn sweep(
        seq: &mut Vec<RectangleAddPointGetEvent<'_, X>>,
        bit: &mut RectangleAddPointGetBit,
        ans: &mut [i64],
    ) {
        seq.sort_unstable_by(|a, b| {
            a.x.cmp(b.x)
                .then_with(|| (a.hi == usize::MAX).cmp(&(b.hi == usize::MAX)))
        });
        for e in seq.iter() {
            if e.hi == usize::MAX {
                ans[e.payload as usize] += bit.prefix(e.lo);
            } else {
                bit.add(e.lo, e.payload);
                bit.add(e.hi, -e.payload);
            }
        }
        seq.clear();
    }

    fn dfs<'a>(
        data: &'a [RectangleAddPointGetRanked<X>],
        l: usize,
        r: usize,
        bit: &mut RectangleAddPointGetBit,
        seq: &mut Vec<RectangleAddPointGetEvent<'a, X>>,
        ans: &mut [i64],
    ) {
        if r - l <= 1 {
            return;
        }
        let m = (l + r) / 2;
        let mut left_add = false;
        for op in &data[l..m] {
            if let RectangleAddPointGetRanked::Add { lx, ly, rx, ry, w } = op {
                if ly < ry {
                    left_add = true;
                    Self::rectangle_events(lx, *ly, rx, *ry, *w, seq);
                }
            }
        }
        let mut right_query = false;
        for op in &data[m..r] {
            if let RectangleAddPointGetRanked::Query { x, y, answer } = op {
                right_query = true;
                if left_add {
                    seq.push(RectangleAddPointGetEvent {
                        x,
                        lo: *y,
                        hi: usize::MAX,
                        payload: *answer as i64,
                    });
                }
            }
        }
        if left_add && right_query {
            Self::sweep(seq, bit, ans);
        } else {
            seq.clear();
        }
        if left_add {
            Self::dfs(data, l, m, bit, seq, ans);
        }
        if right_query {
            Self::dfs(data, m, r, bit, seq, ans);
        }
    }
}

impl<X: Ord, Y: Ord + Clone> Default for RectangleAddPointGet<X, Y> {
    fn default() -> Self {
        Self::new()
    }
}
