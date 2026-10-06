struct RectangleUnionAreaSegtree<T> {
    n: usize,
    count: Vec<usize>,
    span: Vec<T>,
    covered: Vec<T>,
}

impl<T> RectangleUnionAreaSegtree<T>
where
    T: Copy + Default + std::ops::Add<Output = T> + std::ops::Sub<Output = T>,
{
    fn new(ys: &[T]) -> Self {
        let n = (ys.len() - 1).next_power_of_two();
        let mut span = vec![T::default(); 2 * n];
        for (i, w) in ys.windows(2).enumerate() {
            span[n + i] = w[1] - w[0];
        }
        for k in (1..n).rev() {
            span[k] = span[2 * k] + span[2 * k + 1];
        }
        Self {
            n,
            count: vec![0; 2 * n],
            span,
            covered: vec![T::default(); 2 * n],
        }
    }

    #[inline]
    fn pull(&mut self, k: usize) {
        self.covered[k] = if self.count[k] != 0 {
            self.span[k]
        } else if k < self.n {
            self.covered[2 * k] + self.covered[2 * k + 1]
        } else {
            T::default()
        };
    }

    #[inline]
    fn apply(&mut self, k: usize, add: bool) {
        if add {
            self.count[k] += 1;
        } else {
            debug_assert!(self.count[k] != 0);
            self.count[k] -= 1;
        }
        self.pull(k);
    }

    fn apply_range(&mut self, l: usize, r: usize, add: bool) {
        if l >= r {
            return;
        }
        let (mut l, mut r) = (l + self.n, r + self.n);
        let (left, right) = (l, r - 1);
        while l < r {
            if l & 1 != 0 {
                self.apply(l, add);
                l += 1;
            }
            if r & 1 != 0 {
                r -= 1;
                self.apply(r, add);
            }
            l >>= 1;
            r >>= 1;
        }
        let (mut l, mut r) = (left >> 1, right >> 1);
        while l != 0 {
            self.pull(l);
            if l != r {
                self.pull(r);
            }
            l >>= 1;
            r >>= 1;
        }
    }
}

pub fn area_of_union_rectangles<T>(rec: &[(T, T, T, T)]) -> T
where
    T: Copy
        + Ord
        + Default
        + std::ops::Add<Output = T>
        + std::ops::Sub<Output = T>
        + std::ops::Mul<Output = T>,
{
    let mut endpoints = Vec::with_capacity(2 * rec.len());
    let mut events = Vec::with_capacity(2 * rec.len());
    for &(l, d, r, u) in rec {
        assert!(l <= r && d <= u, "rectangle endpoints must be ordered");
        if l == r || d == u {
            continue;
        }
        let id = endpoints.len();
        endpoints.push((d, id));
        endpoints.push((u, id + 1));
        events.push((l, id, id + 1, true));
        events.push((r, id, id + 1, false));
    }
    if events.is_empty() {
        return T::default();
    }

    endpoints.sort_unstable_by(|a, b| a.0.cmp(&b.0));
    let mut indices = vec![0; endpoints.len()];
    let mut ys = Vec::with_capacity(endpoints.len());
    for (y, id) in endpoints {
        if ys.last() != Some(&y) {
            ys.push(y);
        }
        indices[id] = ys.len() - 1;
    }
    for (_, d, u, _) in &mut events {
        *d = indices[*d];
        *u = indices[*u];
    }
    drop(indices);
    events.sort_unstable_by(|a, b| a.0.cmp(&b.0));

    let mut seg = RectangleUnionAreaSegtree::new(&ys);
    let mut ans = T::default();
    let mut prev_x = events[0].0;
    for (x, d, u, add) in events {
        ans = ans + (x - prev_x) * seg.covered[1];
        seg.apply_range(d, u, add);
        prev_x = x;
    }
    ans
}
