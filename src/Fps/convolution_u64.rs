pub fn convolution_u64(a: &[u64], b: &[u64]) -> Vec<u64> {
    if a.is_empty() || b.is_empty() {
        return vec![];
    }
    let size = a.len().checked_add(b.len() - 1).expect("length overflow");
    assert!(size <= 1 << 24, "convolution result length must be <= 2^24");
    if a.len().min(b.len()) <= 60 {
        let mut c = vec![0u64; size];
        for (i, &x) in a.iter().enumerate() {
            for (j, &y) in b.iter().enumerate() {
                c[i + j] = c[i + j].wrapping_add(x.wrapping_mul(y));
            }
        }
        return c;
    }
    let c0 = convolution_u64_detail::convolve::<754974721, 11>(a, b);
    let c1 = convolution_u64_detail::convolve::<1224736769, 3>(a, b);
    let c2 = convolution_u64_detail::convolve::<2013265921, 31>(a, b);
    let c3 = convolution_u64_detail::convolve::<1811939329, 13>(a, b);
    let c4 = convolution_u64_detail::convolve::<2113929217, 5>(a, b);
    const P0: u64 = 754974721;
    const P1: u64 = 1224736769;
    const P2: u64 = 2013265921;
    const P3: u64 = 1811939329;
    const P4: u64 = 2113929217;
    const P01: u64 = P0 * P1;
    const P012: u64 = P01.wrapping_mul(P2);
    const P0123: u64 = P012.wrapping_mul(P3);
    let mut answer = Vec::with_capacity(size);
    for i in 0..size {
        let t0 = c0[i] as u64;
        let t1 = (c1[i] as u64 + P1 - t0) * 656108986 % P1;
        let low = t0 + P0 * t1;
        let t2 = (c2[i] as u64 + P2 - low % P2) * 642531681 % P2;
        let r3 = (low % P3 + (P01 % P3) * t2) % P3;
        let t3 = (c3[i] as u64 + P3 - r3) * 340200806 % P3;
        let r4 = (low % P4 + (P01 % P4) * t2 % P4 + 1137597963 * t3) % P4;
        let t4 = (c4[i] as u64 + P4 - r4) * 1914502476 % P4;
        answer.push(
            low.wrapping_add(P01.wrapping_mul(t2))
                .wrapping_add(P012.wrapping_mul(t3))
                .wrapping_add(P0123.wrapping_mul(t4)),
        );
    }
    answer
}

mod convolution_u64_detail {
    fn pow<const P: u64>(mut x: u64, mut n: u64) -> u64 {
        let mut r = 1;
        while n > 0 {
            if n & 1 != 0 {
                r = r * x % P;
            }
            x = x * x % P;
            n >>= 1;
        }
        r
    }
    fn ntt<const P: u64, const ROOT: u64>(a: &mut [u32], inverse: bool) {
        let n = a.len();
        let mut j = 0;
        for i in 1..n {
            let mut bit = n >> 1;
            while j & bit != 0 {
                j ^= bit;
                bit >>= 1;
            }
            j ^= bit;
            if i < j {
                a.swap(i, j);
            }
        }
        let mut len = 2;
        while len <= n {
            let mut root = pow::<P>(ROOT, (P - 1) / len as u64);
            if inverse {
                root = pow::<P>(root, P - 2);
            }
            for block in a.chunks_exact_mut(len) {
                let mut w = 1u64;
                let (left, right) = block.split_at_mut(len / 2);
                for (x, y) in left.iter_mut().zip(right) {
                    let u = *x as u64;
                    let v = *y as u64 * w % P;
                    let sum = u + v;
                    *x = (if sum >= P { sum - P } else { sum }) as u32;
                    *y = (if u >= v { u - v } else { u + P - v }) as u32;
                    w = w * root % P;
                }
            }
            len <<= 1;
        }
        if inverse {
            let inv = pow::<P>(n as u64, P - 2);
            for x in a {
                *x = (*x as u64 * inv % P) as u32;
            }
        }
    }
    pub(super) fn convolve<const P: u64, const ROOT: u64>(a: &[u64], b: &[u64]) -> Vec<u32> {
        let size = a.len() + b.len() - 1;
        let n = size.next_power_of_two();
        let mut x = vec![0; n];
        let mut y = vec![0; n];
        for (to, &v) in x.iter_mut().zip(a) {
            *to = (v as u64 % P) as u32;
        }
        for (to, &v) in y.iter_mut().zip(b) {
            *to = (v as u64 % P) as u32;
        }
        ntt::<P, ROOT>(&mut x, false);
        ntt::<P, ROOT>(&mut y, false);
        for (x, y) in x.iter_mut().zip(y) {
            *x = (*x as u64 * y as u64 % P) as u32;
        }
        ntt::<P, ROOT>(&mut x, true);
        x.truncate(size);
        x
    }
}
