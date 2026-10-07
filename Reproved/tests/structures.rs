use arcaki_reproved::{
    binaryindexedtree::BIT,
    rollbackunionfind::RollbackUnionFind,
    segtree::{Segtree, SegtreeMonoid},
};
struct Sum;
impl SegtreeMonoid for Sum {
    type S = i64;
    fn identity() -> i64 {
        0
    }
    fn op(a: &i64, b: &i64) -> i64 {
        a + b
    }
}
struct Concat;
impl SegtreeMonoid for Concat {
    type S = String;
    fn identity() -> String {
        String::new()
    }
    fn op(a: &String, b: &String) -> String {
        format!("{a}{b}")
    }
}
fn next(seed: &mut u64, n: usize) -> usize {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    (*seed % n as u64) as usize
}
#[test]
fn sum_structures_match_naive() {
    let mut seed = 913;
    for n in 0..35 {
        let mut a = vec![0i64; n];
        let mut bit = BIT::new(n, 0);
        let mut seg = Segtree::<Sum>::new(n);
        assert_eq!((bit.len(), seg.len()), (n, n));
        assert_eq!((bit.is_empty(), seg.is_empty()), (n == 0, n == 0));
        for _ in 0..100 {
            if n > 0 {
                let i = next(&mut seed, n);
                let x = next(&mut seed, 10) as i64;
                if next(&mut seed, 2) == 0 {
                    a[i] = x;
                    bit.set(i, x);
                    seg.set(i, x);
                } else {
                    a[i] += x;
                    bit.add(i, x);
                    seg.push(i, x);
                }
                assert_eq!(bit.get(i), a[i]);
                assert_eq!(seg.get(i), a[i]);
            }
            let l = next(&mut seed, n + 1);
            let r = l + next(&mut seed, n - l + 1);
            let expected: i64 = a[l..r].iter().sum();
            assert_eq!(bit.prod(l, r), expected);
            assert_eq!(seg.prod(l, r), expected);
            assert_eq!(seg.all_prod(), a.iter().sum());
            let limit = next(&mut seed, 50) as i64;
            let mut right = l;
            let mut sum = 0;
            while right < n && sum + a[right] <= limit {
                sum += a[right];
                right += 1;
            }
            assert_eq!(seg.max_right(l, |s| *s <= limit), right);
            let mut left = r;
            sum = 0;
            while left > 0 && sum + a[left - 1] <= limit {
                left -= 1;
                sum += a[left];
            }
            assert_eq!(seg.min_left(r, |s| *s <= limit), left);
            for target in [-1, 0, 1, limit, a.iter().sum::<i64>() + 1] {
                let mut prefix = 0;
                let expected = if target <= 0 {
                    0
                } else {
                    a.iter()
                        .position(|x| {
                            prefix += x;
                            prefix >= target
                        })
                        .unwrap_or(n)
                };
                assert_eq!(bit.lower_bound(target), expected);
                assert_eq!(BIT::from_vec(a.clone(), 0).lower_bound(target), expected);
            }
        }
    }
}
#[test]
fn noncommutative_products_and_searches() {
    for n in 0..18 {
        let mut a: Vec<String> = (0..n)
            .map(|i| ((b'a' + i as u8) as char).to_string())
            .collect();
        let mut seg = Segtree::<Concat>::from(a.clone());
        if n > 0 {
            seg.set(0, "X".into());
            a[0] = "X".into();
            seg.push(n - 1, "!".into());
            a[n - 1].push('!');
        }
        for l in 0..=n {
            for r in l..=n {
                let target = a[l..r].concat();
                assert_eq!(seg.prod(l, r), target);
                assert_eq!(seg.max_right(l, |s| target.starts_with(s)), r);
                assert_eq!(seg.min_left(r, |s| target.ends_with(s)), l);
            }
            assert_eq!(seg.max_right(l, |_| true), n);
        }
    }
}
#[test]
fn rollback_matches_copied_partitions() {
    let mut seed = 81;
    for n in 1..12 {
        let mut uf = RollbackUnionFind::new(n);
        let mut labels: Vec<usize> = (0..n).collect();
        let mut history = Vec::new();
        for _ in 0..500 {
            match next(&mut seed, 8) {
                0 => {
                    uf.rollback();
                    if let Some(old) = history.pop() {
                        labels = old;
                    }
                }
                1 => {
                    uf.snapshot();
                    history.clear();
                }
                2 => {
                    uf.all_back();
                    if !history.is_empty() {
                        labels = history[0].clone();
                        history.clear();
                    }
                }
                _ => {
                    let u = next(&mut seed, n);
                    let v = next(&mut seed, n);
                    history.push(labels.clone());
                    let (from, to) = (labels[u], labels[v]);
                    assert_eq!(uf.merge(u, v), from != to);
                    for x in &mut labels {
                        if *x == from {
                            *x = to;
                        }
                    }
                }
            }
            for u in 0..n {
                assert_eq!(
                    uf.size(u),
                    labels.iter().filter(|&&x| x == labels[u]).count()
                );
                for v in 0..n {
                    assert_eq!(uf.same(u, v), labels[u] == labels[v]);
                }
            }
        }
    }
}
#[test]
fn duplicate_merge_all_back_regression() {
    let mut uf = RollbackUnionFind::new(2);
    assert!(uf.merge(0, 1));
    assert!(!uf.merge(0, 1));
    uf.all_back();
    assert!(!uf.same(0, 1));
    uf.rollback();
}
#[test]
fn invalid_ranges_fail_explicitly() {
    assert!(std::panic::catch_unwind(|| BIT::new(0, 0i64).add(0, 1)).is_err());
    assert!(std::panic::catch_unwind(|| BIT::new(2, 0i64).prod(2, 1)).is_err());
    assert!(std::panic::catch_unwind(|| Segtree::<Sum>::new(3).set(3, 1)).is_err());
    assert!(std::panic::catch_unwind(|| Segtree::<Sum>::new(3).prod(0, 4)).is_err());
    assert!(std::panic::catch_unwind(|| Segtree::<Sum>::new(3).max_right(4, |_| true)).is_err());
    assert!(std::panic::catch_unwind(|| Segtree::<Sum>::new(3).min_left(4, |_| true)).is_err());
}
