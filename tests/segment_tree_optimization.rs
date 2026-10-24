#![allow(dead_code)]
#[path = "../src/SegmentTree/lazysegtree.rs"]
mod lazy;
#[path = "../src/SegmentTree/segtree.rs"]
mod plain;
#[path = "../src/SegmentTree/segtree_with_monoid.rs"]
mod state;
struct Sum;
impl plain::SegtreeMonoid for Sum {
    type S = u64;
    fn identity() -> u64 {
        0
    }
    fn op(a: &u64, b: &u64) -> u64 {
        a + b
    }
}
impl state::SegtreeMonoid for Sum {
    type S = u64;
    fn identity() -> u64 {
        0
    }
    fn op(&self, a: &u64, b: &u64) -> u64 {
        a + b
    }
}
impl lazy::SegtreeMonoid for Sum {
    type S = (u64, u64);
    fn identity() -> Self::S {
        (0, 0)
    }
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        (a.0 + b.0, a.1 + b.1)
    }
}
struct Affine;
impl lazy::LazySegtreeMonoid for Affine {
    type M = Sum;
    type F = (u64, u64);
    fn identity() -> Self::F {
        (1, 0)
    }
    fn map(f: &Self::F, x: &(u64, u64)) -> (u64, u64) {
        (f.0 * x.0 + f.1 * x.1, x.1)
    }
    fn composition(f: &Self::F, g: &Self::F) -> Self::F {
        (f.0 * g.0, f.0 * g.1 + f.1)
    }
}
fn next(s: &mut u64, n: usize) -> usize {
    *s ^= *s << 13;
    *s ^= *s >> 7;
    *s ^= *s << 17;
    (*s % n as u64) as usize
}
#[test]
fn random_operations_searches_and_slices() {
    let mut seed = 1234567;
    for n in [0usize, 1, 2, 3, 7, 8, 9, 31, 32, 33, 127, 128, 129, 1024] {
        let size = n.next_power_of_two();
        let mut a = vec![1u64; n];
        a.resize(size, 0);
        let input = a
            .iter()
            .enumerate()
            .map(|(i, &x)| (x, u64::from(i < n)))
            .collect();
        let mut t = lazy::LazySegtree::<Affine>::build(&input);
        let mut p = plain::Segtree::<Sum>::from(a.clone());
        let mut m = state::Segtree::<Sum>::from(a.clone(), Sum);
        for step in 0..3000 {
            let x = next(&mut seed, size + 1);
            let y = next(&mut seed, size + 1);
            let (l, r) = (x.min(y), x.max(y));
            match next(&mut seed, 9) {
                0 | 1 | 2 => {
                    let f = (next(&mut seed, 2) as u64, next(&mut seed, 7) as u64);
                    t.apply_range(l, r, f);
                    for i in l..r {
                        a[i] = f.0 * a[i] + f.1 * u64::from(i < n);
                        p.set(i, a[i]);
                        m.set(i, a[i]);
                    }
                }
                3 => {
                    let i = next(&mut seed, size);
                    let v = next(&mut seed, 100) as u64;
                    t.set(i, (v, u64::from(i < n)));
                    a[i] = v;
                    p.set(i, v);
                    m.set(i, v);
                }
                4 => {
                    let i = next(&mut seed, size);
                    assert_eq!(t.get(i).0, a[i]);
                    assert_eq!(p.get(i), a[i]);
                    assert_eq!(m.get(i), a[i]);
                }
                5 => {
                    assert_eq!(
                        t.get_slice(l, r),
                        a[l..r]
                            .iter()
                            .enumerate()
                            .map(|(j, &v)| (v, u64::from(l + j < n)))
                            .collect::<Vec<_>>()
                    );
                }
                _ => {}
            }
            let expected = a[l..r].iter().sum::<u64>();
            assert_eq!(t.prod(l, r).0, expected, "{n} {step}");
            assert_eq!(p.prod(l, r), expected);
            assert_eq!(m.prod(l, r), expected);
            assert_eq!(t.all_prod().0, a.iter().sum::<u64>());
            let limit = next(&mut seed, 200) as u64;
            let (mut right, mut acc) = (l, 0);
            while right < size && acc + a[right] <= limit {
                acc += a[right];
                right += 1;
            }
            assert_eq!(t.max_right(l, |v| v.0 <= limit), right);
            assert_eq!(p.max_right(l, |v| *v <= limit), right);
            assert_eq!(m.max_right(l, |v| *v <= limit), right);
            let (mut left, mut acc) = (r, 0);
            while left > 0 && acc + a[left - 1] <= limit {
                left -= 1;
                acc += a[left];
            }
            assert_eq!(t.min_left(r, |v| v.0 <= limit), left);
            assert_eq!(p.min_left(r, |v| *v <= limit), left);
            assert_eq!(m.min_left(r, |v| *v <= limit), left);
        }
    }
}
struct Text;
impl plain::SegtreeMonoid for Text {
    type S = String;
    fn identity() -> String {
        String::new()
    }
    fn op(a: &String, b: &String) -> String {
        a.clone() + b
    }
}
impl state::SegtreeMonoid for Text {
    type S = String;
    fn identity() -> String {
        String::new()
    }
    fn op(&self, a: &String, b: &String) -> String {
        a.clone() + b
    }
}
impl lazy::SegtreeMonoid for Text {
    type S = String;
    fn identity() -> String {
        String::new()
    }
    fn op(a: &String, b: &String) -> String {
        a.clone() + b
    }
}
struct Remap;
impl lazy::LazySegtreeMonoid for Remap {
    type M = Text;
    type F = Vec<u8>;
    fn identity() -> Self::F {
        (0..=255).collect()
    }
    fn map(f: &Self::F, x: &String) -> String {
        x.bytes().map(|b| f[b as usize] as char).collect()
    }
    fn composition(f: &Self::F, g: &Self::F) -> Self::F {
        g.iter().map(|&x| f[x as usize]).collect()
    }
}
#[test]
fn noncommutative_noncopy_values_and_actions() {
    let mut seed = 813;
    for n in [1usize, 2, 3, 7, 8, 9, 31, 32, 33] {
        let size = n.next_power_of_two();
        let mut a: Vec<String> = (0..size)
            .map(|i| {
                if i < n {
                    ((b'a' + (i % 5) as u8) as char).to_string()
                } else {
                    String::new()
                }
            })
            .collect();
        let mut t = lazy::LazySegtree::<Remap>::build(&a);
        for _ in 0..20 {
            let x = next(&mut seed, size + 1);
            let y = next(&mut seed, size + 1);
            let (l, r) = (x.min(y), x.max(y));
            let mut f = <Remap as lazy::LazySegtreeMonoid>::identity();
            for b in b'a'..=b'e' {
                f[b as usize] = b'a' + next(&mut seed, 5) as u8;
            }
            t.apply_range(l, r, f.clone());
            for v in &mut a[l..r] {
                *v = <Remap as lazy::LazySegtreeMonoid>::map(&f, v);
            }
            let p = plain::Segtree::<Text>::from(a.clone());
            let m = state::Segtree::<Text>::from(a.clone(), Text);
            for l in 0..=size {
                for r in l..=size {
                    let target = a[l..r].concat();
                    assert_eq!(t.prod(l, r), target);
                    assert_eq!(p.prod(l, r), target);
                    assert_eq!(m.prod(l, r), target);
                    let mut expected = r;
                    while expected < size && a[expected].is_empty() {
                        expected += 1;
                    }
                    assert_eq!(t.max_right(l, |s| target.starts_with(&s)), expected);
                    assert_eq!(p.max_right(l, |s| target.starts_with(s)), expected);
                    assert_eq!(m.max_right(l, |s| target.starts_with(s)), expected);
                    let mut expected = l;
                    while expected > 0 && a[expected - 1].is_empty() {
                        expected -= 1;
                    }
                    assert_eq!(t.min_left(r, |s| target.ends_with(&s)), expected);
                    assert_eq!(p.min_left(r, |s| target.ends_with(s)), expected);
                    assert_eq!(m.min_left(r, |s| target.ends_with(s)), expected);
                }
            }
            assert_eq!(t.get_slice(l, r), a[l..r]);
        }
    }
}
#[test]
fn empty_construction_and_boundary_shapes() {
    let mut t = lazy::LazySegtree::<Affine>::new(0);
    assert_eq!(t.all_prod(), (0, 0));
    assert_eq!(t.prod(0, 0), (0, 0));
    assert!(t.get_slice(0, 0).is_empty());
    t.apply_range(0, 0, (0, 7));
    for n in [1usize, 2, 4, 8, 16, 32] {
        for l in 0..=n {
            for r in l..=n {
                let mut t = lazy::LazySegtree::<Affine>::build(&vec![(1, 1); n]);
                t.apply_range(0, n, (0, 3));
                t.apply_range(l, r, (2, 1));
                for i in 0..n {
                    assert_eq!(t.get(i), (if l <= i && i < r { 7 } else { 3 }, 1));
                }
                assert_eq!(
                    t.get_slice(0, n).iter().map(|v| v.0).sum::<u64>(),
                    3 * n as u64 + 4 * (r - l) as u64
                );
            }
        }
    }
}

#[test]
fn min_left_does_not_walk_above_root() {
    let mut t = lazy::LazySegtree::<Affine>::build(&vec![(7, 1)]);
    assert_eq!(t.min_left(1, |x| x.0 <= 6), 1);
    assert_eq!(t.min_left(1, |x| x.0 <= 7), 0);
    t.apply_range(0, 1, (0, 11));
    assert_eq!(t.min_left(1, |x| x.0 <= 10), 1);
}

#[test]
fn stateful_monoid_and_padded_index_compatibility() {
    struct Modulo(u64);
    impl state::SegtreeMonoid for Modulo {
        type S = u64;
        fn identity() -> u64 {
            0
        }
        fn op(&self, a: &u64, b: &u64) -> u64 {
            (a + b) % self.0
        }
    }
    let mut t = state::Segtree::from(vec![3, 5, 6], Modulo(7));
    assert_eq!(t.all_prod(), 0);
    t.set(3, 4);
    assert_eq!(t.prod(0, 4), 4);
    t.push(0, 5);
    assert_eq!(t.get(0), 1);
    let mut l = lazy::LazySegtree::<Affine>::build(&vec![(2, 1); 3]);
    l.set(3, (5, 1));
    l.apply_range(0, 4, (1, 3));
    assert_eq!(l.get_slice(0, 4), vec![(5, 1), (5, 1), (5, 1), (8, 1)]);
    assert_eq!(l.max_right(0, |_| true), 4);
}

#[test]
fn bit_length_matches_pointer_width() {
    assert_eq!(lazy::bit_length(0), 0);
    assert_eq!(lazy::bit_length(1), 0);
    for bit in 1..usize::BITS {
        assert_eq!(lazy::bit_length(1usize << bit), bit as usize);
        assert_eq!(lazy::bit_length((1usize << bit) + 1), bit as usize + 1);
    }
    assert_eq!(lazy::bit_length(usize::MAX), usize::BITS as usize);
}
