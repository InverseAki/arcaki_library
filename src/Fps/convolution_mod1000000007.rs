pub fn convolution_mod1000000007(
    a: &[StaticModInt<Mod1000000007>],
    b: &[StaticModInt<Mod1000000007>],
) -> Vec<StaticModInt<Mod1000000007>> {
    if a.is_empty() || b.is_empty() {
        return vec![];
    }
    let size = a.len().checked_add(b.len() - 1).expect("length overflow");
    assert!(size <= 1 << 24, "convolution result length must be <= 2^24");
    if a.len().min(b.len()) <= 60 {
        let mut c = vec![StaticModInt::<Mod1000000007>::raw(0); size];
        for (i, &x) in a.iter().enumerate() {
            for (j, &y) in b.iter().enumerate() {
                c[i + j] += x * y;
            }
        }
        return c;
    }
    let a: Vec<u32> = a.iter().map(|x| x.val()).collect();
    let b: Vec<u32> = b.iter().map(|x| x.val()).collect();
    let c1 = convolution_1000000007_detail::convolve::<167772161, 3>(&a, &b);
    let c2 = convolution_1000000007_detail::convolve::<469762049, 3>(&a, &b);
    let c3 = convolution_1000000007_detail::convolve::<754974721, 11>(&a, &b);
    const P1: u64 = 167772161;
    const P2: u64 = 469762049;
    const P3: u64 = 754974721;
    const MOD: u64 = 1000000007;
    const INV_P1_MOD_P2: u64 = 104391568;
    const INV_P1P2_MOD_P3: u64 = 190329765;
    c1.into_iter()
        .zip(c2)
        .zip(c3)
        .map(|((x, y), z)| {
            let t1 = (y as u64 + P2 - x as u64) * INV_P1_MOD_P2 % P2;
            let low = x as u64 + P1 * t1;
            let t2 = (z as u64 + P3 - low % P3) * INV_P1P2_MOD_P3 % P3;
            let value = (low % MOD + (P1 * P2 % MOD) * t2) % MOD;
            StaticModInt::<Mod1000000007>::raw(value as u32)
        })
        .collect()
}

mod convolution_1000000007_detail {
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
    pub(super) fn convolve<const P: u64, const ROOT: u64>(a: &[u32], b: &[u32]) -> Vec<u32> {
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
