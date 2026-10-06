#![allow(dead_code)]
#[path = "../src/SegmentTree/rollbacklazysegtree.rs"]
mod lazy;
#[path = "../src/SegmentTree/rollbacksegtree.rs"]
mod point;
#[path = "../src/Graph/weighted_rollback_unionfind.rs"]
mod weighted;
use lazy::{RollbackLazySegtree, RollbackLazySegtreeMonoid};
use point::{RollbackSegtree, RollbackSegtreeMonoid};
use weighted::{RollbackWeightedUnionFind, UFMonoid};
fn next(seed: &mut u64, n: usize) -> usize {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    (*seed % n as u64) as usize
}
struct Sum;
impl RollbackSegtreeMonoid for Sum {
    type S = i64;
    fn identity() -> i64 {
        0
    }
    fn op(a: &i64, b: &i64) -> i64 {
        a + b
    }
}
struct Affine;
impl RollbackLazySegtreeMonoid for Affine {
    type S = (i64, i64); // sum, length
    type F = (i64, i64); // a*x+b
    fn id_e() -> Self::S {
        (0, 0)
    }
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        (a.0 + b.0, a.1 + b.1)
    }
    fn identity() -> Self::F {
        (1, 0)
    }
    fn map(f: &Self::F, x: &Self::S) -> Self::S {
        (f.0 * x.0 + f.1 * x.1, x.1)
    }
    fn composition(f: &Self::F, g: &Self::F) -> Self::F {
        (f.0 * g.0, f.0 * g.1 + f.1)
    }
}
#[test]
fn trees_against_saved_naive_arrays() {
    let mut seed = 91;
    for n in [0, 1, 2, 3, 7, 16, 33] {
        let mut a = vec![0i64; n];
        let mut seg = RollbackSegtree::<Sum>::from(a.clone());
        let mut laz = RollbackLazySegtree::<Affine>::build(&vec![(0, 1); n]);
        let mut hp = Vec::new();
        let mut hl = Vec::new();
        let mut b = a.clone();
        for _ in 0..1500 {
            let l = next(&mut seed, n + 1);
            let r = l + next(&mut seed, n - l + 1);
            match next(&mut seed, 8) {
                0 => {
                    seg.rollback();
                    laz.rollback();
                    if let Some(old) = hp.pop() {
                        a = old;
                    }
                    if let Some(old) = hl.pop() {
                        b = old;
                    }
                }
                1 => {
                    seg.snapshot();
                    laz.snapshot();
                    hp.clear();
                    hl.clear();
                }
                2 => {
                    seg.all_back();
                    laz.all_back();
                    if let Some(old) = hp.first() {
                        a = old.clone();
                    }
                    if let Some(old) = hl.first() {
                        b = old.clone();
                    }
                    hp.clear();
                    hl.clear();
                }
                3 if n > 0 => {
                    let p = next(&mut seed, n);
                    let x = next(&mut seed, 10) as i64;
                    hp.push(a.clone());
                    hl.push(b.clone());
                    seg.push(p, x);
                    a[p] += x;
                    laz.apply(p, (1, x));
                    b[p] += x;
                }
                4 if n > 0 => {
                    let p = next(&mut seed, n);
                    let x = next(&mut seed, 10) as i64;
                    hp.push(a.clone());
                    hl.push(b.clone());
                    seg.set(p, x);
                    a[p] = x;
                    laz.set(p, (x, 1));
                    b[p] = x;
                }
                _ => {
                    let f = (next(&mut seed, 2) as i64, next(&mut seed, 5) as i64);
                    hl.push(b.clone());
                    laz.apply_range(l, r, f);
                    for x in &mut b[l..r] {
                        *x = f.0 * *x + f.1;
                    }
                }
            }
            assert_eq!(seg.len(), n);
            assert_eq!(laz.len(), n);
            assert_eq!(seg.is_empty(), n == 0);
            assert_eq!(laz.is_empty(), n == 0);
            assert_eq!(seg.prod(l, r), a[l..r].iter().sum());
            assert_eq!(laz.prod(l, r), (b[l..r].iter().sum(), (r - l) as i64));
            assert_eq!(seg.all_prod(), a.iter().sum());
            assert_eq!(laz.all_prod(), (b.iter().sum(), n as i64));
            let limit = next(&mut seed, 50) as i64;
            for (v, kind) in [(&a, 0), (&b, 1)] {
                let mut rr = l;
                let mut sum = 0;
                while rr < n && sum + v[rr] <= limit {
                    sum += v[rr];
                    rr += 1;
                }
                let mut ll = r;
                sum = 0;
                while ll > 0 && sum + v[ll - 1] <= limit {
                    ll -= 1;
                    sum += v[ll];
                }
                if kind == 0 {
                    assert_eq!(seg.max_right(l, |s| *s <= limit), rr);
                    assert_eq!(seg.min_left(r, |s| *s <= limit), ll);
                } else {
                    assert_eq!(laz.max_right(l, |s| s.0 <= limit), rr);
                    assert_eq!(laz.min_left(r, |s| s.0 <= limit), ll);
                }
            }
            for p in 0..n {
                assert_eq!(seg.get(p), a[p]);
                assert_eq!(laz.get(p), (b[p], 1));
            }
        }
    }
}
type Perm = [usize; 3];
const ID: Perm = [0, 1, 2];
const PERMS: [Perm; 6] = [
    [0, 1, 2],
    [0, 2, 1],
    [1, 0, 2],
    [1, 2, 0],
    [2, 0, 1],
    [2, 1, 0],
];
fn mul(a: &Perm, b: &Perm) -> Perm {
    [a[b[0]], a[b[1]], a[b[2]]]
}
fn inv(a: &Perm) -> Perm {
    let mut b = ID;
    for i in 0..3 {
        b[a[i]] = i;
    }
    b
}
struct Group;
impl UFMonoid for Group {
    type S = Perm;
    fn identity(&self) -> Perm {
        ID
    }
    fn op(&self, a: &Perm, b: &Perm) -> Perm {
        mul(a, b)
    }
    fn inv(&self, a: &Perm) -> Perm {
        inv(a)
    }
}
fn distances(edges: &[(usize, usize, Perm)], n: usize, u: usize) -> Vec<Option<Perm>> {
    let mut d = vec![None; n];
    d[u] = Some(ID);
    let mut stack = vec![u];
    while let Some(p) = stack.pop() {
        for &(x, y, w) in edges {
            let (v, w) = if x == p {
                (y, w)
            } else if y == p {
                (x, inv(&w))
            } else {
                continue;
            };
            if d[v].is_none() {
                d[v] = Some(mul(&d[p].unwrap(), &w));
                stack.push(v);
            }
        }
    }
    d
}
#[test]
fn weighted_union_find_noncommutative_group_against_graph() {
    let mut seed = 712;
    for n in 1..12 {
        let mut uf = RollbackWeightedUnionFind::new(n, Group);
        let mut edges = Vec::new();
        let mut hist = Vec::new();
        for _ in 0..700 {
            match next(&mut seed, 8) {
                0 => {
                    uf.rollback();
                    if let Some(old) = hist.pop() {
                        edges = old;
                    }
                }
                1 => {
                    uf.snapshot();
                    hist.clear();
                }
                2 => {
                    uf.all_back();
                    if let Some(old) = hist.first() {
                        edges = old.clone();
                    }
                    hist.clear();
                }
                _ => {
                    let u = next(&mut seed, n);
                    let v = next(&mut seed, n);
                    let w = PERMS[next(&mut seed, 6)];
                    let d = distances(&edges, n, u);
                    let ok = d[v].is_none() || d[v] == Some(w);
                    hist.push(edges.clone());
                    assert_eq!(uf.union(u, v, w), ok);
                    if ok && d[v].is_none() {
                        edges.push((u, v, w));
                    }
                }
            }
            for u in 0..n {
                let d = distances(&edges, n, u);
                assert_eq!(uf.size(u), d.iter().filter(|x| x.is_some()).count());
                assert_eq!(uf.dist(u), uf.find(u).1);
                assert_eq!(uf.leader(u), uf.find(u).0);
                for v in 0..n {
                    assert_eq!(uf.diff(u, v), d[v]);
                    assert_eq!(uf.same(u, v), d[v].is_some());
                }
            }
        }
    }
}
struct Concat;
impl RollbackSegtreeMonoid for Concat {
    type S = String;
    fn identity() -> String {
        String::new()
    }
    fn op(a: &String, b: &String) -> String {
        format!("{a}{b}")
    }
}
struct Remap;
impl RollbackLazySegtreeMonoid for Remap {
    type S = String;
    type F = Perm;
    fn id_e() -> String {
        String::new()
    }
    fn op(a: &String, b: &String) -> String {
        format!("{a}{b}")
    }
    fn identity() -> Perm {
        ID
    }
    fn map(f: &Perm, x: &String) -> String {
        x.bytes()
            .map(|c| (b'a' + f[(c - b'a') as usize] as u8) as char)
            .collect()
    }
    fn composition(f: &Perm, g: &Perm) -> Perm {
        mul(f, g)
    }
}
#[test]
fn noncommutative_products_actions_and_searches() {
    let mut seed = 132;
    for n in [0, 1, 3, 7, 16] {
        let mut a = vec!["a".to_owned(); n];
        let mut lazy = RollbackLazySegtree::<Remap>::from(a.clone());
        let mut history = Vec::new();
        for _ in 0..100 {
            if next(&mut seed, 3) == 0 {
                lazy.rollback();
                if let Some(old) = history.pop() {
                    a = old;
                }
            } else {
                let l = next(&mut seed, n + 1);
                let r = l + next(&mut seed, n - l + 1);
                let f = PERMS[next(&mut seed, 6)];
                history.push(a.clone());
                lazy.apply_range(l, r, f);
                for x in &mut a[l..r] {
                    *x = Remap::map(&f, x);
                }
            }
            let mut seg = RollbackSegtree::<Concat>::from(a.clone());
            if n > 0 {
                seg.set(0, "b".into());
                seg.push(0, "c".into());
                seg.rollback();
                seg.rollback();
            }
            assert_eq!(lazy.get_slice(0, n), a);
            for l in 0..=n {
                for r in l..=n {
                    let target = a[l..r].concat();
                    assert_eq!(seg.prod(l, r), target);
                    assert_eq!(lazy.prod(l, r), target);
                    assert_eq!(seg.max_right(l, |s| target.starts_with(s)), r);
                    assert_eq!(seg.min_left(r, |s| target.ends_with(s)), l);
                    assert_eq!(lazy.max_right(l, |s| target.starts_with(s)), r);
                    assert_eq!(lazy.min_left(r, |s| target.ends_with(s)), l);
                }
            }
        }
    }
}
#[test]
fn boundary_validation() {
    let mut e = RollbackLazySegtree::<Affine>::new(0);
    e.apply_range(0, 0, (1, 2));
    e.rollback();
    e.rollback();
    assert_eq!(e.all_prod(), (0, 0));
    assert!(std::panic::catch_unwind(|| RollbackSegtree::<Sum>::new(3).set(3, 1)).is_err());
    assert!(
        std::panic::catch_unwind(|| RollbackLazySegtree::<Affine>::new(3).set(3, (1, 1))).is_err()
    );
    assert!(
        std::panic::catch_unwind(|| RollbackLazySegtree::<Affine>::new(3).apply_range(
            2,
            1,
            (1, 1)
        ))
        .is_err()
    );
    assert!(std::panic::catch_unwind(|| RollbackLazySegtree::<Affine>::new(3).prod(0, 4)).is_err());
    assert!(std::panic::catch_unwind(
        || RollbackLazySegtree::<Affine>::new(3).max_right(4, |_| true)
    )
    .is_err());
    assert!(std::panic::catch_unwind(
        || RollbackLazySegtree::<Affine>::new(3).min_left(4, |_| true)
    )
    .is_err());
}

#[test]
fn large_non_power_of_two_ranges_and_pending_tags() {
    let n = 200001;
    let mut seg = RollbackSegtree::<Sum>::new(n);
    let mut laz = RollbackLazySegtree::<Affine>::from(vec![(0, 1); n]);
    laz.apply_range(0, n, (1, 7));
    laz.snapshot();
    for t in 0..200 {
        let l = (t * 977) % n;
        let r = n - t;
        let count = (r - l) as i64;
        laz.apply_range(l, r, (0, 3));
        assert_eq!(laz.all_prod(), (7 * n as i64 - 4 * count, n as i64));
        laz.set(n - 1, (11, 1));
        let previous = if r == n { 3 } else { 7 };
        assert_eq!(laz.all_prod().0, 7 * n as i64 - 4 * count + 11 - previous);
        assert_eq!(laz.get(n - 1), (11, 1));
        laz.rollback();
        laz.rollback();
        assert_eq!(laz.all_prod(), (7 * n as i64, n as i64));
        assert_eq!(laz.max_right(0, |s| s.0 <= 700), 100);
        assert_eq!(laz.min_left(n, |s| s.0 <= 700), n - 100);
        seg.set(l, 5);
        assert_eq!(seg.prod(0, n), 5);
        assert_eq!(seg.max_right(0, |s| *s == 0), l);
        seg.rollback();
        assert_eq!(seg.all_prod(), 0);
    }
}
