#![allow(dead_code)]
include!("../src/NumberTheory/ratio.rs");
include!("../src/NumberTheory/big_integer.rs");
include!("../src/NumberTheory/big_ratio.rs");
use std::collections::{BTreeSet, HashSet};

fn small(n: i128, d: i128) -> (i128, i128) {
    let g = ratio_detail::gcd(n.unsigned_abs(), d.unsigned_abs()) as i128;
    let (n, d) = (n / g, d / g);
    if d < 0 {
        (-n, -d)
    } else {
        (n, d)
    }
}
#[test]
fn finite_arithmetic_exhaustive() {
    let mut values = Vec::new();
    for d in -7..=7 {
        if d == 0 {
            continue;
        }
        for n in -7..=7 {
            values.push(Ratio::new(d, n));
        }
    }
    for a in &values {
        for b in &values {
            let (n, d, m, e) = (
                *a.numerator() as i128,
                *a.denominator() as i128,
                *b.numerator() as i128,
                *b.denominator() as i128,
            );
            assert_eq!(a.cmp(b), (n * e).cmp(&(m * d)));
            for (actual, expected) in [
                (a + b, small(n * e + m * d, d * e)),
                (a - b, small(n * e - m * d, d * e)),
                (a * b, small(n * m, d * e)),
            ] {
                assert_eq!(
                    (*actual.numerator() as i128, *actual.denominator() as i128),
                    expected
                );
            }
            if m != 0 {
                let c = a / b;
                assert_eq!(
                    (*c.numerator() as i128, *c.denominator() as i128),
                    small(n * e, d * m)
                );
            }
            let aa = Ratio128::from(*a);
            let bb = Ratio128::from(*b);
            assert_eq!(Ratio::try_from(&aa + &bb).unwrap(), a + b);
            assert_eq!(Ratio::try_from(&aa - &bb).unwrap(), a - b);
            assert_eq!(Ratio::try_from(&aa * &bb).unwrap(), a * b);
        }
    }
}
#[test]
fn signed_boundaries_cancellation_and_overflow() {
    for (min, max) in [(i64::MIN as i128, i64::MAX as i128), (i128::MIN, i128::MAX)] {
        assert_eq!(Ratio128::new(min, min), 1);
        assert_eq!(Ratio128::new(min, 0), 0);
        assert_eq!(Ratio128::new(min, 2), Ratio128::from_fraction(-1, min / -2));
        let a = Ratio128::int(min);
        assert_eq!(a.try_sub(&a), Ok(Ratio128::zero()));
        assert_eq!(a.try_div(&a), Ok(Ratio128::one()));
        assert_eq!(Ratio128::zero().try_div(&a), Ok(Ratio128::zero()));
        assert_eq!(
            a.try_neg(),
            if min == i128::MIN {
                Err(RatioError::Overflow)
            } else {
                Ok(Ratio128::int(-min))
            }
        );
        let p = Ratio128::from_fraction(max, max - 1);
        let q = Ratio128::from_fraction(max - 1, max);
        assert_eq!(&p * &q, 1);
        assert!(p > q);
    }
    assert_eq!(Ratio::try_new(i64::MIN, 1), Err(RatioError::Overflow));
    assert_eq!(Ratio128::try_new(i128::MIN, 1), Err(RatioError::Overflow));
    assert_eq!(
        Ratio::int(i64::MAX).try_add(&Ratio::one()),
        Err(RatioError::Overflow)
    );
    assert_eq!(
        Ratio128::int(i128::MAX).try_add(&Ratio128::one()),
        Err(RatioError::Overflow)
    );
    assert_eq!(
        Ratio128::int(i128::MIN).try_div(&Ratio128::int(-1)),
        Err(RatioError::Overflow)
    );
    let n = Ratio::from_fraction(1, i64::MAX);
    assert_eq!(n.try_mul(&n), Err(RatioError::Overflow));
    let g = BigInt::from(1u128 << 64);
    let a = BigInt::from((1u128 << 62) + 1);
    let b = BigInt::from((1u128 << 62) + 3);
    let x = &g * &a;
    let u = &g * &b;
    let n = &x - BigInt::one();
    let residue = (&b * BigInt::from((1u128 << 64) - (1u128 << 62) + 1)) % &g;
    let m = &u - &g + residue;
    let aa = Ratio128::new(x.to_i128().unwrap(), n.to_i128().unwrap());
    let bb = Ratio128::new(u.to_i128().unwrap(), m.to_i128().unwrap());
    let expected = &BigRatio::from(aa) + &BigRatio::from(bb);
    assert_eq!(
        aa.try_add(&bb).unwrap(),
        Ratio128::try_from(expected).unwrap()
    );
    assert!(std::panic::catch_unwind(|| Ratio::int(i64::MAX) + 1).is_err());
    assert!(std::panic::catch_unwind(|| -Ratio::int(i64::MIN)).is_err());
}
#[test]
fn infinities_equality_order_and_hash() {
    let p = Ratio::infinity();
    let n = Ratio::negative_infinity();
    let z = Ratio::zero();
    assert_eq!(Ratio::new(0, 17), p);
    assert_eq!(Ratio::new(0, -17), n);
    assert_ne!(p, n);
    assert_eq!(Ratio::try_new(0, 0), Err(RatioError::Indeterminate));
    assert_eq!(p.try_add(&n), Err(RatioError::Indeterminate));
    assert_eq!(p.try_sub(&p), Err(RatioError::Indeterminate));
    assert_eq!(z.try_mul(&p), Err(RatioError::Indeterminate));
    assert_eq!(p.try_div(&p), Err(RatioError::Indeterminate));
    assert_eq!(z.try_div(&z), Err(RatioError::Indeterminate));
    assert_eq!(p + 5, p);
    assert_eq!(5 - p, n);
    assert_eq!(p * -2, n);
    assert_eq!(p / 0, p);
    assert_eq!(5 / z, p);
    assert_eq!(-5 / z, n);
    assert_eq!(Ratio::int(7) / n, z);
    assert_eq!(n.inv(), z);
    assert_eq!(z.inv(), p);
    assert_eq!(p.try_floor(), Err(RatioError::NotFinite));
    let values = [
        n,
        Ratio::int(i64::MIN),
        Ratio::new(6, 3),
        Ratio::new(2, 1),
        p,
        Ratio::new(0, 2),
    ];
    assert_eq!(values.into_iter().collect::<HashSet<_>>().len(), 4);
    assert_eq!(values.into_iter().collect::<BTreeSet<_>>().len(), 4);
    for a in values {
        for b in values {
            assert_eq!(a == b, a.cmp(&b) == std::cmp::Ordering::Equal);
            assert_eq!(a.cmp(&b), b.cmp(&a).reverse());
        }
    }
    assert!(n < Ratio::int(i64::MIN));
    assert!(p > Ratio::int(i64::MAX));
}
#[test]
fn convenience_parsing_rounding_and_conversion() {
    assert_eq!(Ratio::new(2, 1), Ratio::from_fraction(1, 2));
    assert_eq!(Ratio::from(7), 7);
    assert_eq!(Ratio::default(), 0);
    let mut a: Ratio = " -7 / 3 ".parse().unwrap();
    assert_eq!((a.floor(), a.ceil(), a.trunc()), (-3, -2, -2));
    a += 3;
    a *= 2;
    a -= 1;
    a /= 5;
    assert_eq!(a, Ratio::from_fraction(1, 15));
    assert_eq!(2 + a, a + 2);
    assert_eq!(2 - a, Ratio::int(2) - a);
    assert_eq!(2 * a, a * 2);
    assert_eq!(2 / a, Ratio::int(2) / a);
    for s in ["0", "-42", "7/3", "-7/3", "inf", "-inf"] {
        let a: Ratio = s.parse().unwrap();
        assert_eq!(a.to_string().parse::<Ratio>().unwrap(), a);
    }
    for s in ["", "1//2", "hello", "3/", "0/0"] {
        assert!(s.parse::<Ratio>().is_err());
    }
    for n in -30..=30 {
        for d in 1..=10 {
            let a = Ratio::new(d, n);
            assert_eq!(a.floor(), n.div_euclid(d));
            assert_eq!(a.ceil(), -(-n).div_euclid(d));
            assert_eq!(a.trunc(), n / d);
        }
    }
    let a = Ratio128::int(i128::MIN);
    let b = BigRatio::from(a);
    assert_eq!(Ratio128::try_from(b).unwrap(), a);
    assert_eq!(
        Ratio::try_from(Ratio128::int(i128::MAX)),
        Err(RatioError::Overflow)
    );
    assert_eq!(Ratio::from_fraction(1, 2).to_f64(), 0.5);
    assert_eq!(Ratio128::infinity().to_f64(), f64::INFINITY);
    assert_eq!(Ratio::new(3, -7).pow(3), Ratio::from_fraction(-343, 27));
    assert_eq!(Ratio::zero().pow(0), 1);
}
#[test]
fn big_ratio_arithmetic_and_native_integer_usage() {
    for n in -25..=25 {
        for d in 1..=10 {
            let a = BigRatio::from_fraction(n, d);
            let b = BigRatio::from_fraction(7, 13);
            assert_eq!(
                Ratio::try_from(&a + &b).unwrap(),
                Ratio::new(d, n) + Ratio::new(13, 7)
            );
            assert_eq!(
                Ratio::try_from(&a - &b).unwrap(),
                Ratio::new(d, n) - Ratio::new(13, 7)
            );
            assert_eq!(
                Ratio::try_from(&a * &b).unwrap(),
                Ratio::new(d, n) * Ratio::new(13, 7)
            );
            assert_eq!(
                Ratio::try_from(&a / &b).unwrap(),
                Ratio::new(d, n) / Ratio::new(13, 7)
            );
            assert_eq!(a.floor().to_i128(), Some((n as i128).div_euclid(d as i128)));
            assert_eq!(
                a.ceil().to_i128(),
                Some(-(-(n as i128)).div_euclid(d as i128))
            );
            assert_eq!(a.trunc().to_i128(), Some((n / d) as i128));
        }
    }
    let mut a: BigRatio = "-15/6".parse().unwrap();
    let two = BigInt::from(2);
    assert_eq!(&a + &two, BigRatio::from_fraction(-1, 2));
    assert_eq!(&two / &a, BigRatio::from_fraction(-4, 5));
    a += 2i64;
    a *= &two;
    a -= BigInt::from(3);
    a /= 4u32;
    assert_eq!(a, -1);
    assert_eq!(BigRatio::from(5i64), 5);
    assert!(2 < a + 4);
    assert!(BigRatio::from_fraction(1, 2) < two);
    let h = BigInt::from(10).pow(3000) + BigInt::one();
    let a = BigRatio::from_fraction(&h + BigInt::one(), h.clone());
    let b = BigRatio::from_fraction(h.clone(), &h + BigInt::one());
    assert_eq!(&a * &b, 1);
    assert_eq!(&a / &a, 1);
    assert_eq!(a.to_string().parse::<BigRatio>().unwrap(), a);
    assert!(a > BigRatio::one());
    assert!(b < BigRatio::one());
    let hex: Rational<HexBigInt> = "A/6".parse().unwrap();
    assert_eq!(hex.to_string(), "5/3");
}
#[test]
fn bigint_gcd_signs_zeros_and_structured_cases() {
    for a in -100..=100 {
        for b in -100..=100 {
            let expected =
                ratio_detail::gcd((a as i128).unsigned_abs(), (b as i128).unsigned_abs());
            assert_eq!(
                BigInt::from(a).gcd(&BigInt::from(b)),
                BigInt::from(expected)
            );
            assert_eq!(
                HexBigInt::from(a).gcd(&HexBigInt::from(b)),
                HexBigInt::from(expected)
            );
        }
    }
    let factor = BigInt::from(10).pow(3000) - BigInt::one();
    assert_eq!(
        (&factor * BigInt::from(37)).gcd(&(-&factor * BigInt::from(101))),
        factor
    );
    assert_eq!(factor.gcd(&BigInt::zero()), factor);
    assert_eq!(BigInt::zero().gcd(&BigInt::zero()), BigInt::zero());
    let (mut a, mut b) = (BigInt::one(), BigInt::one());
    for _ in 0..2000 {
        let c = &a + &b;
        a = b;
        b = c;
    }
    assert_eq!(a.gcd(&b), BigInt::one());
    assert_eq!((a * &factor).gcd(&(b * &factor)), factor);
}

#[test]
fn iterator_reductions_and_wide_division() {
    let values = [Ratio::from_fraction(1, 2), Ratio::from_fraction(1, 3)];
    assert_eq!(values.iter().sum::<Ratio>(), Ratio::from_fraction(5, 6));
    assert_eq!(
        values.into_iter().product::<Ratio>(),
        Ratio::from_fraction(1, 6)
    );
    assert_eq!(std::iter::empty::<Ratio>().sum::<Ratio>(), 0);
    assert_eq!(std::iter::empty::<Ratio>().product::<Ratio>(), 1);
    let big = [BigRatio::from_fraction(1, 2), BigRatio::from_fraction(1, 3)];
    assert_eq!(big.iter().sum::<BigRatio>(), BigRatio::from_fraction(5, 6));
    assert_eq!(
        big.into_iter().product::<BigRatio>(),
        BigRatio::from_fraction(1, 6)
    );
    let a = Ratio::from_fraction(1, 2);
    assert_eq!(2 + &a, Ratio::from_fraction(5, 2));
    assert_eq!(2 - &a, Ratio::from_fraction(3, 2));
    assert_eq!(2 * &a, 1);
    assert_eq!(2 / &a, 4);
    let mut seed = 761271891u128;
    let edges = [
        1,
        2,
        u64::MAX as u128,
        1u128 << 64,
        i128::MAX as u128,
        1u128 << 127,
        u128::MAX,
    ];
    for i in 0..700 {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        let a = seed;
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        let b = seed;
        let wide = ratio_detail::Wide::mul(a, b);
        let reference = BigInt::from(a) * BigInt::from(b);
        for d in [edges[i % edges.len()], a.max(b).max(1), u128::MAX] {
            let (q, r) = wide.div_rem(d);
            let (eq, er) = reference.div_rem(&BigInt::from(d));
            assert_eq!(q.to_u128(), eq.to_u128());
            assert_eq!(r, er.to_u128().unwrap());
        }
    }
}
