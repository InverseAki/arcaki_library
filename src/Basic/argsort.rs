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

pub fn argsort_atan2_inplace(ps: &mut [(i64, i64)]){
    ps.sort_unstable_by(|&(x1, y1), &(x2, y2)|{
        let f1 = y1 > 0||(y1==0&&x1<0);
        let f2 = y2 > 0||(y2==0&&x2<0);
        if f1^f2{
            return f1.cmp(&f2);
        } 
        let (u1, v1) = (-y1, x1);
        let (u2, v2) = (-y2, x2);
        if !f1{
            if u1==0||u2==0{
                u2.cmp(&u1)
            } else {
                (u2*v1).cmp(&(u1*v2))
            }
        } else {
            if u1==0||u2==0{
                u1.cmp(&u2)
            } else {
                (u2*v1).cmp(&(u1*v2))
            }
        }
    });
}
