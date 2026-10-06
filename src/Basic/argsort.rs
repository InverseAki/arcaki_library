// Compare arguments exactly. Treat the origin as having argument zero.
mod argsort_detail {
    pub fn cmp(
        (mut x0, y0): (i64, i64),
        (mut x1, y1): (i64, i64),
        atan2_order: bool,
    ) -> std::cmp::Ordering {
        if x0 == 0 && y0 == 0 {
            x0 = 1;
        }
        if x1 == 0 && y1 == 0 {
            x1 = 1;
        }
        let (f0, f1) = if atan2_order {
            // Negative arguments precede zero; the negative x axis is +pi.
            (y0 > 0 || (y0 == 0 && x0 < 0), y1 > 0 || (y1 == 0 && x1 < 0))
        } else {
            (y0 < 0 || (y0 == 0 && x0 < 0), y1 < 0 || (y1 == 0 && x1 < 0))
        };
        f0.cmp(&f1).then_with(|| {
            let cross = x0 as i128 * y1 as i128 - y0 as i128 * x1 as i128;
            // A positive cross product means the first argument is smaller.
            0.cmp(&cross)
        })
    }
}

/// Indices in increasing argument order [0, 2*pi), starting at the positive x axis.
/// Equal arguments retain their input order. The origin has argument zero.
pub fn argsort(ps: &[(i64, i64)]) -> Vec<usize> {
    let mut ord: Vec<usize> = (0..ps.len()).collect();
    ord.sort_by(|&i, &j| argsort_detail::cmp(ps[i], ps[j], false));
    ord
}

/// Sort by increasing argument [0, 2*pi), stably. The origin has argument zero.
pub fn argsort_inplace(ps: &mut [(i64, i64)]) {
    ps.sort_by(|&p, &q| argsort_detail::cmp(p, q, false));
}

/// Sort by increasing atan2 argument (-pi, pi], unstably.
/// The negative x axis has argument +pi; the origin has argument zero.
pub fn argsort_atan2_inplace(ps: &mut [(i64, i64)]) {
    ps.sort_unstable_by(|&p, &q| argsort_detail::cmp(p, q, true));
}
