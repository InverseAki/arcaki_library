// 符号付き多倍長整数。単独で main.rs にコピー可、外部クレート不要。
// BigInt は十進、HexBigInt は十六進の入出力が O(n)。内部は4文字/limb。
// 大きな積は2素数NTT+CRT。除算は正規化した整数逆数のNewton反復。
// NTTの結果長 <= 2^25。一般の任意基数変換・ビット演算は提供しない。

#[derive(Clone, Default, Eq, PartialEq, Hash)]
pub struct RadixBigInt<const BASE: u32> {
    negative: bool,
    digits: Vec<u32>, // little endian、0は空、最上位の0なし
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
    /// 非負の最大公約数。gcd(0,0)=0。符号は結果に影響しない。
    /// 小さい値はu128で計算し、大きい値は既存の剰余によるEuclid互除法。
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
    /// 4文字ずつ直接読み込む。BigIntは10進、HexBigIntは16進。+/-と先頭0可。
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
    /// ネイティブ基数（BigInt=10進、HexBigInt=大文字16進）で追記。
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
            // ASCIIだけを生成。
            output.push_str(std::str::from_utf8(&buf[start..]).unwrap());
        }
    }
    /// Euclid除算。self=q*rhs+r、0<=r<|rhs|。正の除数では商はfloor。
    /// rhs=0ならpanic。/、%、/=、%=もこの規則を使う。
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
    /// 互換名。通常のdiv_remも非負の剰余を返す。
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
    // a>=b。呼び出し側で正規化済み。
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
            // 逐次carryではなく整数係数を集める。短い側<=48ならu64に十分収まる。
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
            // (B-1)^2*min(n,m) <= 65535^2*2^24 < P0*P1。
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
    // DIFの順変換、DITの逆変換。bit reversalを省き、係数積を同じ順序で行う。
    fn ntt<const P: u64, const G: u64>(a: &mut [u32], inverse: bool) {
        let n = a.len();
        if !inverse {
            let mut len = n;
            while len > 1 {
                let root = mod_pow::<P>(G, (P - 1) / len as u64);
                for block in a.chunks_exact_mut(len) {
                    let (lo, hi) = block.split_at_mut(len / 2);
                    let mut w = 1u64;
                    for (x, y) in lo.iter_mut().zip(hi) {
                        let u = *x as u64;
                        let v = *y as u64;
                        *x = (if u + v >= P { u + v - P } else { u + v }) as u32;
                        *y = ((if u >= v { u - v } else { u + P - v }) * w % P) as u32;
                        w = w * root % P;
                    }
                }
                len >>= 1;
            }
        } else {
            let mut len = 2;
            while len <= n {
                let root = mod_pow::<P>(G, P - 1 - (P - 1) / len as u64);
                for block in a.chunks_exact_mut(len) {
                    let (lo, hi) = block.split_at_mut(len / 2);
                    let mut w = 1u64;
                    for (x, y) in lo.iter_mut().zip(hi) {
                        let u = *x as u64;
                        let v = *y as u64 * w % P;
                        *x = (if u + v >= P { u + v - P } else { u + v }) as u32;
                        *y = (if u >= v { u - v } else { u + P - v }) as u32;
                        w = w * root % P;
                    }
                }
                len <<= 1;
            }
            let inv_n = mod_pow::<P>(n as u64, P - 2);
            for x in a {
                *x = (*x as u64 * inv_n % P) as u32;
            }
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
                *v = (*v as u64 * *v as u64 % P) as u32;
            }
        } else {
            let mut y = vec![0; n];
            y[..b.len()].copy_from_slice(b);
            ntt::<P, G>(&mut y, false);
            for (x, y) in x.iter_mut().zip(y) {
                *x = (*x as u64 * y as u64 % P) as u32;
            }
        }
        ntt::<P, G>(&mut x, true);
        x.truncate(size);
        x
    }
    // 正規化済み b（最上位>=B/2）に対するKnuthの筆算除算。
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
    // floor(B^(2n)/a)、aの最上位>=B/2。上半分の厳密逆数から精度を倍増。
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
        // x <- floor(x*(2S-a*x)/S)。常に真の逆数以下へ着地する。
        x = mul::<B>(&x, &correction).into_iter().skip(2 * n).collect();
        trim(&mut x);
        let ax = mul::<B>(a, &x);
        let mut r = sub::<B>(&s, &ax);
        // 上半分由来の誤差<=4B^(n-m)、Newton後の誤差<17。
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
            // 切り捨てた低位がある場合は上へ丸め、商の過大推定を防ぐ。
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
