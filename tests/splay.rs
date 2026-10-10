#![allow(dead_code)]
include!("../src/DataStructure/splay.rs");

struct Sequence;
impl SplayMonoid for Sequence {
    type S = Vec<i64>;
    fn identity() -> Self::S {
        vec![]
    }
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        [a.as_slice(), b.as_slice()].concat()
    }
    fn reverse_prod(x: &mut Self::S) {
        x.reverse();
    }
}
struct Affine;
impl SplayLazyMonoid for Affine {
    type M = Sequence;
    type F = (i64, i64);
    fn identity() -> Self::F {
        (1, 0)
    }
    fn map(&(a, b): &Self::F, x: &Vec<i64>) -> Vec<i64> {
        x.iter().map(|&v| (a * v + b) % 97).collect()
    }
    fn composition(&(a, b): &Self::F, &(c, d): &Self::F) -> Self::F {
        (a * c % 97, (a * d + b) % 97)
    }
}
fn next(seed: &mut u64, n: usize) -> usize {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    (*seed % n as u64) as usize
}
fn check(tree: &mut SplayTree<Affine>, a: &[i64]) {
    assert_eq!(tree.len(), a.len());
    assert_eq!(tree.is_empty(), a.is_empty());
    assert_eq!(tree.prod(0, a.len()), a);
    unsafe {
        assert_eq!((*tree.nil).ac, 0);
        assert_eq!((*tree.nil).p, tree.nil);
        assert!((*tree.nil).prod.is_empty());
        assert!(!(*tree.nil).has_lazy && !(*tree.nil).rev);
        assert_eq!((*tree.r).p, tree.nil);
        let mut stack = vec![(tree.r, false)];
        let mut visited = 0;
        while let Some((c, done)) = stack.pop() {
            if c == tree.nil {
                continue;
            }
            if done {
                let (l, r) = ((*c).l, (*c).r);
                assert_eq!((*c).ac, (*l).ac + (*r).ac + 1);
                assert_eq!(
                    (*c).prod,
                    [
                        (*l).prod.as_slice(),
                        (*c).data.as_slice(),
                        (*r).prod.as_slice()
                    ]
                    .concat()
                );
                assert_eq!((*c).data.len(), 1);
                assert_eq!(tree.data[(*c).idx].get(), c);
                visited += 1;
            } else {
                tree.apply_down(c);
                for child in [(*c).l, (*c).r] {
                    if child != tree.nil {
                        assert_eq!((*child).p, c);
                    }
                }
                stack.extend([(c, true), ((*c).r, false), ((*c).l, false)]);
            }
        }
        assert_eq!(visited, a.len());
        assert_eq!(tree.data.len(), a.len());
    }
}
#[test]
fn random_sequence_and_noncommuting_affine_updates() {
    for initial in [0, 1, 2, 3, 7, 16, 31, 64] {
        for seed0 in [17, 81, 1234567] {
            let mut seed = seed0;
            let mut a = (0..initial).map(|i| i as i64 % 97).collect::<Vec<_>>();
            let mut tree = SplayTree::<Affine>::from_vec(a.iter().map(|&x| vec![x]).collect());
            for step in 0..2500 {
                let n = a.len();
                let l = next(&mut seed, n + 1);
                let r = l + next(&mut seed, n - l + 1);
                match next(&mut seed, 7) {
                    0 if n < 80 => {
                        let x = next(&mut seed, 97) as i64;
                        a.insert(l, x);
                        tree.insert(l, vec![x]);
                    }
                    1 if n > 0 => {
                        let k = next(&mut seed, n);
                        a.remove(k);
                        tree.erase(k);
                    }
                    2 | 3 => {
                        let f = (next(&mut seed, 97) as i64, next(&mut seed, 97) as i64);
                        for v in &mut a[l..r] {
                            *v = (f.0 * *v + f.1) % 97;
                        }
                        tree.apply(l, r, f);
                    }
                    4 => {
                        a[l..r].reverse();
                        tree.reverse(l, r);
                    }
                    _ => assert_eq!(tree.prod(l, r), a[l..r]),
                }
                if step % 25 == 0 {
                    check(&mut tree, &a);
                }
            }
            check(&mut tree, &a);
        }
    }
}
#[test]
fn pending_updates_at_insert_erase_and_empty_ranges() {
    let mut tree = SplayTree::<Affine>::new();
    tree.apply(0, 0, (0, 42));
    tree.reverse(0, 0);
    assert_eq!(tree.prod(0, 0), Vec::<i64>::new());
    for i in 0..20 {
        tree.insert(i, vec![i as i64]);
    }
    let mut a = (0..20i64).collect::<Vec<_>>();
    for _ in 0..50 {
        tree.apply(0, a.len(), (3, 4));
        for x in &mut a {
            *x = (3 * *x + 4) % 97;
        }
        tree.reverse(0, a.len());
        a.reverse();
        tree.insert(0, vec![31]);
        a.insert(0, 31);
        tree.insert(a.len(), vec![17]);
        a.push(17);
        tree.insert(3, vec![61]);
        a.insert(3, 61);
        tree.erase(0);
        a.remove(0);
        tree.erase(a.len() - 1);
        a.pop();
        tree.erase(7);
        a.remove(7);
        for k in 0..=a.len() {
            tree.apply(k, k, (0, 82));
            assert!(tree.prod(k, k).is_empty());
        }
        check(&mut tree, &a);
    }
    while !a.is_empty() {
        tree.erase(a.len() / 2);
        a.remove(a.len() / 2);
    }
    check(&mut tree, &a);
    tree.insert(0, vec![11]);
    check(&mut tree, &[11]);
}
#[test]
fn deep_tree_and_bulk_build() {
    let n = 100000;
    let mut tree = SplayTree::<MM>::new();
    for i in 0..n {
        tree.insert(i, (i as i64, 1));
    }
    assert_eq!(tree.prod(0, 1), (0, 1));
    tree.reverse(0, n);
    tree.apply(0, n, 3);
    assert_eq!(tree.prod(0, 1), (n as i64 + 2, 1));
    assert_eq!(
        tree.prod(0, n),
        (n as i64 * (n as i64 - 1) / 2 + 3 * n as i64, n)
    );
    let mut bulk = SplayTree::<MM>::from_vec((0..n).map(|i| (i as i64, 1)).collect());
    assert_eq!(bulk.prod(n - 1, n), (n as i64 - 1, 1));
}
#[test]
fn invalid_indices_and_ranges_panic() {
    use std::panic::{catch_unwind, AssertUnwindSafe};
    let mut tree = SplayTree::<Affine>::new();
    assert!(catch_unwind(AssertUnwindSafe(|| tree.erase(0))).is_err());
    assert!(catch_unwind(AssertUnwindSafe(|| tree.insert(1, vec![0]))).is_err());
    tree.insert(0, vec![1]);
    assert!(catch_unwind(AssertUnwindSafe(|| tree.prod(0, 2))).is_err());
    assert!(catch_unwind(AssertUnwindSafe(|| tree.apply(1, 0, (1, 0)))).is_err());
    assert!(catch_unwind(AssertUnwindSafe(|| tree.reverse(0, 2))).is_err());
    check(&mut tree, &[1]);
}

#[test]
fn nil_monoid_matches_vector() {
    for initial in [0, 1, 2, 3, 7, 32, 65] {
        for seed0 in [23, 81, 17389] {
            let mut seed = seed0;
            let mut a: Vec<i64> = (0..initial).map(|i| i as i64).collect();
            let mut tree = SplayTree::<NilMonoid<i64>>::from_vec(a.clone());
            for step in 0..2500 {
                let n = a.len();
                let l = next(&mut seed, n + 1);
                let r = l + next(&mut seed, n - l + 1);
                let x = next(&mut seed, 1000) as i64;
                match next(&mut seed, 9) {
                    0 if n < 100 => {
                        tree.insert(l, x);
                        a.insert(l, x);
                    }
                    1 if n > 0 => {
                        let k = next(&mut seed, n);
                        assert_eq!(tree.remove(k), a.remove(k));
                    }
                    2 if n > 0 => {
                        let k = next(&mut seed, n);
                        tree.erase(k);
                        a.remove(k);
                    }
                    3 if n > 0 => {
                        let k = next(&mut seed, n);
                        tree.set(k, x);
                        a[k] = x;
                    }
                    4 if n > 0 => {
                        let k = next(&mut seed, n);
                        assert_eq!(tree.get(k), a[k]);
                    }
                    5 => {
                        tree.reverse(l, r);
                        a[l..r].reverse();
                    }
                    6 if n < 100 => {
                        tree.push(x);
                        a.push(x);
                    }
                    7 => assert_eq!(tree.pop(), a.pop()),
                    _ => {
                        tree.apply(l, r, ());
                        assert_eq!(tree.prod(l, r), ());
                    }
                }
                assert_eq!(tree.len(), a.len());
                assert_eq!(tree.is_empty(), a.is_empty());
                if step % 25 == 0 {
                    assert_eq!(tree.to_vec(), a);
                }
            }
            assert_eq!(tree.to_vec(), a);
        }
    }
}

#[test]
fn prod_monoid_noncommutative_sequence() {
    let mut seed = 83927;
    for n in [0, 1, 3, 7, 16, 63] {
        let mut a: Vec<i64> = (0..n).map(|i| i as i64).collect();
        let mut tree =
            SplayTree::<ProdMonoid<Sequence>>::from_vec(a.iter().map(|&x| vec![x]).collect());
        for step in 0..3000 {
            let n = a.len();
            let l = next(&mut seed, n + 1);
            let r = l + next(&mut seed, n - l + 1);
            let x = next(&mut seed, 1000) as i64;
            match next(&mut seed, 8) {
                0 if n < 80 => {
                    tree.insert(l, vec![x]);
                    a.insert(l, x);
                }
                1 if n > 0 => {
                    let k = next(&mut seed, n);
                    assert_eq!(tree.remove(k), vec![a.remove(k)]);
                }
                2 if n > 0 => {
                    let k = next(&mut seed, n);
                    tree.set(k, vec![x]);
                    a[k] = x;
                }
                3 if n > 0 => {
                    let k = next(&mut seed, n);
                    assert_eq!(tree.get(k), vec![a[k]]);
                }
                4 => {
                    tree.reverse(l, r);
                    a[l..r].reverse();
                }
                5 => {
                    tree.apply(l, r, ());
                    assert_eq!(tree.prod(l, r), a[l..r]);
                }
                6 => assert_eq!(tree.pop(), a.pop().map(|x| vec![x])),
                _ => assert_eq!(tree.prod(l, r), a[l..r]),
            }
            if step % 25 == 0 {
                assert_eq!(tree.prod(0, a.len()), a);
                assert_eq!(tree.to_vec().into_iter().flatten().collect::<Vec<_>>(), a);
                unsafe {
                    assert!(!(*tree.r).has_lazy);
                }
            }
        }
        assert_eq!(tree.prod(0, a.len()), a);
    }
}

#[test]
fn nil_values_without_identity_and_owned_removal() {
    use std::{cell::Cell, rc::Rc};
    #[derive(Debug)]
    struct Value {
        id: usize,
        clones: Rc<Cell<usize>>,
        drops: Rc<Cell<usize>>,
    }
    impl Clone for Value {
        fn clone(&self) -> Self {
            self.clones.set(self.clones.get() + 1);
            Self {
                id: self.id,
                clones: self.clones.clone(),
                drops: self.drops.clone(),
            }
        }
    }
    impl Drop for Value {
        fn drop(&mut self) {
            self.drops.set(self.drops.get() + 1);
        }
    }
    let clones = Rc::new(Cell::new(0));
    let drops = Rc::new(Cell::new(0));
    let values = (0..100)
        .map(|id| Value {
            id,
            clones: clones.clone(),
            drops: drops.clone(),
        })
        .collect();
    let mut tree = SplayTree::<NilMonoid<Value>>::from_vec(values);
    tree.reverse(0, 100);
    let removed = tree.remove(0);
    assert_eq!(removed.id, 99);
    let popped = tree.pop().unwrap();
    assert_eq!(popped.id, 0);
    tree.erase(5);
    assert_eq!(clones.get(), 0, "区間積・削除のためのcloneは不要");
    assert_eq!(tree.get(0).id, 98);
    assert_eq!(clones.get(), 1);
    drop(tree);
    drop(removed);
    drop(popped);
    assert_eq!(drops.get(), 101);
    let mut strings = SplayTree::<NilMonoid<String>>::from_vec(vec!["ab".into(), "cd".into()]);
    strings.reverse(0, 2);
    assert_eq!(strings.to_vec(), vec!["cd", "ab"]);
}

#[test]
fn new_sequence_methods_with_pending_affine_updates() {
    let mut tree = SplayTree::<Affine>::from_vec((0..20).map(|i| vec![i]).collect());
    let mut a: Vec<i64> = (0..20).collect();
    tree.apply(0, 20, (3, 4));
    for x in &mut a {
        *x = (3 * *x + 4) % 97;
    }
    tree.reverse(0, 20);
    a.reverse();
    tree.set(7, vec![82]);
    a[7] = 82;
    assert_eq!(tree.get(8), vec![a[8]]);
    assert_eq!(tree.remove(4), vec![a.remove(4)]);
    tree.push(vec![19]);
    a.push(19);
    assert_eq!(tree.pop(), a.pop().map(|x| vec![x]));
    assert_eq!(tree.to_vec().into_iter().flatten().collect::<Vec<_>>(), a);
    check(&mut tree, &a);
    let mut empty = SplayTree::<NilMonoid<i64>>::new();
    assert_eq!(empty.pop(), None);
    for k in [0, 1, usize::MAX] {
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| empty.get(k))).is_err());
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| empty.set(k, 1))).is_err()
        );
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| empty.remove(k))).is_err()
        );
    }
    assert!(empty.to_vec().is_empty());
}

#[test]
fn original_generic_lazy_interface_remains_compatible() {
    fn use_original_api<F: SplayLazyMonoid>(
        x: <F::M as SplayMonoid>::S,
        f: F::F,
    ) -> <F::M as SplayMonoid>::S {
        let mut tree: SplayTree<F> = SplayTree::new();
        tree.insert(0, x.clone());
        tree.apply(0, 1, f);
        tree.reverse(0, 1);
        let result: <F::M as SplayMonoid>::S = tree.prod(0, 1);
        tree.erase(0);
        let _: Node<F> = Node::new(x, 0, std::ptr::null_mut());
        result
    }
    assert_eq!(use_original_api::<MM>((10, 1), 3), (13, 1));
}
