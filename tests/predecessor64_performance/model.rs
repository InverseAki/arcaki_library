include!("common.rs");
use std::collections::BTreeSet;
use std::ops::Bound::{Excluded, Included, Unbounded};

fn check<S: Set>(s: &S, oracle: &BTreeSet<usize>, p: usize) {
    assert_eq!(s.include(p), oracle.contains(&p), "include({p})");
    assert_eq!(s.prev(p), oracle.range(..p).next_back().copied().unwrap_or(!0), "prev({p})");
    assert_eq!(s.inprev(p), oracle.range(..=p).next_back().copied().unwrap_or(!0), "inprev({p})");
    assert_eq!(s.next(p), oracle.range((Excluded(p), Unbounded)).next().copied().unwrap_or(!0), "next({p})");
    assert_eq!(s.innext(p), oracle.range((Included(p), Unbounded)).next().copied().unwrap_or(!0), "innext({p})");
    assert_eq!(s.min(), oracle.first().copied().unwrap_or(!0));
    assert_eq!(s.max(), oracle.last().copied().unwrap_or(!0));
    assert_eq!(s.is_empty(), oracle.is_empty());
}

fn verify<S: Set>() {
    for n in 1..=10 {
        for bits in 0..1usize << n {
            let mut s = S::new(n);
            let mut oracle = BTreeSet::new();
            for p in 0..n {
                if bits >> p & 1 != 0 { s.insert(p); oracle.insert(p); }
            }
            for p in 0..n { check(&s, &oracle, p); }
            for p in 0..n {
                s.remove(p); s.remove(p); oracle.remove(&p); check(&s, &oracle, p);
                s.insert(p); s.insert(p); oracle.insert(p); check(&s, &oracle, p);
            }
        }
    }
    for n in [63, 64, 65, 4095, 4096, 4097, 262143, 262144, 262145, 16777217] {
        let mut s = S::new(n);
        let mut oracle = BTreeSet::new();
        let mut seed = n as u64;
        let boundaries: Vec<_> = [0, 1, 62, 63, 64, 65, 4094, 4095, 4096, 4097,
            262142, 262143, 262144, 262145, n - 1].into_iter().filter(|&p| p < n).collect();
        for &p in &boundaries { check(&s, &oracle, p); }
        for &p in &boundaries { s.insert(p); oracle.insert(p); }
        for &p in &boundaries { check(&s, &oracle, p); }
        for step in 0..20000 {
            let p = if step % 4 == 0 { boundaries[rand(&mut seed) % boundaries.len()] }
                else { rand(&mut seed) % n };
            match rand(&mut seed) % 3 {
                0 => { s.insert(p); oracle.insert(p); },
                1 => { s.remove(p); oracle.remove(&p); },
                _ => {},
            }
            check(&s, &oracle, p);
            check(&s, &oracle, rand(&mut seed) % n);
        }
        for p in oracle.clone() { s.remove(p); oracle.remove(&p); }
        for &p in &boundaries { check(&s, &oracle, p); }
    }
    for n in [65, 4097, 262145] {
        let mut s = S::new(n);
        for p in 0..n { s.insert(p); }
        for p in 0..n {
            assert!(s.include(p)); assert_eq!(s.innext(p), p); assert_eq!(s.inprev(p), p);
            s.remove(p);
            assert_eq!(s.min(), if p + 1 < n { p + 1 } else { !0 });
        }
        assert!(s.is_empty());
        for p in (0..n).rev() { s.insert(p); }
        for p in (0..n).rev() {
            s.remove(p);
            assert_eq!(s.max(), if p > 0 { p - 1 } else { !0 });
        }
        assert!(s.is_empty());
    }
}

#[test] fn baseline_model() { verify::<baseline::Predecessor64>(); }
#[test] fn update_model() { verify::<update_only::Predecessor64>(); }
#[test] fn incremental_model() { verify::<candidates::Incremental>(); }
#[test] fn leaf_model() { verify::<candidates::LeafFirst>(); }
#[test] fn flat_model() { verify::<flat::Flat>(); }
#[test] fn shifted_model() { verify::<candidates::Shifted>(); }
#[test] fn flat_shifted_model() { verify::<flat::FlatShifted>(); }
#[test] fn current_model() { verify::<current::Predecessor64>(); }
