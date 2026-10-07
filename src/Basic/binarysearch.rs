pub fn binary_search<T>(t: T, a: &Vec<T>) -> usize where T: Copy+PartialOrd+Ord {
    let mut l = 0;
    let mut r = a.len() + 1;
    while l + 1 < r {
        let m = (l + r)/2;
        if a[m - 1] < t {
            l = m;
        } else {
            r = m;
        }
    }
    l
}
