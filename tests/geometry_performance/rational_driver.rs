fn dfs(l:usize,r:usize,ps:&[Point<Ratio>],w:Ratio128)->ConvexHull<Ratio128> {
    if l+1==r {return ConvexHull::new(&[]);}
    let m=(l+r)/2;
    let mut res=dfs(l,m,ps,w);
    res=res.merge(&dfs(m,r,ps,w));
    let a=ConvexHull::new(&ps[l..m]);
    res=res.merge(&a.weighted_minkowski_sum(w,&ConvexHull::new(&ps[m..r]),Ratio128::one()-w));
    res
}
fn main() {
    use std::io::Read;
    let mut input=String::new();std::io::stdin().read_to_string(&mut input).unwrap();
    let mut it=input.split_whitespace();let n:usize=it.next().unwrap().parse().unwrap();
    let p:i128=it.next().unwrap().parse().unwrap();let q:i128=it.next().unwrap().parse().unwrap();
    let ps:Vec<_>=(0..n).map(|_|Point::new(Ratio::int(it.next().unwrap().parse::<i64>().unwrap()),Ratio::int(it.next().unwrap().parse::<i64>().unwrap()))).collect();
    let area=dfs(0,n,&ps,Ratio128::from_fraction(q,p+q)).area();
    println!("{} {}",area.numerator(),area.denominator());
}
