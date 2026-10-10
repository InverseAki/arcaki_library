#![allow(dead_code)]
use std::collections::BTreeMap;
include!("../src/DataStructure/intervalset.rs");

#[test]
fn only_changed_coverage_is_returned() {
    let mut s = IntervalSet::new();
    s.insert(0, 3);
    s.insert(5, 8);
    assert_eq!(s.insert_with_data(2, 6).collect::<Vec<_>>(), vec![(3, 5)]);
    assert_eq!(s.remove_with_data(2, 6).collect::<Vec<_>>(), vec![(2, 6)]);
    assert_eq!(s.remove_with_data(2, 6).collect::<Vec<_>>(), vec![]);
    assert_eq!(s.insert_with_data(8, 10).collect::<Vec<_>>(), vec![(8, 10)]);
}

#[test]
fn exhaustive_delta_model_and_early_drop() {
    for mask in 0u32..256 {
        let mut initial = IntervalSet::new();
        for i in 0..8 { if mask >> i & 1 != 0 { initial.insert(i, i+1); } }
        for l in -1..=9 {
            for r in -1..=9 {
                for inserting in [false, true] {
                    let mut s = initial.clone();
                    let changes: Vec<_> = if inserting { s.insert_with_data(l, r).collect() }
                        else { s.remove_with_data(l, r).collect() };
                    let mut expected = Vec::new();
                    let mut start = None;
                    for i in -2..=11 {
                        let before = (0..8).contains(&i) && mask >> i & 1 != 0;
                        let inside = l <= i && i < r;
                        let after = if inserting { before || inside } else { before && !inside };
                        assert_eq!(s.contains(i), after);
                        if before != after {
                            if start.is_none() { start = Some(i); }
                        } else if let Some(a) = start.take() { expected.push((a, i)); }
                    }
                    assert_eq!(changes, expected);
                    let mut plain = initial.clone();
                    if inserting { plain.insert(l, r); } else { plain.remove(l, r); }
                    assert_eq!(plain.s, s.s);
                    for consumed in 0..=changes.len() {
                        let mut partial = initial.clone();
                        if inserting {
                            let mut it = partial.insert_with_data(l, r);
                            for expected in &changes[..consumed] { assert_eq!(it.next(), Some(*expected)); }
                            drop(it);
                        } else {
                            let mut it = partial.remove_with_data(l, r);
                            for expected in &changes[..consumed] { assert_eq!(it.next(), Some(*expected)); }
                            drop(it);
                        }
                        assert_eq!(partial.s, s.s);
                    }
                }
            }
        }
    }
}

fn net_length(records: &[(i64, i64, bool)]) -> i128 {
    records.iter().map(|&(l, r, add)| (r as i128-l as i128)*if add { 1 } else { -1 }).sum()
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
        assert_eq!(net_length(&a), b.iter().map(|&(l, r)| r as i128 - l as i128).sum::<i128>() * if seed & 1 == 0 { 1 } else { -1 });
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
        assert_eq!(net_length(&a), b.iter().map(|&(l, r)| r as i128 - l as i128).sum::<i128>() * if inserting { 1 } else { -1 });
        assert_eq!(old.snapshot(), new.s.iter().map(|(&l, &r)| (l, r)).collect::<Vec<_>>());
    }
}
