pub fn get_sqrt(x: i64) -> i64 {
    let mut res = (x as f64).sqrt() as i64 - 100;
    while res < 0 || (res + 1) * (res + 1) <= x{
        res += 1;
    }
    res
}

pub fn get_index(mut l: usize, mut r: usize, w: usize)->Vec<usize>{
    l += w; r += w;
    let mut res = Vec::new();
    while l < r{
        if l&1 > 0{
            res.push(l);
            l += 1;
        }
        if r&1 > 0{
            r -= 1;
            res.push(r);
        }
        l >>= 1; r>>=1;
    }
    res
}
