#![allow(dead_code)]
use std::collections::BTreeMap;
include!("../src/Basic/intervalset.rs");

#[test]
fn change_order_and_split() {
    let mut s = IntervalSet::new();
    s.insert(0, 2);
    s.insert(4, 6);
    assert_eq!(s.insert_with_data(2, 4).collect::<Vec<_>>(),
        vec![(0, 2, false), (4, 6, false), (0, 6, true)]);
    assert_eq!(s.remove_with_data(2, 4).collect::<Vec<_>>(),
        vec![(0, 6, false), (0, 2, true), (4, 6, true)]);
    assert_eq!(s.remove_with_data(1, 5).collect::<Vec<_>>(),
        vec![(0, 2, false), (0, 1, true), (4, 6, false), (5, 6, true)]);
}

#[test]
fn exhaustive_changes_and_early_drop() {
    // 全ての小さい被覆状態・操作区間・消費途中位置を調べる。
    for mask in 0u32..256 {
        let mut initial = IntervalSet::new();
        for i in 0..8 {
            if mask >> i & 1 != 0 { initial.insert(i, i + 1); }
        }
        for l in -1..=9 {
            for r in -1..=9 {
                for inserting in [false, true] {
                    let mut s = initial.clone();
                    let changes: Vec<_> = if inserting {
                        s.insert_with_data(l, r).collect()
                    } else {
                        s.remove_with_data(l, r).collect()
                    };
                    let mut replay = initial.s.clone();
                    for &(a, b, add) in &changes {
                        if add {
                            assert_eq!(replay.insert(a, b), None);
                        } else {
                            assert_eq!(replay.remove(&a), Some(b));
                        }
                    }
                    assert_eq!(replay, s.s);
                    let mut plain = initial.clone();
                    if inserting { plain.insert(l, r); }
                    else { plain.remove(l, r); }
                    assert_eq!(plain.s, s.s);
                    for i in -2..=10 {
                        let before = (0..8).contains(&i) && mask >> i & 1 != 0;
                        let inside = l <= i && i < r;
                        assert_eq!(s.contains(i), if inserting { before || inside } else { before && !inside });
                    }
                    for consumed in 0..=changes.len() {
                        let mut partial = initial.clone();
                        if inserting {
                            let mut it = partial.insert_with_data(l, r);
                            for expected in &changes[..consumed] {
                                assert_eq!(it.next(), Some(*expected));
                            }
                            drop(it);
                        } else {
                            let mut it = partial.remove_with_data(l, r);
                            for expected in &changes[..consumed] {
                                assert_eq!(it.next(), Some(*expected));
                            }
                            drop(it);
                        }
                        assert_eq!(partial.s, s.s);
                    }
                    // 戻り値を無視しても更新される。
                    let mut ignored = initial.clone();
                    if inserting { let _ = ignored.insert_with_data(l, r); }
                    else { let _ = ignored.remove_with_data(l, r); }
                    assert_eq!(ignored.s, s.s);
                }
            }
        }
    }
}

mod previous {
    use std::collections::BTreeMap;
    include!("support/intervalset_research_baseline.rs");
    impl<T: Ord + Copy> IntervalSet<T> {
        pub fn snapshot(&self) -> Vec<(T, T)> {
            self.s.iter().map(|(&l, &r)| (l, r)).collect()
        }
    }
}

#[test]
fn exact_records_match_previous_implementation() {
    for mask in 0u32..256 {
        for l in -1..=9 {
            for r in -1..=9 {
                for inserting in [false, true] {
                    let mut old = previous::IntervalSet::new();
                    let mut new = IntervalSet::new();
                    for i in 0..8 {
                        if mask >> i & 1 != 0 { old.insert(i, i+1); new.insert(i, i+1); }
                    }
                    let a: Vec<_> = if inserting { old.insert_with_data(l, r).collect() }
                        else { old.remove_with_data(l, r).collect() };
                    let b: Vec<_> = if inserting { new.insert_with_data(l, r).collect() }
                        else { new.remove_with_data(l, r).collect() };
                    assert_eq!(a, b);
                    assert_eq!(old.snapshot(), new.s.iter().map(|(&l, &r)| (l, r)).collect::<Vec<_>>());
                }
            }
        }
    }
}

#[test]
fn long_operation_sequences_and_boundaries() {
    let mut old = previous::IntervalSet::new();
    let mut new = IntervalSet::new();
    let mut seed = 987654321u64;
    for _ in 0..100000 {
        seed ^= seed << 7;
        seed ^= seed >> 9;
        let l = (seed % 2048) as i64 - 1024;
        let r = l + ((seed >> 32) % 64) as i64;
        let a: Vec<_> = if seed & 1 == 0 { old.insert_with_data(l, r).collect() }
            else { old.remove_with_data(l, r).collect() };
        let b: Vec<_> = if seed & 1 == 0 { new.insert_with_data(l, r).collect() }
            else { new.remove_with_data(l, r).collect() };
        assert_eq!(a, b);
        assert_eq!(old.snapshot(), new.s.iter().map(|(&l, &r)| (l, r)).collect::<Vec<_>>());
    }
    for (l, r, inserting) in [
        (i64::MIN, i64::MAX, true),
        (-1, 1, false),
        (i64::MIN, -1, false),
        (1, i64::MAX, false),
        (i64::MIN, i64::MIN, true),
        (i64::MAX, i64::MAX, false),
    ] {
        let a: Vec<_> = if inserting { old.insert_with_data(l, r).collect() }
            else { old.remove_with_data(l, r).collect() };
        let b: Vec<_> = if inserting { new.insert_with_data(l, r).collect() }
            else { new.remove_with_data(l, r).collect() };
        assert_eq!(a, b);
        assert_eq!(old.snapshot(), new.s.iter().map(|(&l, &r)| (l, r)).collect::<Vec<_>>());
    }
}
