pub trait NttModulus: Modulus {
    const PRIMITIVE_ROOT: u32;
}

impl NttModulus for Mod998244353 {
    const PRIMITIVE_ROOT: u32 = 3;
}

struct ButterflyCache<M: NttModulus> {
    sum_e: [StaticModInt<M>; 30],
    sum_ie: [StaticModInt<M>; 30],
}

fn prepare_butterfly<M: NttModulus>() -> ButterflyCache<M> {
    let zero = StaticModInt::<M>::raw(0);
    let one = StaticModInt::<M>::raw(1);

    let mut es = [zero; 30];
    let mut ies = [zero; 30];

    let g = StaticModInt::<M>::raw(M::PRIMITIVE_ROOT);
    let cnt2 = (M::VALUE - 1).trailing_zeros() as usize;

    let mut e = g.pow(((M::VALUE - 1) >> cnt2) as u64);
    let mut ie = e.inv();

    for i in (2..=cnt2).rev() {
        es[i - 2] = e;
        ies[i - 2] = ie;
        e *= e;
        ie *= ie;
    }

    let mut sum_e = [zero; 30];
    let mut sum_ie = [zero; 30];

    let mut x = one;
    let mut ix = one;

    for i in 0..30 {
        x *= es[i];
        ix *= ies[i];
        sum_e[i] = x;
        sum_ie[i] = ix;
    }

    ButterflyCache { sum_e, sum_ie }
}

fn butterfly<M: NttModulus>(
    a: &mut [StaticModInt<M>],
    cache: &ButterflyCache<M>,
) {
    let n = a.len();
    let h = n.trailing_zeros() as usize;

    for ph in 1..=h {
        let w = 1usize << (ph - 1);
        let p = 1usize << (h - ph);

        let mut now = StaticModInt::<M>::raw(1);

        for s in 0..w {
            let offset = s << (h - ph + 1);

            for i in 0..p {
                let l = a[i + offset];
                let r = a[i + offset + p] * now;

                a[i + offset] = l + r;
                a[i + offset + p] = l - r;
            }

            now *= cache.sum_e[(!s).trailing_zeros() as usize];
        }
    }
}

fn butterfly_inv<M: NttModulus>(
    a: &mut [StaticModInt<M>],
    cache: &ButterflyCache<M>,
) {
    let n = a.len();
    let h = n.trailing_zeros() as usize;

    for ph in (1..=h).rev() {
        let w = 1usize << (ph - 1);
        let p = 1usize << (h - ph);

        let mut inow = StaticModInt::<M>::raw(1);

        for s in 0..w {
            let offset = s << (h - ph + 1);

            for i in 0..p {
                let l = a[i + offset];
                let r = a[i + offset + p];

                a[i + offset] = l + r;
                a[i + offset + p]
                    = StaticModInt::<M>::new(M::VALUE + l.val() - r.val()) * inow;
            }

            inow *= cache.sum_ie[(!s).trailing_zeros() as usize];
        }
    }
}

pub fn convolution<M: NttModulus>(a: &[StaticModInt<M>],b: &[StaticModInt<M>]) -> Vec<StaticModInt<M>> {
    if a.is_empty() || b.is_empty() {
        return vec![];
    }
    let (n, m) = (a.len(), b.len());
    if n.min(m) <= 60 {
        let (n, m, a, b) = if n < m {
            (m, n, b, a)
        } else {
            (n, m, a, b)
        };

        let mut res = vec![StaticModInt::<M>::raw(0); n + m - 1];

        for i in 0..n {
            for j in 0..m {
                res[i + j] += a[i] * b[j];
            }
        }

        return res;
    }
    let z = (n + m - 1).next_power_of_two();
    assert!(
        z <= (1usize << (M::VALUE - 1).trailing_zeros()),
        "NTT length is too large"
    );

    let cache = prepare_butterfly::<M>();

    let mut a = a.to_vec();
    let mut b = b.to_vec();

    a.resize(z, StaticModInt::<M>::raw(0));
    b.resize(z, StaticModInt::<M>::raw(0));

    butterfly(&mut a, &cache);
    butterfly(&mut b, &cache);

    for (x, y) in a.iter_mut().zip(&b) {
        *x *= *y;
    }

    butterfly_inv(&mut a, &cache);

    a.resize(n + m - 1, StaticModInt::<M>::raw(0));

    let iz = StaticModInt::<M>::new(z as u64).inv();
    for x in &mut a {
        *x *= iz;
    }

    a
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Mod754974721 {}

impl Modulus for Mod754974721 {
    const VALUE: u32 = 754_974_721;
}

impl NttModulus for Mod754974721 {
    const PRIMITIVE_ROOT: u32 = 11;
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Mod167772161 {}

impl Modulus for Mod167772161 {
    const VALUE: u32 = 167_772_161;
}

impl NttModulus for Mod167772161 {
    const PRIMITIVE_ROOT: u32 = 3;
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Mod469762049 {}

impl Modulus for Mod469762049 {
    const VALUE: u32 = 469_762_049;
}

impl NttModulus for Mod469762049 {
    const PRIMITIVE_ROOT: u32 = 3;
}

#[inline]
fn safe_mod(mut x: i64, m: i64) -> i64 {
    x %= m;
    if x < 0 {
        x += m;
    }
    x
}

fn inv_gcd(a: i64, b: i64) -> (i64, i64) {
    let a = safe_mod(a, b);

    if a == 0 {
        return (b, 0);
    }

    let mut s = b;
    let mut t = a;
    let mut m0 = 0i64;
    let mut m1 = 1i64;

    while t != 0 {
        let u = s / t;

        s -= t * u;
        m0 -= m1 * u;

        std::mem::swap(&mut s, &mut t);
        std::mem::swap(&mut m0, &mut m1);
    }

    if m0 < 0 {
        m0 += b / s;
    }

    (s, m0)
}

fn convolution_i64_mod<M: NttModulus>(
    a: &[i64],
    b: &[i64],
) -> Vec<i64> {
    let aa = a
        .iter()
        .copied()
        .map(StaticModInt::<M>::new)
        .collect::<Vec<_>>();

    let bb = b
        .iter()
        .copied()
        .map(StaticModInt::<M>::new)
        .collect::<Vec<_>>();

    convolution::<M>(&aa, &bb)
        .into_iter()
        .map(|x| x.val() as i64)
        .collect()
}

pub fn convolution_i64(a: &[i64], b: &[i64]) -> Vec<i64> {
    if a.len().min(b.len()) <= 60{
        let mut res = vec![0; a.len()+b.len()-1];
        for (i,&v1) in a.iter().enumerate(){
            for (j, &v2) in b.iter().enumerate(){
                res[i+j]+=v1*v2;
            }
        }
        return res;
    }
    const M1: u64 = 754_974_721;
    const M2: u64 = 167_772_161;
    const M3: u64 = 469_762_049;

    const M2M3: u64 = M2 * M3;
    const M1M3: u64 = M1 * M3;
    const M1M2: u64 = M1 * M2;
    const M1M2M3: u64 = M1M2.wrapping_mul(M3);

    if a.is_empty() || b.is_empty() {
        return vec![];
    }

    assert!(
        a.len() + b.len() - 1 <= (1usize << 24),
        "convolution_i64: result length must be <= 2^24"
    );

    let (_, i1) = inv_gcd(M2M3 as i64, M1 as i64);
    let (_, i2) = inv_gcd(M1M3 as i64, M2 as i64);
    let (_, i3) = inv_gcd(M1M2 as i64, M3 as i64);

    let c1 = convolution_i64_mod::<Mod754974721>(a, b);
    let c2 = convolution_i64_mod::<Mod167772161>(a, b);
    let c3 = convolution_i64_mod::<Mod469762049>(a, b);

    c1.into_iter()
        .zip(c2)
        .zip(c3)
        .map(|((c1, c2), c3)| {
            const OFFSET: [u64; 5] = [
                0,
                0,
                M1M2M3,
                M1M2M3.wrapping_mul(2),
                M1M2M3.wrapping_mul(3),
            ];

            let mut x = 0i64;

            x = x.wrapping_add(
                c1.wrapping_mul(i1)
                    .rem_euclid(M1 as i64)
                    .wrapping_mul(M2M3 as i64),
            );

            x = x.wrapping_add(
                c2.wrapping_mul(i2)
                    .rem_euclid(M2 as i64)
                    .wrapping_mul(M1M3 as i64),
            );

            x = x.wrapping_add(
                c3.wrapping_mul(i3)
                    .rem_euclid(M3 as i64)
                    .wrapping_mul(M1M2 as i64),
            );

            let mut diff = c1 - safe_mod(x, M1 as i64);
            if diff < 0 {
                diff += M1 as i64;
            }

            x.wrapping_sub(
                OFFSET[diff.rem_euclid(5) as usize] as i64
            )
        })
        .collect()
}

pub fn convolution_i64_2(a: &[i64], b: &[i64]) -> Vec<i64> {
    if a.len().min(b.len()) <= 60{
        let mut res = vec![0; a.len()+b.len()-1];
        for (i,&v1) in a.iter().enumerate(){
            for (j, &v2) in b.iter().enumerate(){
                res[i+j]+=v1*v2;
            }
        }
        return res;
    }
    const M1: i64 = 754_974_721;
    const M2: i64 = 469_762_049;
    const M1M2: i64 = M1 * M2;
    const INV_M1_MOD_M2: i64 = 221_064_492;

    if a.is_empty() || b.is_empty() {
        return vec![];
    }

    assert!(
        a.len() + b.len() - 1 <= (1usize << 24),
        "convolution_i64_2: result length must be <= 2^24"
    );

    let c1 = convolution_i64_mod::<Mod754974721>(a, b);
    let c2 = convolution_i64_mod::<Mod469762049>(a, b);

    c1.into_iter()
        .zip(c2)
        .map(|(x1, x2)| {
            let t =
                safe_mod(x2 - x1, M2)
                * INV_M1_MOD_M2
                % M2;

            let mut x = x1 + M1 * t;
            if x > M1M2 / 2 {
                x -= M1M2;
            }

            x
        })
        .collect()
}
