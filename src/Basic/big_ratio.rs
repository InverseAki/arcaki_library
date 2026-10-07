pub type BigRatio = Rational<BigInt>;
impl<const B: u32> RatioInteger for RadixBigInt<B> {
    fn normalize(mut x: Self, mut y: Self) -> Result<(Self, Self), RatioError> {
        if x.is_zero() {
            return if y.is_zero() {
                Err(RatioError::Indeterminate)
            } else {
                Ok((Self::zero(), Self::from(y.signum())))
            };
        }
        if y.is_zero() {
            return Ok((Self::one(), Self::zero()));
        }
        if x.is_negative() {
            x = -x;
            y = -y;
        }
        let g = x.gcd(&y);
        if g != Self::one() {
            x /= &g;
            y /= &g;
        }
        Ok((x, y))
    }
    fn negate(y: &Self) -> Result<Self, RatioError> {
        Ok(-y)
    }
    fn add_finite(
        x: &Self,
        y: &Self,
        u: &Self,
        v: &Self,
        subtract: bool,
    ) -> Result<(Self, Self), RatioError> {
        let g = x.gcd(u);
        let a = x / &g;
        let b = u / &g;
        let left = y * &b;
        let right = v * &a;
        let n = if subtract { left - right } else { left + right };
        if n.is_zero() {
            return Ok((Self::one(), Self::zero()));
        }
        let h = n.gcd(&g);
        Ok((&a * (u / &h), n / &h))
    }
    fn mul_finite(
        x: &Self,
        y: &Self,
        u: &Self,
        v: &Self,
        divide: bool,
    ) -> Result<(Self, Self), RatioError> {
        let (m, e) = if divide { (u, v) } else { (v, u) };
        let g = y.gcd(e);
        let h = m.gcd(x);
        let mut d = (x / &h) * (e / &g);
        let mut n = (y / &g) * (m / &h);
        if d.is_negative() {
            d = -d;
            n = -n;
        }
        Ok((d, n))
    }
    fn cmp_finite(x: &Self, y: &Self, u: &Self, v: &Self) -> std::cmp::Ordering {
        if x == u {
            return y.cmp(v);
        }
        let a = y.signum();
        let b = v.signum();
        if a != b {
            return a.cmp(&b);
        }
        (y * u).cmp(&(v * x))
    }
    fn round(x: &Self, y: &Self, mode: i8) -> Self {
        let (q, r) = y.div_rem(x);
        if !r.is_zero() && (mode > 0 || (mode == 0 && y.is_negative())) {
            q + Self::one()
        } else {
            q
        }
    }
}
macro_rules! big_ratio_conversion {($($t:ty),*)=>{$(
    impl<const B:u32> From<Rational<$t>> for Rational<RadixBigInt<B>> {
        fn from(r:Rational<$t>)->Self{Self{x:RadixBigInt::from(r.x),y:RadixBigInt::from(r.y)}}
    }
    impl<const B:u32> TryFrom<Rational<RadixBigInt<B>>> for Rational<$t> {
        type Error=RatioError;
        fn try_from(r:Rational<RadixBigInt<B>>)->Result<Self,RatioError>{
            let x=r.x.to_i128().and_then(|x|<$t>::try_from(x).ok()).ok_or(RatioError::Overflow)?;
            let y=r.y.to_i128().and_then(|y|<$t>::try_from(y).ok()).ok_or(RatioError::Overflow)?;
            Ok(Self{x,y})
        }
    }
)*};}
big_ratio_conversion!(i64, i128);

macro_rules! big_ratio_scalar {
    ($($t:ty),* $(,)?) => {$(
        impl<const B:u32> From<$t> for Rational<RadixBigInt<B>> {fn from(x:$t)->Self{Self::int(RadixBigInt::from(x))}}
        impl<const B:u32> PartialEq<$t> for Rational<RadixBigInt<B>> {fn eq(&self,rhs:&$t)->bool{self==&RadixBigInt::from(*rhs)}}
        impl<const B:u32> PartialOrd<$t> for Rational<RadixBigInt<B>> {fn partial_cmp(&self,rhs:&$t)->Option<std::cmp::Ordering>{self.partial_cmp(&RadixBigInt::from(*rhs))}}
        impl<const B:u32> PartialEq<Rational<RadixBigInt<B>>> for $t {fn eq(&self,rhs:&Rational<RadixBigInt<B>>)->bool{rhs==self}}
        impl<const B:u32> PartialOrd<Rational<RadixBigInt<B>>> for $t {fn partial_cmp(&self,rhs:&Rational<RadixBigInt<B>>)->Option<std::cmp::Ordering>{rhs.partial_cmp(self).map(std::cmp::Ordering::reverse)}}
        big_ratio_scalar_ops!($t;Add,add,AddAssign,add_assign;Sub,sub,SubAssign,sub_assign;Mul,mul,MulAssign,mul_assign;Div,div,DivAssign,div_assign);
    )*};
}
macro_rules! big_ratio_scalar_ops {
    ($t:ty;$($op:ident,$method:ident,$assign:ident,$set:ident);* $(;)?)=>{$(
        impl<const B:u32> std::ops::$op<$t> for Rational<RadixBigInt<B>> {type Output=Self;fn $method(self,rhs:$t)->Self{std::ops::$op::$method(self,RadixBigInt::from(rhs))}}
        impl<const B:u32> std::ops::$op<$t> for &Rational<RadixBigInt<B>> {type Output=Rational<RadixBigInt<B>>;fn $method(self,rhs:$t)->Self::Output{std::ops::$op::$method(self,RadixBigInt::from(rhs))}}
        impl<const B:u32> std::ops::$assign<$t> for Rational<RadixBigInt<B>> {fn $set(&mut self,rhs:$t){std::ops::$assign::$set(self,RadixBigInt::from(rhs));}}
        impl<const B:u32> std::ops::$op<Rational<RadixBigInt<B>>> for $t {type Output=Rational<RadixBigInt<B>>;fn $method(self,rhs:Self::Output)->Self::Output{std::ops::$op::$method(Rational::int(RadixBigInt::from(self)),rhs)}}
        impl<const B:u32> std::ops::$op<&Rational<RadixBigInt<B>>> for $t {type Output=Rational<RadixBigInt<B>>;fn $method(self,rhs:&Self::Output)->Self::Output{std::ops::$op::$method(&Rational::int(RadixBigInt::from(self)),rhs)}}
    )*};
}
big_ratio_scalar!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
impl<const B: u32> PartialEq<Rational<RadixBigInt<B>>> for RadixBigInt<B> {
    fn eq(&self, rhs: &Rational<Self>) -> bool {
        rhs == self
    }
}
impl<const B: u32> PartialOrd<Rational<RadixBigInt<B>>> for RadixBigInt<B> {
    fn partial_cmp(&self, rhs: &Rational<Self>) -> Option<std::cmp::Ordering> {
        rhs.partial_cmp(self).map(std::cmp::Ordering::reverse)
    }
}
macro_rules! big_ratio_integer_left {
    ($($op:ident,$method:ident);* $(;)?)=>{$(
        impl<const B:u32> std::ops::$op<Rational<Self>> for RadixBigInt<B> {type Output=Rational<Self>;fn $method(self,rhs:Self::Output)->Self::Output{std::ops::$op::$method(Rational::int(self),rhs)}}
        impl<const B:u32> std::ops::$op<&Rational<Self>> for RadixBigInt<B> {type Output=Rational<Self>;fn $method(self,rhs:&Self::Output)->Self::Output{std::ops::$op::$method(&Rational::int(self),rhs)}}
        impl<const B:u32> std::ops::$op<&Rational<RadixBigInt<B>>> for &RadixBigInt<B> {type Output=Rational<RadixBigInt<B>>;fn $method(self,rhs:&Self::Output)->Self::Output{std::ops::$op::$method(&Rational::int(self.clone()),rhs)}}
    )*};
}
big_ratio_integer_left!(Add,add;Sub,sub;Mul,mul;Div,div);
