use super::*;
fn check<const P: u64>() {
    let mut state = 20261004u64;
    let mut random = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let mut scalar = Plan::new::<P, 3>();
    scalar.simd = false;
    let vector = Plan::new::<P, 3>();
    let r2 = (((1u64 << 32) % P).pow(2) % P) as u32;
    for _ in 0..10000 {
        let a = (random() % (2 * P)) as u32;
        let b = (random() % (2 * P)) as u32;
        let product = ntt_mul::<P>(a, b);
        assert!((product as u64) < 2 * P);
        assert_eq!(
            product as u64 % P,
            a as u64 * b as u64 % P * mod_pow::<P>((1 << 32) % P, P - 2) % P
        );
    }
    for log in 0..=16 {
        let n = 1 << log;
        let source: Vec<_> = (0..n)
            .map(|i| match i % 5 {
                0 => 0,
                1 => P as u32 - 1,
                _ => (random() % P) as u32,
            })
            .collect();
        let mut roundtrip = source.clone();
        ntt::<P, 3>(&mut roundtrip, false);
        ntt::<P, 3>(&mut roundtrip, true);
        assert_eq!(roundtrip, source);
        for offset in [0, 1, 3, 7] {
            let mut a = vec![0; n + offset];
            for (x, &v) in a[offset..].iter_mut().zip(&source) {
                *x = ntt_mul::<P>(v, r2);
            }
            let mut b = a.clone();
            scalar.forward::<P>(&mut a[offset..]);
            vector.forward::<P>(&mut b[offset..]);
            assert_eq!(a, b);
            scalar.inverse::<P>(&mut a[offset..]);
            vector.inverse::<P>(&mut b[offset..]);
            assert_eq!(a, b);
        }
    }
    for n in [1, 2, 4, 7, 31, 48, 49, 63, 64, 65, 127, 128, 129] {
        for m in [1, 3, 49, 64, 129] {
            let a: Vec<_> = (0..n).map(|_| (random() % P) as u32).collect();
            let b: Vec<_> = (0..m).map(|_| (random() % P) as u32).collect();
            let mut expected = vec![0u32; n + m - 1];
            for (i, &x) in a.iter().enumerate() {
                for (j, &y) in b.iter().enumerate() {
                    expected[i + j] =
                        ((expected[i + j] as u64 + x as u64 * y as u64) % P) as u32;
                }
            }
            assert_eq!(convolve::<P, 3>(&a, &b), expected);
        }
    }
}
#[test]
fn montgomery_radix4_scalar_and_simd_prime0() {
    check::<167772161>();
}
#[test]
fn montgomery_radix4_scalar_and_simd_prime1() {
    check::<469762049>();
}
