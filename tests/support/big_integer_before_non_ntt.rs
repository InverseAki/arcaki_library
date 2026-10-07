#[derive(Clone, Default, Eq, PartialEq, Hash)]
pub struct RadixBigInt<const BASE: u32> {
    negative: bool,
    digits: Vec<u32>,
}
pub type BigInt = RadixBigInt<10000>;
pub type HexBigInt = RadixBigInt<65536>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParseBigIntError;
impl std::fmt::Display for ParseBigIntError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("invalid big integer (decimal / hexadecimal native radix)")
    }
}
impl std::error::Error for ParseBigIntError {}

impl<const B: u32> RadixBigInt<B> {
    fn check_base() {
        assert!(B == 10000 || B == 65536, "supported bases: 10000, 65536");
    }
    fn from_digits(negative: bool, mut digits: Vec<u32>) -> Self {
        big_integer_detail::trim(&mut digits);
        Self {
            negative: negative && !digits.is_empty(),
            digits,
        }
    }
    pub fn zero() -> Self {
        Self::default()
    }
    pub fn one() -> Self {
        Self::from(1u64)
    }
    pub fn is_zero(&self) -> bool {
        self.digits.is_empty()
    }
    pub fn is_negative(&self) -> bool {
        self.negative
    }
    pub fn signum(&self) -> i32 {
        if self.is_zero() {
            0
        } else if self.negative {
            -1
        } else {
            1
        }
    }
    pub fn abs(&self) -> Self {
        Self::from_digits(false, self.digits.clone())
    }
    pub fn gcd(&self, rhs: &Self) -> Self {
        let (mut a, mut b) = (self.abs(), rhs.abs());
        if a < b {
            std::mem::swap(&mut a, &mut b);
        }
        while !b.is_zero() {
            if let (Some(mut x), Some(mut y)) = (a.to_u128(), b.to_u128()) {
                while y != 0 {
                    let r = x % y;
                    x = y;
                    y = r;
                }
                return Self::from(x);
            }
            let r = &a % &b;
            a = b;
            b = r;
        }
        a
    }
    pub fn limb_len(&self) -> usize {
        self.digits.len()
    }
    pub fn parse_bytes(s: &[u8]) -> Result<Self, ParseBigIntError> {
        Self::check_base();
        let (negative, s) = match s.first() {
            Some(b'-') => (true, &s[1..]),
            Some(b'+') => (false, &s[1..]),
            _ => (false, s),
        };
        if s.is_empty() {
            return Err(ParseBigIntError);
        }
        let radix = if B == 10000 { 10 } else { 16 };
        let mut digits = Vec::with_capacity((s.len() + 3) / 4);
        for chunk in s.rchunks(4) {
            let mut x = 0;
            for &c in chunk {
                let d = match c {
                    b'0'..=b'9' => (c - b'0') as u32,
                    b'a'..=b'f' => (c - b'a' + 10) as u32,
                    b'A'..=b'F' => (c - b'A' + 10) as u32,
                    _ => return Err(ParseBigIntError),
                };
                if d >= radix {
                    return Err(ParseBigIntError);
                }
                x = x * radix + d;
            }
            digits.push(x);
        }
        Ok(Self::from_digits(negative, digits))
    }
    pub fn append_to(&self, output: &mut String) {
        Self::check_base();
        if self.is_zero() {
            output.push('0');
            return;
        }
        if self.negative {
            output.push('-');
        }
        let radix = if B == 10000 { 10 } else { 16 };
        const DIGITS: &[u8; 16] = b"0123456789ABCDEF";
        for (i, &d) in self.digits.iter().rev().enumerate() {
            let mut x = d;
            let mut buf = [b'0'; 4];
            for c in buf.iter_mut().rev() {
                *c = DIGITS[(x % radix) as usize];
                x /= radix;
            }
            let start = if i == 0 {
                buf.iter().position(|&c| c != b'0').unwrap_or(3)
            } else {
                0
            };
            output.push_str(std::str::from_utf8(&buf[start..]).unwrap());
        }
    }
    pub fn div_rem(&self, rhs: &Self) -> (Self, Self) {
        assert!(!rhs.is_zero(), "division by zero");
        let (q, r) = big_integer_detail::div_rem::<B>(&self.digits, &rhs.digits);
        let mut q = Self::from_digits(self.negative ^ rhs.negative, q);
        let mut r = Self::from_digits(false, r);
        if self.negative && !r.is_zero() {
            r = &rhs.abs() - &r;
            q = if rhs.negative {
                &q + &Self::one()
            } else {
                &q - &Self::one()
            };
        }
        (q, r)
    }
    pub fn checked_div_rem(&self, rhs: &Self) -> Option<(Self, Self)> {
        if rhs.is_zero() {
            None
        } else {
            Some(self.div_rem(rhs))
        }
    }
    pub fn div_rem_euclid(&self, rhs: &Self) -> (Self, Self) {
        self.div_rem(rhs)
    }
    pub fn pow(&self, mut exponent: u64) -> Self {
        let mut answer = Self::one();
        let mut x = self.clone();
        while exponent != 0 {
            if exponent & 1 != 0 {
                answer = &answer * &x;
            }
            exponent >>= 1;
            if exponent != 0 {
                x = &x * &x;
            }
        }
        answer
    }
    pub fn to_u128(&self) -> Option<u128> {
        if self.negative {
            return None;
        }
        let mut x = 0u128;
        for &d in self.digits.iter().rev() {
            x = x.checked_mul(B as u128)?.checked_add(d as u128)?;
        }
        Some(x)
    }
    pub fn to_i128(&self) -> Option<i128> {
        let mut x = 0u128;
        for &d in self.digits.iter().rev() {
            x = x.checked_mul(B as u128)?.checked_add(d as u128)?;
        }
        if self.negative {
            if x == 1u128 << 127 {
                Some(i128::MIN)
            } else {
                i128::try_from(x).ok().map(|x| -x)
            }
        } else {
            i128::try_from(x).ok()
        }
    }
}
impl<const B: u32> std::str::FromStr for RadixBigInt<B> {
    type Err = ParseBigIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse_bytes(s.as_bytes())
    }
}
impl<const B: u32> std::fmt::Display for RadixBigInt<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::with_capacity(self.digits.len() * 4 + 1);
        self.append_to(&mut s);
        f.pad(&s)
    }
}
impl<const B: u32> std::fmt::Debug for RadixBigInt<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}
impl<const B: u32> From<u128> for RadixBigInt<B> {
    fn from(mut x: u128) -> Self {
        Self::check_base();
        let mut digits = Vec::new();
        while x != 0 {
            digits.push((x % B as u128) as u32);
            x /= B as u128;
        }
        Self::from_digits(false, digits)
    }
}
impl<const B: u32> From<i128> for RadixBigInt<B> {
    fn from(x: i128) -> Self {
        let mut a = Self::from(x.unsigned_abs());
        a.negative = x < 0;
        a
    }
}
macro_rules! big_integer_from {
    ($($t:ty => $wide:ty),* $(,)?) => {$(impl<const B:u32> From<$t> for RadixBigInt<B> {
        fn from(x:$t)->Self {Self::from(x as $wide)}
    })*};
}
big_integer_from!(u8=>u128,u16=>u128,u32=>u128,u64=>u128,usize=>u128,i8=>i128,i16=>i128,i32=>i128,i64=>i128,isize=>i128);
impl<const B: u32> Ord for RadixBigInt<B> {
    fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
        self.negative.cmp(&rhs.negative).reverse().then_with(|| {
            let c = big_integer_detail::cmp(&self.digits, &rhs.digits);
            if self.negative {
                c.reverse()
            } else {
                c
            }
        })
    }
}
impl<const B: u32> PartialOrd for RadixBigInt<B> {
    fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(rhs))
    }
}
impl<const B: u32> std::ops::Neg for RadixBigInt<B> {
    type Output = Self;
    fn neg(mut self) -> Self {
        if !self.is_zero() {
            self.negative = !self.negative;
        }
        self
    }
}
impl<const B: u32> std::ops::Neg for &RadixBigInt<B> {
    type Output = RadixBigInt<B>;
    fn neg(self) -> Self::Output {
        -self.clone()
    }
}
impl<const B: u32> std::ops::Add for &RadixBigInt<B> {
    type Output = RadixBigInt<B>;
    fn add(self, rhs: Self) -> Self::Output {
        if self.negative == rhs.negative {
            RadixBigInt::from_digits(
                self.negative,
                big_integer_detail::add::<B>(&self.digits, &rhs.digits),
            )
        } else {
            match big_integer_detail::cmp(&self.digits, &rhs.digits) {
                std::cmp::Ordering::Less => RadixBigInt::from_digits(
                    rhs.negative,
                    big_integer_detail::sub::<B>(&rhs.digits, &self.digits),
                ),
                _ => RadixBigInt::from_digits(
                    self.negative,
                    big_integer_detail::sub::<B>(&self.digits, &rhs.digits),
                ),
            }
        }
    }
}
impl<const B: u32> std::ops::Sub for &RadixBigInt<B> {
    type Output = RadixBigInt<B>;
    fn sub(self, rhs: Self) -> Self::Output {
        if self.negative != rhs.negative {
            RadixBigInt::from_digits(
                self.negative,
                big_integer_detail::add::<B>(&self.digits, &rhs.digits),
            )
        } else {
            match big_integer_detail::cmp(&self.digits, &rhs.digits) {
                std::cmp::Ordering::Less => RadixBigInt::from_digits(
                    !self.negative,
                    big_integer_detail::sub::<B>(&rhs.digits, &self.digits),
                ),
                _ => RadixBigInt::from_digits(
                    self.negative,
                    big_integer_detail::sub::<B>(&self.digits, &rhs.digits),
                ),
            }
        }
    }
}
impl<const B: u32> std::ops::Mul for &RadixBigInt<B> {
    type Output = RadixBigInt<B>;
    fn mul(self, rhs: Self) -> Self::Output {
        RadixBigInt::from_digits(
            self.negative ^ rhs.negative,
            big_integer_detail::mul::<B>(&self.digits, &rhs.digits),
        )
    }
}
impl<const B: u32> std::ops::Div for &RadixBigInt<B> {
    type Output = RadixBigInt<B>;
    fn div(self, rhs: Self) -> Self::Output {
        self.div_rem(rhs).0
    }
}
impl<const B: u32> std::ops::Rem for &RadixBigInt<B> {
    type Output = RadixBigInt<B>;
    fn rem(self, rhs: Self) -> Self::Output {
        self.div_rem(rhs).1
    }
}
macro_rules! big_integer_ops {
    ($($trait:ident,$method:ident,$assign:ident,$assign_method:ident);* $(;)?)=>{$(
        impl<const B:u32> std::ops::$trait for RadixBigInt<B> {
            type Output=Self;
            fn $method(self,rhs:Self)->Self {std::ops::$trait::$method(&self,&rhs)}
        }
        impl<const B:u32> std::ops::$trait<&Self> for RadixBigInt<B> {
            type Output=Self;
            fn $method(self,rhs:&Self)->Self {std::ops::$trait::$method(&self,rhs)}
        }
        impl<const B:u32> std::ops::$trait<RadixBigInt<B>> for &RadixBigInt<B> {
            type Output=RadixBigInt<B>;
            fn $method(self,rhs:RadixBigInt<B>)->Self::Output {std::ops::$trait::$method(self,&rhs)}
        }
        impl<const B:u32> std::ops::$assign<&Self> for RadixBigInt<B> {
            fn $assign_method(&mut self,rhs:&Self) {*self=std::ops::$trait::$method(&*self,rhs);}
        }
        impl<const B:u32> std::ops::$assign for RadixBigInt<B> {
            fn $assign_method(&mut self,rhs:Self) {std::ops::$assign::$assign_method(self,&rhs);}
        }
    )*};
}
big_integer_ops!(Add,add,AddAssign,add_assign;Sub,sub,SubAssign,sub_assign;Mul,mul,MulAssign,mul_assign;Div,div,DivAssign,div_assign;Rem,rem,RemAssign,rem_assign);

mod big_integer_detail {
    use std::cmp::Ordering;
    pub(super) fn trim(a: &mut Vec<u32>) {
        while a.last() == Some(&0) {
            a.pop();
        }
    }
    pub(super) fn cmp(a: &[u32], b: &[u32]) -> Ordering {
        a.len()
            .cmp(&b.len())
            .then_with(|| a.iter().rev().cmp(b.iter().rev()))
    }
    pub(super) fn add<const B: u32>(a: &[u32], b: &[u32]) -> Vec<u32> {
        let n = a.len().max(b.len());
        let mut c = Vec::with_capacity(n + 1);
        let mut carry = 0;
        for i in 0..n {
            let x = a.get(i).copied().unwrap_or(0) + b.get(i).copied().unwrap_or(0) + carry;
            carry = (x >= B) as u32;
            c.push(if x >= B { x - B } else { x });
        }
        if carry != 0 {
            c.push(carry);
        }
        c
    }
    pub(super) fn sub<const B: u32>(a: &[u32], b: &[u32]) -> Vec<u32> {
        debug_assert!(cmp(a, b) != Ordering::Less);
        let mut c = a.to_vec();
        sub_assign::<B>(&mut c, b);
        c
    }
    fn sub_assign<const B: u32>(a: &mut Vec<u32>, b: &[u32]) {
        let mut borrow = 0i64;
        for (i, x) in a.iter_mut().enumerate() {
            let y = *x as i64 - b.get(i).copied().unwrap_or(0) as i64 - borrow;
            borrow = (y < 0) as i64;
            *x = (y + borrow * B as i64) as u32;
        }
        debug_assert_eq!(borrow, 0);
        trim(a);
    }
    fn increment<const B: u32>(a: &mut Vec<u32>) {
        for x in a.iter_mut() {
            if *x + 1 < B {
                *x += 1;
                return;
            }
            *x = 0;
        }
        a.push(1);
    }
    fn mul_small<const B: u32>(a: &[u32], b: u32) -> Vec<u32> {
        if b == 0 {
            return vec![];
        }
        let mut c = Vec::with_capacity(a.len() + 1);
        let mut carry = 0u64;
        for &x in a {
            let t = x as u64 * b as u64 + carry;
            c.push((t % B as u64) as u32);
            carry = t / B as u64;
        }
        while carry != 0 {
            c.push((carry % B as u64) as u32);
            carry /= B as u64;
        }
        trim(&mut c);
        c
    }
    fn div_small<const B: u32>(a: &[u32], b: u32) -> (Vec<u32>, u32) {
        let mut c = vec![0; a.len()];
        let mut r = 0u64;
        for i in (0..a.len()).rev() {
            let x = r * B as u64 + a[i] as u64;
            c[i] = (x / b as u64) as u32;
            r = x % b as u64;
        }
        trim(&mut c);
        (c, r as u32)
    }
    pub(super) fn mul<const B: u32>(a: &[u32], b: &[u32]) -> Vec<u32> {
        if a.is_empty() || b.is_empty() {
            return vec![];
        }
        if a.len() == 1 {
            return mul_small::<B>(b, a[0]);
        }
        if b.len() == 1 {
            return mul_small::<B>(a, b[0]);
        }
        let size = a.len().checked_add(b.len() - 1).expect("length overflow");
        let coefficients = if a.len().min(b.len()) <= 48 {
            let mut c = vec![0u64; size];
            for (i, &x) in a.iter().enumerate() {
                for (j, &y) in b.iter().enumerate() {
                    c[i + j] += x as u64 * y as u64;
                }
            }
            c
        } else {
            assert!(size <= 1 << 25, "NTT result length exceeds 2^25");
            let c0 = convolve::<167772161, 3>(a, b);
            let c1 = convolve::<469762049, 3>(a, b);
            c0.into_iter()
                .zip(c1)
                .map(|(x, y)| {
                    let t = (y as u64 + 469762049 - x as u64) * 104391568 % 469762049;
                    x as u64 + 167772161 * t
                })
                .collect()
        };
        let mut answer = Vec::with_capacity(size + 1);
        let mut carry = 0u64;
        for c in coefficients {
            let x = c + carry;
            answer.push((x % B as u64) as u32);
            carry = x / B as u64;
        }
        while carry != 0 {
            answer.push((carry % B as u64) as u32);
            carry /= B as u64;
        }
        trim(&mut answer);
        answer
    }
    fn mod_pow<const P: u64>(mut x: u64, mut e: u64) -> u64 {
        let mut y = 1;
        while e != 0 {
            if e & 1 != 0 {
                y = y * x % P;
            }
            x = x * x % P;
            e >>= 1;
        }
        y
    }
    const fn ntt_neg_inverse(p: u32) -> u32 {
        let mut x = p;
        let mut i = 0;
        while i < 5 {
            x = x.wrapping_mul(2u32.wrapping_sub(p.wrapping_mul(x)));
            i += 1;
        }
        x.wrapping_neg()
    }
    #[inline(always)]
    fn ntt_one<const P: u64>() -> u32 {
        ((1u64 << 32) % P) as u32
    }
    #[inline(always)]
    fn ntt_add<const P: u64>(a: u32, b: u32) -> u32 {
        let x = a + b;
        if x >= 2 * P as u32 {
            x - 2 * P as u32
        } else {
            x
        }
    }
    #[inline(always)]
    fn ntt_sub<const P: u64>(a: u32, b: u32) -> u32 {
        if a >= b {
            a - b
        } else {
            a + 2 * P as u32 - b
        }
    }
    #[inline(always)]
    fn ntt_mul<const P: u64>(a: u32, b: u32) -> u32 {
        let t = a as u64 * b as u64;
        let m = (t as u32).wrapping_mul(ntt_neg_inverse(P as u32));
        ((t + m as u64 * P) >> 32) as u32
    }
    fn ntt_pow<const P: u64>(a: u32, e: u64) -> u32 {
        ((mod_pow::<P>(a as u64, e) << 32) % P) as u32
    }
    #[cfg(target_arch = "aarch64")]
    mod ntt_neon {
        use std::arch::aarch64::*;
        const LANES: usize = 4;
        type V = uint32x4_t;
        #[inline(always)]
        unsafe fn splat(x: u32) -> V {
            vdupq_n_u32(x)
        }
        #[inline(always)]
        unsafe fn load(x: *const u32) -> V {
            vld1q_u32(x)
        }
        #[inline(always)]
        unsafe fn store(x: *mut u32, v: V) {
            vst1q_u32(x, v)
        }
        #[inline(always)]
        unsafe fn add<const P: u64>(a: V, b: V) -> V {
            let s = vaddq_u32(a, b);
            vminq_u32(s, vsubq_u32(s, splat(2 * P as u32)))
        }
        #[inline(always)]
        unsafe fn sub<const P: u64>(a: V, b: V) -> V {
            let s = vsubq_u32(a, b);
            vminq_u32(s, vaddq_u32(s, splat(2 * P as u32)))
        }
        #[inline(always)]
        unsafe fn mul<const P: u64>(a: V, b: V) -> V {
            let m = vmulq_u32(vmulq_u32(a, b), splat(super::ntt_neg_inverse(P as u32)));
            let p = vdup_n_u32(P as u32);
            let lo = vaddq_u64(
                vmull_u32(vget_low_u32(a), vget_low_u32(b)),
                vmull_u32(vget_low_u32(m), p),
            );
            let hi = vaddq_u64(
                vmull_u32(vget_high_u32(a), vget_high_u32(b)),
                vmull_u32(vget_high_u32(m), p),
            );
            vcombine_u32(vshrn_n_u64::<32>(lo), vshrn_n_u64::<32>(hi))
        }
        #[inline]
        #[target_feature(enable = "neon")]
        pub(super) unsafe fn forward4<const P: u64, const UNIT: bool>(
            a0: &mut [u32],
            a1: &mut [u32],
            a2: &mut [u32],
            a3: &mut [u32],
            r1: u32,
            r2: u32,
            r3: u32,
            imag: u32,
        ) {
            let (r1, r2, r3, im) = (splat(r1), splat(r2), splat(r3), splat(imag));
            for i in (0..a0.len()).step_by(LANES) {
                let u = load(a0.as_ptr().add(i));
                let v = load(a1.as_ptr().add(i));
                let w = load(a2.as_ptr().add(i));
                let z = load(a3.as_ptr().add(i));
                let v = if UNIT { v } else { mul::<P>(v, r1) };
                let w = if UNIT { w } else { mul::<P>(w, r2) };
                let z = if UNIT { z } else { mul::<P>(z, r3) };
                let s02 = add::<P>(u, w);
                let d02 = sub::<P>(u, w);
                let s13 = add::<P>(v, z);
                let d13 = mul::<P>(sub::<P>(v, z), im);
                store(a0.as_mut_ptr().add(i), add::<P>(s02, s13));
                store(a1.as_mut_ptr().add(i), sub::<P>(s02, s13));
                store(a2.as_mut_ptr().add(i), add::<P>(d02, d13));
                store(a3.as_mut_ptr().add(i), sub::<P>(d02, d13));
            }
        }
        #[inline]
        #[target_feature(enable = "neon")]
        pub(super) unsafe fn inverse4<const P: u64, const UNIT: bool>(
            a0: &mut [u32],
            a1: &mut [u32],
            a2: &mut [u32],
            a3: &mut [u32],
            r1: u32,
            r2: u32,
            r3: u32,
            imag: u32,
        ) {
            let (r1, r2, r3, im) = (splat(r1), splat(r2), splat(r3), splat(imag));
            for i in (0..a0.len()).step_by(LANES) {
                let u = load(a0.as_ptr().add(i));
                let v = load(a1.as_ptr().add(i));
                let w = load(a2.as_ptr().add(i));
                let z = load(a3.as_ptr().add(i));
                let s01 = add::<P>(u, v);
                let d01 = sub::<P>(u, v);
                let s23 = add::<P>(w, z);
                let d23 = mul::<P>(sub::<P>(w, z), im);
                let t1 = add::<P>(d01, d23);
                let t2 = sub::<P>(s01, s23);
                let t3 = sub::<P>(d01, d23);
                store(a0.as_mut_ptr().add(i), add::<P>(s01, s23));
                store(
                    a1.as_mut_ptr().add(i),
                    if UNIT { t1 } else { mul::<P>(t1, r1) },
                );
                store(
                    a2.as_mut_ptr().add(i),
                    if UNIT { t2 } else { mul::<P>(t2, r2) },
                );
                store(
                    a3.as_mut_ptr().add(i),
                    if UNIT { t3 } else { mul::<P>(t3, r3) },
                );
            }
        }
    }
    #[cfg(target_arch = "x86_64")]
    mod ntt_avx2 {
        use std::arch::x86_64::*;
        const LANES: usize = 8;
        type V = __m256i;
        #[inline(always)]
        unsafe fn splat(x: u32) -> V {
            _mm256_set1_epi32(x as i32)
        }
        #[inline(always)]
        unsafe fn load(x: *const u32) -> V {
            _mm256_loadu_si256(x.cast())
        }
        #[inline(always)]
        unsafe fn store(x: *mut u32, v: V) {
            _mm256_storeu_si256(x.cast(), v)
        }
        #[inline(always)]
        unsafe fn add<const P: u64>(a: V, b: V) -> V {
            let s = _mm256_add_epi32(a, b);
            _mm256_min_epu32(s, _mm256_sub_epi32(s, splat(2 * P as u32)))
        }
        #[inline(always)]
        unsafe fn sub<const P: u64>(a: V, b: V) -> V {
            let s = _mm256_sub_epi32(a, b);
            _mm256_min_epu32(s, _mm256_add_epi32(s, splat(2 * P as u32)))
        }
        #[inline(always)]
        unsafe fn mul<const P: u64>(a: V, b: V) -> V {
            let m = _mm256_mullo_epi32(
                _mm256_mullo_epi32(a, b),
                splat(super::ntt_neg_inverse(P as u32)),
            );
            let p = splat(P as u32);
            let even = _mm256_add_epi64(_mm256_mul_epu32(a, b), _mm256_mul_epu32(m, p));
            let odd = _mm256_add_epi64(
                _mm256_mul_epu32(_mm256_srli_epi64::<32>(a), _mm256_srli_epi64::<32>(b)),
                _mm256_mul_epu32(_mm256_srli_epi64::<32>(m), p),
            );
            _mm256_blend_epi32::<0xAA>(_mm256_srli_epi64::<32>(even), odd)
        }
        #[inline]
        #[target_feature(enable = "avx2")]
        pub(super) unsafe fn forward4<const P: u64, const UNIT: bool>(
            a0: &mut [u32],
            a1: &mut [u32],
            a2: &mut [u32],
            a3: &mut [u32],
            r1: u32,
            r2: u32,
            r3: u32,
            imag: u32,
        ) {
            let (r1, r2, r3, im) = (splat(r1), splat(r2), splat(r3), splat(imag));
            for i in (0..a0.len()).step_by(LANES) {
                let u = load(a0.as_ptr().add(i));
                let v = load(a1.as_ptr().add(i));
                let w = load(a2.as_ptr().add(i));
                let z = load(a3.as_ptr().add(i));
                let v = if UNIT { v } else { mul::<P>(v, r1) };
                let w = if UNIT { w } else { mul::<P>(w, r2) };
                let z = if UNIT { z } else { mul::<P>(z, r3) };
                let s02 = add::<P>(u, w);
                let d02 = sub::<P>(u, w);
                let s13 = add::<P>(v, z);
                let d13 = mul::<P>(sub::<P>(v, z), im);
                store(a0.as_mut_ptr().add(i), add::<P>(s02, s13));
                store(a1.as_mut_ptr().add(i), sub::<P>(s02, s13));
                store(a2.as_mut_ptr().add(i), add::<P>(d02, d13));
                store(a3.as_mut_ptr().add(i), sub::<P>(d02, d13));
            }
        }
        #[inline]
        #[target_feature(enable = "avx2")]
        pub(super) unsafe fn inverse4<const P: u64, const UNIT: bool>(
            a0: &mut [u32],
            a1: &mut [u32],
            a2: &mut [u32],
            a3: &mut [u32],
            r1: u32,
            r2: u32,
            r3: u32,
            imag: u32,
        ) {
            let (r1, r2, r3, im) = (splat(r1), splat(r2), splat(r3), splat(imag));
            for i in (0..a0.len()).step_by(LANES) {
                let u = load(a0.as_ptr().add(i));
                let v = load(a1.as_ptr().add(i));
                let w = load(a2.as_ptr().add(i));
                let z = load(a3.as_ptr().add(i));
                let s01 = add::<P>(u, v);
                let d01 = sub::<P>(u, v);
                let s23 = add::<P>(w, z);
                let d23 = mul::<P>(sub::<P>(w, z), im);
                let t1 = add::<P>(d01, d23);
                let t2 = sub::<P>(s01, s23);
                let t3 = sub::<P>(d01, d23);
                store(a0.as_mut_ptr().add(i), add::<P>(s01, s23));
                store(
                    a1.as_mut_ptr().add(i),
                    if UNIT { t1 } else { mul::<P>(t1, r1) },
                );
                store(
                    a2.as_mut_ptr().add(i),
                    if UNIT { t2 } else { mul::<P>(t2, r2) },
                );
                store(
                    a3.as_mut_ptr().add(i),
                    if UNIT { t3 } else { mul::<P>(t3, r3) },
                );
            }
        }
    }

    fn ntt_simd_available() -> bool {
        #[cfg(target_arch = "aarch64")]
        {
            return std::arch::is_aarch64_feature_detected!("neon");
        }
        #[cfg(target_arch = "x86_64")]
        {
            return std::arch::is_x86_feature_detected!("avx2");
        }
        #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
        {
            false
        }
    }

    struct Plan {
        simd: bool,
        root: [u32; 26],
        iroot: [u32; 26],
        rate2: [u32; 24],
        irate2: [u32; 24],
        rate3: [u32; 23],
        irate3: [u32; 23],
    }
    impl Plan {
        fn new<const P: u64, const G: u64>() -> Self {
            let mut s = Self {
                simd: ntt_simd_available(),
                root: [0; 26],
                iroot: [0; 26],
                rate2: [0; 24],
                irate2: [0; 24],
                rate3: [0; 23],
                irate3: [0; 23],
            };
            s.root[25] = ntt_pow::<P>(G as u32, (P - 1) >> 25);
            s.iroot[25] = ntt_pow::<P>(G as u32, P - 1 - ((P - 1) >> 25));
            for i in (0..25).rev() {
                s.root[i] = ntt_mul::<P>(s.root[i + 1], s.root[i + 1]);
                s.iroot[i] = ntt_mul::<P>(s.iroot[i + 1], s.iroot[i + 1]);
            }
            let (mut p, mut ip) = (ntt_one::<P>(), ntt_one::<P>());
            for i in 0..24 {
                s.rate2[i] = ntt_mul::<P>(s.root[i + 2], p);
                s.irate2[i] = ntt_mul::<P>(s.iroot[i + 2], ip);
                p = ntt_mul::<P>(p, s.iroot[i + 2]);
                ip = ntt_mul::<P>(ip, s.root[i + 2]);
            }
            let (mut p, mut ip) = (ntt_one::<P>(), ntt_one::<P>());
            for i in 0..23 {
                s.rate3[i] = ntt_mul::<P>(s.root[i + 3], p);
                s.irate3[i] = ntt_mul::<P>(s.iroot[i + 3], ip);
                p = ntt_mul::<P>(p, s.iroot[i + 3]);
                ip = ntt_mul::<P>(ip, s.root[i + 3]);
            }
            s
        }
        #[inline]
        fn forward4<const P: u64, const UNIT: bool>(
            &self,
            a0: &mut [u32],
            a1: &mut [u32],
            a2: &mut [u32],
            a3: &mut [u32],
            r1: u32,
            r2: u32,
            r3: u32,
            imag: u32,
        ) {
            assert!(a0.len().is_power_of_two());
            assert_eq!(a0.len(), a1.len());
            assert_eq!(a0.len(), a2.len());
            assert_eq!(a0.len(), a3.len());
            #[cfg(target_arch = "aarch64")]
            if self.simd && a0.len() >= 4 {
                unsafe {
                    ntt_neon::forward4::<P, UNIT>(a0, a1, a2, a3, r1, r2, r3, imag);
                }
                return;
            }
            #[cfg(target_arch = "x86_64")]
            if self.simd && a0.len() >= 8 {
                unsafe {
                    ntt_avx2::forward4::<P, UNIT>(a0, a1, a2, a3, r1, r2, r3, imag);
                }
                return;
            }
            Self::forward4_scalar::<P, UNIT>(a0, a1, a2, a3, r1, r2, r3, imag);
        }
        #[inline]
        fn forward4_scalar<const P: u64, const UNIT: bool>(
            a0: &mut [u32],
            a1: &mut [u32],
            a2: &mut [u32],
            a3: &mut [u32],
            r1: u32,
            r2: u32,
            r3: u32,
            imag: u32,
        ) {
            for (((x0, x1), x2), x3) in a0.iter_mut().zip(a1).zip(a2).zip(a3) {
                let u = *x0;
                let v = if UNIT { *x1 } else { ntt_mul::<P>(*x1, r1) };
                let w = if UNIT { *x2 } else { ntt_mul::<P>(*x2, r2) };
                let z = if UNIT { *x3 } else { ntt_mul::<P>(*x3, r3) };
                let s02 = ntt_add::<P>(u, w);
                let d02 = ntt_sub::<P>(u, w);
                let s13 = ntt_add::<P>(v, z);
                let d13 = ntt_mul::<P>(ntt_sub::<P>(v, z), imag);
                *x0 = ntt_add::<P>(s02, s13);
                *x1 = ntt_sub::<P>(s02, s13);
                *x2 = ntt_add::<P>(d02, d13);
                *x3 = ntt_sub::<P>(d02, d13);
            }
        }
        #[inline]
        fn inverse4<const P: u64, const UNIT: bool>(
            &self,
            a0: &mut [u32],
            a1: &mut [u32],
            a2: &mut [u32],
            a3: &mut [u32],
            r1: u32,
            r2: u32,
            r3: u32,
            imag: u32,
        ) {
            assert!(a0.len().is_power_of_two());
            assert_eq!(a0.len(), a1.len());
            assert_eq!(a0.len(), a2.len());
            assert_eq!(a0.len(), a3.len());
            #[cfg(target_arch = "aarch64")]
            if self.simd && a0.len() >= 4 {
                unsafe {
                    ntt_neon::inverse4::<P, UNIT>(a0, a1, a2, a3, r1, r2, r3, imag);
                }
                return;
            }
            #[cfg(target_arch = "x86_64")]
            if self.simd && a0.len() >= 8 {
                unsafe {
                    ntt_avx2::inverse4::<P, UNIT>(a0, a1, a2, a3, r1, r2, r3, imag);
                }
                return;
            }
            Self::inverse4_scalar::<P, UNIT>(a0, a1, a2, a3, r1, r2, r3, imag);
        }
        #[inline]
        fn inverse4_scalar<const P: u64, const UNIT: bool>(
            a0: &mut [u32],
            a1: &mut [u32],
            a2: &mut [u32],
            a3: &mut [u32],
            r1: u32,
            r2: u32,
            r3: u32,
            imag: u32,
        ) {
            for (((x0, x1), x2), x3) in a0.iter_mut().zip(a1).zip(a2).zip(a3) {
                let u = *x0;
                let v = *x1;
                let w = *x2;
                let z = *x3;
                let s01 = ntt_add::<P>(u, v);
                let d01 = ntt_sub::<P>(u, v);
                let s23 = ntt_add::<P>(w, z);
                let d23 = ntt_mul::<P>(ntt_sub::<P>(w, z), imag);
                let t1 = ntt_add::<P>(d01, d23);
                let t2 = ntt_sub::<P>(s01, s23);
                let t3 = ntt_sub::<P>(d01, d23);
                *x0 = ntt_add::<P>(s01, s23);
                *x1 = if UNIT { t1 } else { ntt_mul::<P>(t1, r1) };
                *x2 = if UNIT { t2 } else { ntt_mul::<P>(t2, r2) };
                *x3 = if UNIT { t3 } else { ntt_mul::<P>(t3, r3) };
            }
        }
        fn forward<const P: u64>(&self, a: &mut [u32]) {
            let h = a.len().trailing_zeros() as usize;
            let mut len = 0;
            while len < h {
                if h - len == 1 {
                    let p = 1 << (h - len - 1);
                    let blocks = 1 << len;
                    let mut rot = ntt_one::<P>();
                    for (s, block) in a.chunks_exact_mut(2 * p).enumerate() {
                        let (l, r) = block.split_at_mut(p);
                        for (x, y) in l.iter_mut().zip(r) {
                            let u = *x;
                            let v = ntt_mul::<P>(*y, rot);
                            *x = ntt_add::<P>(u, v);
                            *y = ntt_sub::<P>(u, v);
                        }
                        if s + 1 < blocks {
                            rot = ntt_mul::<P>(rot, self.rate2[(!s).trailing_zeros() as usize]);
                        }
                    }
                    len += 1;
                } else {
                    let p = 1 << (h - len - 2);
                    let blocks = 1 << len;
                    let mut rot = ntt_one::<P>();
                    let imag = self.root[2];
                    for (s, block) in a.chunks_exact_mut(4 * p).enumerate() {
                        let rot2 = ntt_mul::<P>(rot, rot);
                        let rot3 = ntt_mul::<P>(rot2, rot);
                        let (a0, rest) = block.split_at_mut(p);
                        let (a1, rest) = rest.split_at_mut(p);
                        let (a2, a3) = rest.split_at_mut(p);
                        if s == 0 {
                            self.forward4::<P, true>(a0, a1, a2, a3, rot, rot2, rot3, imag);
                        } else {
                            self.forward4::<P, false>(a0, a1, a2, a3, rot, rot2, rot3, imag);
                        }
                        if s + 1 < blocks {
                            rot = ntt_mul::<P>(rot, self.rate3[(!s).trailing_zeros() as usize]);
                        }
                    }
                    len += 2;
                }
            }
        }
        fn inverse<const P: u64>(&self, a: &mut [u32]) {
            let h = a.len().trailing_zeros() as usize;
            let mut len = h;
            while len > 0 {
                if len == 1 {
                    let p = 1 << (h - len);
                    let blocks = 1 << (len - 1);
                    let mut rot = ntt_one::<P>();
                    for (s, block) in a.chunks_exact_mut(2 * p).enumerate() {
                        let (l, r) = block.split_at_mut(p);
                        for (x, y) in l.iter_mut().zip(r) {
                            let u = *x;
                            let v = *y;
                            *x = ntt_add::<P>(u, v);
                            *y = ntt_mul::<P>(ntt_sub::<P>(u, v), rot);
                        }
                        if s + 1 < blocks {
                            rot = ntt_mul::<P>(rot, self.irate2[(!s).trailing_zeros() as usize]);
                        }
                    }
                    len -= 1;
                } else {
                    let p = 1 << (h - len);
                    let blocks = 1 << (len - 2);
                    let mut rot = ntt_one::<P>();
                    let imag = self.iroot[2];
                    for (s, block) in a.chunks_exact_mut(4 * p).enumerate() {
                        let rot2 = ntt_mul::<P>(rot, rot);
                        let rot3 = ntt_mul::<P>(rot2, rot);
                        let (a0, rest) = block.split_at_mut(p);
                        let (a1, rest) = rest.split_at_mut(p);
                        let (a2, a3) = rest.split_at_mut(p);
                        if s == 0 {
                            self.inverse4::<P, true>(a0, a1, a2, a3, rot, rot2, rot3, imag);
                        } else {
                            self.inverse4::<P, false>(a0, a1, a2, a3, rot, rot2, rot3, imag);
                        }
                        if s + 1 < blocks {
                            rot = ntt_mul::<P>(rot, self.irate3[(!s).trailing_zeros() as usize]);
                        }
                    }
                    len -= 2;
                }
            }
        }
    }
    fn ntt<const P: u64, const G: u64>(a: &mut [u32], inverse: bool) {
        static P0: std::sync::OnceLock<Plan> = std::sync::OnceLock::new();
        static P1: std::sync::OnceLock<Plan> = std::sync::OnceLock::new();
        assert!(G == 3 && (P == 167772161 || P == 469762049));
        let plan = if P == 167772161 {
            P0.get_or_init(Plan::new::<P, G>)
        } else {
            P1.get_or_init(Plan::new::<P, G>)
        };
        if inverse {
            plan.inverse::<P>(a);
            let inv = mod_pow::<P>(a.len() as u64, P - 2) as u32;
            for x in a {
                let v = ntt_mul::<P>(*x, inv);
                *x = if v >= P as u32 { v - P as u32 } else { v };
            }
        } else {
            let r2 = (((1u64 << 32) % P) * ((1u64 << 32) % P) % P) as u32;
            for x in a.iter_mut() {
                *x = ntt_mul::<P>(*x, r2);
            }
            plan.forward::<P>(a);
        }
    }
    #[cfg(test)]
    mod ntt_tests {
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
    }
    fn convolve<const P: u64, const G: u64>(a: &[u32], b: &[u32]) -> Vec<u32> {
        let size = a.len() + b.len() - 1;
        let n = size.next_power_of_two();
        let mut x = vec![0; n];
        x[..a.len()].copy_from_slice(a);
        ntt::<P, G>(&mut x, false);
        if std::ptr::eq(a, b) {
            for v in &mut x {
                *v = ntt_mul::<P>(*v, *v);
            }
        } else {
            let mut y = vec![0; n];
            y[..b.len()].copy_from_slice(b);
            ntt::<P, G>(&mut y, false);
            for (x, y) in x.iter_mut().zip(y) {
                *x = ntt_mul::<P>(*x, y);
            }
        }
        ntt::<P, G>(&mut x, true);
        x.truncate(size);
        x
    }
    fn school_div<const B: u32>(a: &[u32], b: &[u32]) -> (Vec<u32>, Vec<u32>) {
        if cmp(a, b) == Ordering::Less {
            return (vec![], a.to_vec());
        }
        if b.len() == 1 {
            let (q, r) = div_small::<B>(a, b[0]);
            return (q, if r == 0 { vec![] } else { vec![r] });
        }
        let n = b.len();
        let mut u = a.to_vec();
        u.push(0);
        let mut q = vec![0; a.len() - n + 1];
        let base = B as u64;
        for j in (0..q.len()).rev() {
            let numerator = u[j + n] as u64 * base + u[j + n - 1] as u64;
            let mut digit = (numerator / b[n - 1] as u64).min(base - 1);
            let mut rem = numerator - digit * b[n - 1] as u64;
            while rem < base && digit * b[n - 2] as u64 > rem * base + u[j + n - 2] as u64 {
                digit -= 1;
                rem += b[n - 1] as u64;
            }
            let mut borrow = 0u64;
            for i in 0..n {
                let p = digit * b[i] as u64 + borrow;
                let diff = u[j + i] as i64 - (p % base) as i64;
                borrow = p / base + (diff < 0) as u64;
                u[j + i] = (diff + if diff < 0 { B as i64 } else { 0 }) as u32;
            }
            let tail = u[j + n] as i64 - borrow as i64;
            if tail < 0 {
                digit -= 1;
                let mut carry = 0u32;
                for i in 0..n {
                    let s = u[j + i] + b[i] + carry;
                    carry = (s >= B) as u32;
                    u[j + i] = if s >= B { s - B } else { s };
                }
                u[j + n] = ((tail + B as i64 + carry as i64) % B as i64) as u32;
            } else {
                u[j + n] = tail as u32;
            }
            q[j] = digit as u32;
        }
        u.truncate(n);
        trim(&mut u);
        trim(&mut q);
        (q, u)
    }
    fn power(position: usize, value: u32) -> Vec<u32> {
        let mut a = vec![0; position + 1];
        a[position] = value;
        a
    }
    fn reciprocal<const B: u32>(a: &[u32]) -> Vec<u32> {
        let n = a.len();
        debug_assert!(a[n - 1] >= B / 2);
        let s = power(2 * n, 1);
        if n <= 32 {
            return school_div::<B>(&s, a).0;
        }
        let m = (n + 1) / 2;
        let mut x = vec![0; n - m];
        x.extend(reciprocal::<B>(&a[n - m..]));
        let ax = mul::<B>(a, &x);
        let correction = sub::<B>(&power(2 * n, 2), &ax);
        x = mul::<B>(&x, &correction).into_iter().skip(2 * n).collect();
        trim(&mut x);
        let ax = mul::<B>(a, &x);
        let mut r = sub::<B>(&s, &ax);
        while cmp(&r, a) != Ordering::Less {
            sub_assign::<B>(&mut r, a);
            increment::<B>(&mut x);
        }
        x
    }
    struct Divider {
        cut: usize,
        n: usize,
        inverse: Vec<u32>,
        power_of_base: bool,
    }
    impl Divider {
        fn new<const B: u32>(b: &[u32], quotient_len: usize) -> Self {
            let p = b.len().min(quotient_len + 2);
            let cut = b.len() - p;
            let mut high = b[cut..].to_vec();
            if cut != 0 {
                increment::<B>(&mut high);
            }
            let is_power =
                high.last() == Some(&1) && high[..high.len() - 1].iter().all(|&x| x == 0);
            let inverse = if is_power {
                vec![]
            } else {
                reciprocal::<B>(&high)
            };
            Self {
                cut,
                n: high.len(),
                inverse,
                power_of_base: is_power,
            }
        }
        fn divide<const B: u32>(&self, a: &[u32], b: &[u32]) -> (Vec<u32>, Vec<u32>) {
            let high = &a[self.cut..];
            let mut q = if self.power_of_base {
                high.iter().skip(self.n - 1).copied().collect()
            } else {
                mul::<B>(high, &self.inverse)
                    .into_iter()
                    .skip(2 * self.n)
                    .collect()
            };
            trim(&mut q);
            let product = mul::<B>(&q, b);
            let mut r = sub::<B>(a, &product);
            while cmp(&r, b) != Ordering::Less {
                sub_assign::<B>(&mut r, b);
                increment::<B>(&mut q);
            }
            (q, r)
        }
    }
    pub(super) fn div_rem<const B: u32>(a: &[u32], b: &[u32]) -> (Vec<u32>, Vec<u32>) {
        if cmp(a, b) == Ordering::Less {
            return (vec![], a.to_vec());
        }
        if b.len() == 1 {
            let (q, r) = div_small::<B>(a, b[0]);
            return (q, if r == 0 { vec![] } else { vec![r] });
        }
        let scale = B / (b[b.len() - 1] + 1);
        let a = mul_small::<B>(a, scale);
        let b = mul_small::<B>(b, scale);
        let n = b.len();
        let q_len = a.len() - n + 1;
        let (q, r) = if n <= 32 || q_len <= 32 {
            school_div::<B>(&a, &b)
        } else if a.len() <= 2 * n {
            Divider::new::<B>(&b, q_len).divide::<B>(&a, &b)
        } else {
            let divider = Divider::new::<B>(&b, n);
            let mut q = vec![0; a.len()];
            let mut r = Vec::new();
            for start in (0..a.len()).step_by(n).rev() {
                let end = (start + n).min(a.len());
                let mut current = a[start..end].to_vec();
                current.extend(r);
                trim(&mut current);
                if cmp(&current, &b) == Ordering::Less {
                    r = current;
                    continue;
                }
                let (chunk, rest) = divider.divide::<B>(&current, &b);
                q[start..start + chunk.len()].copy_from_slice(&chunk);
                r = rest;
            }
            trim(&mut q);
            (q, r)
        };
        let (r, rem) = div_small::<B>(&r, scale);
        debug_assert_eq!(rem, 0);
        (q, r)
    }
}
