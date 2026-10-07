#![allow(dead_code)]
include!("../src/Basic/ratio.rs");
include!("../src/Gemetory/geometry.rs");
fn p(x: i64, y: i64) -> IntPoint {
    Point::new(x, y)
}
fn r(n: i128, d: i128) -> Ratio128 {
    Ratio128::from_fraction(n, d)
}
#[test]
fn point_and_area() {
    let (a, b, c) = (p(1, 2), p(4, 2), p(1, 6));
    assert_eq!(b - a, Point::new(3i128, 0));
    assert_eq!(a + p(2, 3), Point::new(3i128, 5));
    assert_eq!(a * 3, Point::new(3i128, 6));
    assert_eq!(-p(i64::MIN, 0), Point::new(-(i64::MIN as i128), 0));
    assert_eq!(a.dot(b), 8);
    assert_eq!(a.cross(b), -6);
    assert_eq!(a.rotated90(), Point::new(-2i128, 1));
    assert_eq!(a.distance_squared(c), 16);
    assert_eq!(a.distance(c), 4.0);
    assert_eq!(a.manhattan_distance(c), 4);
    assert_eq!(signed_triangle_area2(a, b, c), 12);
    assert_eq!(signed_triangle_area2(a, c, b), -12);
    assert_eq!(triangle_area(a, b, c), r(6, 1));
    assert_eq!(triangle_centroid(a, b, c), Point::new(r(2, 1), r(10, 3)));
    assert_eq!(a.midpoint(b), Point::new(r(5, 2), r(2, 1)));
    assert_eq!(polygon_area(&[a, b, p(4, 6), c]), r(12, 1));
    assert_eq!(polygon_area::<i64>(&[]), r(0, 1));
    assert_eq!(ccw(a, b, p(0, 2)), Ccw::Behind);
    assert_eq!(ccw(a, b, p(5, 2)), Ccw::Beyond);
    assert_eq!(ccw(a, a, a), Ccw::OnSegment);
    assert_eq!(ccw(a, a, b), Ccw::Beyond);
    assert_eq!(p(0, 0).angle(), None);
    let q = p(1, 0).rotated(std::f64::consts::FRAC_PI_2);
    assert!(q.x.abs() < 1e-12 && (q.y - 1.0).abs() < 1e-12);
    assert_eq!(a / 2, Point::new(r(1, 2), r(1, 1)));
    assert!(p(1, 0).is_perpendicular(p(0, 1)));
    assert!(p(1, 2).is_parallel(p(-2, -4)));
    assert!((p(1, 0).angle_to(p(0, 1)).unwrap() - std::f64::consts::FRAC_PI_2).abs() < 1e-12);
    assert_eq!(
        triangle_circumcenter(a, b, c),
        Some(Point::new(r(5, 2), r(4, 1)))
    );
    assert_eq!(triangle_circumcenter(a, a, c), None);
    assert_eq!(
        p(i64::MIN, 0).manhattan_distance(p(i64::MAX, 0)),
        u64::MAX as i128
    );
}
#[test]
fn canonical_lines_and_projection() {
    assert_eq!(Line::new(2, -4, 6), Line::new(-3, 6, -9));
    assert_eq!(Line::new(0, -2, 4).unwrap().coefficients(), (0, 1, -2));
    assert_eq!(Line::new(0, 0, 1), None);
    assert_eq!(Line::through(p(0, 0), p(0, 0)), None);
    assert_eq!(
        Line::new(i128::MIN, 0, 0).unwrap().coefficients(),
        (1, 0, 0)
    );
    let l = Line::through(p(0, 1), p(1, 0)).unwrap();
    assert_eq!(l.coefficients(), (1, 1, -1));
    assert_eq!(l, Line::through(p(1, 0), p(0, 1)).unwrap());
    assert_eq!(l.projection(p(0, 0)), Point::new(r(1, 2), r(1, 2)));
    assert_eq!(l.reflection(p(0, 0)), p(1, 1).to_rational());
    assert_eq!(l.distance_squared(p(0, 0)), r(1, 2));
    let vertical = Line::new(1, 0, 0).unwrap();
    assert_eq!(
        l.intersection(vertical),
        LineIntersection::Point(p(0, 1).to_rational())
    );
    assert_eq!(l.intersection(l), LineIntersection::Coincident);
    assert_eq!(
        l.intersection(l.parallel_through(p(0, 0))),
        LineIntersection::Parallel
    );
    assert!(l.is_perpendicular(l.perpendicular_through(p(2, 3))));
    assert!(l.perpendicular_through(p(2, 3)).contains(p(2, 3)));
    assert_eq!(
        Line::from_rational_coefficients(r(1, 2), r(1, 3), r(-1, 6))
            .unwrap()
            .coefficients(),
        (3, 2, -1)
    );
    let a = Point::new(Ratio::from_fraction(1, 2), Ratio::zero());
    let b = Point::new(Ratio::zero(), Ratio::from_fraction(1, 3));
    assert_eq!(Line::through(a, b).unwrap().coefficients(), (2, 3, -1));
    assert_eq!(
        triangle_area(a, b, Point::new(Ratio::zero(), Ratio::zero())),
        r(1, 12)
    );
}
#[test]
fn segments_and_degeneracies() {
    let s = Segment::new(p(0, 0), p(4, 0));
    assert_eq!(s.closest_point(p(2, 3)), p(2, 0).to_rational());
    assert_eq!(s.distance_squared(p(5, 2)), r(5, 1));
    assert_eq!(
        s.intersection(Segment::new(p(2, -1), p(2, 1))),
        SegmentIntersection::Point(p(2, 0).to_rational())
    );
    assert_eq!(
        s.intersection(Segment::new(p(6, 0), p(2, 0))),
        SegmentIntersection::Overlap(Segment::new(p(2, 0).to_rational(), p(4, 0).to_rational()))
    );
    assert_eq!(
        s.intersection(Segment::new(p(4, 0), p(5, 1))),
        SegmentIntersection::Point(p(4, 0).to_rational())
    );
    assert_eq!(
        s.intersection(Segment::new(p(0, 1), p(4, 1))),
        SegmentIntersection::None
    );
    assert_eq!(
        s.segment_distance_squared(Segment::new(p(0, 1), p(4, 1))),
        r(1, 1)
    );
    let z = Segment::new(p(2, 0), p(2, 0));
    assert!(s.intersects(z));
    assert_eq!(z.distance_squared(p(2, 3)), r(9, 1));
    assert_eq!(
        z.intersection(z),
        SegmentIntersection::Point(p(2, 0).to_rational())
    );
    let a = Segment::new(p(0, 0), p(2, 1));
    let b = Segment::new(p(0, 1), p(1, 0));
    assert_eq!(
        a.intersection(b),
        SegmentIntersection::Point(Point::new(r(2, 3), r(1, 3)))
    );
}
#[test]
fn exhaustive_small_grid_against_parametric_intersection() {
    let points: Vec<_> = (-2..=2)
        .flat_map(|x| (-2..=2).map(move |y| p(x, y)))
        .collect();
    for &a in &points {
        for &b in &points {
            for &c in &points {
                for &d in &points {
                    let s = Segment::new(a, b);
                    let t = Segment::new(c, d);
                    let (ux, uy, vx, vy, wx, wy) = (
                        b.x - a.x,
                        b.y - a.y,
                        d.x - c.x,
                        d.y - c.y,
                        c.x - a.x,
                        c.y - a.y,
                    );
                    let det = ux * vy - uy * vx;
                    let expected = if det != 0 {
                        let mut n = wx * vy - wy * vx;
                        let mut m = wx * uy - wy * ux;
                        let mut den = det;
                        if den < 0 {
                            n = -n;
                            m = -m;
                            den = -den;
                        }
                        n >= 0 && n <= den && m >= 0 && m <= den
                    } else {
                        let on = |q: IntPoint, x: IntPoint, y: IntPoint| {
                            (q.x - x.x) * (y.y - x.y) == (q.y - x.y) * (y.x - x.x)
                                && q.x >= x.x.min(y.x)
                                && q.x <= x.x.max(y.x)
                                && q.y >= x.y.min(y.y)
                                && q.y <= x.y.max(y.y)
                        };
                        on(a, c, d) || on(b, c, d) || on(c, a, b) || on(d, a, b)
                    };
                    assert_eq!(s.intersects(t), expected);
                    if expected {
                        match s.intersection(t) {
                            SegmentIntersection::Point(q) => {
                                assert!(Segment::new(a.to_rational(), b.to_rational()).contains(q));
                                assert!(Segment::new(c.to_rational(), d.to_rational()).contains(q));
                            }
                            SegmentIntersection::Overlap(q) => {
                                assert!(q.a < q.b);
                                assert!(
                                    Segment::new(a.to_rational(), b.to_rational()).contains(q.a)
                                );
                                assert!(
                                    Segment::new(c.to_rational(), d.to_rational()).contains(q.b)
                                );
                            }
                            _ => panic!(),
                        }
                    }
                    if a != b {
                        let l = Line::through(a, b).unwrap();
                        assert!(l.contains(a) && l.contains(b));
                        assert_eq!(l, Line::through(b, a).unwrap());
                    }
                }
            }
        }
    }
}
#[test]
fn overflow_and_infinity_are_rejected() {
    assert!(std::panic::catch_unwind(
        || p(i64::MIN, i64::MIN).distance_squared(p(i64::MAX, i64::MAX))
    )
    .is_err());
    assert!(std::panic::catch_unwind(|| Point::new(Ratio::infinity(), Ratio::zero())).is_err());
    assert!(std::panic::catch_unwind(|| Line::new(i128::MIN, 1, 0)).is_err());
}

#[test]
fn rational_coordinate_geometry() {
    let a = Point::new(r(1, 2), r(1, 3));
    let b = Point::new(r(3, 2), r(-2, 3));
    assert_eq!(a.dot(b), r(19, 36));
    assert_eq!(a.cross(b), r(-5, 6));
    assert_eq!(a.distance_squared(b), r(2, 1));
    assert_eq!(a / r(1, 2), Point::new(r(1, 1), r(2, 3)));
    let l = Line::through(a, b).unwrap();
    assert_eq!(l.coefficients(), (6, 6, -5));
    let q = Point::new(r(5, 7), r(8, 11));
    let foot = l.projection(q);
    assert!(l.contains(foot));
    assert!((q - foot).dot(l.direction()).is_zero());
    assert_eq!(l.reflection(l.reflection(q)), q);
    assert_eq!(l.distance_squared(q), q.distance_squared(foot));
    let s = Segment::new(a, b);
    assert_eq!(s.closest_point(a), a);
    assert_eq!(s.closest_point(b), b);
    assert_eq!(
        s.intersection(Segment::new(a, a)),
        SegmentIntersection::Point(a)
    );
}

#[test]
fn exact_angle_and_slope_ordering() {
    use std::cmp::Ordering::*;
    let origin = p(0, 0);
    let directions = [
        p(1, 0),
        p(2, 1),
        p(1, 1),
        p(0, 1),
        p(-1, 1),
        p(-1, 0),
        p(-1, -1),
        p(0, -1),
        p(1, -1),
    ];
    for (i, &a) in directions.iter().enumerate() {
        for (j, &b) in directions.iter().enumerate() {
            assert_eq!(a.cmp_angle(b), Some(i.cmp(&j)));
            assert_eq!(
                Segment::new(origin, a).cmp_angle(Segment::new(origin, b)),
                Some(i.cmp(&j))
            );
        }
    }
    assert_eq!(p(0, 0).cmp_angle(p(1, 0)), None);
    assert_eq!(
        Segment::new(origin, origin).cmp_angle(Segment::new(origin, p(1, 0))),
        None
    );
    assert_eq!(p(2, 1).cmp_angle(p(4, 2)), Some(Equal));
    let mut lines = vec![
        Line::new(1, 1, 0).unwrap(),
        Line::new(1, 0, 0).unwrap(),
        Line::new(0, 1, 0).unwrap(),
        Line::new(1, -1, 0).unwrap(),
    ];
    lines.sort_by(|a, b| a.cmp_angle(*b));
    assert_eq!(
        lines.iter().map(|l| l.coefficients()).collect::<Vec<_>>(),
        vec![(0, 1, 0), (1, -1, 0), (1, 0, 0), (1, 1, 0)]
    );
    assert_eq!(lines[1].cmp_angle(Line::new(2, -2, 3).unwrap()), Equal);
    lines.sort_by(|a, b| a.cmp_slope(*b));
    assert_eq!(
        lines.iter().map(|l| l.coefficients()).collect::<Vec<_>>(),
        vec![(1, 1, 0), (0, 1, 0), (1, -1, 0), (1, 0, 0)]
    );
    let m = i128::MAX;
    assert_eq!(
        Line::new(m, m - 1, 0)
            .unwrap()
            .cmp_angle(Line::new(m - 1, m - 2, 0).unwrap()),
        Greater
    );
    assert_eq!(
        Line::new(1, i128::MIN, 0)
            .unwrap()
            .cmp_angle(Line::new(1, -m, 0).unwrap()),
        Less
    );
    assert_eq!(
        Point::new(i128::MIN, i128::MIN).cmp_angle(Point::new(-m, -m)),
        Some(Equal)
    );
    assert_eq!(
        Point::new(m, m - 1).cmp_angle(Point::new(m - 1, m - 2)),
        Some(Greater)
    );
    let huge = Segment::new(p(i64::MIN, i64::MIN), p(i64::MAX, i64::MAX));
    assert_eq!(huge.cmp_angle(Segment::new(origin, p(1, 1))), Some(Equal));
    let rational = Point::new(r(1, 2), r(1, 3));
    assert_eq!(
        rational.cmp_angle(Point::new(r(3, 2), r(1, 1))),
        Some(Equal)
    );
}
#[test]
fn integer_product_comparator_boundaries_and_reference() {
    use std::cmp::Ordering::*;
    let values = [
        i128::MIN,
        i128::MIN + 1,
        -i128::MAX,
        -3,
        -1,
        0,
        1,
        2,
        3,
        i128::MAX - 1,
        i128::MAX,
    ];
    for &a in &values {
        for &b in &values {
            for &c in &values {
                for &d in &values {
                    let sign = |a: i128, b: i128| {
                        if a == 0 || b == 0 {
                            0i8
                        } else if (a < 0) != (b < 0) {
                            -1
                        } else {
                            1
                        }
                    };
                    let (s, t) = (sign(a, b), sign(c, d));
                    let expected = if s != t {
                        s.cmp(&t)
                    } else if s == 0 {
                        Equal
                    } else {
                        let order = ratio_detail::Wide::mul(a.unsigned_abs(), b.unsigned_abs())
                            .cmp(&ratio_detail::Wide::mul(c.unsigned_abs(), d.unsigned_abs()));
                        if s < 0 {
                            order.reverse()
                        } else {
                            order
                        }
                    };
                    assert_eq!(
                        geometry_cmp_integer_products(a, b, c, d),
                        expected,
                        "{a}*{b} vs {c}*{d}"
                    );
                }
            }
        }
    }
    let vectors: Vec<_> = (-8..=8)
        .flat_map(|x| (-8..=8).map(move |y| p(x, y)))
        .filter(|q| *q != p(0, 0))
        .collect();
    let angle = |q: IntPoint| q.angle().unwrap().rem_euclid(2.0 * std::f64::consts::PI);
    for &a in &vectors {
        for &b in &vectors {
            let expected = if a.cross(b) == 0 && a.dot(b) > 0 {
                Equal
            } else {
                angle(a).partial_cmp(&angle(b)).unwrap()
            };
            assert_eq!(a.cmp_angle(b), Some(expected));
            let (l, k) = (
                Line::through(p(0, 0), a).unwrap(),
                Line::through(p(0, 0), b).unwrap(),
            );
            let theta = |q: IntPoint| q.angle().unwrap().rem_euclid(std::f64::consts::PI);
            let expected = if a.cross(b) == 0 {
                Equal
            } else {
                theta(a).partial_cmp(&theta(b)).unwrap()
            };
            assert_eq!(l.cmp_angle(k), expected);
            let slope = |q: IntPoint| {
                if q.x == 0 {
                    f64::INFINITY
                } else {
                    q.y as f64 / q.x as f64
                }
            };
            assert_eq!(l.cmp_slope(k), slope(a).partial_cmp(&slope(b)).unwrap());
        }
    }
}

#[test]
fn concave_polygon_orientation_and_area() {
    let boundary = [p(0, 0), p(3, 0), p(3, 1), p(1, 1), p(1, 3), p(0, 3)];
    let polygon = Polygon::new(&boundary);
    assert_eq!(polygon.vertices(), boundary);
    assert_eq!(polygon.area2(), 10i128);
    assert_eq!(polygon.area(), r(5, 1));
    assert_eq!(polygon.area_f64(), 5.0);
    assert_eq!(polygon.len(), 6);
    let mut clockwise = boundary.to_vec();
    clockwise[1..].reverse();
    assert_eq!(Polygon::new(&clockwise), polygon);
    let mut closed = vec![boundary[0], boundary[0]];
    closed.extend(boundary[1..].iter().copied());
    closed.push(boundary[0]);
    assert_eq!(Polygon::new(&closed), polygon);
    for shift in 0..boundary.len() {
        let mut cycle = boundary.to_vec();
        cycle.rotate_left(shift);
        let poly = Polygon::new(&cycle);
        assert_eq!(poly.vertices()[0], boundary[shift]);
        assert_eq!(poly.area2(), 10);
    }
    let shifted: Vec<_> = boundary
        .iter()
        .map(|p| IntPoint::new(p.x + 1_000_000_000, p.y - 1_000_000_000))
        .collect();
    assert_eq!(Polygon::new(&shifted).area(), polygon.area());
    let rational: Vec<_> = boundary.iter().map(|p| p.to_rational() / r(2, 1)).collect();
    assert_eq!(Polygon::new(&rational).area2(), r(5, 2));
    assert_eq!(Polygon::new(&rational).area(), r(5, 4));
    let small: Vec<_> = boundary
        .iter()
        .map(|p| Point::new(Ratio::from_fraction(p.x, 2), Ratio::from_fraction(p.y, 2)))
        .collect();
    assert_eq!(Polygon::new(&small).area(), r(5, 4));
    let triangle = Polygon::new(&[p(0, 0), p(1, 0), p(0, 1)]);
    assert_eq!(triangle.area2(), 1);
    assert_eq!(triangle.area(), r(1, 2));
}
#[test]
fn polygon_degenerate_boundaries() {
    let empty = Polygon::<i64>::new(&[]);
    assert!(empty.is_empty());
    assert_eq!(empty.area2(), 0);
    assert_eq!(empty.area(), r(0, 1));
    for points in [
        vec![p(2, 3)],
        vec![p(0, 0), p(1, 1)],
        vec![p(0, 0), p(1, 1), p(2, 2)],
        vec![p(1, 1), p(1, 1), p(1, 1)],
    ] {
        let poly = Polygon::new(&points);
        assert_eq!(poly.area2(), 0);
        assert_eq!(poly.area(), r(0, 1));
    }
    assert!(std::panic::catch_unwind(|| Polygon::new(&[Point {
        x: Ratio::infinity(),
        y: Ratio::zero()
    }]))
    .is_err());
}

#[test]
fn optimized_rational_predicates_and_fallbacks() {
    use std::cmp::Ordering::*;
    let mut state = 478u64;
    let mut next = || {
        state ^= state << 7;
        state ^= state >> 9;
        state
    };
    for _ in 0..4000 {
        let mut point = || {
            Point::new(
                r((next() % 201) as i128 - 100, (next() % 13 + 1) as i128),
                r((next() % 201) as i128 - 100, (next() % 13 + 1) as i128),
            )
        };
        let (a, b, c) = (point(), point(), point());
        let expected = signed_triangle_area2(a, b, c).cmp(&Ratio128::zero());
        assert_eq!(Ratio128::cmp_turn(a, b, c), expected);
        let narrow = |p: RationalPoint| {
            Point::new(Ratio::try_from(p.x).unwrap(), Ratio::try_from(p.y).unwrap())
        };
        assert_eq!(Ratio::cmp_turn(narrow(a), narrow(b), narrow(c)), expected);
    }
    let a = Point::new(r(1, 10i128.pow(20)), r(1, 10i128.pow(20) + 1));
    assert_eq!(Ratio128::cmp_turn(a, a, a), Equal);
    let (a, b) = (r(i128::MAX, 2), r(2, i128::MAX));
    assert_eq!(Ratio128::cmp_products(a, b, r(1, 1), r(1, 1)), Equal);
    assert_eq!(Ratio128::cmp_products(a, b, r(2, 1), r(1, 1)), Less);
    let k = i128::MAX / 8;
    let (a, b, c) = (
        Point::new(r(0, 1), r(0, 1)),
        Point::new(r(k, 1), r(0, 1)),
        Point::new(r(0, 1), r(k, 1)),
    );
    assert_eq!(Ratio128::cmp_turn(a, b, c), Greater);
}
