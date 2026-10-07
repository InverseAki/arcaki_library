pub fn crt(ss: &Vec<(usize, usize)>)->(usize, usize){
    let mut r = 0; let mut m = 1;
    for &(bi, mi) in ss{
        let (g, p, _) = ext_gcd(m, mi as i64);
        if (bi as i64-r)%g!=0{return (!0, !0);}
        let t = modulo(floor(bi as i64-r, g)*p, floor(mi as i64, g));
        r += m*t;
        m = floor(m*mi as i64, g);
        r = modulo(r, m);
    }
    (r as usize, m as usize)
}
