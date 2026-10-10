include!("../src/Gemetory/argsort.rs");

#[test]
fn comparator_obeys_total_order() {
    let coordinates = [i64::MIN, -1, 0, 1, i64::MAX];
    let ps: Vec<_> = coordinates
        .iter()
        .flat_map(|&x| coordinates.iter().map(move |&y| (x, y)))
        .collect();
    for atan2_order in [false, true] {
        for &a in &ps {
            for &b in &ps {
                let ab = argsort_detail::cmp(a, b, atan2_order);
                assert_eq!(ab, argsort_detail::cmp(b, a, atan2_order).reverse());
                for &c in &ps {
                    if ab.is_le() && argsort_detail::cmp(b, c, atan2_order).is_le() {
                        assert!(argsort_detail::cmp(a, c, atan2_order).is_le());
                    }
                }
            }
        }
    }
    let mut axes = [(-1, 0), (1, 0)];
    argsort_atan2_inplace(&mut axes);
    assert_eq!(axes, [(1, 0), (-1, 0)]);
}

#[test]
fn axes_and_quadrants() {
    let ps = [
        (0, -1),
        (-1, 0),
        (1, 1),
        (1, 0),
        (-1, -1),
        (0, 1),
        (1, -1),
        (-1, 1),
    ];
    let expected = [3, 2, 5, 7, 1, 4, 0, 6];
    assert_eq!(argsort(&ps), expected);
    let mut actual = ps;
    argsort_inplace(&mut actual);
    assert_eq!(actual, expected.map(|i| ps[i]));
    argsort_atan2_inplace(&mut actual);
    assert_eq!(actual, [4, 0, 6, 3, 2, 5, 7, 1].map(|i| ps[i]));
}

#[test]
fn origin_and_equal_angles_are_stable() {
    let ps = [(0, 1), (0, 0), (2, 0), (1, 1), (0, 0), (1, 0), (2, 2)];
    assert_eq!(argsort(&ps), [1, 2, 4, 5, 3, 6, 0]);
    let mut actual = ps;
    argsort_inplace(&mut actual);
    assert_eq!(actual, [1, 2, 4, 5, 3, 6, 0].map(|i| ps[i]));
    assert!(argsort(&[]).is_empty());
    argsort_inplace(&mut []);
    argsort_atan2_inplace(&mut []);
    assert_eq!(argsort(&[(0, 0)]), [0]);
}

#[test]
fn grid_matches_atan2_reference() {
    let mut ps = Vec::new();
    for x in -4..=4 {
        for y in -4..=4 {
            ps.push((x, y));
        }
    }
    for seed in 0..40_u64 {
        let mut state = seed + 1;
        for i in (1..ps.len()).rev() {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            ps.swap(i, state as usize % (i + 1));
        }
        let angle = |(x, y): (i64, i64)| (y as f64).atan2(x as f64);
        let positive_angle = |p| {
            let a = angle(p);
            if a < 0.0 {
                a + std::f64::consts::TAU
            } else {
                a
            }
        };
        let mut expected: Vec<_> = (0..ps.len()).collect();
        expected.sort_by(|&i, &j| {
            positive_angle(ps[i])
                .partial_cmp(&positive_angle(ps[j]))
                .unwrap()
        });
        assert_eq!(argsort(&ps), expected);
        let mut actual = ps.clone();
        argsort_inplace(&mut actual);
        assert_eq!(actual, expected.iter().map(|&i| ps[i]).collect::<Vec<_>>());
        argsort_atan2_inplace(&mut actual);
        assert!(actual.windows(2).all(|w| angle(w[0]) <= angle(w[1])));
    }
}

#[test]
fn full_i64_coordinates() {
    let lo = i64::MIN;
    let hi = i64::MAX;
    let ps = [
        (lo, lo),
        (lo, hi),
        (hi, lo),
        (hi, hi),
        (lo, 0),
        (0, lo),
        (hi, 0),
        (0, hi),
    ];
    let expected = [6, 3, 7, 1, 4, 0, 5, 2];
    assert_eq!(argsort(&ps), expected);
    let mut actual = ps;
    argsort_inplace(&mut actual);
    assert_eq!(actual, expected.map(|i| ps[i]));
    argsort_atan2_inplace(&mut actual);
    assert_eq!(actual, [0, 5, 2, 6, 3, 7, 1, 4].map(|i| ps[i]));
}
