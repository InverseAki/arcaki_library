use std::mem::take;
include!("../src/Basic/bit_2d.rs");

#[test]
fn empty_and_sparse_x() {
    let empty = BIT2DArray::<i32>::new(0, []);
    assert_eq!(empty.prod(0, &-5, 0, &5), 0);
    let empty = BIT2DArray::<i32>::new(8, []);
    assert_eq!(empty.prefix(8, &10), 0);
    let mut bit = BIT2DArray::new(100, [(0, -10), (99, 20), (99, 20)]);
    bit.add(0, &-10, 4);
    bit.add(99, &20, -7);
    assert_eq!(bit.prod(1, &-100, 99, &100), 0);
    assert_eq!(bit.prod(99, &20, 100, &21), -7);
    assert_eq!(bit.prod(0, &-10, 100, &20), 4);
    assert_eq!(bit.prod(0, &-10, 100, &21), -3);
    assert_eq!(bit.prod(0, &20, 100, &20), 0);
}

#[test]
fn owned_ord_without_clone_or_copy() {
    #[derive(PartialEq, Eq, PartialOrd, Ord)]
    struct Key(String);
    let key = |s: &str| Key(s.into());
    let mut bit = BIT2DArray::new(4, [(3, key("pear")), (0, key("apple"))]);
    bit.add(3, &key("pear"), 7);
    bit.add(0, &key("apple"), 2);
    assert_eq!(bit.prod(0, &key("banana"), 4, &key("z")), 7);
    assert_eq!(bit.prefix(4, &key("pear")), 2);
}

#[test]
fn exhaustive_small_registered_subsets_and_rectangles() {
    let coords: Vec<_> = (0..3).flat_map(|x| [-2, 0, 3].map(|y| (x, y))).collect();
    for mask in 0..1usize << coords.len() {
        let points: Vec<_> = coords.iter().enumerate()
            .filter(|(i, _)| mask >> i & 1 != 0).map(|(_, &p)| p).collect();
        // 逆順かつ重複した登録で構築順と dedup を検証。
        let mut bit = BIT2DArray::new(3, points.iter().rev().chain(&points).copied());
        let mut weights = vec![0; points.len()];
        for round in 0..3 {
            for (i, &(x, y)) in points.iter().enumerate() {
                let w = (i as i64 + 1) * if round == 1 { -2 } else { 1 };
                weights[i] += w;
                bit.add(x, &y, w);
            }
            for lx in 0..=3 {
                for rx in lx..=3 {
                    for ly in -3..=4 {
                        for ry in ly..=4 {
                            let expected: i64 = points.iter().zip(&weights)
                                .filter(|((x, y), _)| lx <= *x && *x < rx && ly <= *y && *y < ry)
                                .map(|(_, w)| *w).sum();
                            assert_eq!(bit.prod(lx, &ly, rx, &ry), expected);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn larger_updates_match_naive_and_original() {
    let h = 31;
    let points: Vec<_> = (0..h).flat_map(|x| (0..13)
        .filter(move |y| (x * 7 + y * 3) % 5 == 0)
        .map(move |y| (x, y as i32 * 10 - 60))).collect();
    let mut bit = BIT2DArray::new(h, points.iter().copied());
    let old_points: Vec<_> = points.iter().map(|&(x, y)| (x as i32, y)).collect();
    let mut old = BIT2D::new(&old_points);
    let mut weights = vec![0i64; points.len()];
    let mut seed = 123456789u64;
    let mut next = || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    for _ in 0..5000 {
        let i = next() as usize % points.len();
        let (x, y) = points[i];
        let w = (next() % 101) as i64 - 50;
        weights[i] += w;
        bit.add(x, &y, w);
        old.add(x as i32, y, w);
        let a = next() as usize % (h + 1);
        let b = next() as usize % (h + 1);
        let (lx, rx) = (a.min(b), a.max(b));
        let a = (next() % 151) as i32 - 75;
        let b = (next() % 151) as i32 - 75;
        let (ly, ry) = (a.min(b), a.max(b));
        let expected: i64 = points.iter().zip(&weights)
            .filter(|((x, y), _)| lx <= *x && *x < rx && ly <= *y && *y < ry)
            .map(|(_, w)| *w).sum();
        assert_eq!(bit.prod(lx, &ly, rx, &ry), expected);
        assert_eq!(bit.prod(lx, &ly, rx, &ry), old.prod(lx as i32, ly, rx as i32, ry));
    }
}

#[test]
#[should_panic(expected = "x must be in 0..h")]
fn registration_outside_array_is_rejected() {
    let _ = BIT2DArray::new(3, [(3, 0)]);
}
