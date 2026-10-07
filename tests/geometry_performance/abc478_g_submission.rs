#![allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Rational<T: RatioInteger> {
    x: T,
    y: T,
}
pub type Ratio = Rational<i64>;
pub type Ratio128 = Rational<i128>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RatioError {
    Overflow,
    Indeterminate,
    NotFinite,
    InvalidFormat,
}
impl std::fmt::Display for RatioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Overflow => "reduced ratio does not fit the integer type",
            Self::Indeterminate => "indeterminate ratio (0/0, inf-inf, 0*inf, inf/inf)",
            Self::NotFinite => "cannot round an infinite ratio",
            Self::InvalidFormat => "invalid ratio",
        })
    }
}
impl std::error::Error for RatioError {}

#[doc(hidden)]
pub trait RatioInteger:
    Clone + Ord + std::hash::Hash + std::fmt::Display + std::str::FromStr + From<i32>
{
    fn normalize(x: Self, y: Self) -> Result<(Self, Self), RatioError>;
    fn negate(y: &Self) -> Result<Self, RatioError>;
    fn add_finite(
        x: &Self,
        y: &Self,
        u: &Self,
        v: &Self,
        subtract: bool,
    ) -> Result<(Self, Self), RatioError>;
    fn mul_finite(
        x: &Self,
        y: &Self,
        u: &Self,
        v: &Self,
        divide: bool,
    ) -> Result<(Self, Self), RatioError>;
    fn cmp_finite(x: &Self, y: &Self, u: &Self, v: &Self) -> std::cmp::Ordering;
    fn round(x: &Self, y: &Self, mode: i8) -> Self;
}
impl<T: RatioInteger> Rational<T> {
    pub fn try_new(x: T, y: T) -> Result<Self, RatioError> {
        let (x, y) = T::normalize(x, y)?;
        Ok(Self { x, y })
    }
    pub fn new(x: T, y: T) -> Self {
        Self::try_new(x, y).expect("invalid ratio")
    }
    pub fn try_from_fraction<N: Into<T>, D: Into<T>>(
        numerator: N,
        denominator: D,
    ) -> Result<Self, RatioError> {
        Self::try_new(denominator.into(), numerator.into())
    }
    pub fn from_fraction<N: Into<T>, D: Into<T>>(numerator: N, denominator: D) -> Self {
        Self::try_from_fraction(numerator, denominator).expect("invalid ratio")
    }
    pub fn int<I: Into<T>>(value: I) -> Self {
        Self {
            x: T::from(1),
            y: value.into(),
        }
    }
    pub fn zero() -> Self {
        Self::int(T::from(0))
    }
    pub fn one() -> Self {
        Self::int(T::from(1))
    }
    pub fn infinity() -> Self {
        Self {
            x: T::from(0),
            y: T::from(1),
        }
    }
    pub fn negative_infinity() -> Self {
        Self {
            x: T::from(0),
            y: T::from(-1),
        }
    }
    pub fn numerator(&self) -> &T {
        &self.y
    }
    pub fn denominator(&self) -> &T {
        &self.x
    }
    pub fn into_parts(self) -> (T, T) {
        (self.y, self.x)
    }
    pub fn is_inf(&self) -> bool {
        self.x == T::from(0)
    }
    pub fn is_finite(&self) -> bool {
        !self.is_inf()
    }
    pub fn is_zero(&self) -> bool {
        self.y == T::from(0)
    }
    pub fn signum(&self) -> i8 {
        match self.y.cmp(&T::from(0)) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        }
    }
    fn signed_infinity(sign: i8) -> Self {
        if sign < 0 {
            Self::negative_infinity()
        } else {
            Self::infinity()
        }
    }
    pub fn try_neg(&self) -> Result<Self, RatioError> {
        Ok(Self {
            x: self.x.clone(),
            y: T::negate(&self.y)?,
        })
    }
    pub fn try_abs(&self) -> Result<Self, RatioError> {
        if self.signum() < 0 {
            self.try_neg()
        } else {
            Ok(self.clone())
        }
    }
    pub fn abs(&self) -> Self {
        self.try_abs().expect("ratio overflow")
    }
    pub fn try_inv(&self) -> Result<Self, RatioError> {
        if self.is_inf() {
            Ok(Self::zero())
        } else {
            Self::try_new(self.y.clone(), self.x.clone())
        }
    }
    pub fn inv(&self) -> Self {
        self.try_inv().expect("ratio overflow")
    }
    pub fn recip(&self) -> Self {
        self.inv()
    }
    fn add_sub(&self, rhs: &Self, subtract: bool) -> Result<Self, RatioError> {
        if self.is_inf() || rhs.is_inf() {
            let sign = rhs.signum() * if subtract { -1 } else { 1 };
            if self.is_inf() && rhs.is_inf() && self.signum() != sign {
                return Err(RatioError::Indeterminate);
            }
            return Ok(Self::signed_infinity(if self.is_inf() {
                self.signum()
            } else {
                sign
            }));
        }
        let (x, y) = T::add_finite(&self.x, &self.y, &rhs.x, &rhs.y, subtract)?;
        Ok(Self { x, y })
    }
    pub fn try_add(&self, rhs: &Self) -> Result<Self, RatioError> {
        self.add_sub(rhs, false)
    }
    pub fn try_sub(&self, rhs: &Self) -> Result<Self, RatioError> {
        self.add_sub(rhs, true)
    }
    pub fn try_mul(&self, rhs: &Self) -> Result<Self, RatioError> {
        if self.is_inf() || rhs.is_inf() {
            if self.is_zero() || rhs.is_zero() {
                return Err(RatioError::Indeterminate);
            }
            return Ok(Self::signed_infinity(self.signum() * rhs.signum()));
        }
        let (x, y) = T::mul_finite(&self.x, &self.y, &rhs.x, &rhs.y, false)?;
        Ok(Self { x, y })
    }
    pub fn try_div(&self, rhs: &Self) -> Result<Self, RatioError> {
        if rhs.is_inf() {
            return if self.is_inf() {
                Err(RatioError::Indeterminate)
            } else {
                Ok(Self::zero())
            };
        }
        if self.is_inf() {
            return Ok(Self::signed_infinity(
                self.signum() * if rhs.signum() < 0 { -1 } else { 1 },
            ));
        }
        if rhs.is_zero() {
            return if self.is_zero() {
                Err(RatioError::Indeterminate)
            } else {
                Ok(Self::signed_infinity(self.signum()))
            };
        }
        let (x, y) = T::mul_finite(&self.x, &self.y, &rhs.x, &rhs.y, true)?;
        Ok(Self { x, y })
    }
    pub fn try_pow(&self, mut exponent: u64) -> Result<Self, RatioError> {
        let mut result = Self::one();
        let mut base = self.clone();
        while exponent > 0 {
            if exponent & 1 != 0 {
                result = result.try_mul(&base)?;
            }
            exponent >>= 1;
            if exponent != 0 {
                base = base.try_mul(&base)?;
            }
        }
        Ok(result)
    }
    pub fn pow(&self, exponent: u64) -> Self {
        self.try_pow(exponent).expect("invalid ratio power")
    }
    fn try_round(&self, mode: i8) -> Result<T, RatioError> {
        if self.is_inf() {
            Err(RatioError::NotFinite)
        } else {
            Ok(T::round(&self.x, &self.y, mode))
        }
    }
    pub fn try_floor(&self) -> Result<T, RatioError> {
        self.try_round(-1)
    }
    pub fn try_ceil(&self) -> Result<T, RatioError> {
        self.try_round(1)
    }
    pub fn try_trunc(&self) -> Result<T, RatioError> {
        self.try_round(0)
    }
    pub fn floor(&self) -> T {
        self.try_floor().expect("infinite ratio")
    }
    pub fn ceil(&self) -> T {
        self.try_ceil().expect("infinite ratio")
    }
    pub fn trunc(&self) -> T {
        self.try_trunc().expect("infinite ratio")
    }
}
impl<T: RatioInteger> Default for Rational<T> {
    fn default() -> Self {
        Self::zero()
    }
}
impl<T: RatioInteger> From<T> for Rational<T> {
    fn from(value: T) -> Self {
        Self::int(value)
    }
}
impl<T: RatioInteger> std::fmt::Display for Rational<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_inf() {
            f.write_str(if self.signum() < 0 { "-inf" } else { "inf" })
        } else if self.x == T::from(1) {
            std::fmt::Display::fmt(&self.y, f)
        } else {
            write!(f, "{}/{}", self.y, self.x)
        }
    }
}
impl<T: RatioInteger> std::str::FromStr for Rational<T> {
    type Err = RatioError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        match s {
            "inf" | "+inf" => return Ok(Self::infinity()),
            "-inf" => return Ok(Self::negative_infinity()),
            _ => (),
        }
        if let Some((n, d)) = s.split_once('/') {
            let y = n.trim().parse().map_err(|_| RatioError::InvalidFormat)?;
            let x = d.trim().parse().map_err(|_| RatioError::InvalidFormat)?;
            Self::try_new(x, y)
        } else {
            Ok(Self::int(
                s.parse::<T>().map_err(|_| RatioError::InvalidFormat)?,
            ))
        }
    }
}
impl<T: RatioInteger> Ord for Rational<T> {
    fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
        match (self.is_inf(), rhs.is_inf()) {
            (true, true) => self.y.cmp(&rhs.y),
            (true, false) => self.signum().cmp(&0),
            (false, true) => 0.cmp(&rhs.signum()),
            (false, false) => T::cmp_finite(&self.x, &self.y, &rhs.x, &rhs.y),
        }
    }
}
impl<T: RatioInteger> PartialOrd for Rational<T> {
    fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(rhs))
    }
}
impl<T: RatioInteger> std::ops::Neg for &Rational<T> {
    type Output = Rational<T>;
    fn neg(self) -> Self::Output {
        self.try_neg().expect("ratio overflow")
    }
}
impl<T: RatioInteger> std::ops::Neg for Rational<T> {
    type Output = Self;
    fn neg(self) -> Self {
        -&self
    }
}
macro_rules! ratio_ops {
    ($($op:ident,$method:ident,$try:ident,$assign:ident,$set:ident);* $(;)?) => {$(
        impl<T:RatioInteger> std::ops::$op for &Rational<T> {type Output=Rational<T>;fn $method(self,rhs:Self)->Self::Output{self.$try(rhs).expect("invalid ratio arithmetic")}}
        impl<T:RatioInteger> std::ops::$op for Rational<T> {type Output=Self;fn $method(self,rhs:Self)->Self{std::ops::$op::$method(&self,&rhs)}}
        impl<T:RatioInteger> std::ops::$op<&Self> for Rational<T> {type Output=Self;fn $method(self,rhs:&Self)->Self{std::ops::$op::$method(&self,rhs)}}
        impl<T:RatioInteger> std::ops::$op<Rational<T>> for &Rational<T> {type Output=Rational<T>;fn $method(self,rhs:Rational<T>)->Self::Output{std::ops::$op::$method(self,&rhs)}}
        impl<T:RatioInteger> std::ops::$assign for Rational<T> {fn $set(&mut self,rhs:Self){*self=std::ops::$op::$method(&*self,&rhs);}}
        impl<T:RatioInteger> std::ops::$assign<&Self> for Rational<T> {fn $set(&mut self,rhs:&Self){*self=std::ops::$op::$method(&*self,rhs);}}
        impl<T:RatioInteger> std::ops::$op<T> for Rational<T> {type Output=Self;fn $method(self,rhs:T)->Self{std::ops::$op::$method(&self,&Self::int(rhs))}}
        impl<T:RatioInteger> std::ops::$op<T> for &Rational<T> {type Output=Rational<T>;fn $method(self,rhs:T)->Self::Output{std::ops::$op::$method(self,&Rational::int(rhs))}}
        impl<T:RatioInteger> std::ops::$assign<T> for Rational<T> {fn $set(&mut self,rhs:T){*self=std::ops::$op::$method(&*self,rhs);}}
        impl<T:RatioInteger> std::ops::$op<&T> for Rational<T> {type Output=Self;fn $method(self,rhs:&T)->Self{std::ops::$op::$method(&self,rhs.clone())}}
        impl<T:RatioInteger> std::ops::$op<&T> for &Rational<T> {type Output=Rational<T>;fn $method(self,rhs:&T)->Self::Output{std::ops::$op::$method(self,rhs.clone())}}
        impl<T:RatioInteger> std::ops::$assign<&T> for Rational<T> {fn $set(&mut self,rhs:&T){*self=std::ops::$op::$method(&*self,rhs);}}
    )*};
}
ratio_ops!(Add,add,try_add,AddAssign,add_assign;Sub,sub,try_sub,SubAssign,sub_assign;Mul,mul,try_mul,MulAssign,mul_assign;Div,div,try_div,DivAssign,div_assign);
impl<T: RatioInteger> PartialEq<T> for Rational<T> {
    fn eq(&self, rhs: &T) -> bool {
        self.x == T::from(1) && self.y == *rhs
    }
}
impl<T: RatioInteger> PartialOrd<T> for Rational<T> {
    fn partial_cmp(&self, rhs: &T) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&Self::int(rhs.clone())))
    }
}

mod ratio_detail {
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
    pub(super) struct Wide {
        hi: u128,
        lo: u128,
    }
    impl Wide {
        pub fn from(x: u128) -> Self {
            Self { hi: 0, lo: x }
        }
        pub fn mul(a: u128, b: u128) -> Self {
            if let Some(lo) = a.checked_mul(b) {
                return Self::from(lo);
            }
            let mask = u64::MAX as u128;
            let (a0, a1, b0, b1) = (a & mask, a >> 64, b & mask, b >> 64);
            let p = a0 * b0;
            let q = a0 * b1;
            let r = a1 * b0;
            let mid = (p >> 64) + (q & mask) + (r & mask);
            Self {
                lo: (p & mask) | (mid << 64),
                hi: a1 * b1 + (q >> 64) + (r >> 64) + (mid >> 64),
            }
        }
        pub fn add(self, rhs: Self) -> Self {
            let (lo, c) = self.lo.overflowing_add(rhs.lo);
            Self {
                lo,
                hi: self.hi + rhs.hi + c as u128,
            }
        }
        pub fn sub(self, rhs: Self) -> Self {
            debug_assert!(self >= rhs);
            let (lo, c) = self.lo.overflowing_sub(rhs.lo);
            Self {
                lo,
                hi: self.hi - rhs.hi - c as u128,
            }
        }
        pub fn to_u128(self) -> Option<u128> {
            if self.hi == 0 {
                Some(self.lo)
            } else {
                None
            }
        }
        pub fn is_zero(self) -> bool {
            self.hi == 0 && self.lo == 0
        }
        pub fn div_rem(self, d: u128) -> (Self, u128) {
            assert!(d != 0);
            if self.hi == 0 {
                return (Self::from(self.lo / d), self.lo % d);
            }
            let mut q = Self::from(0);
            let mut r = 0u128;
            for i in (0..256).rev() {
                let bit = if i >= 128 {
                    (self.hi >> (i - 128)) & 1
                } else {
                    (self.lo >> i) & 1
                };
                let carry = r >> 127;
                r = (r << 1) | bit;
                if carry != 0 || r >= d {
                    r = r.wrapping_sub(d);
                    if i >= 128 {
                        q.hi |= 1u128 << (i - 128);
                    } else {
                        q.lo |= 1u128 << i;
                    }
                }
            }
            (q, r)
        }
    }
    pub(super) fn gcd(mut a: u128, mut b: u128) -> u128 {
        while b != 0 {
            let r = a % b;
            a = b;
            b = r;
        }
        a
    }
    pub(super) fn signed_sum(a: Wide, an: bool, b: Wide, bn: bool) -> (Wide, bool) {
        if an == bn {
            (a.add(b), an)
        } else if a >= b {
            (a.sub(b), an)
        } else {
            (b.sub(a), bn)
        }
    }
}
macro_rules! ratio_fixed {
    ($t:ty) => {
        impl RatioInteger for $t {
            fn normalize(x: Self, y: Self) -> Result<(Self, Self), RatioError> {
                if x == 0 {
                    return if y == 0 {
                        Err(RatioError::Indeterminate)
                    } else {
                        Ok((0, if y < 0 { -1 } else { 1 }))
                    };
                }
                if y == 0 {
                    return Ok((1, 0));
                }
                let g = ratio_detail::gcd(x.unsigned_abs() as u128, y.unsigned_abs() as u128);
                let d = x.unsigned_abs() as u128 / g;
                let n = y.unsigned_abs() as u128 / g;
                let neg = (x < 0) ^ (y < 0);
                let x = Self::try_from(d).map_err(|_| RatioError::Overflow)?;
                let y = Self::ratio_from_magnitude(n, neg)?;
                Ok((x, y))
            }
            fn negate(y: &Self) -> Result<Self, RatioError> {
                y.checked_neg().ok_or(RatioError::Overflow)
            }
            fn add_finite(
                x: &Self,
                y: &Self,
                u: &Self,
                v: &Self,
                subtract: bool,
            ) -> Result<(Self, Self), RatioError> {
                if *x == 1 && *u == 1 {
                    let value = if subtract {
                        y.checked_sub(*v)
                    } else {
                        y.checked_add(*v)
                    };
                    return value.map(|n| (1, n)).ok_or(RatioError::Overflow);
                }
                let (d, e) = (*x as u128, *u as u128);
                let g = ratio_detail::gcd(d, e);
                let a = ratio_detail::Wide::mul(y.unsigned_abs() as u128, e / g);
                let b = ratio_detail::Wide::mul(v.unsigned_abs() as u128, d / g);
                let (n, neg) = ratio_detail::signed_sum(a, *y < 0, b, (*v < 0) ^ subtract);
                if n.is_zero() {
                    return Ok((1, 0));
                }
                let h = ratio_detail::gcd(n.div_rem(g).1, g);
                let n = n.div_rem(h).0.to_u128().ok_or(RatioError::Overflow)?;
                let d = (d / g).checked_mul(e / h).ok_or(RatioError::Overflow)?;
                Ok((
                    Self::try_from(d).map_err(|_| RatioError::Overflow)?,
                    Self::ratio_from_magnitude(n, neg)?,
                ))
            }
            fn mul_finite(
                x: &Self,
                y: &Self,
                u: &Self,
                v: &Self,
                divide: bool,
            ) -> Result<(Self, Self), RatioError> {
                if !divide && *x == 1 && *u == 1 {
                    return y
                        .checked_mul(*v)
                        .map(|n| (1, n))
                        .ok_or(RatioError::Overflow);
                }
                let (mut n, mut d, mut m, mut e) = (
                    y.unsigned_abs() as u128,
                    *x as u128,
                    v.unsigned_abs() as u128,
                    *u as u128,
                );
                if divide {
                    std::mem::swap(&mut m, &mut e);
                }
                let g = ratio_detail::gcd(n, e);
                n /= g;
                e /= g;
                let h = ratio_detail::gcd(m, d);
                m /= h;
                d /= h;
                let n = n.checked_mul(m).ok_or(RatioError::Overflow)?;
                let d = d.checked_mul(e).ok_or(RatioError::Overflow)?;
                Ok((
                    Self::try_from(d).map_err(|_| RatioError::Overflow)?,
                    Self::ratio_from_magnitude(n, (*y < 0) ^ (*v < 0))?,
                ))
            }
            fn cmp_finite(x: &Self, y: &Self, u: &Self, v: &Self) -> std::cmp::Ordering {
                if x == u {
                    return y.cmp(v);
                }
                let a = (*y).cmp(&0);
                let b = (*v).cmp(&0);
                if a != b {
                    return a.cmp(&b);
                }
                let c = ratio_detail::Wide::mul(y.unsigned_abs() as u128, *u as u128).cmp(
                    &ratio_detail::Wide::mul(v.unsigned_abs() as u128, *x as u128),
                );
                if *y < 0 {
                    c.reverse()
                } else {
                    c
                }
            }
            fn round(x: &Self, y: &Self, mode: i8) -> Self {
                let q = y / x;
                let r = y % x;
                if r < 0 && mode < 0 {
                    q - 1
                } else if r > 0 && mode > 0 {
                    q + 1
                } else {
                    q
                }
            }
        }
        impl Rational<$t> {
            pub fn to_f64(&self) -> f64 {
                self.y as f64 / self.x as f64
            }
        }
        impl PartialEq<Rational<$t>> for $t {
            fn eq(&self, rhs: &Rational<$t>) -> bool {
                rhs == self
            }
        }
        impl PartialOrd<Rational<$t>> for $t {
            fn partial_cmp(&self, rhs: &Rational<$t>) -> Option<std::cmp::Ordering> {
                rhs.partial_cmp(self).map(std::cmp::Ordering::reverse)
            }
        }
    };
}
trait RatioSignedMagnitude: Sized {
    fn ratio_from_magnitude(n: u128, negative: bool) -> Result<Self, RatioError>;
}
macro_rules! ratio_signed_magnitude {($($t:ty),*)=>{$(impl RatioSignedMagnitude for $t {
    fn ratio_from_magnitude(n:u128,negative:bool)->Result<Self,RatioError>{
        if negative && n==(<$t>::MIN.unsigned_abs()as u128) {return Ok(<$t>::MIN);}
        let x=<$t>::try_from(n).map_err(|_|RatioError::Overflow)?;Ok(if negative {-x}else{x})
    }
})*};}
ratio_signed_magnitude!(i64, i128);
ratio_fixed!(i64);
ratio_fixed!(i128);
macro_rules! ratio_scalar_left {($($t:ty),*)=>{$(
    impl std::ops::Add<Rational<$t>> for $t {type Output=Rational<$t>;fn add(self,rhs:Self::Output)->Self::Output{Rational::int(self)+rhs}}
    impl std::ops::Sub<Rational<$t>> for $t {type Output=Rational<$t>;fn sub(self,rhs:Self::Output)->Self::Output{Rational::int(self)-rhs}}
    impl std::ops::Mul<Rational<$t>> for $t {type Output=Rational<$t>;fn mul(self,rhs:Self::Output)->Self::Output{Rational::int(self)*rhs}}
    impl std::ops::Div<Rational<$t>> for $t {type Output=Rational<$t>;fn div(self,rhs:Self::Output)->Self::Output{Rational::int(self)/rhs}}
    impl std::ops::Add<&Rational<$t>> for $t {type Output=Rational<$t>;fn add(self,rhs:&Self::Output)->Self::Output{Rational::int(self)+rhs}}
    impl std::ops::Sub<&Rational<$t>> for $t {type Output=Rational<$t>;fn sub(self,rhs:&Self::Output)->Self::Output{Rational::int(self)-rhs}}
    impl std::ops::Mul<&Rational<$t>> for $t {type Output=Rational<$t>;fn mul(self,rhs:&Self::Output)->Self::Output{Rational::int(self)*rhs}}
    impl std::ops::Div<&Rational<$t>> for $t {type Output=Rational<$t>;fn div(self,rhs:&Self::Output)->Self::Output{Rational::int(self)/rhs}}
)*};}
ratio_scalar_left!(i64, i128);
impl From<Ratio> for Ratio128 {
    fn from(r: Ratio) -> Self {
        Self {
            x: r.x as i128,
            y: r.y as i128,
        }
    }
}
impl TryFrom<Ratio128> for Ratio {
    type Error = RatioError;
    fn try_from(r: Ratio128) -> Result<Self, Self::Error> {
        Ok(Self {
            x: i64::try_from(r.x).map_err(|_| RatioError::Overflow)?,
            y: i64::try_from(r.y).map_err(|_| RatioError::Overflow)?,
        })
    }
}

pub fn floor(a: i64, b: i64) -> i64 {
    let q = a.checked_div(b).expect("invalid floor division");
    let r = a % b;
    if r != 0 && (r < 0) != (b < 0) {
        q - 1
    } else {
        q
    }
}

impl<T: RatioInteger> std::iter::Sum for Rational<T> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::zero(), |a, b| a + b)
    }
}
impl<'a, T: RatioInteger + 'a> std::iter::Sum<&'a Self> for Rational<T> {
    fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Self::zero(), |a, b| a + b)
    }
}
impl<T: RatioInteger> std::iter::Product for Rational<T> {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::one(), |a, b| a * b)
    }
}
impl<'a, T: RatioInteger + 'a> std::iter::Product<&'a Self> for Rational<T> {
    fn product<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Self::one(), |a, b| a * b)
    }
}


pub trait GeometryCoordinate: Copy + Ord {
    type Wide: GeometryNumber;
    fn wide(self) -> Self::Wide;
    fn cmp_turn(a: Point<Self>, b: Point<Self>, c: Point<Self>) -> std::cmp::Ordering {
        let (u, v) = (b.difference(a), c.difference(a));
        Self::Wide::cmp_products(u.x, v.y, u.y, v.x)
    }
}
pub trait GeometryNumber: GeometryCoordinate<Wide = Self> {
    fn zero() -> Self;
    fn add(self, rhs: Self) -> Self;
    fn sub(self, rhs: Self) -> Self;
    fn mul(self, rhs: Self) -> Self;
    fn neg(self) -> Self;
    fn rational(self) -> Ratio128;
    fn cmp_products(a: Self, b: Self, c: Self, d: Self) -> std::cmp::Ordering {
        a.mul(b).cmp(&c.mul(d))
    }

    fn abs(self) -> Self {
        if self < Self::zero() {
            self.neg()
        } else {
            self
        }
    }
    fn sign(self) -> i8 {
        match self.cmp(&Self::zero()) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        }
    }
}
impl GeometryCoordinate for i64 {
    type Wide = i128;
    fn wide(self) -> i128 {
        self as i128
    }
}
impl GeometryCoordinate for i128 {
    type Wide = i128;
    fn wide(self) -> i128 {
        self
    }
}
impl GeometryNumber for i128 {
    fn cmp_products(a: Self, b: Self, c: Self, d: Self) -> std::cmp::Ordering {
        geometry_cmp_integer_products(a, b, c, d)
    }

    fn zero() -> Self {
        0
    }
    fn add(self, rhs: Self) -> Self {
        self.checked_add(rhs).expect("geometry overflow")
    }
    fn sub(self, rhs: Self) -> Self {
        self.checked_sub(rhs).expect("geometry overflow")
    }
    fn mul(self, rhs: Self) -> Self {
        self.checked_mul(rhs).expect("geometry overflow")
    }
    fn neg(self) -> Self {
        self.checked_neg().expect("geometry overflow")
    }
    fn rational(self) -> Ratio128 {
        Ratio128::int(self)
    }
}
impl GeometryCoordinate for Ratio {
    fn cmp_turn(a: Point<Self>, b: Point<Self>, c: Point<Self>) -> std::cmp::Ordering {
        geometry_cmp_rational_turn(a.to_rational(), b.to_rational(), c.to_rational())
    }

    type Wide = Ratio128;
    fn wide(self) -> Ratio128 {
        assert!(self.is_finite(), "infinite coordinate");
        self.into()
    }
}
impl GeometryCoordinate for Ratio128 {
    fn cmp_turn(a: Point<Self>, b: Point<Self>, c: Point<Self>) -> std::cmp::Ordering {
        a.wide();
        b.wide();
        c.wide();
        geometry_cmp_rational_turn(a, b, c)
    }

    type Wide = Ratio128;
    fn wide(self) -> Self {
        assert!(self.is_finite(), "infinite coordinate");
        self
    }
}
impl GeometryNumber for Ratio128 {
    fn cmp_products(a: Self, b: Self, c: Self, d: Self) -> std::cmp::Ordering {
        assert!(a.is_finite() && b.is_finite() && c.is_finite() && d.is_finite());
        let fast = (|| {
            let left = a.numerator().checked_mul(*b.numerator())?;
            let right = c.numerator().checked_mul(*d.numerator())?;
            let ld = a.denominator().checked_mul(*b.denominator())?;
            let rd = c.denominator().checked_mul(*d.denominator())?;
            Some(geometry_cmp_integer_products(left, rd, right, ld))
        })();
        fast.unwrap_or_else(|| (a * b).cmp(&(c * d)))
    }

    fn zero() -> Self {
        Ratio128::zero()
    }
    fn add(self, rhs: Self) -> Self {
        self + rhs
    }
    fn sub(self, rhs: Self) -> Self {
        self - rhs
    }
    fn mul(self, rhs: Self) -> Self {
        self * rhs
    }
    fn neg(self) -> Self {
        -self
    }
    fn rational(self) -> Self {
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Point<T = i64> {
    pub x: T,
    pub y: T,
}
pub type IntPoint = Point<i64>;
pub type RationalPoint = Point<Ratio128>;
impl<T: GeometryCoordinate> Point<T> {
    pub fn new(x: T, y: T) -> Self {
        x.wide();
        y.wide();
        Self { x, y }
    }
    pub fn wide(self) -> Point<T::Wide> {
        Point {
            x: self.x.wide(),
            y: self.y.wide(),
        }
    }
    pub fn to_rational(self) -> RationalPoint {
        let p = self.wide();
        Point::new(p.x.rational(), p.y.rational())
    }
    pub fn translated(self, v: Self) -> Point<T::Wide> {
        let (p, v) = (self.wide(), v.wide());
        Point::new(p.x.add(v.x), p.y.add(v.y))
    }
    pub fn difference(self, other: Self) -> Point<T::Wide> {
        let (p, q) = (self.wide(), other.wide());
        Point::new(p.x.sub(q.x), p.y.sub(q.y))
    }
    pub fn scaled(self, k: T::Wide) -> Point<T::Wide> {
        let p = self.wide();
        let k = k.wide();
        Point::new(p.x.mul(k), p.y.mul(k))
    }
    pub fn rotated90(self) -> Point<T::Wide> {
        let p = self.wide();
        Point::new(p.y.neg(), p.x)
    }
    pub fn dot(self, other: Self) -> T::Wide {
        let (p, q) = (self.wide(), other.wide());
        p.x.mul(q.x).add(p.y.mul(q.y))
    }
    pub fn cross(self, other: Self) -> T::Wide {
        let (p, q) = (self.wide(), other.wide());
        p.x.mul(q.y).sub(p.y.mul(q.x))
    }
    pub fn norm_squared(self) -> T::Wide {
        self.dot(self)
    }
    pub fn norm(self) -> f64 {
        self.norm_squared().rational().to_f64().sqrt()
    }
    pub fn distance_squared(self, other: Self) -> T::Wide {
        self.difference(other).norm_squared()
    }
    pub fn distance(self, other: Self) -> f64 {
        self.distance_squared(other).rational().to_f64().sqrt()
    }
    pub fn manhattan_distance(self, other: Self) -> T::Wide {
        let p = self.difference(other);
        p.x.abs().add(p.y.abs())
    }
    pub fn midpoint(self, other: Self) -> RationalPoint {
        let (p, q) = (self.to_rational(), other.to_rational());
        Point::new((p.x + q.x) / 2, (p.y + q.y) / 2)
    }
    pub fn angle(self) -> Option<f64> {
        let p = self.to_rational();
        if p.x.is_zero() && p.y.is_zero() {
            None
        } else {
            Some(p.y.to_f64().atan2(p.x.to_f64()))
        }
    }
    pub fn angle_to(self, other: Self) -> Option<f64> {
        let delta = other.angle()? - self.angle()?;
        Some(delta.sin().atan2(delta.cos()))
    }

    pub fn cmp_angle(self, other: Self) -> Option<std::cmp::Ordering> {
        let (p, q) = (self.wide(), other.wide());
        let z = T::Wide::zero();
        if (p.x == z && p.y == z) || (q.x == z && q.y == z) {
            return None;
        }
        let half = |p: Point<T::Wide>| p.y < z || (p.y == z && p.x < z);
        let order = half(p).cmp(&half(q));
        Some(if order.is_eq() {
            T::Wide::cmp_products(p.x, q.y, p.y, q.x).reverse()
        } else {
            order
        })
    }
    pub fn is_parallel(self, other: Self) -> bool {
        self.cross(other) == T::Wide::zero()
    }
    pub fn is_perpendicular(self, other: Self) -> bool {
        self.dot(other) == T::Wide::zero()
    }
    pub fn divided(self, divisor: T) -> RationalPoint {
        let divisor = divisor.wide().rational();
        assert!(!divisor.is_zero(), "zero divisor");
        let p = self.to_rational();
        Point::new(p.x / divisor, p.y / divisor)
    }
    pub fn rotated(self, radians: f64) -> Point<f64> {
        let p = self.to_rational();
        let (s, c) = radians.sin_cos();
        Point {
            x: p.x.to_f64() * c - p.y.to_f64() * s,
            y: p.x.to_f64() * s + p.y.to_f64() * c,
        }
    }
}
impl<T: GeometryCoordinate> std::ops::Add for Point<T> {
    type Output = Point<T::Wide>;
    fn add(self, rhs: Self) -> Self::Output {
        self.translated(rhs)
    }
}
impl<T: GeometryCoordinate> std::ops::Sub for Point<T> {
    type Output = Point<T::Wide>;
    fn sub(self, rhs: Self) -> Self::Output {
        self.difference(rhs)
    }
}
impl<T: GeometryCoordinate> std::ops::Neg for Point<T> {
    type Output = Point<T::Wide>;
    fn neg(self) -> Self::Output {
        let p = self.wide();
        Point::new(p.x.neg(), p.y.neg())
    }
}
impl<T: GeometryCoordinate> std::ops::Mul<T> for Point<T> {
    type Output = Point<T::Wide>;
    fn mul(self, rhs: T) -> Self::Output {
        self.scaled(rhs.wide())
    }
}
impl<T: GeometryCoordinate> std::ops::Div<T> for Point<T> {
    type Output = RationalPoint;
    fn div(self, rhs: T) -> Self::Output {
        self.divided(rhs)
    }
}

pub fn orientation<T: GeometryCoordinate>(a: Point<T>, b: Point<T>, c: Point<T>) -> i8 {
    signed_triangle_area2(a, b, c).sign()
}
pub fn signed_triangle_area2<T: GeometryCoordinate>(
    a: Point<T>,
    b: Point<T>,
    c: Point<T>,
) -> T::Wide {
    b.difference(a).cross(c.difference(a))
}
pub fn triangle_area2<T: GeometryCoordinate>(a: Point<T>, b: Point<T>, c: Point<T>) -> T::Wide {
    signed_triangle_area2(a, b, c).abs()
}
pub fn triangle_area<T: GeometryCoordinate>(a: Point<T>, b: Point<T>, c: Point<T>) -> Ratio128 {
    triangle_area2(a, b, c).rational() / 2
}
pub fn triangle_centroid<T: GeometryCoordinate>(
    a: Point<T>,
    b: Point<T>,
    c: Point<T>,
) -> RationalPoint {
    (a.to_rational() + b.to_rational() + c.to_rational()) / Ratio128::int(3)
}
pub fn triangle_circumcenter<T: GeometryCoordinate>(
    a: Point<T>,
    b: Point<T>,
    c: Point<T>,
) -> Option<RationalPoint> {
    let (a, b, c) = (a.to_rational(), b.to_rational(), c.to_rational());
    let (u, v) = (b - a, c - a);
    let det = u.cross(v) * 2;
    if det.is_zero() {
        return None;
    }
    let (s, t) = (u.norm_squared(), v.norm_squared());
    Some(a + Point::new((s * v.y - t * u.y) / det, (u.x * t - v.x * s) / det))
}

pub fn signed_polygon_area2<T: GeometryCoordinate>(points: &[Point<T>]) -> T::Wide {
    let mut sum = T::Wide::zero();
    if points.len() < 3 {
        return sum;
    }
    for i in 0..points.len() {
        sum = sum.add(points[i].cross(points[(i + 1) % points.len()]));
    }
    sum
}
pub fn polygon_area<T: GeometryCoordinate>(points: &[Point<T>]) -> Ratio128 {
    signed_polygon_area2(points).abs().rational() / 2
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Polygon<T: GeometryCoordinate = i64> {
    vertices: Vec<Point<T>>,
}
impl<T: GeometryCoordinate> Polygon<T> {
    pub fn new(vertices: &[Point<T>]) -> Self {
        let mut vertices = vertices.to_vec();
        for p in &vertices {
            p.wide();
        }
        vertices.dedup();
        if vertices.len() > 1 && vertices.first() == vertices.last() {
            vertices.pop();
        }
        if signed_polygon_area2(&vertices) < T::Wide::zero() {
            vertices[1..].reverse();
        }
        Self { vertices }
    }
    pub fn vertices(&self) -> &[Point<T>] {
        &self.vertices
    }
    pub fn into_vertices(self) -> Vec<Point<T>> {
        self.vertices
    }
    pub fn len(&self) -> usize {
        self.vertices.len()
    }
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }
    pub fn area2(&self) -> T::Wide {
        signed_polygon_area2(&self.vertices).abs()
    }
    pub fn area(&self) -> Ratio128 {
        self.area2().rational() / 2
    }
    pub fn area_f64(&self) -> f64 {
        self.area().to_f64()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Ccw {
    CounterClockwise,
    Clockwise,
    Behind,
    Beyond,
    OnSegment,
}
pub fn ccw<T: GeometryCoordinate>(a: Point<T>, b: Point<T>, c: Point<T>) -> Ccw {
    let (u, v) = (b.difference(a), c.difference(a));
    match u.cross(v).sign() {
        1 => Ccw::CounterClockwise,
        -1 => Ccw::Clockwise,
        _ => {
            if u.dot(v) < T::Wide::zero() {
                Ccw::Behind
            } else if u.norm_squared() < v.norm_squared() {
                Ccw::Beyond
            } else {
                Ccw::OnSegment
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Line {
    a: i128,
    b: i128,
    c: i128,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LineIntersection {
    Parallel,
    Coincident,
    Point(RationalPoint),
}
impl Line {
    pub fn new(a: i128, b: i128, c: i128) -> Option<Self> {
        if a == 0 && b == 0 {
            return None;
        }
        let g = geometry_gcd(
            geometry_gcd(a.unsigned_abs(), b.unsigned_abs()),
            c.unsigned_abs(),
        );
        fn reduce(v: i128, g: u128) -> i128 {
            if g > i128::MAX as u128 {
                if v == 0 {
                    0
                } else {
                    -1
                }
            } else {
                v / (g as i128)
            }
        }
        let (mut a, mut b, mut c) = (reduce(a, g), reduce(b, g), reduce(c, g));
        if a < 0 || (a == 0 && b < 0) {
            a = GeometryNumber::neg(a);
            b = GeometryNumber::neg(b);
            c = GeometryNumber::neg(c);
        }
        Some(Self { a, b, c })
    }
    pub fn from_rational_coefficients(a: Ratio128, b: Ratio128, c: Ratio128) -> Option<Self> {
        assert!(a.is_finite() && b.is_finite() && c.is_finite());
        if a.is_zero() && b.is_zero() {
            return None;
        }
        let mut lcm = 1i128;
        for r in [a, b, c] {
            let d = *r.denominator();
            lcm = GeometryNumber::mul(lcm / (geometry_gcd(lcm as u128, d as u128) as i128), d);
        }
        Self::new(
            GeometryNumber::mul(*a.numerator(), lcm / *a.denominator()),
            GeometryNumber::mul(*b.numerator(), lcm / *b.denominator()),
            GeometryNumber::mul(*c.numerator(), lcm / *c.denominator()),
        )
    }
    pub fn through<T: GeometryCoordinate>(p: Point<T>, q: Point<T>) -> Option<Self> {
        let (p, q) = (p.wide(), q.wide());
        let a = p.y.sub(q.y);
        let b = q.x.sub(p.x);
        let c = p.x.mul(q.y).sub(p.y.mul(q.x));
        Self::from_rational_coefficients(a.rational(), b.rational(), c.rational())
    }
    pub fn coefficients(self) -> (i128, i128, i128) {
        (self.a, self.b, self.c)
    }
    pub fn evaluate<T: GeometryCoordinate>(self, p: Point<T>) -> Ratio128 {
        let p = p.to_rational();
        Ratio128::int(self.a) * p.x + Ratio128::int(self.b) * p.y + Ratio128::int(self.c)
    }
    pub fn contains<T: GeometryCoordinate>(self, p: Point<T>) -> bool {
        self.evaluate(p).is_zero()
    }
    pub fn side<T: GeometryCoordinate>(self, p: Point<T>) -> i8 {
        self.evaluate(p).signum()
    }

    pub fn cmp_angle(self, other: Self) -> std::cmp::Ordering {
        match (self.a == 0, other.a == 0) {
            (true, true) => std::cmp::Ordering::Equal,
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            (false, false) => geometry_cmp_integer_products(self.b, other.a, other.b, self.a),
        }
    }
    pub fn cmp_slope(self, other: Self) -> std::cmp::Ordering {
        match (self.b == 0, other.b == 0) {
            (true, true) => std::cmp::Ordering::Equal,
            (true, false) => std::cmp::Ordering::Greater,
            (false, true) => std::cmp::Ordering::Less,
            (false, false) => {
                let c = geometry_cmp_integer_products(self.a, other.b, other.a, self.b).reverse();
                if (self.b < 0) != (other.b < 0) {
                    c.reverse()
                } else {
                    c
                }
            }
        }
    }
    pub fn is_parallel(self, other: Self) -> bool {
        self.normal().cross(other.normal()).is_zero()
    }
    pub fn is_perpendicular(self, other: Self) -> bool {
        self.normal().dot(other.normal()).is_zero()
    }
    pub fn normal(self) -> RationalPoint {
        Point::new(Ratio128::int(self.a), Ratio128::int(self.b))
    }
    pub fn direction(self) -> RationalPoint {
        self.normal().rotated90()
    }
    pub fn intersection(self, other: Self) -> LineIntersection {
        let (a, b, c) = (
            Ratio128::int(self.a),
            Ratio128::int(self.b),
            Ratio128::int(self.c),
        );
        let (d, e, f) = (
            Ratio128::int(other.a),
            Ratio128::int(other.b),
            Ratio128::int(other.c),
        );
        let det = a * e - b * d;
        if det.is_zero() {
            if self == other {
                LineIntersection::Coincident
            } else {
                LineIntersection::Parallel
            }
        } else {
            LineIntersection::Point(Point::new((b * f - c * e) / det, (c * d - a * f) / det))
        }
    }
    pub fn projection<T: GeometryCoordinate>(self, p: Point<T>) -> RationalPoint {
        let p = p.to_rational();
        let n = self.normal();
        p - n * (self.evaluate(p) / n.norm_squared())
    }
    pub fn reflection<T: GeometryCoordinate>(self, p: Point<T>) -> RationalPoint {
        let p = p.to_rational();
        self.projection(p) * Ratio128::int(2) - p
    }
    pub fn distance_squared<T: GeometryCoordinate>(self, p: Point<T>) -> Ratio128 {
        let e = self.evaluate(p);
        e * e / self.normal().norm_squared()
    }
    pub fn distance<T: GeometryCoordinate>(self, p: Point<T>) -> f64 {
        self.distance_squared(p).to_f64().sqrt()
    }
    pub fn parallel_through<T: GeometryCoordinate>(self, p: Point<T>) -> Self {
        let p = p.to_rational();
        let (a, b) = (Ratio128::int(self.a), Ratio128::int(self.b));
        Self::from_rational_coefficients(a, b, -(a * p.x + b * p.y)).unwrap()
    }
    pub fn perpendicular_through<T: GeometryCoordinate>(self, p: Point<T>) -> Self {
        let p = p.to_rational();
        let (a, b) = (Ratio128::int(self.b), -Ratio128::int(self.a));
        Self::from_rational_coefficients(a, b, -(a * p.x + b * p.y)).unwrap()
    }
}

fn geometry_cmp_rational_turn(
    a: RationalPoint,
    b: RationalPoint,
    c: RationalPoint,
) -> std::cmp::Ordering {
    let values = [a.x, a.y, b.x, b.y, c.x, c.y];
    let fast = (|| {
        let mut denominator = 1i128;
        for value in values {
            let d = *value.denominator();
            denominator = (denominator / (geometry_gcd(denominator as u128, d as u128) as i128))
                .checked_mul(d)?;
        }
        let mut v = [0i128; 6];
        for (i, value) in values.iter().enumerate() {
            v[i] = value
                .numerator()
                .checked_mul(denominator / *value.denominator())?;
        }
        Some(geometry_cmp_integer_products(
            v[2].checked_sub(v[0])?,
            v[5].checked_sub(v[1])?,
            v[3].checked_sub(v[1])?,
            v[4].checked_sub(v[0])?,
        ))
    })();
    fast.unwrap_or_else(|| {
        let (u, v) = (b - a, c - a);
        Ratio128::cmp_products(u.x, v.y, u.y, v.x)
    })
}
fn geometry_cmp_integer_products(a: i128, b: i128, c: i128, d: i128) -> std::cmp::Ordering {
    if let (Some(left), Some(right)) = (a.checked_mul(b), c.checked_mul(d)) {
        return left.cmp(&right);
    }

    let sign = |a: i128, b: i128| {
        if a == 0 || b == 0 {
            0i8
        } else if (a < 0) != (b < 0) {
            -1
        } else {
            1
        }
    };
    let (s, t) = (sign(a, b), sign(c, d));
    let order = s.cmp(&t);
    if !order.is_eq() || s == 0 {
        return order;
    }
    let order = geometry_cmp_unsigned_fractions(
        a.unsigned_abs(),
        c.unsigned_abs(),
        d.unsigned_abs(),
        b.unsigned_abs(),
    );
    if s < 0 {
        order.reverse()
    } else {
        order
    }
}
fn geometry_cmp_unsigned_fractions(
    mut a: u128,
    mut b: u128,
    mut c: u128,
    mut d: u128,
) -> std::cmp::Ordering {
    let mut reverse = false;
    loop {
        let order = (a / b).cmp(&(c / d));
        if !order.is_eq() {
            return if reverse { order.reverse() } else { order };
        }
        let (r, s) = (a % b, c % d);
        if r == 0 || s == 0 {
            let order = (r != 0).cmp(&(s != 0));
            return if reverse { order.reverse() } else { order };
        }
        (a, b, c, d) = (b, r, d, s);
        reverse = !reverse;
    }
}

fn geometry_gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Segment<T = i64> {
    pub a: Point<T>,
    pub b: Point<T>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SegmentIntersection {
    None,
    Point(RationalPoint),
    Overlap(Segment<Ratio128>),
}
impl<T: GeometryCoordinate> Segment<T> {
    pub fn new(a: Point<T>, b: Point<T>) -> Self {
        a.wide();
        b.wide();
        Self { a, b }
    }

    pub fn cmp_angle(self, other: Self) -> Option<std::cmp::Ordering> {
        self.b
            .difference(self.a)
            .cmp_angle(other.b.difference(other.a))
    }
    pub fn line(self) -> Option<Line> {
        Line::through(self.a, self.b)
    }
    pub fn length_squared(self) -> T::Wide {
        self.a.distance_squared(self.b)
    }
    pub fn length(self) -> f64 {
        self.a.distance(self.b)
    }
    pub fn contains(self, p: Point<T>) -> bool {
        orientation(self.a, self.b, p) == 0
            && p.x >= self.a.x.min(self.b.x)
            && p.x <= self.a.x.max(self.b.x)
            && p.y >= self.a.y.min(self.b.y)
            && p.y <= self.a.y.max(self.b.y)
    }
    pub fn intersects(self, other: Self) -> bool {
        let (a, b, c, d) = (self.a, self.b, other.a, other.b);
        let (s, t, u, v) = (
            orientation(a, b, c),
            orientation(a, b, d),
            orientation(c, d, a),
            orientation(c, d, b),
        );
        (s * t < 0 && u * v < 0)
            || self.contains(c)
            || self.contains(d)
            || other.contains(a)
            || other.contains(b)
    }
    pub fn intersection(self, other: Self) -> SegmentIntersection {
        if !self.intersects(other) {
            return SegmentIntersection::None;
        }
        let mut common = Vec::new();
        for p in [self.a, self.b, other.a, other.b] {
            if self.contains(p) && other.contains(p) {
                common.push(p);
            }
        }
        common.sort();
        common.dedup();
        if let Some(&p) = common.first() {
            let q = *common.last().unwrap();
            return if p == q {
                SegmentIntersection::Point(p.to_rational())
            } else {
                SegmentIntersection::Overlap(Segment::new(p.to_rational(), q.to_rational()))
            };
        }
        match self.line().unwrap().intersection(other.line().unwrap()) {
            LineIntersection::Point(p) => SegmentIntersection::Point(p),
            _ => unreachable!(),
        }
    }
    pub fn closest_point(self, p: Point<T>) -> RationalPoint {
        let (a, b, p) = (self.a.to_rational(), self.b.to_rational(), p.to_rational());
        let v = b - a;
        let n = v.norm_squared();
        if n.is_zero() {
            return a;
        }
        let t = ((p - a).dot(v) / n)
            .max(Ratio128::zero())
            .min(Ratio128::one());
        a + v * t
    }
    pub fn distance_squared(self, p: Point<T>) -> Ratio128 {
        self.closest_point(p).distance_squared(p.to_rational())
    }
    pub fn distance(self, p: Point<T>) -> f64 {
        self.distance_squared(p).to_f64().sqrt()
    }
    pub fn segment_distance_squared(self, other: Self) -> Ratio128 {
        if self.intersects(other) {
            Ratio128::zero()
        } else {
            self.distance_squared(other.a)
                .min(self.distance_squared(other.b))
                .min(other.distance_squared(self.a))
                .min(other.distance_squared(self.b))
        }
    }
    pub fn segment_distance(self, other: Self) -> f64 {
        self.segment_distance_squared(other).to_f64().sqrt()
    }
}


fn hull_turn<T: GeometryCoordinate>(a: Point<T>, b: Point<T>, c: Point<T>) -> std::cmp::Ordering {
    T::cmp_turn(a,b,c)
}

pub fn convex_hull_chains<T: GeometryCoordinate>(
    points: &[Point<T>],
) -> (Vec<Point<T>>, Vec<Point<T>>) {
    let mut points = points.to_vec();
    for p in &points {
        p.wide();
    }
    points.sort_unstable();
    points.dedup();
    hull_chains_from_sorted(&points)
}

fn hull_chains_from_sorted<T: GeometryCoordinate>(
    points: &[Point<T>],
) -> (Vec<Point<T>>, Vec<Point<T>>) {
    let (mut upper, mut lower) = (Vec::new(), Vec::new());
    for &p in points {
        while upper.len() >= 2
            && !hull_turn(upper[upper.len() - 2], upper[upper.len() - 1], p).is_lt()
        {
            upper.pop();
        }
        while lower.len() >= 2
            && !hull_turn(lower[lower.len() - 2], lower[lower.len() - 1], p).is_gt()
        {
            lower.pop();
        }
        upper.push(p);
        lower.push(p);
    }
    (upper, lower)
}

pub fn convex_hull_polygon<T: GeometryCoordinate>(points: &[Point<T>]) -> Vec<Point<T>> {
    let (upper, lower) = convex_hull_chains(points);
    hull_join_chains(upper, lower)
}
fn hull_join_chains<T: GeometryCoordinate>(
    upper: Vec<Point<T>>,
    mut lower: Vec<Point<T>>,
) -> Vec<Point<T>> {
    if upper.len() > 2 {
        lower.extend(upper[1..upper.len() - 1].iter().rev().copied());
    }
    lower
}

pub fn convex_hull(points: &[(i64, i64)]) -> (Vec<(i64, i64)>, Vec<(i64, i64)>) {
    let points: Vec<_> = points.iter().map(|&(x, y)| IntPoint::new(x, y)).collect();
    let (upper, lower) = convex_hull_chains(&points);
    let tuples = |points: Vec<IntPoint>| points.into_iter().map(|p| (p.x, p.y)).collect();
    (tuples(upper), tuples(lower))
}
pub fn cross_product(a: (i64, i64), b: (i64, i64), c: (i64, i64)) -> i128 {
    signed_triangle_area2(
        IntPoint::new(a.0, a.1),
        IntPoint::new(b.0, b.1),
        IntPoint::new(c.0, c.1),
    )
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConvexHull<T: GeometryCoordinate = i64> {
    vertices: Vec<Point<T>>,
}
impl<T: GeometryCoordinate> ConvexHull<T> {
    pub fn new(points: &[Point<T>]) -> Self {
        Self {
            vertices: convex_hull_polygon(points),
        }
    }
    pub fn vertices(&self) -> &[Point<T>] {
        &self.vertices
    }
    pub fn into_vertices(self) -> Vec<Point<T>> {
        self.vertices
    }
    pub fn len(&self) -> usize {
        self.vertices.len()
    }
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }

    pub fn merge(&self, other: &Self) -> Self {
        let a = hull_sorted_vertices(&self.vertices);
        let b = hull_sorted_vertices(&other.vertices);
        let points = hull_merge_sorted(a.into_iter(), b.into_iter());
        let (upper, lower) = hull_chains_from_sorted(&points);
        Self {
            vertices: hull_join_chains(upper, lower),
        }
    }
    pub fn area2(&self) -> T::Wide {
        signed_polygon_area2(&self.vertices).abs()
    }
    pub fn area(&self) -> Ratio128 {
        self.area2().rational() / 2
    }
    pub fn area_f64(&self) -> f64 {
        self.area().to_f64()
    }
    pub fn to_polygon(&self) -> Polygon<T> {
        Polygon::new(&self.vertices)
    }
    pub fn scaled<C: HullCoefficient<T>>(&self, coefficient: C) -> ConvexHull<C::Output> {
        coefficient.validate();
        let vertices = self
            .vertices
            .iter()
            .map(|&p| coefficient.scale(p))
            .collect();
        ConvexHull {
            vertices: hull_normalize_cycle(vertices),
        }
    }
    pub fn weighted_minkowski_sum<C: HullCoefficient<T>>(
        &self,
        alpha: C,
        other: &Self,
        beta: C,
    ) -> ConvexHull<C::Output> {
        let (a, b) = (self.scaled(alpha), other.scaled(beta));
        hull_merge_minkowski(&a, &b)
    }
    pub fn minkowski_sum(&self, other: &Self) -> ConvexHull<T::Wide>
    where
        i64: HullCoefficient<T, Output = T::Wide>,
    {
        self.weighted_minkowski_sum(1i64, other, 1i64)
    }
}

fn hull_sorted_vertices<T: GeometryCoordinate>(points: &[Point<T>]) -> Vec<Point<T>> {
    let Some((right, _)) = points.iter().enumerate().max_by_key(|(_, p)| **p) else {
        return Vec::new();
    };
    let lower = points[..=right].iter().copied();
    let upper = std::iter::once(points[0]).chain(points[right..].iter().rev().copied());
    hull_merge_sorted(lower, upper)
}
fn hull_merge_sorted<T: GeometryCoordinate>(
    left: impl Iterator<Item = Point<T>>,
    right: impl Iterator<Item = Point<T>>,
) -> Vec<Point<T>> {
    let (mut left, mut right) = (left.peekable(), right.peekable());
    let mut points = Vec::new();
    loop {
        let point = match (left.peek(), right.peek()) {
            (Some(a), Some(b)) => {
                if a <= b {
                    left.next().unwrap()
                } else {
                    right.next().unwrap()
                }
            }
            (Some(_), None) => left.next().unwrap(),
            (None, Some(_)) => right.next().unwrap(),
            (None, None) => break,
        };
        if points.last() != Some(&point) {
            points.push(point);
        }
    }
    points
}

pub trait HullScalar: GeometryNumber {
    fn from_integer(value: i128) -> Self;
}
impl HullScalar for i128 {
    fn from_integer(value: i128) -> Self {
        value
    }
}
impl HullScalar for Ratio128 {
    fn from_integer(value: i128) -> Self {
        Ratio128::int(value)
    }
}
pub trait HullCoefficient<T: GeometryCoordinate>: Copy {
    type Output: GeometryNumber;
    fn validate(self) {}
    fn scale(self, point: Point<T>) -> Point<Self::Output>;
}
impl<T: GeometryCoordinate> HullCoefficient<T> for i64
where
    T::Wide: HullScalar,
{
    type Output = T::Wide;
    fn scale(self, point: Point<T>) -> Point<Self::Output> {
        point.scaled(T::Wide::from_integer(self as i128))
    }
}
impl<T: GeometryCoordinate> HullCoefficient<T> for i128
where
    T::Wide: HullScalar,
{
    type Output = T::Wide;
    fn scale(self, point: Point<T>) -> Point<Self::Output> {
        point.scaled(T::Wide::from_integer(self))
    }
}
impl<T: GeometryCoordinate> HullCoefficient<T> for Ratio {
    type Output = Ratio128;
    fn validate(self) {
        self.wide();
    }
    fn scale(self, point: Point<T>) -> RationalPoint {
        point.to_rational().scaled(self.wide())
    }
}
impl<T: GeometryCoordinate> HullCoefficient<T> for Ratio128 {
    type Output = Ratio128;
    fn validate(self) {
        self.wide();
    }
    fn scale(self, point: Point<T>) -> RationalPoint {
        point.to_rational().scaled(self.wide())
    }
}

fn hull_normalize_cycle<T: GeometryNumber>(mut points: Vec<Point<T>>) -> Vec<Point<T>> {
    points.dedup();
    if points.len() > 1 && points.first() == points.last() {
        points.pop();
    }
    let mut stack = Vec::with_capacity(points.len());
    for p in points {
        while stack.len() >= 2
            && hull_turn(stack[stack.len() - 2], stack[stack.len() - 1], p).is_eq()
        {
            stack.pop();
        }
        stack.push(p);
    }
    while stack.len() > 2
        && hull_turn(stack[stack.len() - 2], stack[stack.len() - 1], stack[0]).is_eq()
    {
        stack.pop();
    }
    let mut start = 0;
    while stack.len() - start > 2
        && hull_turn(stack[stack.len() - 1], stack[start], stack[start + 1]).is_eq()
    {
        start += 1;
    }
    if start > 0 {
        stack.drain(..start);
    }
    if let Some((index, _)) = stack.iter().enumerate().min_by_key(|(_, p)| **p) {
        stack.rotate_left(index);
    }
    stack
}

fn hull_edge_start<T: GeometryCoordinate>(points: &[Point<T>]) -> usize {
    points
        .iter()
        .enumerate()
        .min_by_key(|(_, p)| (p.y, p.x))
        .unwrap()
        .0
}
fn hull_merge_minkowski<T: GeometryNumber>(a: &ConvexHull<T>, b: &ConvexHull<T>) -> ConvexHull<T> {
    if a.is_empty() || b.is_empty() {
        return ConvexHull {
            vertices: Vec::new(),
        };
    }
    if a.len() == 1 {
        return ConvexHull {
            vertices: b.vertices.iter().map(|&p| p + a.vertices[0]).collect(),
        };
    }
    if b.len() == 1 {
        return ConvexHull {
            vertices: a.vertices.iter().map(|&p| p + b.vertices[0]).collect(),
        };
    }
    let (sa, sb) = (hull_edge_start(&a.vertices), hull_edge_start(&b.vertices));
    let (mut i, mut j) = (0, 0);
    let mut vertices = Vec::with_capacity(a.len() + b.len());
    while i < a.len() || j < b.len() {
        vertices.push(a.vertices[(sa + i) % a.len()] + b.vertices[(sb + j) % b.len()]);
        let order = if i == a.len() {
            std::cmp::Ordering::Greater
        } else if j == b.len() {
            std::cmp::Ordering::Less
        } else {
            let u = a.vertices[(sa + i + 1) % a.len()] - a.vertices[(sa + i) % a.len()];
            let v = b.vertices[(sb + j + 1) % b.len()] - b.vertices[(sb + j) % b.len()];
            u.cmp_angle(v).expect("zero edge in convex hull")
        };
        if !order.is_gt() {
            i += 1;
        }
        if !order.is_lt() {
            j += 1;
        }
    }
    ConvexHull {
        vertices: hull_normalize_cycle(vertices),
    }
}


fn dfs(points: &[Point<i128>], p: i128, q: i128) -> (ConvexHull<i128>, ConvexHull<i128>) {
    if points.len() == 1 {
        return (ConvexHull::new(points), ConvexHull::new(&[]));
    }
    let m = points.len() / 2;
    let (a, left) = dfs(&points[..m], p, q);
    let (b, right) = dfs(&points[m..], p, q);
    let crossing = a.weighted_minkowski_sum(q, &b, p);
    (a.merge(&b), left.merge(&right).merge(&crossing))
}
fn main() {
    use std::io::Read;
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    let mut it = input.split_whitespace();
    let n: usize = it.next().unwrap().parse().unwrap();
    let p: i128 = it.next().unwrap().parse().unwrap();
    let q: i128 = it.next().unwrap().parse().unwrap();
    let points: Vec<_> = (0..n)
        .map(|_| {
            Point::new(
                it.next().unwrap().parse::<i128>().unwrap(),
                it.next().unwrap().parse::<i128>().unwrap(),
            )
        })
        .collect();
    let (_, hull) = dfs(&points, p, q);
    let area = Ratio128::from_fraction(hull.area2(), 2 * (p + q) * (p + q));
    println!("{} {}", area.numerator(), area.denominator());
}
