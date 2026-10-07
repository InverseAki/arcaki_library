#[derive(Clone, Copy, Debug)]
struct LiChaoSegmentLine {
    a: i64,
    b: i128,
}

impl LiChaoSegmentLine {
    #[inline]
    fn eval(self, x: i64) -> i128 {
        self.a as i128 * x as i128 + self.b
    }
}

#[derive(Clone, Debug)]
struct LiChaoSegmentNode {
    line: Option<LiChaoSegmentLine>,
    child: [usize; 2],
}

#[derive(Clone, Debug)]
pub struct LiChaoSegmentTree<const LICAOISMIN: bool> {
    x_min: i64,
    x_max: i64,
    nodes: Vec<LiChaoSegmentNode>,
}

impl<const LICAOISMIN: bool> LiChaoSegmentTree<LICAOISMIN> {
    const NONE: usize = usize::MAX;

    pub fn new(x_min: i64, x_max: i64) -> Self {
        assert!(x_min <= x_max);
        Self {
            x_min,
            x_max,
            nodes: Vec::new(),
        }
    }

    fn push_node(&mut self) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(LiChaoSegmentNode {
            line: None,
            child: [Self::NONE; 2],
        });
        idx
    }

    fn ensure_child(&mut self, idx: usize, dir: usize) -> usize {
        if self.nodes[idx].child[dir] == Self::NONE {
            let next = self.push_node();
            self.nodes[idx].child[dir] = next;
        }
        self.nodes[idx].child[dir]
    }

    #[inline]
    fn midpoint(l: i64, r: i64) -> i64 {
        (l as i128 + (r as i128 - l as i128) / 2) as i64
    }

    #[inline]
    fn better(x: i128, y: i128) -> bool {
        if LICAOISMIN {
            x < y
        } else {
            x > y
        }
    }

    fn insert_line(&mut self, mut idx: usize, mut l: i64, mut r: i64, mut line: LiChaoSegmentLine) {
        loop {
            let Some(mut current) = self.nodes[idx].line else {
                self.nodes[idx].line = Some(line);
                return;
            };
            let m = Self::midpoint(l, r);
            if Self::better(line.eval(m), current.eval(m)) {
                std::mem::swap(&mut line, &mut current);
                self.nodes[idx].line = Some(current);
            }
            if l == r {
                return;
            }
            let dir = if Self::better(line.eval(l), current.eval(l)) {
                r = m;
                0
            } else if Self::better(line.eval(r), current.eval(r)) {
                l = m + 1;
                1
            } else {
                return;
            };
            idx = self.ensure_child(idx, dir);
        }
    }

    pub fn add_line(&mut self, a: i64, b: i128) {
        if self.nodes.is_empty() {
            self.push_node();
        }
        self.insert_line(0, self.x_min, self.x_max, LiChaoSegmentLine { a, b });
    }

    pub fn add_segment(&mut self, a: i64, b: i128, l: i64, r: i64) {
        assert!(l <= r);
        let (l, r) = (l.max(self.x_min), r.min(self.x_max));
        if l > r {
            return;
        }
        if self.nodes.is_empty() {
            self.push_node();
        }
        self.insert_segment(0, self.x_min, self.x_max, l, r, LiChaoSegmentLine { a, b });
    }

    fn insert_segment(
        &mut self,
        idx: usize,
        l: i64,
        r: i64,
        ql: i64,
        qr: i64,
        line: LiChaoSegmentLine,
    ) {
        if ql <= l && r <= qr {
            self.insert_line(idx, l, r, line);
            return;
        }
        let m = Self::midpoint(l, r);
        if ql <= m {
            let next = self.ensure_child(idx, 0);
            self.insert_segment(next, l, m, ql, qr, line);
        }
        if m < qr {
            let next = self.ensure_child(idx, 1);
            self.insert_segment(next, m + 1, r, ql, qr, line);
        }
    }

    pub fn query(&self, x: i64) -> Option<i128> {
        assert!(self.x_min <= x && x <= self.x_max);
        if self.nodes.is_empty() {
            return None;
        }
        let (mut l, mut r) = (self.x_min, self.x_max);
        let mut idx = 0;
        let mut ans = None;
        loop {
            if let Some(line) = self.nodes[idx].line {
                let value = line.eval(x);
                if ans.map_or(true, |old| Self::better(value, old)) {
                    ans = Some(value);
                }
            }
            if l == r {
                break;
            }
            let m = Self::midpoint(l, r);
            let dir = if x <= m {
                r = m;
                0
            } else {
                l = m + 1;
                1
            };
            let next = self.nodes[idx].child[dir];
            if next == Self::NONE {
                break;
            }
            idx = next;
        }
        ans
    }
}
