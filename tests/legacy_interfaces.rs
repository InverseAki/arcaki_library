#![allow(dead_code, unused_imports)]
mod math {
    const MOD: i64 = 998244353;
    include!("../src/Basic/math.rs");
}
mod legacy_mint {
    include!("../src/NumberTheory/mint.rs");
    type MI = Mint<998244353>;
    include!("../src/NumberTheory/mint_combination.rs");
    fn convolution(a: &[MI], b: &[MI]) -> Vec<MI> {
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
    fn original_mint_and_helper_location() {
        let c = MintCombination::new(20);
        assert_eq!(c.c(5, 2).val(), 10);
        assert_eq!(c.p(5, 2).val(), 20);
        assert_eq!((c.f(20) * c.fi(20)).val(), 1);
        assert_eq!((c.inv(7) * 7usize).val(), 1);
        assert_eq!(MintCombination::new(0).c(0, 0).val(), 1);
        let mut vs = vec![vec![MI::new(1), MI::new(2)], vec![MI::new(3), MI::new(4)]];
        assert_eq!(
            convolution_merge(&mut vs)
                .iter()
                .map(|x| x.val())
                .collect::<Vec<_>>(),
            vec![3, 10, 8]
        );
        assert!(vs.iter().all(Vec::is_empty));
        assert_eq!(convolution_merge(&mut vec![])[0].val(), 1);
    }
}
#[test]
fn original_math_functions_and_signatures() {
    let gcd: fn(i64, i64) -> i64 = math::gcd;
    let floor: fn(i64, i64) -> i64 = math::floor;
    let modulo: fn(i64, i64) -> i64 = math::modulo;
    let egcd: fn(i64, i64) -> (i64, i64, i64) = math::extended_gcd;
    assert_eq!((gcd(24, 18), floor(-7, 3), modulo(-7, 3)), (6, -3, 2));
    let (g, x, y) = egcd(24, 18);
    assert_eq!(24 * x + 18 * y, g);
    for m in [7i64, 12, 97, 998244353] {
        for a in -100i64..=100 {
            if gcd(a.rem_euclid(m), m) == 1 {
                let inv = math::mod_inverse(a, m);
                assert!(0 <= inv && inv < m);
                assert_eq!((a as i128 * inv as i128).rem_euclid(m as i128), 1);
            }
        }
    }
    let (fact, inv_fact) = math::factorial_i64(30);
    let table = math::factorial(30);
    for n in 0..=30 {
        assert_eq!(table[n], (fact[n], inv_fact[n]));
        assert_eq!(fact[n] * inv_fact[n] % 998244353, 1);
    }
    assert_eq!(math::comb(5, 2, &table), 10);
    assert_eq!(math::comb(3, 4, &table), 0);
    assert_eq!(math::fast_mod_pow(3, 10, 97), 73);
}
