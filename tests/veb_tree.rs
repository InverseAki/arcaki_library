#![allow(dead_code)]
include!("predecessor64_performance/common.rs");
include!("predecessor64_performance/model_checks.rs");
#[path = "../src/DataStructure/veb_tree.rs"]
mod veb;
impl_set!(veb::VebTree);
type TestedSet = veb::VebTree;
include!("predecessor64_performance/constructor_checks.rs");
#[test]
fn veb_model() { verify::<TestedSet>(); }

#[test]
fn large_sparse_recursive_boundaries() {
    let n = (1 << 28) + 129;
    let mut s = TestedSet::new(n);
    let mut oracle = BTreeSet::new();
    let mut seed = 831_749;
    for i in 0..20000 {
        let p = rand(&mut seed) % n;
        s.insert(p);
        oracle.insert(p);
        if i % 3 == 0 {
            let p = *oracle.first().unwrap();
            s.remove(p);
            oracle.remove(&p);
        }
        if i % 7 == 0 && !oracle.is_empty() {
            let p = *oracle.last().unwrap();
            s.remove(p);
            oracle.remove(&p);
        }
        check(&s, &oracle, p);
        check(&s, &oracle, rand(&mut seed) % n);
    }
    for p in oracle.clone() {
        s.remove(p);
        oracle.remove(&p);
        check(&s, &oracle, p);
    }
}

#[test]
fn constructors_across_recursive_summary_boundaries() {
    for len in [262145usize, 4194305] {
        for pattern in 0..4 {
            let mut seed = 12997;
            let words: Vec<u64> = (0..len).map(|i| match pattern {
                0 => 0,
                1 => u64::MAX,
                2 => if i % 4096 == 0 { 1 << (i % 64) } else { 0 },
                _ => rand(&mut seed) as u64,
            }).collect();
            let mut expected = current::Predecessor64::from_vec_u64(words.clone());
            let mut actual = TestedSet::from_vec_u64(words);
            for step in 0..10000 {
                let p = rand(&mut seed) % (len * 64);
                assert_eq!(actual.include(p), expected.include(p));
                assert_eq!(actual.prev(p), expected.prev(p));
                assert_eq!(actual.next(p), expected.next(p));
                assert_eq!(actual.inprev(p), expected.inprev(p));
                assert_eq!(actual.innext(p), expected.innext(p));
                assert_eq!(actual.min(), expected.min());
                assert_eq!(actual.max(), expected.max());
                if step % 2 == 0 { actual.insert(p); expected.insert(p); }
                else { actual.remove(p); expected.remove(p); }
            }
        }
    }
}
