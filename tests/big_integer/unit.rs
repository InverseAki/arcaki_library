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
                assert_eq!(q.to_i128(), Some((a / b) as i128));
                assert_eq!(r.to_i128(), Some((a % b) as i128));
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
