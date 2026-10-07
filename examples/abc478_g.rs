#![allow(dead_code)]
include!("../src/Basic/ratio.rs");
include!("../src/Gemetory/geometry.rs");
include!("../src/Gemetory/convexhull.rs");

fn dfs(points: &[Point<i128>], p: i128, q: i128) -> (ConvexHull<i128>, ConvexHull<i128>) {
    if points.len() == 1 {
        return (ConvexHull::new(points), ConvexHull::new(&[]));
    }
    let m = points.len() / 2;
    let (a, left) = dfs(&points[..m], p, q);
    let (b, right) = dfs(&points[m..], p, q);
    let crossing = a.weighted_minkowski_sum(q, &b, p);
    (a.merge(&b), left.merge(&right).merge(&crossing))
}
fn main() {
    use std::io::Read;
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    let mut it = input.split_whitespace();
    let n: usize = it.next().unwrap().parse().unwrap();
    let p: i128 = it.next().unwrap().parse().unwrap();
    let q: i128 = it.next().unwrap().parse().unwrap();
    let points: Vec<_> = (0..n)
        .map(|_| {
            Point::new(
                it.next().unwrap().parse::<i128>().unwrap(),
                it.next().unwrap().parse::<i128>().unwrap(),
            )
        })
        .collect();
    let (_, hull) = dfs(&points, p, q);
    let area = Ratio128::from_fraction(hull.area2(), 2 * (p + q) * (p + q));
    println!("{} {}", area.numerator(), area.denominator());
}
