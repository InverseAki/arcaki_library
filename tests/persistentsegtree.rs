#![allow(dead_code)]

#[path = "../src/SegmentTree/persistentsegtree.rs"]
mod persistent;

use persistent::{PersistentSegtree, SegtreeMonoid};

struct Sum;

impl SegtreeMonoid for Sum {
    type S = u64;
    fn identity() -> u64 {
        0
    }
    fn op(a: &u64, b: &u64) -> u64 {
        a + b
    }
}

fn next(seed: &mut u64, n: usize) -> usize {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed as usize % n
}

#[test]
fn branching_versions_match_linear_search() {
    let mut seed = 0x1234_5678_9abc_def0;
    for size in [0usize, 1, 2, 3, 5, 8, 17, 33] {
        let n = size.next_power_of_two();
        let base: Vec<u64> = (0..size).map(|_| next(&mut seed, 6) as u64).collect();
        let mut tree = PersistentSegtree::<Sum>::build(&base);
        let mut padded = base;
        padded.resize(n, 0);
        let mut versions = vec![padded];
        for _ in 0..40 {
            let t = next(&mut seed, versions.len());
            let p = next(&mut seed, n);
            let x = next(&mut seed, 6) as u64;
            tree.update(t, p, x);
            let mut a = versions[t].clone();
            a[p] = x;
            versions.push(a);
        }
        let count = tree.versions();
        for k in [0, 1, 2, 3, 7] {
            for _ in 0..12 {
                let ts: Vec<usize> = (0..k).map(|_| next(&mut seed, count)).collect();
                let limits: Vec<u64> = (0..k).map(|_| next(&mut seed, n * 8 + 1) as u64).collect();
                let predicate =
                    |values: &[u64]| values.iter().zip(&limits).all(|(v, lim)| v <= lim);
                for x in 0..=n {
                    let mut right = x;
                    while right < n
                        && ts
                            .iter()
                            .zip(&limits)
                            .all(|(&t, &lim)| versions[t][x..=right].iter().sum::<u64>() <= lim)
                    {
                        right += 1;
                    }
                    let mut left = x;
                    while left > 0
                        && ts
                            .iter()
                            .zip(&limits)
                            .all(|(&t, &lim)| versions[t][left - 1..x].iter().sum::<u64>() <= lim)
                    {
                        left -= 1;
                    }
                    assert_eq!(tree.parallel_max_right(&ts, x, predicate), right);
                    assert_eq!(tree.parallel_min_left(&ts, x, predicate), left);
                    if k == 1 {
                        assert_eq!(tree.max_right(ts[0], x, |v| *v <= limits[0]), right);
                        assert_eq!(tree.min_left(ts[0], x, |v| *v <= limits[0]), left);
                    }
                }
            }
        }
        assert_eq!(tree.versions(), count);
        for (t, a) in versions.iter().enumerate() {
            for (p, &value) in a.iter().enumerate() {
                assert_eq!(tree.get(t, p), value);
            }
        }
    }
}

struct Concat;

impl SegtreeMonoid for Concat {
    type S = String;
    fn identity() -> String {
        String::new()
    }
    fn op(a: &String, b: &String) -> String {
        a.clone() + b
    }
}

#[test]
fn noncommutative_products_preserve_order() {
    let base: Vec<String> = "abracadabra".chars().map(|c| c.to_string()).collect();
    let mut tree = PersistentSegtree::<Concat>::build(&base);
    tree.update(0, 3, "z".into());
    tree.update(0, 7, "y".into());
    let ts = [2, 0, 1, 0];
    for x in 0..=16 {
        for boundary in 0..=16 {
            let targets: Vec<String> = ts
                .iter()
                .map(|&t| tree.prod(t, x, boundary.max(x)))
                .collect();
            let predicate = |v: &[String]| {
                v.iter()
                    .zip(&targets)
                    .all(|(s, target)| target.starts_with(s))
            };
            let expected = (x..=16)
                .take_while(|&r| {
                    ts.iter()
                        .zip(&targets)
                        .all(|(&t, target)| target.starts_with(&tree.prod(t, x, r)))
                })
                .last()
                .unwrap();
            assert_eq!(tree.parallel_max_right(&ts, x, predicate), expected);

            let targets: Vec<String> = ts
                .iter()
                .map(|&t| tree.prod(t, boundary.min(x), x))
                .collect();
            let predicate = |v: &[String]| {
                v.iter()
                    .zip(&targets)
                    .all(|(s, target)| target.ends_with(s))
            };
            let expected = (0..=x)
                .rev()
                .take_while(|&l| {
                    ts.iter()
                        .zip(&targets)
                        .all(|(&t, target)| target.ends_with(&tree.prod(t, l, x)))
                })
                .last()
                .unwrap();
            assert_eq!(tree.parallel_min_left(&ts, x, predicate), expected);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Hash {
    value: u64,
    power: u64,
}

struct RollingHash;

impl SegtreeMonoid for RollingHash {
    type S = Hash;
    fn identity() -> Hash {
        Hash { value: 0, power: 1 }
    }
    fn op(a: &Hash, b: &Hash) -> Hash {
        Hash {
            value: a.value.wrapping_mul(b.power).wrapping_add(b.value),
            power: a.power.wrapping_mul(b.power),
        }
    }
}

#[test]
fn rolling_hash_common_prefix_and_suffix() {
    let base = b"abracadabra";
    let leaf = |c: u8| Hash {
        value: c as u64,
        power: 1_000_000_007,
    };
    let mut tree = PersistentSegtree::<RollingHash>::build(
        &base.iter().copied().map(leaf).collect::<Vec<_>>(),
    );
    let mut versions = vec![base.to_vec()];
    for (t, p, c) in [(0, 3, b'z'), (0, 7, b'y'), (1, 7, b'y')] {
        tree.update(t, p, leaf(c));
        let mut a = versions[t].clone();
        a[p] = c;
        versions.push(a);
    }
    for ts in [[0, 1, 2], [1, 3, 1], [0, 0, 0], [3, 2, 0]] {
        for x in 0..=base.len() {
            let equal = |v: &[Hash]| v.windows(2).all(|w| w[0] == w[1]);
            let mut right = x;
            while right < base.len()
                && ts
                    .windows(2)
                    .all(|w| versions[w[0]][right] == versions[w[1]][right])
            {
                right += 1;
            }
            if right == base.len() {
                right = 16;
            }
            let mut left = x;
            while left > 0
                && ts
                    .windows(2)
                    .all(|w| versions[w[0]][left - 1] == versions[w[1]][left - 1])
            {
                left -= 1;
            }
            assert_eq!(tree.parallel_max_right(&ts, x, equal), right);
            assert_eq!(tree.parallel_min_left(&ts, x, equal), left);
        }
    }
}

#[test]
#[should_panic]
fn rejects_false_identity() {
    PersistentSegtree::<Sum>::new(4).parallel_max_right(&[0, 0], 4, |_| false);
}

#[test]
#[should_panic]
fn rejects_false_empty_identity() {
    PersistentSegtree::<Sum>::new(4).parallel_min_left(&[], 0, |_| false);
}

#[test]
#[should_panic]
fn rejects_invalid_version_at_boundary() {
    PersistentSegtree::<Sum>::new(4).parallel_min_left(&[1], 0, |_| true);
}

#[test]
#[should_panic]
fn rejects_right_start_out_of_bounds() {
    PersistentSegtree::<Sum>::new(4).parallel_max_right(&[0], 5, |_| true);
}

#[test]
#[should_panic]
fn rejects_left_end_out_of_bounds() {
    PersistentSegtree::<Sum>::new(4).parallel_min_left(&[0], 5, |_| true);
}
