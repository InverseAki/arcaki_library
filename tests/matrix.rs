#![allow(dead_code, unused_imports)]
include!("../src/Basic/matrix.rs");
mod doubling_compat {
    const MOD: i64 = 998244353;
    include!("../src/Basic/doubling_matrix.rs");
}
mod mint_compat {
    include!("../src/NumberTheory/mint.rs");
    type MI = Mint<998244353>;
    include!("../src/Basic/mint_matrix.rs");
    #[test]
    fn old_mint_interface_without_modulus_method() {
        for n in [0, 1, 3, 16, 17, 32, 65] {
            let values: Vec<_> = (0..n * n).map(|i| MI::new(i % 19)).collect();
            let mut a = Matrix::new(n, values.clone());
            if n > 0 {
                a[(0, 0)] = MI::new(12);
            }
            let c = a.mul(&Matrix::identity(n));
            assert_eq!(
                c.as_slice().iter().map(|x| x.val()).collect::<Vec<_>>(),
                a.as_slice().iter().map(|x| x.val()).collect::<Vec<_>>()
            );
            let d = a.mul(&a);
            for i in 0..n {
                for j in 0..n {
                    let expected = (0..n).fold(0usize, |s, k| {
                        (s + a[(i, k)].val() * a[(k, j)].val()) % 998244353
                    });
                    assert_eq!(d[(i, j)].val(), expected);
                }
            }
        }
        let a = Matrix::new(2, vec![MI::new(0), MI::new(1), MI::new(1), MI::new(3)]);
        let inv = a.inv();
        let prod = a.mul(&inv);
        assert_eq!(
            prod.as_slice().iter().map(|x| x.val()).collect::<Vec<_>>(),
            vec![1, 0, 0, 1]
        );
    }
}
fn next(seed: &mut u64) -> u64 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed
}
fn naive_mod(n: usize, a: &[u32], b: &[u32], m: u32) -> Vec<u32> {
    let mut out = vec![0; n * n];
    for i in 0..n {
        for j in 0..n {
            let mut sum = 0u128;
            for k in 0..n {
                sum += a[i * n + k] as u128 * b[k * n + j] as u128;
            }
            out[i * n + j] = (sum % m as u128) as u32;
        }
    }
    out
}
fn check_mod<const P: u32>() {
    let mut seed = 718293;
    for n in [0, 1, 2, 15, 16, 17, 18, 19, 31, 32, 33, 63, 64, 65] {
        for mode in 0..4 {
            let mut make = || {
                (0..n * n)
                    .map(|i| match mode {
                        0 => (next(&mut seed) % P as u64) as u32,
                        1 => P - 1,
                        2 => {
                            if i % 17 == 0 {
                                P - 1
                            } else {
                                0
                            }
                        }
                        _ => 0,
                    })
                    .collect::<Vec<_>>()
            };
            let a = make();
            let b = make();
            let expected = naive_mod(n, &a, &b, P);
            let lhs = SquareMatrix::<ModMatrixMonoid<P>>::new(n, a);
            let rhs = SquareMatrix::<ModMatrixMonoid<P>>::new(n, b);
            assert_eq!(
                lhs.prod(&rhs).as_slice(),
                expected,
                "P={P},n={n},mode={mode}"
            );
            let mut out = SquareMatrix::new(n, vec![P - 1; n * n]);
            lhs.mul_into(&rhs, &mut out);
            assert_eq!(out.as_slice(), expected);
            assert_eq!((&lhs * &rhs).as_slice(), expected);
        }
    }
}
#[test]
fn residues_limits_sparse_and_tile_boundaries() {
    check_mod::<1>();
    check_mod::<2>();
    check_mod::<17>();
    check_mod::<65537>();
    check_mod::<998244353>();
    check_mod::<1000000007>();
    check_mod::<2147483647>();
    check_mod::<4294967291>();
    check_mod::<{ u32::MAX }>();
    for n in [127, 128, 129] {
        let a = vec![998244352; n * n];
        let b = a.clone();
        let c = SquareMatrix::<ModMatrixMonoid<998244353>>::new(n, a).mul(&SquareMatrix::new(n, b));
        assert!(c.as_slice().iter().all(|&x| x == n as u32));
    }
}
#[test]
fn powers_inverse_and_constructor_compatibility() {
    type Mat = SquareMatrix<ModMatrixMonoid<97>>;
    for n in 0..=6 {
        let a: Vec<u32> = (0..n * n).map(|i| (i * 17 % 97) as u32).collect();
        let a = Mat::new(n, &a);
        let mut expected = Mat::identity(n);
        for e in 0..25 {
            assert_eq!(a.pow(e), expected);
            expected = expected.mul(&a);
        }
    }
    let a = Mat::new(2, [0, 1, 1, 3]);
    let b = a.try_inv().unwrap();
    assert_eq!(a.mul(&b), Mat::identity(2));
    assert_eq!(b.mul(&a), Mat::identity(2));
    assert!(Mat::zeros(3).try_inv().is_none());
    assert_eq!(Mat::zeros(0).inv(), Mat::identity(0));
    let mut seed = 1234;
    for n in 1..15 {
        let mut a = Mat::identity(n);
        // 可逆な行基本変形から構造的に可逆な行列を作る。
        for _ in 0..n * 12 {
            let i = (next(&mut seed) % n as u64) as usize;
            let j = (next(&mut seed) % n as u64) as usize;
            if i == j {
                continue;
            }
            let factor = (next(&mut seed) % 97) as u32;
            for k in 0..n {
                a[(i, k)] = (a[(i, k)] + factor * a[(j, k)]) % 97;
            }
        }
        let b = a.inv();
        assert_eq!(a.mul(&b), Mat::identity(n));
        assert_eq!(b.mul(&a), Mat::identity(n));
    }
    let values = vec![1i64, 1, 1, 0];
    let a = doubling_compat::DoublingMatrix::<doubling_compat::AddMulMonoid>::new(2, &values);
    assert_eq!(a.pow(10).get(0, 0), &89);
    assert_eq!(
        a.prod(&doubling_compat::DoublingMatrix::e(2)).as_slice(),
        values
    );
    let a = doubling_compat::DoublingMatrix::<doubling_compat::AddMulMonoid>::new(
        17,
        vec![998244352; 17 * 17],
    );
    assert!(a.prod(&a).as_slice().iter().all(|&x| x == 17));
}
#[test]
fn min_plus_and_boolean_semirings() {
    let inf = MinPlusMonoid::INF;
    assert_eq!(MinPlusMonoid::mul(&inf, &-7), inf);
    let a = SquareMatrix::<MinPlusMonoid>::new(3, vec![inf, -3, inf, inf, inf, 7, 2, inf, inf]);
    assert_eq!(
        a.pow(2).as_slice(),
        &[inf, inf, 4, 9, inf, inf, inf, -1, inf]
    );
    assert_eq!(a.mul(&SquareMatrix::e(3)), a);
    let b = SquareMatrix::<BoolMatrixMonoid>::new(
        3,
        [false, true, false, false, false, true, true, false, false],
    );
    assert_eq!(b.pow(3), SquareMatrix::identity(3));
    for n in [0, 1, 63, 64, 65] {
        let a = SquareMatrix::<MinPlusMonoid>::new(
            n,
            (0..n * n)
                .map(|i| if i % 7 == 0 { (i % 19) as i64 - 9 } else { inf })
                .collect::<Vec<_>>(),
        );
        let c = a.mul(&a);
        for i in 0..n {
            for j in 0..n {
                let expected = (0..n)
                    .filter(|&k| a[(i, k)] != inf && a[(k, j)] != inf)
                    .map(|k| a[(i, k)] + a[(k, j)])
                    .min()
                    .unwrap_or(inf);
                assert_eq!(c[(i, j)], expected);
            }
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Scalar(Vec<u32>);
struct Noncommutative;
impl MatrixMonoid for Noncommutative {
    type S = Scalar;
    fn zero() -> Scalar {
        Scalar(vec![0; 4])
    }
    fn one() -> Scalar {
        Scalar(vec![1, 0, 0, 1])
    }
    fn sum(a: &Scalar, b: &Scalar) -> Scalar {
        Scalar(a.0.iter().zip(&b.0).map(|(x, y)| (x + y) % 97).collect())
    }
    fn mul(a: &Scalar, b: &Scalar) -> Scalar {
        Scalar(vec![
            (a.0[0] * b.0[0] + a.0[1] * b.0[2]) % 97,
            (a.0[0] * b.0[1] + a.0[1] * b.0[3]) % 97,
            (a.0[2] * b.0[0] + a.0[3] * b.0[2]) % 97,
            (a.0[2] * b.0[1] + a.0[3] * b.0[3]) % 97,
        ])
    }
}
#[test]
fn generic_noncopy_noncommutative_multiplication() {
    let mut seed = 27289;
    for n in [0, 1, 2, 4, 64, 65] {
        let mut make = || {
            (0..n * n)
                .map(|_| Scalar((0..4).map(|_| (next(&mut seed) % 97) as u32).collect()))
                .collect::<Vec<_>>()
        };
        let a = make();
        let b = make();
        let c = SquareMatrix::<Noncommutative>::new(n, &a).mul(&SquareMatrix::new(n, &b));
        for i in 0..n {
            for j in 0..n {
                for x in 0..2 {
                    for y in 0..2 {
                        let mut sum = 0;
                        for k in 0..n {
                            for z in 0..2 {
                                sum = (sum + a[i * n + k].0[x * 2 + z] * b[k * n + j].0[z * 2 + y])
                                    % 97;
                            }
                        }
                        assert_eq!(c[(i, j)].0[x * 2 + y], sum);
                    }
                }
            }
        }
    }
}
#[test]
fn invalid_shapes_indices_and_modulus_are_detected() {
    use std::panic::{catch_unwind, AssertUnwindSafe};
    type Mat = SquareMatrix<ModMatrixMonoid<97>>;
    assert!(catch_unwind(|| Mat::new(2, vec![0; 3])).is_err());
    assert!(catch_unwind(|| Mat::zeros(usize::MAX)).is_err());
    let mut a = Mat::identity(2);
    assert!(catch_unwind(|| a.mul(&Mat::identity(3))).is_err());
    assert!(catch_unwind(|| a[(0, 2)]).is_err());
    assert!(catch_unwind(AssertUnwindSafe(|| {
        a[(2, 0)] = 0;
    }))
    .is_err());
    assert!(catch_unwind(|| SquareMatrix::<ModMatrixMonoid<4>>::identity(2).inv()).is_err());
    assert!(catch_unwind(|| Mat::new(1, [97]).mul(&Mat::identity(1))).is_err());
    assert_eq!(a, Mat::identity(2));
}

mod large_legacy_mod {
    const MOD: i64 = i64::MAX;
    include!("../src/Basic/doubling_matrix.rs");
    #[test]
    fn large_modulus_fallback_does_not_overflow() {
        for n in [0, 1, 2, 17] {
            let a = DoublingMatrix::<AddMulMonoid>::new(n, vec![MOD - 1; n * n]);
            let c = a.prod(&a);
            assert!(c.as_slice().iter().all(|&x| x == n as i64));
        }
    }
}
