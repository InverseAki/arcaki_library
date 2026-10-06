#![allow(dead_code)]
include!("../src/Basic/ratio.rs");
include!("../src/Gemetory/geometry.rs");
include!("../src/Gemetory/convexhull.rs");
fn p(x: i64, y: i64) -> IntPoint {
    IntPoint::new(x, y)
}
fn r(n: i128, d: i128) -> Ratio128 {
    Ratio128::from_fraction(n, d)
}
// 独立なgift wrapping。小さい検証座標では通常の厳密外積で十分。
fn jarvis<T: GeometryNumber>(points: &[Point<T>]) -> Vec<Point<T>> {
    let mut points = points.to_vec();
    points.sort();
    points.dedup();
    if points.len() <= 1 {
        return points;
    }
    let mut hull = Vec::new();
    let start = points[0];
    let mut current = start;
    loop {
        hull.push(current);
        let mut next = *points.iter().find(|&&q| q != current).unwrap();
        for &q in &points {
            let cross = (next - current).cross(q - current);
            if cross < T::zero()
                || (cross == T::zero()
                    && current.distance_squared(q) > current.distance_squared(next))
            {
                next = q;
            }
        }
        current = next;
        if current == start {
            break;
        }
        assert!(hull.len() <= points.len());
    }
    hull
}
fn naive_weighted<T: GeometryCoordinate, C: HullCoefficient<T>>(
    a: &ConvexHull<T>,
    b: &ConvexHull<T>,
    ka: C,
    kb: C,
) -> Vec<Point<C::Output>> {
    let points: Vec<_> = a
        .vertices()
        .iter()
        .flat_map(|&p| b.vertices().iter().map(move |&q| ka.scale(p) + kb.scale(q)))
        .collect();
    jarvis(&points)
}
#[test]
fn hull_degenerate_and_compatibility() {
    let points = vec![
        p(0, 0),
        p(0, 2),
        p(2, 0),
        p(2, 2),
        p(1, 1),
        p(1, 0),
        p(0, 0),
    ];
    assert_eq!(
        convex_hull_polygon(&points),
        vec![p(0, 0), p(2, 0), p(2, 2), p(0, 2)]
    );
    let (upper, lower) = convex_hull_chains(&points);
    assert_eq!(upper, vec![p(0, 0), p(0, 2), p(2, 2)]);
    assert_eq!(lower, vec![p(0, 0), p(2, 0), p(2, 2)]);
    let tuples: Vec<_> = points.iter().map(|p| (p.x, p.y)).collect();
    assert_eq!(
        convex_hull(&tuples),
        (vec![(0, 0), (0, 2), (2, 2)], vec![(0, 0), (2, 0), (2, 2)])
    );
    assert_eq!(cross_product((0, 0), (2, 0), (0, 2)), 4i128);
    assert!(convex_hull_polygon::<i64>(&[]).is_empty());
    assert_eq!(convex_hull_polygon(&[p(3, 4), p(3, 4)]), vec![p(3, 4)]);
    assert_eq!(
        convex_hull_polygon(&[p(3, 3), p(1, 1), p(2, 2), p(1, 1)]),
        vec![p(1, 1), p(3, 3)]
    );
    let rational = vec![
        Point::new(r(0, 1), r(0, 1)),
        Point::new(r(1, 2), r(0, 1)),
        Point::new(r(0, 1), r(1, 3)),
        Point::new(r(1, 4), r(0, 1)),
    ];
    assert_eq!(convex_hull_polygon(&rational), jarvis(&rational));
    let small: Vec<_> = rational
        .iter()
        .map(|p| {
            Point::new(
                Ratio::from_fraction(*p.x.numerator() as i64, *p.x.denominator() as i64),
                Ratio::from_fraction(*p.y.numerator() as i64, *p.y.denominator() as i64),
            )
        })
        .collect();
    assert_eq!(
        convex_hull_polygon(&small)
            .into_iter()
            .map(|p| p.to_rational())
            .collect::<Vec<_>>(),
        jarvis(&rational)
    );
}
#[test]
fn all_grid_subsets_against_gift_wrapping() {
    let grid: Vec<_> = (-1..=1)
        .flat_map(|x| (-1..=1).map(move |y| p(x, y)))
        .collect();
    for mask in 0..1usize << grid.len() {
        let mut points: Vec<_> = grid
            .iter()
            .enumerate()
            .filter(|(i, _)| mask >> i & 1 != 0)
            .map(|(_, &p)| p)
            .collect();
        let wide: Vec<_> = points.iter().map(|p| p.wide()).collect();
        let expected = jarvis(&wide);
        points.reverse();
        if let Some(&p) = points.first() {
            points.push(p);
        }
        assert_eq!(
            ConvexHull::new(&points)
                .vertices()
                .iter()
                .map(|p| p.wide())
                .collect::<Vec<_>>(),
            expected
        );
    }
}
#[test]
fn minkowski_structured_and_weights() {
    let sets = vec![
        vec![],
        vec![p(2, -1)],
        vec![p(-1, 0), p(2, 0)],
        vec![p(0, -2), p(0, 1)],
        vec![p(-2, -2), p(2, 2)],
        vec![p(0, 0), p(2, 0), p(0, 2)],
        vec![p(-1, -1), p(1, -1), p(1, 1), p(-1, 1)],
        vec![p(-2, 0), p(0, -2), p(2, 0), p(0, 2)],
    ];
    for a in &sets {
        for b in &sets {
            let (a, b) = (ConvexHull::new(a), ConvexHull::new(b));
            assert_eq!(
                a.minkowski_sum(&b).into_vertices(),
                naive_weighted(&a, &b, 1i64, 1i64)
            );
            for ka in [-2i64, 0, 1, 3] {
                for kb in [-2i64, 0, 1, 3] {
                    let sum = a.weighted_minkowski_sum(ka, &b, kb);
                    assert_eq!(sum.vertices(), naive_weighted(&a, &b, ka, kb));
                    assert_eq!(sum, b.weighted_minkowski_sum(kb, &a, ka));
                }
            }
            for ka in [r(-1, 2), r(0, 1), r(2, 3)] {
                for kb in [r(-1, 2), r(0, 1), r(2, 3)] {
                    let sum = a.weighted_minkowski_sum(ka, &b, kb);
                    assert_eq!(sum.vertices(), naive_weighted(&a, &b, ka, kb));
                    let small_a =
                        Ratio::from_fraction(*ka.numerator() as i64, *ka.denominator() as i64);
                    let small_b =
                        Ratio::from_fraction(*kb.numerator() as i64, *kb.denominator() as i64);
                    assert_eq!(sum, a.weighted_minkowski_sum(small_a, &b, small_b));
                }
            }
        }
    }
}
#[test]
fn deterministic_clouds_and_rational_coordinates() {
    let mut state = 712367821u64;
    let mut next = || {
        state ^= state << 7;
        state ^= state >> 9;
        state
    };
    for _ in 0..2000 {
        let (n, m) = ((next() % 20) as usize, (next() % 20) as usize);
        let mut cloud = |n| {
            (0..n)
                .map(|_| p((next() % 31) as i64 - 15, (next() % 31) as i64 - 15))
                .collect::<Vec<_>>()
        };
        let (a, b) = (cloud(n), cloud(m));
        let (ka, kb) = ((next() % 9) as i64 - 4, (next() % 9) as i64 - 4);
        let (a, b) = (ConvexHull::new(&a), ConvexHull::new(&b));
        assert_eq!(
            a.weighted_minkowski_sum(ka, &b, kb).into_vertices(),
            naive_weighted(&a, &b, ka, kb)
        );
        let (ar, br) = (a.scaled(r(1, 3)), b.scaled(r(-2, 5)));
        let all: Vec<_> = ar
            .vertices()
            .iter()
            .flat_map(|&p| br.vertices().iter().map(move |&q| p + q))
            .collect();
        assert_eq!(ar.minkowski_sum(&br).into_vertices(), jarvis(&all));
        assert_eq!(
            ar.weighted_minkowski_sum(ka, &br, kb).into_vertices(),
            naive_weighted(&ar, &br, ka, kb)
        );
        assert_eq!(
            ar.weighted_minkowski_sum(r(ka as i128, 3), &br, r(kb as i128, 2))
                .into_vertices(),
            naive_weighted(&ar, &br, r(ka as i128, 3), r(kb as i128, 2))
        );
    }
}
#[test]
fn extreme_integer_coordinates_and_overflow() {
    let (lo, hi) = (i64::MIN, i64::MAX);
    let points = [p(lo, lo), p(hi, lo), p(hi, hi), p(lo, hi)];
    let a = ConvexHull::new(&points);
    assert_eq!(a.vertices(), points);
    assert_eq!(a.minkowski_sum(&a).vertices(), points.map(|p| p * 2));
    assert_eq!(a.scaled(-1i64).len(), 4);
    assert_eq!(a.scaled(0i64).vertices(), [Point::new(0i128, 0)]);
    assert!(std::panic::catch_unwind(|| a.scaled(Ratio::infinity())).is_err());
    let big = ConvexHull::new(&[Point::new(i128::MAX, 0)]);
    assert!(std::panic::catch_unwind(|| big.scaled(2i64)).is_err());
}

#[test]
fn large_convex_cycles() {
    let n = 5000i64;
    let mut points = Vec::new();
    for x in -n..=n {
        points.push(p(x, x * x));
        points.push(p(x, 2 * n * n - x * x));
    }
    let hull = ConvexHull::new(&points);
    assert_eq!(hull.len(), (4 * n) as usize);
    let start = std::time::Instant::now();
    let result = hull.minkowski_sum(&hull);
    let elapsed = start.elapsed();
    assert_eq!(result, hull.scaled(2i64));
    eprintln!(
        "Minkowski merge: {} + {} vertices -> {}, {:?}",
        hull.len(),
        hull.len(),
        result.len(),
        elapsed
    );
    assert_eq!(hull.scaled(-1i64).len(), hull.len());
    assert!(ConvexHull::<i64>::new(&[]).scaled(Ratio::zero()).is_empty());
    assert!(
        std::panic::catch_unwind(|| ConvexHull::<i64>::new(&[]).scaled(Ratio::infinity())).is_err()
    );
}

#[test]
fn convex_hull_area_and_polygon_conversion() {
    let points = [p(0, 0), p(3, 0), p(3, 1), p(1, 1), p(1, 3), p(0, 3)];
    let hull = ConvexHull::new(&points);
    assert_eq!(hull.area2(), 14i128);
    assert_eq!(hull.area(), r(7, 1));
    assert_eq!(hull.area_f64(), 7.0);
    assert_eq!(hull.to_polygon().vertices(), hull.vertices());
    assert_eq!(hull.to_polygon().area(), hull.area());
    for k in [-3i64, 0, 1, 2] {
        assert_eq!(hull.scaled(k).area(), hull.area() * (k as i128 * k as i128));
    }
    assert_eq!(hull.scaled(r(-1, 2)).area(), r(7, 4));
    assert_eq!(hull.minkowski_sum(&hull).area(), r(28, 1));
    for points in [vec![], vec![p(2, 3)], vec![p(0, 0), p(1, 1)]] {
        let h = ConvexHull::new(&points);
        assert_eq!(h.area2(), 0);
        assert_eq!(h.area(), r(0, 1));
        assert_eq!(h.to_polygon().area(), r(0, 1));
    }
    let triangle = ConvexHull::new(&[p(0, 0), p(1, 0), p(0, 1)]);
    assert_eq!(triangle.area(), r(1, 2));
}

#[test]
fn merge_all_small_grid_hulls() {
    let grid: Vec<_> = (-1..=1)
        .flat_map(|x| (-1..=1).map(move |y| p(x, y)))
        .collect();
    let mut hulls = Vec::new();
    for mask in 0..1usize << grid.len() {
        let points: Vec<_> = grid
            .iter()
            .enumerate()
            .filter(|(i, _)| mask >> i & 1 != 0)
            .map(|(_, &p)| p)
            .collect();
        let hull = ConvexHull::new(&points);
        // 共有頂点・同一x座標・点と線分の鎖も辞書順に取得できる。
        let mut sorted = hull.vertices().to_vec();
        sorted.sort();
        sorted.dedup();
        assert_eq!(hull_sorted_vertices(hull.vertices()), sorted);
        if !hulls.contains(&hull) {
            hulls.push(hull);
        }
    }
    for a in &hulls {
        for b in &hulls {
            let points: Vec<_> = a
                .vertices()
                .iter()
                .chain(b.vertices())
                .map(|p| p.wide())
                .collect();
            let expected = jarvis(&points);
            let result = a.merge(b);
            assert_eq!(
                result
                    .vertices()
                    .iter()
                    .map(|p| p.wide())
                    .collect::<Vec<_>>(),
                expected
            );
            assert_eq!(result, b.merge(a));
            assert_eq!(result.merge(a), result);
        }
    }
    eprintln!(
        "merge: {} distinct grid hulls, {} pairs",
        hulls.len(),
        hulls.len() * hulls.len()
    );
}
#[test]
fn merge_structured_and_rational() {
    let sets = vec![
        vec![],
        vec![p(0, 0)],
        vec![p(0, -3), p(0, 3)],
        vec![p(-3, 0), p(3, 0)],
        vec![p(-1, -1), p(1, -1), p(1, 1), p(-1, 1)],
        vec![p(-4, -4), p(4, -4), p(4, 4), p(-4, 4)],
        vec![p(10, 0), p(11, 0), p(10, 1)],
        vec![p(-3, -2), p(-1, 0), p(-2, 3)],
    ];
    for a in &sets {
        for b in &sets {
            let (a, b) = (ConvexHull::new(a), ConvexHull::new(b));
            let points: Vec<_> = a.vertices().iter().chain(b.vertices()).copied().collect();
            assert_eq!(a.merge(&b), ConvexHull::new(&points));
            let (ar, br) = (a.scaled(r(-1, 2)), b.scaled(r(2, 3)));
            let points: Vec<_> = ar.vertices().iter().chain(br.vertices()).copied().collect();
            assert_eq!(ar.merge(&br).vertices(), jarvis(&points));
            let small = |h: &ConvexHull<Ratio128>| {
                ConvexHull::new(
                    &h.vertices()
                        .iter()
                        .map(|p| {
                            Point::new(
                                Ratio::from_fraction(
                                    *p.x.numerator() as i64,
                                    *p.x.denominator() as i64,
                                ),
                                Ratio::from_fraction(
                                    *p.y.numerator() as i64,
                                    *p.y.denominator() as i64,
                                ),
                            )
                        })
                        .collect::<Vec<_>>(),
                )
            };
            assert_eq!(
                small(&ar)
                    .merge(&small(&br))
                    .vertices()
                    .iter()
                    .map(|p| p.to_rational())
                    .collect::<Vec<_>>(),
                ar.merge(&br).into_vertices()
            );
        }
    }
    let (lo, hi) = (i64::MIN, i64::MAX);
    let a = ConvexHull::new(&[p(lo, lo), p(hi, hi)]);
    let b = ConvexHull::new(&[p(lo, hi), p(hi, lo)]);
    assert_eq!(
        a.merge(&b).vertices(),
        [p(lo, lo), p(hi, lo), p(hi, hi), p(lo, hi)]
    );
}
#[test]
fn merge_large_interleaved_cycles() {
    let n = 5000i64;
    let (mut even, mut odd) = (Vec::new(), Vec::new());
    for x in -n..=n {
        let target = if x % 2 == 0 { &mut even } else { &mut odd };
        target.push(p(x, x * x));
        target.push(p(x, 2 * n * n - x * x));
    }
    let (a, b) = (ConvexHull::new(&even), ConvexHull::new(&odd));
    let start = std::time::Instant::now();
    let result = a.merge(&b);
    let elapsed = start.elapsed();
    assert_eq!(result.len(), (4 * n) as usize);
    even.extend(odd);
    assert_eq!(result, ConvexHull::new(&even));
    eprintln!(
        "convex hull merge: {} + {} -> {}, {:?}",
        a.len(),
        b.len(),
        result.len(),
        elapsed
    );
}
