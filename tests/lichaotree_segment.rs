#![allow(dead_code)]
include!("../src/SegmentTree/lichaotree_segment.rs");
mod whole {
    use std::mem::swap;
    include!("../src/SegmentTree/lichaotree_line.rs");
}

type Segment = (i64, i128, i64, i64);
fn check<const MIN: bool>(tree: &LiChaoSegmentTree<MIN>, lines: &[Segment], xs: &[i64]) {
    for &x in xs {
        let values = lines
            .iter()
            .filter(|&&(_, _, l, r)| l <= x && x <= r)
            .map(|&(a, b, _, _)| a as i128 * x as i128 + b);
        let expected = if MIN { values.min() } else { values.max() };
        assert_eq!(tree.query(x), expected, "x={x}, min={MIN}");
    }
}
fn exhaustive<const MIN: bool>() {
    let xs: Vec<_> = (-2..=2).collect();
    let mut candidates = Vec::new();
    for l in -2..=2 {
        for r in l..=2 {
            for a in -1..=1 {
                for b in -1..=1 {
                    candidates.push((a, b, l, r));
                }
            }
        }
    }
    for &first in &candidates {
        for &second in &candidates {
            let mut tree = LiChaoSegmentTree::<MIN>::new(-2, 2);
            for (a, b, l, r) in [first, second] {
                tree.add_segment(a, b, l, r);
            }
            check(&tree, &[first, second], &xs);
        }
    }
}
#[test]
fn exhaustive_pairs_min_max() {
    exhaustive::<true>();
    exhaustive::<false>();
}

fn differential<const MIN: bool>() {
    let xs: Vec<_> = (-24..=24).collect();
    let mut state = 123456789u64;
    let mut next = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    for _ in 0..40 {
        let mut tree = LiChaoSegmentTree::<MIN>::new(-24, 24);
        let mut lines = Vec::new();
        check(&tree, &lines, &xs);
        for step in 0..150 {
            let a = (next() % 101) as i64 - 50;
            let b = (next() % 1001) as i128 - 500;
            let l = (next() % 81) as i64 - 40;
            let r = (next() % 81) as i64 - 40;
            let (l, r) = (l.min(r), l.max(r));
            if step % 17 == 0 {
                tree.add_line(a, b);
                lines.push((a, b, -24, 24));
            } else {
                tree.add_segment(a, b, l, r);
                lines.push((a, b, l, r));
            }
            check(&tree, &lines, &xs);
        }
    }
}
#[test]
fn random_mixed_insertions_min_max() {
    differential::<true>();
    differential::<false>();
}

fn boundaries<const MIN: bool>() {
    let xs = [i64::MIN, i64::MIN + 1, -1, 0, 1, i64::MAX - 1, i64::MAX];
    let mut tree = LiChaoSegmentTree::<MIN>::new(i64::MIN, i64::MAX);
    let mut lines = Vec::new();
    check(&tree, &lines, &xs);
    for (a, b, l, r) in [
        (i64::MAX, 0, i64::MIN, i64::MIN),
        (i64::MIN, 0, i64::MAX, i64::MAX),
        (0, i128::MAX, 0, 0),
        (0, i128::MIN, 1, 1),
        (-1, 7, i64::MIN + 1, i64::MAX - 1),
        (i64::MIN, 0, i64::MIN, i64::MAX),
        (i64::MAX, 0, i64::MIN, i64::MAX),
    ] {
        tree.add_segment(a, b, l, r);
        lines.push((a, b, l, r));
        check(&tree, &lines, &xs);
    }
    let mut point = LiChaoSegmentTree::<MIN>::new(i64::MAX, i64::MAX);
    point.add_segment(0, 3, i64::MIN, i64::MAX - 1);
    assert_eq!(point.query(i64::MAX), None);
    point.add_segment(0, 5, i64::MIN, i64::MAX);
    point.add_line(0, -5);
    assert_eq!(point.query(i64::MAX), Some(if MIN { -5 } else { 5 }));
}
#[test]
fn full_i64_domain_and_single_point() {
    boundaries::<true>();
    boundaries::<false>();
}
#[test]
fn uncovered_clipped_and_adjacent_segments() {
    let mut tree = LiChaoSegmentTree::<true>::new(-3, 3);
    tree.add_segment(0, 0, 4, 10);
    assert!(tree.nodes.is_empty());
    tree.add_segment(0, 7, i64::MIN, -1);
    tree.add_segment(0, 9, 1, i64::MAX);
    assert_eq!(tree.query(-1), Some(7));
    assert_eq!(tree.query(0), None);
    assert_eq!(tree.query(1), Some(9));
    tree.add_segment(0, 5, 0, 0);
    assert_eq!(tree.query(0), Some(5));
    assert_eq!(tree.clone().query(3), Some(9));
}
fn compare_whole<const MIN: bool>() {
    let mut tree = LiChaoSegmentTree::<MIN>::new(-100, 100);
    let mut original = whole::LiChaoTree::<MIN>::new(-100, 100);
    for a in -10..=10 {
        tree.add_line(a, -(a as i128).pow(2));
        original.add_line(a, -(a as i128).pow(2));
        for x in -100..=100 {
            assert_eq!(tree.query(x), original.query(x));
        }
    }
}
#[test]
fn same_api_as_whole_line_and_coexistence() {
    compare_whole::<true>();
    compare_whole::<false>();
}
#[test]
#[should_panic]
fn invalid_domain() {
    LiChaoSegmentTree::<true>::new(1, 0);
}
#[test]
#[should_panic]
fn reversed_segment() {
    LiChaoSegmentTree::<true>::new(0, 10).add_segment(0, 0, 2, 1);
}
#[test]
#[should_panic]
fn query_outside_domain() {
    LiChaoSegmentTree::<true>::new(0, 10).query(-1);
}
