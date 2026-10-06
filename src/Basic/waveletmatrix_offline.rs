#[allow(dead_code)]
mod wavelet_offline_detail {
    include!("waveletmatrix.rs");

    pub(super) struct Fenwick {
        counts: Vec<usize>,
        sums: Vec<u128>,
    }
    impl Fenwick {
        fn new(active: &[bool], values: &[usize]) -> Self {
            let n = active.len();
            let mut counts = vec![0; n + 1];
            let mut sums = vec![0; n + 1];
            for i in 0..n {
                if active[i] {
                    counts[i + 1] = 1;
                    sums[i + 1] = values[i] as u128;
                }
            }
            for i in 1..=n {
                let j = i + (i & i.wrapping_neg());
                if j <= n {
                    counts[j] += counts[i];
                    sums[j] += sums[i];
                }
            }
            Self { counts, sums }
        }
        #[inline]
        pub(super) fn add(&mut self, i: usize, value: usize) {
            let mut p = i + 1;
            while p < self.counts.len() {
                self.counts[p] += 1;
                self.sums[p] += value as u128;
                p += p & p.wrapping_neg();
            }
        }
        #[inline]
        pub(super) fn remove(&mut self, i: usize, value: usize) {
            let mut p = i + 1;
            while p < self.counts.len() {
                self.counts[p] -= 1;
                self.sums[p] -= value as u128;
                p += p & p.wrapping_neg();
            }
        }
        #[inline]
        fn prefix_count(&self, mut r: usize) -> usize {
            let mut ans = 0;
            while r > 0 {
                ans += self.counts[r];
                r &= r - 1;
            }
            ans
        }
        #[inline]
        fn prefix_sum(&self, mut r: usize) -> u128 {
            let mut ans = 0;
            while r > 0 {
                ans += self.sums[r];
                r &= r - 1;
            }
            ans
        }
        #[inline]
        pub(super) fn count(&self, l: usize, r: usize) -> usize {
            self.prefix_count(r) - self.prefix_count(l)
        }
        #[inline]
        pub(super) fn sum(&self, l: usize, r: usize) -> u128 {
            self.prefix_sum(r) - self.prefix_sum(l)
        }
        #[inline]
        pub(super) fn count_sum(&self, l: usize, r: usize) -> (usize, u128) {
            let (mut count, mut sum) = (0, 0);
            let mut p = r;
            while p > 0 {
                count += self.counts[p];
                sum += self.sums[p];
                p &= p - 1;
            }
            p = l;
            // Compute each prefix separately: intermediate subtractions cannot underflow.
            let (mut left_count, mut left_sum) = (0, 0);
            while p > 0 {
                left_count += self.counts[p];
                left_sum += self.sums[p];
                p &= p - 1;
            }
            (count - left_count, sum - left_sum)
        }
    }

    pub(super) fn build(
        ranks: &[usize],
        mut active: Vec<bool>,
        mut values: Vec<usize>,
    ) -> (WaveletMatrix, Vec<Fenwick>) {
        let wm = WaveletMatrix::new(ranks);
        let n = ranks.len();
        let mut trees = Vec::with_capacity(wm.max_log + 1);
        trees.push(Fenwick::new(&active, &values));
        for d in 0..wm.max_log {
            let mut next_active = vec![false; n];
            let mut next_values = vec![0; n];
            let (mut zero, mut one) = (0, wm.mids[d]);
            for i in 0..n {
                let j = if wm.bitvecs[d].access(i) {
                    let j = one;
                    one += 1;
                    j
                } else {
                    let j = zero;
                    zero += 1;
                    j
                };
                next_active[j] = active[i];
                next_values[j] = values[i];
            }
            active = next_active;
            values = next_values;
            trees.push(Fenwick::new(&active, &values));
        }
        (wm, trees)
    }
    #[inline]
    pub(super) fn depth(wm: &WaveletMatrix) -> usize {
        wm.max_log
    }
    #[inline]
    pub(super) fn route(wm: &WaveletMatrix, d: usize, i: usize) -> usize {
        let zero = wm.bitvecs[d].rank0(i);
        if wm.bitvecs[d].access(i) {
            wm.mids[d] + i - zero
        } else {
            zero
        }
    }
    #[inline]
    pub(super) fn children(
        wm: &WaveletMatrix,
        d: usize,
        l: usize,
        r: usize,
    ) -> (usize, usize, usize, usize) {
        let a = wm.bitvecs[d].rank0(l);
        let b = wm.bitvecs[d].rank0(r);
        (a, b, wm.mids[d] + l - a, wm.mids[d] + r - b)
    }
}

/// A point-set wavelet matrix with pre-registered (index, value) candidates.
/// Values are nonnegative usize; sums and sum targets are u128.
/// Exactly one candidate is active at each original index.
/// All index/value/rank ranges are half-open; ranks are zero-based.
pub struct WaveletMatrixOffline {
    offsets: Vec<usize>,
    candidates: Vec<usize>,
    current: Vec<usize>,
    values: Vec<usize>,
    wm: wavelet_offline_detail::WaveletMatrix,
    trees: Vec<wavelet_offline_detail::Fenwick>,
}

impl WaveletMatrixOffline {
    /// Initial values are registered automatically. Duplicate candidates are allowed.
    /// The execution order of future updates is not needed, only their candidates.
    pub fn new(initial: &[usize], updates: &[(usize, usize)]) -> Self {
        let n = initial.len();
        let mut points: Vec<_> = initial.iter().copied().enumerate().collect();
        for &(i, x) in updates {
            assert!(i < n, "candidate index out of bounds");
            points.push((i, x));
        }
        points.sort_unstable();
        points.dedup();
        let candidates: Vec<_> = points.iter().map(|&(_, x)| x).collect();
        let mut offsets = vec![0; n + 1];
        for &(i, _) in &points {
            offsets[i + 1] += 1;
        }
        for i in 0..n {
            offsets[i + 1] += offsets[i];
        }
        let current: Vec<_> = (0..n)
            .map(|i| {
                offsets[i]
                    + candidates[offsets[i]..offsets[i + 1]]
                        .binary_search(&initial[i])
                        .unwrap()
            })
            .collect();
        let mut active = vec![false; points.len()];
        for &p in &current {
            active[p] = true;
        }
        let mut values = candidates.clone();
        values.sort_unstable();
        values.dedup();
        let ranks: Vec<_> = candidates
            .iter()
            .map(|x| values.binary_search(x).unwrap())
            .collect();
        let (wm, trees) = wavelet_offline_detail::build(&ranks, active, candidates.clone());
        Self {
            offsets,
            candidates,
            current,
            values,
            wm,
            trees,
        }
    }

    pub fn len(&self) -> usize {
        self.current.len()
    }
    pub fn is_empty(&self) -> bool {
        self.current.is_empty()
    }
    /// Number of distinct registered (index, value) points, including initial values.
    pub fn candidate_len(&self) -> usize {
        self.candidates.len()
    }
    pub fn get(&self, i: usize) -> usize {
        assert!(i < self.len());
        self.candidates[self.current[i]]
    }
    /// Set a pre-registered value. Invalid updates panic before changing the state.
    pub fn set(&mut self, i: usize, x: usize) {
        assert!(i < self.len());
        let next = self.offsets[i]
            + self.candidates[self.offsets[i]..self.offsets[i + 1]]
                .binary_search(&x)
                .expect("unregistered (index, value) candidate");
        let old = self.current[i];
        if old == next {
            return;
        }
        let old_value = self.candidates[old];
        let (mut p, mut q) = (old, next);
        for d in 0..=wavelet_offline_detail::depth(&self.wm) {
            self.trees[d].remove(p, old_value);
            self.trees[d].add(q, x);
            if d < wavelet_offline_detail::depth(&self.wm) {
                p = wavelet_offline_detail::route(&self.wm, d, p);
                q = wavelet_offline_detail::route(&self.wm, d, q);
            }
        }
        self.current[i] = next;
    }
    #[inline]
    fn interval(&self, l: usize, r: usize) -> (usize, usize) {
        assert!(l <= r && r <= self.len());
        (self.offsets[l], self.offsets[r])
    }
    pub fn range_sum(&self, l: usize, r: usize) -> u128 {
        let (l, r) = self.interval(l, r);
        self.trees[0].sum(l, r)
    }
    pub fn kth_smallest(&self, l: usize, r: usize, mut k: usize) -> usize {
        let (mut a, mut b) = self.interval(l, r);
        assert!(k < r - l);
        let depth = wavelet_offline_detail::depth(&self.wm);
        let mut rank = 0;
        for d in 0..depth {
            let (zl, zr, ol, or) = wavelet_offline_detail::children(&self.wm, d, a, b);
            let zeros = self.trees[d + 1].count(zl, zr);
            if k < zeros {
                a = zl;
                b = zr;
            } else {
                k -= zeros;
                rank |= 1usize << (depth - 1 - d);
                a = ol;
                b = or;
            }
        }
        self.values[rank]
    }
    pub fn kth_largest(&self, l: usize, r: usize, k: usize) -> usize {
        self.interval(l, r);
        assert!(k < r - l);
        self.kth_smallest(l, r, r - l - 1 - k)
    }
    pub fn rank_range(&self, x: usize, l: usize, r: usize) -> usize {
        let (mut l, mut r) = self.interval(l, r);
        let rank = match self.values.binary_search(&x) {
            Ok(rank) => rank,
            Err(_) => return 0,
        };
        let depth = wavelet_offline_detail::depth(&self.wm);
        for d in 0..depth {
            let (zl, zr, ol, or) = wavelet_offline_detail::children(&self.wm, d, l, r);
            (l, r) = if rank >> (depth - 1 - d) & 1 == 0 {
                (zl, zr)
            } else {
                (ol, or)
            };
        }
        self.trees[depth].count(l, r)
    }
    /// Number of active values strictly below upper.
    pub fn range_freq(&self, l: usize, r: usize, upper: usize) -> usize {
        let (mut a, mut b) = self.interval(l, r);
        let upper = self.values.partition_point(|&v| v < upper);
        if upper == self.values.len() {
            return r - l;
        }
        let depth = wavelet_offline_detail::depth(&self.wm);
        let mut ans = 0;
        for d in 0..depth {
            let (zl, zr, ol, or) = wavelet_offline_detail::children(&self.wm, d, a, b);
            if upper >> (depth - 1 - d) & 1 == 1 {
                ans += self.trees[d + 1].count(zl, zr);
                a = ol;
                b = or;
            } else {
                a = zl;
                b = zr;
            }
        }
        ans
    }
    pub fn range_freq_between(&self, l: usize, r: usize, lower: usize, upper: usize) -> usize {
        self.interval(l, r);
        if lower >= upper {
            0
        } else {
            self.range_freq(l, r, upper) - self.range_freq(l, r, lower)
        }
    }
    pub fn sum_less(&self, l: usize, r: usize, upper: usize) -> u128 {
        let (mut l, mut r) = self.interval(l, r);
        let upper = self.values.partition_point(|&v| v < upper);
        if upper == self.values.len() {
            return self.trees[0].sum(l, r);
        }
        let depth = wavelet_offline_detail::depth(&self.wm);
        let mut ans = 0;
        for d in 0..depth {
            let (zl, zr, ol, or) = wavelet_offline_detail::children(&self.wm, d, l, r);
            if upper >> (depth - 1 - d) & 1 == 1 {
                ans += self.trees[d + 1].sum(zl, zr);
                l = ol;
                r = or;
            } else {
                l = zl;
                r = zr;
            }
        }
        ans
    }
    pub fn sum_between(&self, l: usize, r: usize, lower: usize, upper: usize) -> u128 {
        self.interval(l, r);
        if lower >= upper {
            0
        } else {
            self.sum_less(l, r, upper) - self.sum_less(l, r, lower)
        }
    }
    /// Sum of the k smallest active values in [l,r). k=0 is allowed.
    pub fn sum_smallest(&self, l: usize, r: usize, mut k: usize) -> u128 {
        let (mut a, mut b) = self.interval(l, r);
        assert!(k <= r - l);
        if k == 0 {
            return 0;
        }
        let depth = wavelet_offline_detail::depth(&self.wm);
        let (mut ans, mut rank) = (0, 0);
        for d in 0..depth {
            let (zl, zr, ol, or) = wavelet_offline_detail::children(&self.wm, d, a, b);
            let (count, sum) = self.trees[d + 1].count_sum(zl, zr);
            if k <= count {
                a = zl;
                b = zr;
            } else {
                ans += sum;
                k -= count;
                rank |= 1usize << (depth - 1 - d);
                a = ol;
                b = or;
            }
        }
        ans + (self.values[rank] as u128) * (k as u128)
    }
    pub fn sum_largest(&self, l: usize, r: usize, k: usize) -> u128 {
        self.interval(l, r);
        assert!(k <= r - l);
        self.range_sum(l, r) - self.sum_smallest(l, r, r - l - k)
    }
    /// Sum of zero-based ascending ranks [start,end) in the original interval [l,r).
    pub fn sum_sorted(&self, l: usize, r: usize, start: usize, end: usize) -> u128 {
        self.interval(l, r);
        assert!(start <= end && end <= r - l);
        self.sum_smallest(l, r, end) - self.sum_smallest(l, r, start)
    }
    /// Minimum number of values selected from [l,r) whose sum is >= target.
    /// Selects the largest values first; returns None if the target is unreachable.
    /// target=0 returns Some(0), including for an empty interval.
    pub fn min_count_for_sum(&self, l: usize, r: usize, mut target: u128) -> Option<usize> {
        let (mut l, mut r) = self.interval(l, r);
        if target == 0 {
            return Some(0);
        }
        if self.trees[0].sum(l, r) < target {
            return None;
        }
        let depth = wavelet_offline_detail::depth(&self.wm);
        let (mut ans, mut rank) = (0, 0);
        for d in 0..depth {
            let (zl, zr, ol, or) = wavelet_offline_detail::children(&self.wm, d, l, r);
            let (count, sum) = self.trees[d + 1].count_sum(ol, or);
            if sum >= target {
                rank |= 1usize << (depth - 1 - d);
                l = ol;
                r = or;
            } else {
                ans += count;
                target -= sum;
                l = zl;
                r = zr;
            }
        }
        // The maintained remaining sum is >= target > 0, so this value is positive.
        let value = self.values[rank] as u128;
        Some(ans + ((target - 1) / value + 1) as usize)
    }
    pub fn prev_value(&self, l: usize, r: usize, upper: usize) -> Option<usize> {
        let count = self.range_freq(l, r, upper);
        if count == 0 {
            None
        } else {
            Some(self.kth_smallest(l, r, count - 1))
        }
    }
    pub fn next_value(&self, l: usize, r: usize, lower: usize) -> Option<usize> {
        let count = self.range_freq(l, r, lower);
        if count == r - l {
            None
        } else {
            Some(self.kth_smallest(l, r, count))
        }
    }
}
