use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign};
include!("../src/DataStructure/bitset.rs");

fn next(seed: &mut u64) -> u64 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed
}
fn make(h: usize, w: usize, mode: usize, seed: &mut u64) -> BitMatrix {
    let mut a = BitMatrix::new(h, w);
    for i in 0..h {
        for j in 0..w {
            a.set(i, j, match mode {
                0 => false,
                1 => true,
                2 => i == j,
                3 => (i + j) % 2 == 0,
                4 => j % 64 == 63 || j % 64 == 0,
                _ => next(seed) & 1 != 0,
            });
        }
    }
    a
}
fn naive(a: &BitMatrix, b: &BitMatrix, op: usize) -> Vec<Vec<usize>> {
    let lhs: Vec<Vec<bool>> = (0..a.height()).map(|i|
        (0..a.width()).map(|k| a.get(i, k)).collect()).collect();
    let columns: Vec<Vec<bool>> = (0..b.width()).map(|j|
        (0..b.height()).map(|k| b.get(k, j)).collect()).collect();
    lhs.iter().map(|row| columns.iter().map(|col| {
        match op {
            0 => row.iter().zip(col).map(|(&x, &y)| usize::from(x & y)).sum(),
            1 => row.iter().zip(col).map(|(&x, &y)| usize::from(x | y)).sum(),
            _ => row.iter().zip(col).map(|(&x, &y)| usize::from(x ^ y)).sum(),
        }
    }).collect()).collect()
}
#[test]
fn boundary_shapes_and_structured_products() {
    let mut seed = 1234567;
    for k in [0, 1, 2, 63, 64, 65, 127, 128, 129] {
        for (h, w) in [(0, 3), (3, 0), (1, 1), (3, 5), (65, 2), (2, 65)] {
            for mode in 0..6 {
                let a = make(h, k, mode, &mut seed);
                let b = make(k, w, (mode + 1) % 6, &mut seed);
                assert_eq!(a.transpose().transpose(), a);
                assert_eq!(b.transpose().transpose(), b);
                assert_eq!(a.mul_and(&b), naive(&a, &b, 0));
                assert_eq!(a.mul_or(&b), naive(&a, &b, 1));
                assert_eq!(a.mul_xor(&b), naive(&a, &b, 2));
            }
        }
    }
}
#[test]
fn exhaustive_two_by_two() {
    for x in 0..16 {
        for y in 0..16 {
            let mut a = BitMatrix::new(2, 2);
            let mut b = a.clone();
            for i in 0..2 { for j in 0..2 {
                a.set(i, j, x >> (i * 2 + j) & 1 != 0);
                b.set(i, j, y >> (i * 2 + j) & 1 != 0);
            }}
            assert_eq!(a.mul_and(&b), naive(&a, &b, 0));
            assert_eq!(a.mul_or(&b), naive(&a, &b, 1));
            assert_eq!(a.mul_xor(&b), naive(&a, &b, 2));
        }
    }
}
#[test]
fn padding_and_invalid_shapes() {
    let a = BitMatrix::from_rows(65, vec![BitSet::build(65, vec![!0, !0])]);
    let b = BitMatrix::from_rows(1, vec![BitSet::build(1, vec![!0]); 65]);
    assert_eq!(a.mul_and(&b), vec![vec![65]]);
    assert_eq!(a.mul_or(&b), vec![vec![65]]);
    assert_eq!(a.mul_xor(&b), vec![vec![0]]);
    use std::panic::{catch_unwind, AssertUnwindSafe};
    assert!(catch_unwind(|| BitMatrix::from_rows(2, vec![BitSet::new(1)])).is_err());
    assert!(catch_unwind(|| a.mul_and(&BitMatrix::new(64, 1))).is_err());
    assert!(catch_unwind(|| a.get(0, 65)).is_err());
    let mut a = a;
    assert!(catch_unwind(AssertUnwindSafe(|| a.set(0, 65, true))).is_err());
}
#[test]
#[ignore = "release performance measurement"]
fn benchmark() {
    use std::time::Instant;
    let mut seed = 246813579;
    for n in [256, 512] {
        let a = make(n, n, 5, &mut seed);
        let b = make(n, n, 5, &mut seed);
        for op in 0..3 {
            let start = Instant::now();
            let fast = match op { 0 => a.mul_and(&b), 1 => a.mul_or(&b), _ => a.mul_xor(&b) };
            let packed = start.elapsed();
            let start = Instant::now();
            let expected = naive(&a, &b, op);
            let scalar = start.elapsed();
            assert_eq!(fast, expected);
            eprintln!("n={n} op={op} packed={packed:?} scalar={scalar:?} speedup={:.1}", scalar.as_secs_f64()/packed.as_secs_f64());
        }
    }
}

fn logical_naive(a: &BitMatrix, b: &BitMatrix, op: usize) -> BitMatrix {
    let mut out = BitMatrix::new(a.height(), b.width());
    for i in 0..a.height() { for j in 0..b.width() {
        let mut value = op == 0;
        for k in 0..a.width() {
            let (x, y) = (a.get(i, k), b.get(k, j));
            match op {
                0 => value &= x | y,
                1 => value |= x & y,
                _ => value ^= x & y,
            }
        }
        out.set(i, j, value);
    }}
    out
}
fn prod(a: &BitMatrix, b: &BitMatrix, op: usize) -> BitMatrix {
    match op { 0 => a.prod_and(b), 1 => a.prod_or(b), _ => a.prod_xor(b) }
}
fn power(a: &BitMatrix, e: u64, op: usize) -> BitMatrix {
    match op { 0 => a.pow_and(e), 1 => a.pow_or(e), _ => a.pow_xor(e) }
}
fn logical_identity(n: usize, op: usize) -> BitMatrix {
    let mut out = BitMatrix::new(n, n);
    for i in 0..n { for j in 0..n { out.set(i, j, (i == j) ^ (op == 0)); }}
    out
}
#[test]
fn logical_products_and_powers() {
    for x in 0..16 {
        let mut a = BitMatrix::new(2, 2);
        for i in 0..2 { for j in 0..2 { a.set(i, j, x >> (i * 2 + j) & 1 != 0); }}
        for op in 0..3 {
            let identity = logical_identity(2, op);
            assert_eq!(prod(&a, &identity, op), a);
            assert_eq!(prod(&identity, &a, op), a);
            let mut expected = identity;
            for e in 0..=12 {
                assert_eq!(power(&a, e, op), expected);
                expected = logical_naive(&expected, &a, op);
            }
            for y in 0..16 {
                let mut b = BitMatrix::new(2, 2);
                for i in 0..2 { for j in 0..2 { b.set(i, j, y >> (i * 2 + j) & 1 != 0); }}
                assert_eq!(prod(&a, &b, op), logical_naive(&a, &b, op));
            }
        }
    }
    let mut seed = 13579;
    for n in [0, 1, 63, 64, 65] {
        let a = make(n, n, 5, &mut seed);
        for op in 0..3 {
            let mut expected = logical_identity(n, op);
            for e in 0..=3 {
                assert_eq!(power(&a, e, op), expected);
                expected = logical_naive(&expected, &a, op);
            }
        }
    }
}
#[test]
fn logical_empty_rectangular_and_large_exponent() {
    let mut seed = 97531;
    for k in [0, 1, 63, 64, 65] {
        let a = make(3, k, 5, &mut seed);
        let b = make(k, 5, 5, &mut seed);
        for op in 0..3 { assert_eq!(prod(&a, &b, op), logical_naive(&a, &b, op)); }
    }
    for (h, k, w) in [(0, 0, 3), (0, 2, 3), (3, 2, 0)] {
        for op in 0..3 {
            let out = prod(&BitMatrix::new(h, k), &BitMatrix::new(k, w), op);
            assert_eq!((out.height(), out.width()), (h, w));
        }
    }
    let mut cycle = BitMatrix::new(3, 3);
    for i in 0..3 { cycle.set(i, (i + 1) % 3, true); }
    for op in 1..3 { assert_eq!(power(&cycle, u64::MAX, op), logical_identity(3, op)); }
    let mut dual = cycle.clone();
    for i in 0..3 { for j in 0..3 { dual.set(i, j, !cycle.get(i, j)); }}
    assert_eq!(dual.pow_and(u64::MAX), logical_identity(3, 0));
    for op in 0..3 {
        assert!(std::panic::catch_unwind(|| power(&BitMatrix::new(2, 3), 0, op)).is_err());
        assert!(std::panic::catch_unwind(|| prod(&BitMatrix::new(2, 3), &BitMatrix::new(2, 3), op)).is_err());
    }
}
