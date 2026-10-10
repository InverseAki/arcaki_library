#![allow(dead_code)]
const MOD: i64 = 998244353;
include!("../src/NumberTheory/math.rs");
include!("../src/NumberTheory/ratio.rs");
include!("../src/NumberTheory/crt.rs");

#[path = "../src/NumberTheory/modcombination.rs"]
mod legacy_combination;

#[test]
fn shared_integer_division_signed_boundaries() {
    for a in -100i64..=100 {
        for b in -20i64..=20 {
            if b == 0 {
                continue;
            }
            let q = floor(a, b);
            let r = modulo(a, b);
            assert_eq!(q, a.div_euclid(b));
            assert_eq!(a as i128, b as i128 * q as i128 + r as i128);
            assert!(0 <= r && r < b.abs());
            assert_eq!(legacy_combination::floor(a, b), q);
            assert_eq!(legacy_combination::modulo(a, b), r);
            let ratio = Ratio::from_fraction(a, b);
            let (n, d) = (*ratio.numerator(), *ratio.denominator());
            assert_eq!(ratio.floor(), floor(n, d));
            assert_eq!(ratio.ceil(), -floor(-n, d));
        }
    }
    for a in [i64::MIN, i64::MIN + 1, -1, 0, 1, i64::MAX] {
        for b in [i64::MIN, -7, -1, 1, 7, i64::MAX] {
            if a == i64::MIN && b == -1 {
                assert!(std::panic::catch_unwind(|| floor(a, b)).is_err());
                assert!(std::panic::catch_unwind(|| legacy_combination::floor(a, b)).is_err());
            } else {
                let q = floor(a, b);
                let r = modulo(a, b);
                assert_eq!(a as i128, b as i128 * q as i128 + r as i128);
                assert!(r >= 0 && (r as u64) < b.unsigned_abs());
                assert_eq!(legacy_combination::floor(a, b), q);
            }
        }
    }
    assert!(std::panic::catch_unwind(|| floor(1i64, 0)).is_err());
    assert!(std::panic::catch_unwind(|| modulo(1i64, 0)).is_err());
    assert_eq!(modulo(i64::MIN, -1), 0);
}

#[test]
fn crt_matches_exhaustive_residues() {
    assert_eq!(crt(&vec![]), (0, 1));
    for m in 1..=15usize {
        for n in 1..=15usize {
            let lcm = m / gcd(m, n) * n;
            for a in 0..m {
                for b in 0..n {
                    let expected = (0..lcm).find(|&x| x % m == a && x % n == b);
                    let actual = crt(&vec![(a, m), (b, n)]);
                    assert_eq!(actual, expected.map_or((usize::MAX, usize::MAX), |r| (r, lcm)));
                }
            }
        }
    }
    assert_eq!(crt(&vec![(2, 3), (3, 5), (2, 7)]), (23, 105));
    for a in 0..=50i64 {
        for b in 0..=50 {
            assert_eq!(ext_gcd(a, b), extended_gcd(a, b));
            assert_eq!(legacy_combination::extended_gcd(a, b), extended_gcd(a, b));
        }
    }
}

#[test]
fn legacy_combinations_use_shared_math() {
    for a in -50..=50i64 {
        for m in [3i64, 5, 7, 11, 97] {
            if modulo(a, m) != 0 {
                assert_eq!(legacy_combination::mod_inverse(a, m), mod_inverse(a, m));
                assert_eq!(modulo(a * mod_inverse(a, m), m), 1);
            }
            for p in 0..=7 {
                assert_eq!(legacy_combination::fast_mod_pow(a, p, m), fast_mod_pow(a, p, m));
            }
        }
    }
    let f = legacy_combination::factorial(40);
    let mut row = vec![1i64];
    for n in 0..=40 {
        for k in 0..=n {
            assert_eq!(legacy_combination::comb(n as i64, k as i64, &f), row[k]);
        }
        let mut next = vec![1; row.len() + 1];
        for k in 1..row.len() {
            next[k] = (row[k - 1] + row[k]) % 1_000_000_007;
        }
        row = next;
    }
}
