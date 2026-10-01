#[inline(always)]
fn gcd_u64(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a
}

#[derive(Clone, Copy)]
struct Montgomery64 {
    n: u64,
    ni: u64,
    r2: u64,
}

impl Montgomery64 {
    #[inline]
    fn new(n: u64) -> Self {
        debug_assert!(n >= 3 && n & 1 == 1);
        let mut inv = n;
        for _ in 0..6 {
            inv = inv.wrapping_mul(
                2u64.wrapping_sub(n.wrapping_mul(inv))
            );
        }
        let ni = inv.wrapping_neg();
        let r2 =
            (0u128.wrapping_sub(n as u128) % n as u128) as u64;

        Self { n, ni, r2 }
    }

    #[inline(always)]
    fn reduce(&self, x: u128) -> u64 {
        let q = (x as u64).wrapping_mul(self.ni);
        let qn = q as u128 * self.n as u128;
        let (s, carry) = x.overflowing_add(qn);
        let hi = (s >> 64) as u64;

        if carry {
            hi.wrapping_sub(self.n)
        } else if hi >= self.n {
            hi - self.n
        } else {
            hi
        }
    }

    #[inline(always)]
    fn make(&self, x: u64) -> u64 {
        self.reduce(x as u128 * self.r2 as u128)
    }

    #[inline(always)]
    fn mul(&self, a: u64, b: u64) -> u64 {
        self.reduce(a as u128 * b as u128)
    }

    #[inline(always)]
    fn add(&self, a: u64, b: u64) -> u64 {
        let (s, carry) = a.overflowing_add(b);

        if carry || s >= self.n {
            s.wrapping_sub(self.n)
        } else {
            s
        }
    }

    #[inline(always)]
    fn one(&self) -> u64 {
        self.make(1)
    }

    #[inline(always)]
    fn pow(&self, mut a: u64, mut e: u64) -> u64 {
        let mut res = self.one();

        while e != 0 {
            if e & 1 != 0 {
                res = self.mul(res, a);
            }

            a = self.mul(a, a);
            e >>= 1;
        }

        res
    }
}

#[inline]
pub fn is_prime_u64(n: u64) -> bool {
    if n < 2 {
        return false;
    }

    const SMALL: [u64; 12] = [
        2, 3, 5, 7, 11, 13,
        17, 19, 23, 29, 31, 37,
    ];

    for p in SMALL {
        if n % p == 0 {
            return n == p;
        }
    }
    let s = (n - 1).trailing_zeros();
    let d = (n - 1) >> s;

    let mont = Montgomery64::new(n);

    let one = mont.one();
    let minus_one = mont.make(n - 1);

    #[inline(always)]
    fn test(
        n: u64,
        a: u64,
        d: u64,
        s: u32,
        mont: &Montgomery64,
        one: u64,
        minus_one: u64,
    ) -> bool {
        let a = a % n;

        if a == 0 {
            return true;
        }

        let mut x = mont.pow(mont.make(a), d);

        if x == one || x == minus_one {
            return true;
        }

        for _ in 1..s {
            x = mont.mul(x, x);

            if x == minus_one {
                return true;
            }
        }

        false
    }

    if n < (1u64 << 32) {
        for a in [2u64, 7, 61] {
            if !test(
                n, a, d, s,
                &mont, one, minus_one
            ) {
                return false;
            }
        }
    } else {
        for a in [
            2u64,
            325,
            9375,
            28178,
            450775,
            9780504,
            1795265022,
        ] {
            if !test(
                n, a, d, s,
                &mont, one, minus_one
            ) {
                return false;
            }
        }
    }

    true
}

#[inline]
pub fn miller_rabin(n: usize) -> bool {
    is_prime_u64(n as u64)
}

#[derive(Clone, Copy)]
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    #[inline]
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    #[inline(always)]
    fn next(&mut self) -> u64 {
        self.state =self.state.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }
}

fn pollard_rho(n: u64) -> u64 {
    debug_assert!(n > 1);
    debug_assert!(!is_prime_u64(n));
    if n % 2 == 0 {return 2;}
    if n % 3 == 0 {return 3;}
    if n % 5 == 0 {return 5;}
    let mont = Montgomery64::new(n);
    let one = mont.one();
    let mut rng = SplitMix64::new(n ^ 0xA0761D6478BD642F);
    loop {
        let c = mont.make(rng.next() % (n - 1) + 1);
        let mut y = mont.make(rng.next() % (n - 2) + 2);
        let mut x = 0;
        let mut ys = 0;
        let mut g = 1u64;
        let mut r = 1usize;
        let mut q = one;
        #[inline(always)]
        fn f(
            mont: &Montgomery64,
            x: u64,
            c: u64,
        ) -> u64 {
            mont.add(mont.mul(x, x), c)
        }

        while g == 1 {
            x = y;

            for _ in 0..r {
                y = f(&mont, y, c);
            }

            let mut k = 0usize;

            while k < r && g == 1 {
                ys = y;
                let len = 128.min(r - k);
                for _ in 0..len {
                    y = f(&mont, y, c);
                    let d = x.abs_diff(y);
                    q = mont.mul(q, d);
                }
                g = gcd_u64(q, n);
                k += len;
            }
            r <<= 1;
        }
        if g == n {
            loop {
                ys = f(&mont, ys, c);
                g = gcd_u64(x.abs_diff(ys), n);
                if g > 1 {
                    break;
                }
            }
        }
        if g != n {
            return g;
        }
    }
}

pub fn factorize_vec_u64(mut n: u64) -> Vec<u64> {
    if n <= 1 {
        return Vec::new();
    }

    let mut res = Vec::new();
    for p in [
        2u64, 3, 5, 7, 11, 13,
        17, 19, 23, 29, 31, 37,
    ] {
        while n % p == 0 {
            res.push(p);
            n /= p;
        }
    }

    if n == 1 {
        return res;
    }

    let mut stack = vec![n];

    while let Some(x) = stack.pop() {
        if x == 1 {
            continue;
        }

        if is_prime_u64(x) {
            res.push(x);
            continue;
        }

        let d = pollard_rho(x);

        stack.push(d);
        stack.push(x / d);
    }

    res.sort_unstable();
    res
}

pub fn factorize_pairs_u64(n: u64) -> Vec<(u64, usize)> {
    let v = factorize_vec_u64(n);
    let mut res: Vec<(u64, usize)> = Vec::new();
    for p in v {
        if let Some(last) = res.last_mut() {
            if last.0 == p {
                last.1 += 1;
                continue;
            }
        }
        res.push((p, 1));
    }
    res
}

pub fn factorize(n: usize) -> HashMap<usize, usize> {
    let mut res = HashMap::new();

    for p in factorize_vec_u64(n as u64) {
        *res.entry(p as usize).or_insert(0) += 1;
    }

    res
}
