#[path = "../Fps/convolution_u64.rs"]
mod implementation;
use implementation::convolution_u64;
fn naive(a: &[u64], b: &[u64]) -> Vec<u64> {
    if a.is_empty() || b.is_empty() {
        return vec![];
    }
    let mut c = vec![0u64; a.len() + b.len() - 1];
    for (i, &x) in a.iter().enumerate() {
        for (j, &y) in b.iter().enumerate() {
            c[i + j] = c[i + j].wrapping_add(x.wrapping_mul(y));
        }
    }
    c
}
#[test]
fn random_full_width_and_thresholds() {
    let mut seed = 73919u64;
    for (n, m) in [
        (0, 0),
        (0, 64),
        (64, 0),
        (1, 1),
        (1, 70),
        (60, 100),
        (61, 61),
        (64, 65),
        (127, 129),
        (255, 258),
        (512, 513),
    ] {
        for trial in 0..12 {
            let mut make = |len| {
                (0..len)
                    .map(|_| {
                        seed ^= seed << 13;
                        seed ^= seed >> 7;
                        seed ^= seed << 17;
                        match trial % 3 {
                            0 => seed,
                            1 => seed & 0xffffffff,
                            _ => seed % 1000,
                        }
                    })
                    .collect::<Vec<_>>()
            };
            let a = make(n);
            let b = make(m);
            assert_eq!(
                convolution_u64(&a, &b),
                naive(&a, &b),
                "n={n},m={m},trial={trial}"
            );
        }
    }
}
#[test]
fn full_width_patterns() {
    // 真の整数係数はu128にも収まらないが、(-1)*(-1)=1 mod 2^64。
    let n = 4097;
    let m = 3000;
    let a = vec![u64::MAX; n];
    let b = vec![u64::MAX; m];
    let c = convolution_u64(&a, &b);
    for (i, &v) in c.iter().enumerate() {
        assert_eq!(v, (i + 1).min(n).min(m).min(n + m - 1 - i) as u64);
    }
    let a = vec![1u64 << 63; 91];
    let b = vec![1u64 << 63; 113];
    assert!(convolution_u64(&a, &b).iter().all(|&x| x == 0));
    let a = vec![u64::MAX; 128];
    let mut b = vec![0; 97];
    b[63] = 1;
    assert_eq!(convolution_u64(&a, &b), naive(&a, &b));
    assert_eq!(convolution_u64(&a, &vec![0; 97]), vec![0; 224]);
}
#[test]
fn primes_roots_and_crt_constants() {
    fn pow(mut a: u64, mut n: u64, p: u64) -> u64 {
        let mut r = 1;
        while n > 0 {
            if n & 1 != 0 {
                r = r * a % p;
            }
            a = a * a % p;
            n >>= 1;
        }
        r
    }
    let ps = [754974721u64, 1224736769, 2013265921, 1811939329, 2113929217];
    let roots = [11, 3, 31, 13, 5];
    let inverses = [656108986, 642531681, 340200806, 1914502476];
    for (i, (&p, &root)) in ps.iter().zip(&roots).enumerate() {
        for d in 2..=((p as f64).sqrt() as u64) {
            assert_ne!(p % d, 0);
        }
        assert_eq!((p - 1) % (1 << 24), 0);
        let w = pow(root, (p - 1) / (1 << 24), p);
        assert_eq!(pow(w, 1 << 24, p), 1);
        assert_ne!(pow(w, 1 << 23, p), 1);
        if i > 0 {
            let prefix = ps[..i].iter().fold(1u64, |x, &q| x * (q % p) % p);
            assert_eq!(prefix * inverses[i - 1] % p, 1);
        }
    }
    assert_eq!(
        ps[..3].iter().fold(1u64, |x, &q| x * (q % ps[4]) % ps[4]),
        1137597963
    );
    // 5素数の積が係数上限2^151を超えることを、多倍長積で確認。
    let mut limbs = vec![1u64];
    for p in ps {
        let mut carry = 0u128;
        for x in &mut limbs {
            let y = *x as u128 * p as u128 + carry;
            *x = y as u64;
            carry = y >> 64;
        }
        if carry != 0 {
            limbs.push(carry as u64);
        }
    }
    assert!(limbs.len() == 3 && limbs[2] > (1 << 23));
}
