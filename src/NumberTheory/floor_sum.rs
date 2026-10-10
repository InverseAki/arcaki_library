pub fn floor_sum(n: i64, m: i64, a: i64, b: i64)->i64{
    let (a1, a2) = (a/m, a%m);
    let s = n*(n-1)/2*a1;
    let (b1, b2) = (b/m, b%m);
    if a2==0{s+b1*n}
    else{
        let k = (a2*(n-1)+b2)/m;
        s+n*(k+b1)-floor_sum(k, a2, m, m+a2-b2-1)
    }
}
