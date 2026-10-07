#[derive(Clone)]
pub struct StaticRectangleAddRectangleSum<X = i32, Y = X, W = i128> {
    rectangles: Vec<(X, Y, X, Y, W)>,
    queries: Vec<(X, Y, X, Y)>,
}

struct StaticRectangleAddRectangleSumBits<W> {
    data: [Vec<W>; 4],
    zero: W,
}

impl<W> StaticRectangleAddRectangleSumBits<W>
where
    W: Copy + std::ops::Add<Output = W>,
{
    fn new(n: usize, zero: W) -> Self {
        Self {
            data: std::array::from_fn(|_| vec![zero; n + 1]),
            zero,
        }
    }

    #[inline]
    fn add(&mut self, mut p: usize, values: [W; 4]) {
        p += 1;
        while p < self.data[0].len() {
            for k in 0..4 {
                self.data[k][p] = self.data[k][p] + values[k];
            }
            p += p & p.wrapping_neg();
        }
    }

    #[inline]
    fn prefix(&self, mut r: usize) -> [W; 4] {
        let mut values = [self.zero; 4];
        while r != 0 {
            for k in 0..4 {
                values[k] = values[k] + self.data[k][r];
            }
            r &= r - 1;
        }
        values
    }
}

struct StaticRectangleAddRectangleSumEvent<'a, X> {
    x: &'a X,
    index: usize,
}

impl<X: Ord, Y: Ord, W> StaticRectangleAddRectangleSum<X, Y, W> {
    pub fn new() -> Self {
        Self {
            rectangles: Vec::new(),
            queries: Vec::new(),
        }
    }

    pub fn build(rectangles: impl IntoIterator<Item = (X, Y, X, Y, W)>) -> Self {
        let mut solver = Self::new();
        for (lx, ly, rx, ry, w) in rectangles {
            solver.push_add(lx, ly, rx, ry, w);
        }
        solver
    }

    #[inline]
    pub fn push_add(&mut self, lx: X, ly: Y, rx: X, ry: Y, w: W) {
        assert!(lx <= rx && ly <= ry, "invalid rectangle");
        if lx == rx || ly == ry {
            return;
        }
        self.rectangles.push((lx, ly, rx, ry, w));
    }

    #[inline]
    pub fn push_query(&mut self, lx: X, ly: Y, rx: X, ry: Y) {
        assert!(lx <= rx && ly <= ry, "invalid rectangle");
        self.queries.push((lx, ly, rx, ry));
    }
}

impl<X: Ord, Y: Ord, W> StaticRectangleAddRectangleSum<X, Y, W>
where
    W: Copy + std::ops::Add<Output = W> + std::ops::Sub<Output = W> + std::ops::Mul<Output = W>,
{
    pub fn solve_with(
        &mut self,
        zero: W,
        convert_x: impl Fn(&X) -> W,
        convert_y: impl Fn(&Y) -> W,
    ) -> Vec<W> {
        let mut answers = vec![zero; self.queries.len()];
        if !self.rectangles.is_empty()
            && self
                .queries
                .iter()
                .any(|(lx, ly, rx, ry)| lx < rx && ly < ry)
        {
            let mut ys: Vec<&Y> = self.rectangles.iter().flat_map(|r| [&r.1, &r.3]).collect();
            ys.sort_unstable();
            ys.dedup();
            let cy: Vec<W> = ys.iter().map(|&y| convert_y(y)).collect();
            let rectangle_ranks: Vec<_> = self
                .rectangles
                .iter()
                .map(|r| {
                    (
                        ys.binary_search(&&r.1).unwrap(),
                        ys.binary_search(&&r.3).unwrap(),
                    )
                })
                .collect();
            let query_data: Vec<_> = self
                .queries
                .iter()
                .map(|(_, ly, _, ry)| {
                    (
                        ys.partition_point(|y| *y < ly),
                        ys.partition_point(|y| *y < ry),
                        convert_y(ly),
                        convert_y(ry),
                    )
                })
                .collect();
            let mut events = Vec::with_capacity(2 * (self.rectangles.len() + self.queries.len()));
            for (i, (lx, _, rx, _, _)) in self.rectangles.iter().enumerate() {
                events.push(StaticRectangleAddRectangleSumEvent {
                    x: lx,
                    index: i << 2,
                });
                events.push(StaticRectangleAddRectangleSumEvent {
                    x: rx,
                    index: (i << 2) | 1,
                });
            }
            for (i, (lx, ly, rx, ry)) in self.queries.iter().enumerate() {
                if lx < rx && ly < ry {
                    events.push(StaticRectangleAddRectangleSumEvent {
                        x: lx,
                        index: (i << 2) | 2,
                    });
                    events.push(StaticRectangleAddRectangleSumEvent {
                        x: rx,
                        index: (i << 2) | 3,
                    });
                }
            }
            events.sort_unstable_by_key(|e| (e.x, (e.index & 3) < 2));
            let mut bits = StaticRectangleAddRectangleSumBits::new(ys.len(), zero);
            for e in events {
                let tag = e.index & 3;
                let i = e.index >> 2;
                let x = convert_x(e.x);
                if tag < 2 {
                    let w = self.rectangles[i].4;
                    let w = if tag == 0 { w } else { zero - w };
                    let (lo, hi) = rectangle_ranks[i];
                    let wx = w * x;
                    let wl = w * cy[lo];
                    bits.add(lo, [w, wx, wl, wx * cy[lo]]);
                    let w = zero - w;
                    let wx = zero - wx;
                    bits.add(hi, [w, wx, w * cy[hi], wx * cy[hi]]);
                } else {
                    let (lo, hi, ly, ry) = query_data[i];
                    let upper = bits.prefix(hi);
                    let lower = bits.prefix(lo);
                    let value = Self::area_prefix(x, ry, upper) - Self::area_prefix(x, ly, lower);
                    if tag == 2 {
                        answers[i] = answers[i] - value;
                    } else {
                        answers[i] = answers[i] + value;
                    }
                }
            }
        }
        self.rectangles.clear();
        self.queries.clear();
        answers
    }

    #[inline]
    fn area_prefix(x: W, y: W, sums: [W; 4]) -> W {
        x * y * sums[0] - y * sums[1] - x * sums[2] + sums[3]
    }
}

impl<X, Y, W> StaticRectangleAddRectangleSum<X, Y, W>
where
    X: Ord + Clone,
    Y: Ord + Clone,
    W: Copy
        + Default
        + From<X>
        + From<Y>
        + std::ops::Add<Output = W>
        + std::ops::Sub<Output = W>
        + std::ops::Mul<Output = W>,
{
    pub fn solve(&mut self) -> Vec<W> {
        self.solve_with(W::default(), |x| W::from(x.clone()), |y| W::from(y.clone()))
    }
}

impl<X: Ord, Y: Ord, W> Default for StaticRectangleAddRectangleSum<X, Y, W> {
    fn default() -> Self {
        Self::new()
    }
}
