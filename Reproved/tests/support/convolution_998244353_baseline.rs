pub fn convolution_mod998244353(
    a: &[StaticModInt<Mod998244353>],
    b: &[StaticModInt<Mod998244353>],
) -> Vec<StaticModInt<Mod998244353>> {
    convolution_998244353_detail::check_lengths(a.len(), b.len());
    if a.is_empty() || b.is_empty() {
        return vec![];
    }
    let size = a.len() + b.len() - 1;
    if a.len().min(b.len()) <= 60 {
        let mut c = vec![StaticModInt::<Mod998244353>::raw(0); size];
        for (i, &x) in a.iter().enumerate() {
            for (j, &y) in b.iter().enumerate() {
                c[i + j] += x * y;
            }
        }
        return c;
    }
    if size <= 1 << 23 {
        return convolution_998244353_detail::convolve::<998244353, 3>(a, b)
            .into_iter()
            .map(StaticModInt::<Mod998244353>::raw)
            .collect();
    }
    convolution_998244353_detail::crt(a, b)
}

mod convolution_998244353_detail {
    pub(super) fn check_lengths(n: usize, m: usize) {
        assert!(
            n <= 1 << 24 && m <= 1 << 24,
            "each input length must be <= 2^24"
        );
    }
    pub(super) fn crt(
        a: &[super::StaticModInt<super::Mod998244353>],
        b: &[super::StaticModInt<super::Mod998244353>],
    ) -> Vec<super::StaticModInt<super::Mod998244353>> {
        let c1 = convolve::<167772161, 3>(a, b);
        let c2 = convolve::<469762049, 3>(a, b);
        let c3 = convolve::<2013265921, 31>(a, b);
        const P1: u64 = 167772161;
        const P2: u64 = 469762049;
        const P3: u64 = 2013265921;
        const MOD: u64 = 998244353;
        c1.into_iter()
            .zip(c2)
            .zip(c3)
            .map(|((x, y), z)| {
                let t1 = (y as u64 + P2 - x as u64) * 104391568 % P2;
                let low = x as u64 + P1 * t1;
                let t2 = (z as u64 + P3 - low % P3) * 1066314758 % P3;
                let v = (low % MOD + (P1 * P2 % MOD) * t2) % MOD;
                super::StaticModInt::<super::Mod998244353>::raw(v as u32)
            })
            .collect()
    }

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
    pub(super) fn convolve<const P: u64, const ROOT: u64>(
        a: &[super::StaticModInt<super::Mod998244353>],
        b: &[super::StaticModInt<super::Mod998244353>],
    ) -> Vec<u32> {
        let size = a.len() + b.len() - 1;
        let n = size.next_power_of_two();
        let mut x = vec![0; n];
        let mut y = vec![0; n];
        for (to, &v) in x.iter_mut().zip(a) {
            *to = (v.val() as u64 % P) as u32;
        }
        for (to, &v) in y.iter_mut().zip(b) {
            *to = (v.val() as u64 % P) as u32;
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
