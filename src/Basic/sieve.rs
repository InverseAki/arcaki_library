pub fn linear_sieve(mx: usize)->Vec<usize>{
    let mut lpf = vec![0; mx+1];
    lpf[1] = 1;
    let mut ps = Vec::new();
    for i in 2..=mx{
        if lpf[i]==0{
            lpf[i] = i;
            ps.push(i);
        }
        for &p in &ps{
            let ip = i*p;
            if ip>mx{break;}
            lpf[ip]=p;
            if p==lpf[i]{break;}
        }
    }
    lpf
}

#[inline(always)]
pub fn lpf_factorize(mut x: usize, lpf: &[usize])->Vec<(usize, usize)>{
    let mut res = Vec::new();
    while x > 1 {
        let d = lpf[x];
        let mut e = 0;
        while x%d==0{
            x /= d;
            e += 1;
        }
        res.push((d, e));
    }
    res
}

#[inline(always)]
pub fn lpf_factorize_add(mut x: usize, lpf: &[usize], res: &mut Vec<(usize, usize)>){
    res.clear();
    while x > 1 {
        let d = lpf[x];
        let mut e = 0;
        while x%d==0{
            x /= d;
            e += 1;
        }
        res.push((d, e));
    }
    res
}
