// run_math_migration.py からテンプレートの実物を結合して実行する。
#[test]
fn integer_math_contracts() {
    for a in -100i64..=100 {
        for b in -20i64..=20 {
            if b == 0 {
                continue;
            }
            let (q, r) = (math::floor(a, b), math::modulo(a, b));
            assert_eq!(a, b * q + r);
            assert!(0 <= r && r < b.abs());
        }
    }
    assert_eq!(math::modulo(i64::MIN, -1), 0);
    assert_eq!(math::floor(i64::MIN, 3), -3074457345618258603);
    assert_eq!(math::modulo(i64::MAX, i64::MIN), i64::MAX);
    assert!(std::panic::catch_unwind(|| math::floor(i64::MIN, -1)).is_err());
    for a in 0..100i64 {
        for b in 0..100i64 {
            let (g, x, y) = math::extended_gcd(a, b);
            assert_eq!(a * x + b * y, g);
            assert_eq!(math::gcd(a, b), g);
            if g > 0 {
                assert_eq!((a % g, b % g), (0, 0));
            }
        }
    }
    for (a, b) in [(i64::MAX, i64::MAX - 2), (0, i64::MAX), (i64::MAX, 0)] {
        let (g, x, y) = math::extended_gcd(a, b);
        assert_eq!(a as i128 * x as i128 + b as i128 * y as i128, g as i128);
    }
}
#[test]
fn runtime_mod_against_wide_arithmetic() {
    let mut seed = 918273u64;
    for m in [2u32, 3, 4, 6, 97, 998244353, 2147483647, u32::MAX] {
        let bt = barrett::Barrett32::new(m);
        for _ in 0..5000 {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            let a = seed as u32;
            let b = (seed >> 32) as u32;
            assert_eq!(bt.mul(a, b), (a as u128 * b as u128 % m as u128) as u32);
            let e = b % 40;
            let expected = (0..e).fold(1u128, |v, _| v * a as u128 % m as u128);
            assert_eq!(bt.pow(a, e as u64), expected as u32);
            match bt.try_inv(a) {
                Some(i) => assert_eq!(bt.mul(a, i), 1),
                None => assert_ne!(math::gcd(a as i64, m as i64), 1),
            }
        }
        assert_eq!(bt.pow(0, 0), 1);
        assert_eq!(bt.try_inv(0), None);
    }
    assert_eq!(barrett::Barrett32::new(12).inv(5), 5);
    assert!(std::panic::catch_unwind(|| barrett::Barrett32::new(12).inv(6)).is_err());
}
#[test]
fn combinations_against_pascal() {
    let c = MintCombination::new(50);
    let mut row = vec![1u64];
    for n in 0..=50 {
        for k in 0..=n {
            assert_eq!(c.c(n, k).val() as u64, row[k]);
        }
        assert_eq!(c.c(n, n + 1).val(), 0);
        let mut next = vec![1; row.len() + 1];
        for k in 1..row.len() {
            next[k] = (row[k - 1] + row[k]) % MI::modulus() as u64;
        }
        row = next;
    }
    assert_eq!(MintCombination::new(0).c(0, 0).val(), 1);
    assert!(std::panic::catch_unwind(|| c.inv(0)).is_err());
    for i in 1..=50 {
        assert_eq!((c.inv(i) * i).val(), 1);
    }
}
fn naive_product(a: &[MI], b: &[MI]) -> Vec<MI> {
    if a.is_empty() || b.is_empty() {
        return vec![];
    }
    let mut c = vec![MI::new(0); a.len() + b.len() - 1];
    for (i, &x) in a.iter().enumerate() {
        for (j, &y) in b.iter().enumerate() {
            c[i + j] += x * y;
        }
    }
    c
}
#[test]
fn polynomial_products_and_truncation() {
    let mut seed = 1729u64;
    for n in 0..12 {
        for trial in 0..40 {
            let mut vs = Vec::new();
            for _ in 0..n {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                let len = if trial == 0 {
                    0
                } else {
                    (seed % 7 + 1) as usize
                };
                vs.push(
                    (0..len)
                        .map(|i| MI::new(((seed >> i) % 100) as u32))
                        .collect::<Vec<_>>(),
                );
            }
            let expected = vs
                .iter()
                .fold(vec![MI::new(1)], |a, b| naive_product(&a, b));
            let vals = |a: Vec<MI>| a.into_iter().map(|x| x.val()).collect::<Vec<_>>();
            assert_eq!(
                vals(convolution_merge(&mut vs.clone())),
                vals(expected.clone())
            );
            for mx in 0..15 {
                let mut e = expected.clone();
                e.truncate(mx);
                assert_eq!(
                    vals(convolution_merge_with_mx(&mut vs.clone(), mx)),
                    vals(e)
                );
            }
        }
    }
}
