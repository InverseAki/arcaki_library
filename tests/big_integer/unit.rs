#[path = "../../src/NumberTheory/big_integer.rs"]
mod big_integer;
use big_integer::{BigInt, HexBigInt};
#[test]
fn parsing_and_canonical_zero() {
    for s in ["0", "-0", "+000000", "-00000"] {
        let a: BigInt = s.parse().unwrap();
        assert_eq!(a.to_string(), "0");
        assert_eq!(a, BigInt::zero());
    }
    for s in ["", "+", "-", " 1", "1 ", "1_0", "0xAB", "A", "１２"] {
        assert!(s.parse::<BigInt>().is_err(), "{s}");
    }
    assert_eq!("-00aBcD".parse::<HexBigInt>().unwrap().to_string(), "-ABCD");
    assert_eq!("+00123".parse::<BigInt>().unwrap(), BigInt::from(123));
}
#[test]
fn conversions_and_ordering() {
    for x in [i128::MIN, i128::MAX, -1, 0, 1, 10000, -10000] {
        let a = BigInt::from(x);
        assert_eq!(a.to_i128(), Some(x));
        assert_eq!(a.to_string(), x.to_string());
        let h = HexBigInt::from(x);
        assert_eq!(h.to_i128(), Some(x));
    }
    let a = BigInt::from(u128::MAX);
    assert_eq!(a.to_u128(), Some(u128::MAX));
    assert_eq!(a.to_i128(), None);
    let bigger = &a + &BigInt::one();
    assert_eq!(bigger.to_u128(), None);
    assert_eq!(BigInt::from(-1).to_u128(), None);
    let mut v = vec![10001, -10001, 0, -1, 1, 9999, -9999];
    let mut b: Vec<_> = v.iter().map(|&x| BigInt::from(x)).collect();
    v.sort();
    b.sort();
    assert_eq!(
        b.iter().map(|x| x.to_i128().unwrap()).collect::<Vec<_>>(),
        v.iter().map(|&x| x as i128).collect::<Vec<_>>()
    );
}
#[test]
fn signs_division_and_operators() {
    for a in -100..=100 {
        for b in -100..=100 {
            let x = BigInt::from(a);
            let y = BigInt::from(b);
            assert_eq!((&x + &y).to_i128(), Some((a + b) as i128));
            assert_eq!((&x - &y).to_i128(), Some((a - b) as i128));
            assert_eq!((&x * &y).to_i128(), Some((a * b) as i128));
            if b != 0 {
                let (q, r) = x.div_rem(&y);
                assert_eq!(q.to_i128(), Some((a as i128).div_euclid(b as i128)));
                assert_eq!(r.to_i128(), Some((a as i128).rem_euclid(b as i128)));
                let (q, r) = x.div_rem_euclid(&y);
                assert_eq!((&q * &y + &r), x);
                assert!(r >= BigInt::zero() && r < y.abs());
            } else {
                assert!(x.checked_div_rem(&y).is_none());
            }
        }
    }
    let mut x = BigInt::from(17);
    x += BigInt::from(3);
    x -= BigInt::from(1);
    x *= BigInt::from(4);
    x /= BigInt::from(3);
    x %= BigInt::from(7);
    assert_eq!(x, BigInt::from(4));
    assert_eq!(BigInt::from(-3).pow(5), BigInt::from(-243));
    assert_eq!(BigInt::zero().pow(0), BigInt::one());
}
#[test]
#[should_panic(expected = "division by zero")]
fn zero_divisor_panics() {
    BigInt::one().div_rem(&BigInt::zero());
}
#[test]
fn ntt_square_and_full_carry() {
    for n in [197, 256, 257, 1024, 4097] {
        let a: BigInt = "9".repeat(n).parse().unwrap();
        let expected = "9".repeat(n - 1) + "8" + &"0".repeat(n - 1) + "1";
        assert_eq!((&a * &a).to_string(), expected);
        assert_eq!(a.pow(2).to_string(), expected);
        let h: HexBigInt = "F".repeat(n).parse().unwrap();
        let expected = "F".repeat(n - 1) + "E" + &"0".repeat(n - 1) + "1";
        assert_eq!((&h * &h).to_string(), expected);
    }
}

fn check_euclid<const B: u32>() {
    use big_integer::RadixBigInt;
    type Ordering = std::cmp::Ordering;
    for a in -35i128..=35 {
        for b in -20i128..=20 {
            let x = RadixBigInt::<B>::from(a);
            let y = RadixBigInt::<B>::from(b);
            if b == 0 {
                assert!(x.checked_div_rem(&y).is_none());
                continue;
            }
            let (q, r) = x.div_rem(&y);
            assert_eq!(q.to_i128(), Some(a.div_euclid(b)));
            assert_eq!(r.to_i128(), Some(a.rem_euclid(b)));
            assert_eq!(&q * &y + &r, x);
            assert!(!r.is_negative() && r < y.abs());
            assert_eq!(x.div_rem_euclid(&y), (q.clone(), r.clone()));
            assert_eq!(x.checked_div_rem(&y), Some((q.clone(), r.clone())));
            assert_eq!(&x / &y, q);
            assert_eq!(&x % &y, r);
            assert_eq!(x.clone() / y.clone(), q);
            assert_eq!(x.clone() % y.clone(), r);
            assert_eq!(x.clone() / &y, q);
            assert_eq!(&x / y.clone(), q);
            assert_eq!(x.clone() % &y, r);
            assert_eq!(&x % y.clone(), r);
            let mut quotient = x.clone();
            quotient /= &y;
            assert_eq!(quotient, q);
            let mut remainder = x.clone();
            remainder %= &y;
            assert_eq!(remainder, r);
            quotient = x.clone();
            quotient /= y.clone();
            assert_eq!(quotient, q);
            remainder = x.clone();
            remainder %= y.clone();
            assert_eq!(remainder, r);
            assert_eq!(r.cmp(&RadixBigInt::zero()) == Ordering::Less, false);
        }
    }
    // 大きな負数、割り切れる値、剰余1・d-1、絶対値が除数未満。
    let d = RadixBigInt::<B>::from(10).pow(1200) + RadixBigInt::from(37);
    let q = RadixBigInt::<B>::from(10).pow(1800) + RadixBigInt::from(11);
    for r in [
        RadixBigInt::zero(),
        RadixBigInt::one(),
        &d - RadixBigInt::one(),
    ] {
        let x = -(&q * &d + &r);
        for y in [d.clone(), -d.clone()] {
            let (quotient, remainder) = x.div_rem(&y);
            assert_eq!(&quotient * &y + &remainder, x);
            assert!(!remainder.is_negative() && remainder < d);
            let expected = if r.is_zero() { r.clone() } else { &d - &r };
            assert_eq!(remainder, expected);
        }
    }
    for y in [d.clone(), -d.clone()] {
        let x = -RadixBigInt::one();
        let (q, r) = x.div_rem(&y);
        assert_eq!(q, RadixBigInt::from(if y.is_negative() { 1 } else { -1 }));
        assert_eq!(r, &d - RadixBigInt::one());
    }
}
#[test]
fn euclidean_division_all_entrypoints_and_radices() {
    check_euclid::<10000>();
    check_euclid::<65536>();
}

#[test]
fn fast_ascii_append_preserves_utf8_and_formatting() {
    for x in [
        i128::MIN,
        -10001,
        -10000,
        -9999,
        -1,
        0,
        1,
        99,
        100,
        9999,
        10000,
        10001,
        i128::MAX,
    ] {
        let a = BigInt::from(x);
        let mut out = String::from("前置き😀:");
        a.append_to(&mut out);
        assert_eq!(out, format!("前置き😀:{}", x));
        assert_eq!(format!("{:>45}", a), format!("{:>45}", x));
        let h = HexBigInt::from(x);
        let mut out = String::from("前置き😀:");
        h.append_to(&mut out);
        let expected = if x < 0 {
            format!("-{:X}", x.unsigned_abs())
        } else {
            format!("{:X}", x)
        };
        assert_eq!(out, format!("前置き😀:{}", expected));
    }
    for s in [
        "1",
        "123",
        "1234",
        "10000",
        "99999999999999999999999999999999",
    ] {
        let a: BigInt = s.parse().unwrap();
        let mut out = String::new();
        a.append_to(&mut out);
        assert_eq!(out, s);
        assert_eq!(a.limb_len(), (s.len() + 3) / 4);
    }
}

#[test]
fn crt_reuse_at_exact_ntt_capacity_boundary() {
    for (n, m) in [(49usize, 80usize), (65, 64), (129, 128), (257, 256)] {
        let (n, m) = (n.max(m) * 4, n.min(m) * 4);
        let a: BigInt = "9".repeat(n).parse().unwrap();
        let b: BigInt = "9".repeat(m).parse().unwrap();
        let expected = "9".repeat(m - 1) + "8" + &"9".repeat(n - m) + &"0".repeat(m - 1) + "1";
        assert_eq!((&a * &b).to_string(), expected);
        let a: HexBigInt = "F".repeat(n).parse().unwrap();
        let b: HexBigInt = "F".repeat(m).parse().unwrap();
        let expected = "F".repeat(m - 1) + "E" + &"F".repeat(n - m) + &"0".repeat(m - 1) + "1";
        assert_eq!((&a * &b).to_string(), expected);
    }
}

#[test]
fn shifted_single_limb_divisor_and_equal_values() {
    fn check<const B: u32>() {
        use big_integer::RadixBigInt;
        let base = RadixBigInt::<B>::from(B);
        let q = base.pow(57) + RadixBigInt::from(7);
        for shift in [1, 2, 3, 31, 32, 33, 128, 513] {
            for digit in [1, B / 2, B - 1] {
                let d = base.pow(shift) * RadixBigInt::from(digit);
                assert_eq!(d.div_rem(&d), (RadixBigInt::one(), RadixBigInt::zero()));
                for r in [
                    RadixBigInt::zero(),
                    RadixBigInt::one(),
                    &d - RadixBigInt::one(),
                ] {
                    let x = &d * &q + &r;
                    assert_eq!(x.div_rem(&d), (q.clone(), r.clone()));
                    let (nq, nr) = (-x.clone()).div_rem(&d);
                    assert_eq!(&nq * &d + &nr, -x);
                    assert!(nr >= RadixBigInt::zero() && nr < d);
                }
            }
        }
    }
    check::<10000>();
    check::<65536>();
}
