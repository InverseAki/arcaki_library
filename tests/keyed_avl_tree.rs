#![allow(dead_code)]
#[path = "../src/SegmentTree/sortable_sequence.rs"]
mod sortable;
use sortable::{KeyedAvlMonoid, KeyedAvlTree, SortableSequence};
const MOD: u64 = 998_244_353;
struct Affine;
impl KeyedAvlMonoid for Affine {
    type S = (u64, u64);
    fn identity() -> Self::S {
        (1, 0)
    }
    fn op(f: &Self::S, g: &Self::S) -> Self::S {
        (f.0 * g.0 % MOD, (f.1 * g.0 + g.1) % MOD)
    }
}
type Tree = KeyedAvlTree<usize, Affine>;
type Seq = SortableSequence<usize, Affine>;
fn prod(a: &[(usize, (u64, u64))]) -> (u64, u64) {
    a.iter()
        .fold(Affine::identity(), |p, x| Affine::op(&p, &x.1))
}
fn value(i: usize) -> (u64, u64) {
    (i as u64 % 997 + 1, i as u64 % 991)
}
fn next(s: &mut u64, n: usize) -> usize {
    *s ^= *s << 13;
    *s ^= *s >> 7;
    *s ^= *s << 17;
    (*s % n as u64) as usize
}
fn check_tree(t: &Tree, a: &[(usize, (u64, u64))]) {
    t.check_invariants();
    assert_eq!(t.to_vec(), a);
    assert_eq!(t.len(), a.len());
    assert_eq!(t.all_prod(), prod(a));
    assert_eq!(
        t.all_prod_reverse(),
        a.iter()
            .rev()
            .fold(Affine::identity(), |p, x| Affine::op(&p, &x.1))
    );
    for (i, x) in a.iter().enumerate() {
        assert_eq!(t.get_index(i), (&x.0, &x.1));
        assert_eq!(t.get(&x.0), Some(&x.1));
    }
}
#[test]
fn every_rank_and_key_split_noncommutative() {
    for n in [0, 1, 2, 3, 7, 16, 33, 100] {
        let a: Vec<_> = (0..n).map(|i| (i * 2, value(i))).collect();
        for at in 0..=n {
            let (l, r) = Tree::from_sorted(a.clone()).split(at);
            check_tree(&l, &a[..at]);
            check_tree(&r, &a[at..]);
            check_tree(&l.concat(r), &a);
        }
        for key in 0..=2 * n + 1 {
            let at = a.partition_point(|x| x.0 < key);
            let (l, r) = Tree::from_sorted(a.clone()).split_key(&key);
            check_tree(&l, &a[..at]);
            check_tree(&r, &a[at..]);
            check_tree(&r.merge(l), &a);
        }
        let t = Tree::from_sorted(a.clone());
        for l in 0..=n {
            for r in l..=n {
                assert_eq!(t.prod(l, r), prod(&a[l..r]));
                assert_eq!(
                    t.prod_reverse(l, r),
                    a[l..r]
                        .iter()
                        .rev()
                        .fold(Affine::identity(), |p, x| Affine::op(&p, &x.1))
                );
            }
        }
    }
}
#[test]
fn ordered_updates_and_overlapping_key_ranges() {
    let mut s = 19237;
    let mut t = Tree::new();
    let mut a = std::collections::BTreeMap::new();
    for _ in 0..10000 {
        let k = next(&mut s, 300);
        if next(&mut s, 3) == 0 {
            assert_eq!(t.remove(&k), a.remove(&k));
        } else {
            let v = value(next(&mut s, 100000));
            assert_eq!(t.insert(k, v), a.insert(k, v));
        }
        check_tree(&t, &a.iter().map(|(&k, &v)| (k, v)).collect::<Vec<_>>());
    }
    for n in [1, 2, 3, 17, 128, 1023, 4096] {
        let a: Vec<_> = (0..n).map(|i| (i, value(i))).collect();
        for modulus in [2, 3, 7, 19] {
            let mut t = Tree::new();
            for rem in 0..modulus {
                let sub =
                    Tree::from_sorted(a.iter().copied().filter(|x| x.0 % modulus == rem).collect());
                t = t.merge(sub);
                t.check_invariants();
            }
            check_tree(&t, &a);
        }
    }
}
fn check_seq(seq: &Seq, a: &[(usize, (u64, u64))]) {
    seq.check_invariants();
    assert_eq!(seq.len(), a.len());
    assert_eq!(seq.all_prod(), prod(a));
    for (i, x) in a.iter().enumerate() {
        assert_eq!(seq.get(i), (&x.0, &x.1));
    }
}
#[test]
fn sortable_against_naive_all_query_types() {
    let mut s = 6789123;
    for n in [0, 1, 2, 3, 7, 16, 65, 257] {
        let mut a: Vec<_> = (0..n).map(|i| (i, value(i))).collect();
        let mut seq = Seq::from_vec(a.clone());
        let mut fresh = n;
        for step in 0..4000 {
            let l = next(&mut s, n + 1);
            let r = l + next(&mut s, n - l + 1);
            match next(&mut s, 4) {
                0 if n > 0 => {
                    let i = next(&mut s, n);
                    let v = value(next(&mut s, 100000));
                    seq.set(i, fresh, v);
                    a[i] = (fresh, v);
                    fresh += 1;
                }
                1 => assert_eq!(seq.prod(l, r), prod(&a[l..r])),
                2 => {
                    seq.sort_asc(l, r);
                    a[l..r].sort_unstable_by_key(|x| x.0);
                }
                _ => {
                    seq.sort_desc(l, r);
                    a[l..r].sort_unstable_by(|x, y| y.0.cmp(&x.0));
                }
            }
            check_seq(&seq, &a);
            if step % 41 == 0 {
                for l in 0..=n {
                    for r in l..=n {
                        assert_eq!(seq.prod(l, r), prod(&a[l..r]));
                    }
                }
            }
        }
    }
}
#[test]
fn structured_split_merge_and_alternating_sorts() {
    let n = 4096;
    let a: Vec<_> = (0..n).map(|i| (i, value(i))).collect();
    let mut t = Tree::new();
    for (k, v) in a.clone() {
        t.insert(k, v);
    }
    check_tree(&t, &a);
    for at in [1, n / 2, n - 1, 0, n] {
        let mut r = t.split_off(at);
        t.append(&mut r);
        assert!(r.is_empty());
        t.check_invariants();
    }
    for i in (0..n).rev() {
        assert_eq!(t.remove(&i), Some(value(i)));
        t.check_invariants();
    }
    let mut a: Vec<_> = (0..n).map(|i| ((i * 2053) % n, value(i))).collect();
    let mut seq = Seq::from_vec(a.clone());
    for step in 0..200 {
        let (l, r) = if step % 3 == 0 {
            (0, n)
        } else {
            (step, n - step)
        };
        let desc = step % 2 != 0;
        seq.sort(l, r, desc);
        a[l..r].sort_unstable_by(|x, y| if desc { y.0.cmp(&x.0) } else { x.0.cmp(&y.0) });
        check_seq(&seq, &a);
        assert_eq!(seq.prod(1, n - 1), prod(&a[1..n - 1]));
    }
}
#[test]
fn invalid_inputs_panic_before_mutation() {
    use std::panic::{catch_unwind, AssertUnwindSafe};
    let mut t = Tree::singleton(1, (2, 3));
    assert!(catch_unwind(AssertUnwindSafe(|| t.split_off(2))).is_err());
    assert_eq!(t.len(), 1);
    assert!(catch_unwind(|| t.prod(1, 0)).is_err());
    assert!(catch_unwind(|| Tree::singleton(1, (2, 3)).merge(Tree::singleton(1, (3, 4)))).is_err());
    assert!(catch_unwind(|| Tree::from_sorted(vec![(2, (1, 0)), (1, (1, 0))])).is_err());
    assert!(
        catch_unwind(|| Tree::singleton(2, (1, 0)).concat(Tree::singleton(1, (1, 0)))).is_err()
    );
    let mut seq = Seq::from_vec(vec![(1, (2, 3))]);
    assert!(catch_unwind(AssertUnwindSafe(|| seq.sort(0, 2, false))).is_err());
    assert!(catch_unwind(AssertUnwindSafe(|| seq.set(1, 2, (1, 0)))).is_err());
    check_seq(&seq, &[(1, (2, 3))]);
}

#[test]
fn key_needs_only_ord_and_monoid_needs_no_debug() {
    #[derive(PartialEq, Eq, PartialOrd, Ord)]
    struct Key(usize);
    #[derive(Clone, PartialEq)]
    struct Value(i64);
    struct Sum;
    impl KeyedAvlMonoid for Sum {
        type S = Value;
        fn identity() -> Value {
            Value(0)
        }
        fn op(a: &Value, b: &Value) -> Value {
            Value(a.0 + b.0)
        }
    }
    let a = KeyedAvlTree::<Key, Sum>::singleton(Key(1), Value(7));
    let b = KeyedAvlTree::<Key, Sum>::singleton(Key(0), Value(3));
    let t = a.merge(b);
    assert!(t.all_prod() == Value(10));
    let (a, b) = t.split_key(&Key(1));
    assert!(a.all_prod() == Value(3) && b.all_prod() == Value(7));
    let mut seq =
        SortableSequence::<Key, Sum>::from_vec(vec![(Key(2), Value(5)), (Key(1), Value(3))]);
    seq.sort_asc(0, 2);
    assert!(seq.get(0).0 == &Key(1));
    assert!(seq.prod(0, 2) == Value(8));
}
