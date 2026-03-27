pub fn argsort(ps: &[(i64, i64)]) -> Vec<usize> {
    let mut ord: Vec<usize> = (0..ps.len()).collect();
    ord.sort_by(|&i, &j| {
        let (x0, y0) = ps[i];
        let (x1, y1) = ps[j];
        let f0 = y0 < 0 || (y0 == 0 && x0 < 0);
        let f1 = y1 < 0 || (y1 == 0 && x0 < 0);
        f0.cmp(&f1).then_with(|| {let c = x0 as i128 * y1 as i128 -y0 as i128 * x1 as i128;c.cmp(&0)})
    });
    ord
}

pub fn argsort_inplace(ps: &mut [(i64, i64)]) -> Vec<usize> {
    ps.sort_by(|&(x0, y0), &(x1, y1)| {
        let f0 = y0 < 0 || (y0 == 0 && x0 < 0);
        let f1 = y1 < 0 || (y1 == 0 && x0 < 0);
        f0.cmp(&f1).then_with(|| {let c = x0 as i128 * y1 as i128 -y0 as i128 * x1 as i128;c.cmp(&0)})
    });
}