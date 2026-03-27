#[inline]
pub fn cmp_concat(sa: &[char], sb: &[char], za: &[usize], zb: &[usize])->Ordering{
    let (an, bn) = (sa.len(), sb.len());
    if an==bn{
        for i in 0..an{
            if sa[i]!=sb[i]{
                return sa[i].cmp(&sb[i]);
            }
        }
        Ordering::Equal
    } else if an > bn{
        for i in 0..bn{
            if sa[i]!=sb[i]{
                return sa[i].cmp(&sb[i])
            }
        }
        if za[bn] >= an-bn{
            for i in 0..bn{
                if sb[i]!=sa[an-bn+i]{
                    return sb[i].cmp(&sa[an-bn+i]);
                }
            }
            Ordering::Equal
        } else {
            let p = za[bn];
            sa[p+bn].cmp(&sa[p])
        }
    } else {
        for i in 0..an{
            if sa[i]!=sb[i]{
                return sa[i].cmp(&sb[i])
            }
        }
        if zb[an] >= bn-an{
            for i in 0..an{
                if sb[i+bn-an]!=sa[i]{
                    return sb[bn-an+i].cmp(&sa[i]);
                }
            }
            Ordering::Equal
        } else {
            let p = zb[an];
            sb[p].cmp(&sb[p+an])
        }
    }
}

#[inline]
pub fn cmp_concat_idx(i: usize, j: usize, ss: &[Vec<char>], za: &[Vec<usize>])->Ordering{
    cmp_concat(&ss[i], &ss[j], &za[i], &za[j])
}

pub fn build_concat_order(ord: &mut [usize], ss: &[Vec<char>], za: &[Vec<usize>]){
    ord.sort_unstable_by(|&i, &j| cmp_concat_idx(i, j, ss, za));
}
