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
