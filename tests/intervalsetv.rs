#![allow(dead_code)]
use std::collections::BTreeMap;
include!("../src/Basic/intervalsetv.rs");
mod previous {
    use std::collections::BTreeMap;
    include!("support/intervalsetv_before_extract.rs");
}

#[test]
fn exhaustive_records_model_and_early_drop() {
    for mut mask in 0..729 {
        let mut initial = IntervalSetV::new();
        let mut old_initial = previous::IntervalSetV::new();
        let mut model = [None; 6];
        for i in 0..6 {
            let state = mask % 3;
            mask /= 3;
            if state != 0 {
                let v = state-1;
                model[i] = Some(v);
                initial.insert(i as i32, i as i32+1, v);
                old_initial.insert(i as i32, i as i32+1, v);
            }
        }
        for l in -1..=7 {
            for r in -1..=7 {
                for value in [None, Some(0), Some(1)] {
                    let mut new = initial.clone();
                    let mut old = old_initial.clone();
                    let changes: Vec<_> = if let Some(v) = value { new.insert_with_data(l, r, v).collect() }
                        else { new.remove_with_data(l, r).collect() };
                    let before = if let Some(v) = value { old.insert_with_data(l, r, v) }
                        else { old.remove_with_data(l, r) };
                    assert_eq!(changes, before, "l={l}, r={r}, value={value:?}");
                    assert_eq!(new.iter().collect::<Vec<_>>(), old.iter().collect::<Vec<_>>());
                    let mut replay = initial.s.clone();
                    for &(a, b, v, add) in &changes {
                        if add { assert_eq!(replay.insert(a, (b, v)), None); }
                        else { assert_eq!(replay.remove(&a), Some((b, v))); }
                    }
                    assert_eq!(replay, new.s);
                    let mut plain = initial.clone();
                    if let Some(v) = value { plain.insert(l, r, v); }
                    else { plain.remove(l, r); }
                    assert_eq!(plain.s, new.s);
                    for i in -2..=8 {
                        let expected = if l <= i && i < r { value }
                            else if (0..6).contains(&i) { model[i as usize] } else { None };
                        assert_eq!(new.get(i), expected);
                        assert_eq!(new.contains(i), expected.is_some());
                    }
                    let sections: Vec<_> = new.iter().collect();
                    for pair in sections.windows(2) {
                        assert!(pair[0].1 <= pair[1].0);
                    }
                    for consumed in 0..=changes.len() {
                        let mut partial = initial.clone();
                        if let Some(v) = value {
                            let mut it = partial.insert_with_data(l, r, v);
                            for expected in &changes[..consumed] { assert_eq!(it.next(), Some(*expected)); }
                            drop(it);
                        } else {
                            let mut it = partial.remove_with_data(l, r);
                            for expected in &changes[..consumed] { assert_eq!(it.next(), Some(*expected)); }
                            drop(it);
                        }
                        assert_eq!(partial.s, new.s);
                    }
                }
            }
        }
    }
}

#[test]
fn sequences_and_extreme_endpoints() {
    let mut new = IntervalSetV::new();
    let mut old = previous::IntervalSetV::new();
    let mut seed = 123456789u64;
    for _ in 0..100000 {
        seed ^= seed << 7;
        seed ^= seed >> 9;
        let l = (seed % 2048) as i64 - 1024;
        let r = l + ((seed >> 32) % 64) as i64;
        let value = (seed >> 16) % 3;
        let a = if seed & 1 == 0 { old.insert_with_data(l, r, value) }
            else { old.remove_with_data(l, r) };
        let b: Vec<_> = if seed & 1 == 0 { new.insert_with_data(l, r, value).collect() }
            else { new.remove_with_data(l, r).collect() };
        assert_eq!(a, b);
        assert_eq!(old.iter().collect::<Vec<_>>(), new.iter().collect::<Vec<_>>());
    }
    for (l, r, value) in [(i64::MIN, i64::MAX, Some(0)), (-1, 1, Some(1)),
        (i64::MIN, -1, None), (1, i64::MAX, None), (i64::MIN, i64::MIN, Some(0)),
        (i64::MAX, i64::MAX, None)] {
        let a = if let Some(v) = value { old.insert_with_data(l, r, v) } else { old.remove_with_data(l, r) };
        let b: Vec<_> = if let Some(v) = value { new.insert_with_data(l, r, v).collect() } else { new.remove_with_data(l, r).collect() };
        assert_eq!(a, b);
        assert_eq!(old.iter().collect::<Vec<_>>(), new.iter().collect::<Vec<_>>());
    }
}

#[test]
fn consecutive_same_value_sections_keep_record_order() {
    let mut s = IntervalSetV::new();
    s.insert(0, 1, 1);
    s.insert(1, 2, 0);
    s.insert(2, 3, 1);
    s.insert(3, 4, 0);
    s.insert(1, 2, 1);
    assert_eq!(s.iter().collect::<Vec<_>>(), vec![(0, 1, 1), (1, 3, 1), (3, 4, 0)]);
    assert_eq!(s.insert_with_data(-1, 0, 1).collect::<Vec<_>>(),
        vec![(0, 1, 1, false), (1, 3, 1, false), (-1, 3, 1, true)]);
    assert_eq!(s.iter().collect::<Vec<_>>(), vec![(-1, 3, 1), (3, 4, 0)]);
}
