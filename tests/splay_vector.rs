#![allow(dead_code)]
include!("../src/SegmentTree/splay.rs");
fn next(seed: &mut u64, n: usize) -> usize {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    (*seed % n as u64) as usize
}
fn compare(v: &SplayVector<i64>, a: &[i64]) {
    assert_eq!(v.len(), a.len());
    assert_eq!(v.is_empty(), a.is_empty());
    assert_eq!(v.iter().copied().collect::<Vec<_>>(), a);
    assert_eq!(
        v.iter().rev().copied().collect::<Vec<_>>(),
        a.iter().rev().copied().collect::<Vec<_>>()
    );
    assert_eq!(v.first(), a.first());
    assert_eq!(v.last(), a.last());
    for k in 0..=a.len() {
        assert_eq!(v.get(k), a.get(k));
        if k < a.len() {
            assert_eq!(v[k], a[k]);
        }
    }
}
#[test]
fn vector_operations_against_vec() {
    for initial in [0, 1, 3, 7, 32, 65] {
        for seed0 in [81, 1729, 987239] {
            let mut seed = seed0;
            let mut a: Vec<i64> = (0..initial).map(|i| i as i64).collect();
            let mut v = SplayVector::from(a.clone());
            for step in 0..2000 {
                let n = a.len();
                let k = next(&mut seed, n + 1);
                let r = k + next(&mut seed, n - k + 1);
                let x = next(&mut seed, 1000) as i64;
                match next(&mut seed, 18) {
                    0 if n < 100 => {
                        v.push(x);
                        a.push(x);
                    }
                    1 => assert_eq!(v.pop(), a.pop()),
                    2 if n < 100 => {
                        v.insert(k, x);
                        a.insert(k, x);
                    }
                    3 if k < n => assert_eq!(v.remove(k), a.remove(k)),
                    4 if k < n => {
                        v[k] = x;
                        a[k] = x;
                    }
                    5 if k < n => {
                        *v.get_mut(k).unwrap() += 1;
                        a[k] += 1;
                    }
                    6 => {
                        v.reverse();
                        a.reverse();
                    }
                    7 => {
                        v.reverse_range(k..r);
                        a[k..r].reverse();
                    }
                    8 if k < n => assert_eq!(v.swap_remove(k), a.swap_remove(k)),
                    9 if k < n => {
                        let j = next(&mut seed, n);
                        v.swap(k, j);
                        a.swap(k, j);
                    }
                    10 => {
                        let len = next(&mut seed, 100);
                        v.resize(len, x);
                        a.resize(len, x);
                    }
                    11 => {
                        v.truncate(k);
                        a.truncate(k);
                    }
                    12 => {
                        v.retain(|x| x % 3 != 0);
                        a.retain(|x| x % 3 != 0);
                    }
                    13 => {
                        v.retain_mut(|x| {
                            *x += 1;
                            *x % 4 != 0
                        });
                        a.retain_mut(|x| {
                            *x += 1;
                            *x % 4 != 0
                        });
                    }
                    14 => {
                        let actual = v.drain(k..r).collect::<Vec<_>>();
                        let expected = a.drain(k..r).collect::<Vec<_>>();
                        assert_eq!(actual, expected);
                    }
                    15 => {
                        let mut tail = v.split_off(k);
                        let mut tail_a = a.split_off(k);
                        compare(&tail, &tail_a);
                        v.append(&mut tail);
                        a.append(&mut tail_a);
                        assert!(tail.is_empty());
                    }
                    16 => {
                        for x in &mut v {
                            *x += 2;
                        }
                        for x in &mut a {
                            *x += 2;
                        }
                    }
                    _ => {
                        assert_eq!(v.get(k), a.get(k));
                        assert_eq!(v.get_splayed(k), a.get(k));
                    }
                }
                assert_eq!(v.len(), a.len());
                if step % 25 == 0 {
                    compare(&v, &a);
                }
            }
            compare(&v, &a);
            assert_eq!(v.into_vec(), a);
        }
    }
}
#[test]
fn references_and_double_ended_iterators_after_reversals() {
    for n in 0usize..=10 {
        for pattern in 0usize..(1usize << n) {
            let mut a: Vec<_> = (0..n as i64).collect();
            let mut v = SplayVector::from(a.clone());
            v.reverse();
            a.reverse();
            if n > 0 {
                v.reverse_range(n / 3..=n - 1);
                a[n / 3..n].reverse();
            }
            let mut it = v.iter();
            let mut expected = std::collections::VecDeque::from(a.clone());
            for step in 0..n {
                assert_eq!(it.len(), expected.len());
                let (x, y) = if pattern >> step & 1 == 0 {
                    (it.next(), expected.pop_front())
                } else {
                    (it.next_back(), expected.pop_back())
                };
                assert_eq!(x.copied(), y);
            }
            assert_eq!(it.next(), None);
            assert_eq!(it.next_back(), None);
            let mut it = v.iter_mut();
            let mut refs = Vec::new();
            let mut expected = std::collections::VecDeque::from(a.clone());
            for step in 0..n {
                let (x, y) = if pattern >> step & 1 == 0 {
                    (it.next(), expected.pop_front())
                } else {
                    (it.next_back(), expected.pop_back())
                };
                let x = x.unwrap();
                assert_eq!(*x, y.unwrap());
                refs.push(x);
            }
            drop(it);
            for x in refs {
                *x += 7;
            }
            for x in &mut a {
                *x += 7;
            }
            compare(&v, &a);
            let mut it = v.into_iter();
            let mut expected = std::collections::VecDeque::from(a);
            for step in 0..n {
                let (x, y) = if pattern >> step & 1 == 0 {
                    (it.next(), expected.pop_front())
                } else {
                    (it.next_back(), expected.pop_back())
                };
                assert_eq!(x, y);
            }
            assert_eq!(it.next(), None);
            assert_eq!(it.next_back(), None);
        }
    }
    let mut v = SplayVector::from([10, 20, 30, 40]);
    v.reverse_range(1..4);
    let a = &v[0];
    let b = v.get(2).unwrap();
    let it = v.iter();
    assert_eq!((*a, *b), (10, 30));
    assert_eq!(it.copied().collect::<Vec<_>>(), vec![10, 40, 30, 20]);
    *v.first_mut().unwrap() = 1;
    *v.last_mut().unwrap() = 2;
    assert_eq!(v.to_vec(), vec![1, 40, 30, 2]);
}
#[test]
fn non_clone_non_debug_values_drop_exactly_once() {
    use std::{cell::Cell, rc::Rc};
    struct Resource {
        id: usize,
        drops: Rc<Cell<usize>>,
    }
    impl Drop for Resource {
        fn drop(&mut self) {
            self.drops.set(self.drops.get() + 1);
        }
    }
    let drops = Rc::new(Cell::new(0));
    let mut v: SplayVector<_> = (0..100)
        .map(|id| Resource {
            id,
            drops: drops.clone(),
        })
        .collect();
    v.reverse();
    assert_eq!(v[0].id, 99);
    v[0].id = 300;
    let first = v.remove(0);
    assert_eq!(first.id, 300);
    drop(first);
    let tail = v.split_off(50);
    let mut owned = tail.into_iter();
    drop(owned.next());
    drop(owned.next_back());
    drop(owned);
    let mut drain = v.drain(10..30);
    drop(drain.next());
    drop(drain);
    v.retain(|x| x.id % 2 == 0);
    for x in v.iter_mut() {
        x.id += 1;
    }
    v.clear();
    assert!(v.is_empty());
    assert_eq!(drops.get(), 100);
    v.push(Resource {
        id: 1,
        drops: drops.clone(),
    });
    let values = v.into_vec();
    assert_eq!(values[0].id, 1);
    drop(values);
    assert_eq!(drops.get(), 101);
    let mut t = SplayTree::<NilMonoid<Resource>>::new();
    t.push(Resource {
        id: 2,
        drops: drops.clone(),
    });
    drop(t.pop());
    assert_eq!(drops.get(), 102);
}
#[test]
fn construction_capacity_ranges_and_conversion() {
    let mut v = SplayVector::with_capacity(100);
    assert!(v.capacity() >= 100);
    v.extend(0..20);
    let cap = v.capacity();
    v.clear();
    assert_eq!(v.capacity(), cap);
    let mut counter = 0;
    v.resize_with(12, || {
        counter += 1;
        counter
    });
    assert_eq!(counter, 12);
    v.extend_from_slice(&[13, 14]);
    v.extend(&[15, 16]);
    v.reverse_range(..=3);
    v.reverse_range(4..);
    v.reverse_range(..);
    let clone = v.clone();
    assert_eq!(clone, v);
    assert_eq!(format!("{:?}", v), format!("{:?}", v.to_vec()));
    v.reserve(200);
    assert!(v.capacity() >= v.len() + 200);
    v.shrink_to(50);
    assert!(v.capacity() >= 50);
    v.shrink_to_fit();
    assert!(v.capacity() >= v.len());
    let values: Vec<_> = v.into();
    assert_eq!(values, clone.to_vec());
    let mut left = SplayVector::from([1, 2]);
    let mut right = SplayVector::with_capacity(20);
    right.extend([3, 4]);
    let capacity = right.capacity();
    left.append(&mut right);
    assert_eq!(left.to_vec(), vec![1, 2, 3, 4]);
    assert!(right.is_empty());
    assert_eq!(right.capacity(), capacity);
    let mut v: SplayVector<i32> = [1, 2, 3].into();
    use std::panic::{catch_unwind, AssertUnwindSafe};
    assert!(catch_unwind(AssertUnwindSafe(|| v[3])).is_err());
    assert!(catch_unwind(AssertUnwindSafe(|| {
        v[3] = 5;
    }))
    .is_err());
    assert!(catch_unwind(AssertUnwindSafe(|| v.drain(0..=usize::MAX).count())).is_err());
    assert!(catch_unwind(AssertUnwindSafe(|| v.reverse_range(2..1))).is_err());
    assert_eq!(v.into_vec(), vec![1, 2, 3]);
}
