// Source: https://github.com/InverseAki/arcaki_library/blob/main/src/NumberTheory/mint.rs
use std::{ops::*, fmt::{Display, Formatter}, sync::Once};

#[derive(Copy, Clone, Debug)]
pub struct Mint<const M: usize>{
    val: usize,
}

impl<const M: usize> Display for Mint<M>{
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.val)
    }
}

impl<const M: usize> Mint<M>{
    #[inline(always)]
    pub fn new(v: usize) -> Self{
        Self{val: v % M}
    }

    #[inline(always)]
    pub fn val(&self) -> usize{
        self.val
    }

    pub fn pow_u(&self, mut x: usize)->Self{
        let mut v = self.val();
        let mut res = 1;
        while x > 0{
            if x&1==1{
                res = (res*v)%M;
            }
            v = (v*v)%M;
            x >>= 1;
        }
        Self{val: res}
    }

    pub fn pow(&self, other: Self)->Self{
        self.pow_u(other.val)
    }

    pub fn inv(&self)->Self{
        self.pow_u(M-2)
    }
}

impl<const M: usize> Add for Mint<M>{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self{
            val: (self.val+rhs.val)%M,
        }
    }
}

impl<const M: usize> AddAssign for Mint<M>{
    fn add_assign(&mut self, rhs: Self) {
        *self = Self{
            val: (self.val+rhs.val)%M,
        }
    }
}

impl<const M: usize> Sub for Mint<M>{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self{
            val: (M+self.val-rhs.val)%M,
        }
    }
}

impl<const M: usize> SubAssign for Mint<M>{
    fn sub_assign(&mut self, rhs: Self) {
        *self = Self{
            val: (M+self.val-rhs.val)%M,
        }
    }
}

impl<const M: usize> Mul for Mint<M>{
    type Output = Mint<M>;

    fn mul(self, rhs: Self) -> Self::Output {
        Self{
            val: (self.val*rhs.val)%M,
        }
    }
}

impl<const M: usize> MulAssign for Mint<M>{
    fn mul_assign(&mut self, rhs: Self) {
        *self = Self{
            val: (self.val*rhs.val)%M,
        }
    }
}

impl<const M: usize> Div for Mint<M>{
    type Output = Self;

    fn div(self, rhs: Self) -> Self{
        if rhs.val==0{
            panic!("Mint: 0-division-error");
        }
        self*rhs.inv()
    }
}

impl<const M: usize> DivAssign for Mint<M>{
    fn div_assign(&mut self, rhs: Self) {
        if rhs.val==0{
            panic!("Mint: 0-division-error");
        }
        *self *= rhs.inv();
    }
}

impl<const M: usize, T> From<T> for Mint<M> where T: Into<usize>{
    fn from(value: T) -> Self {
        Mint::new(value.into())
    }
}

impl<const M: usize, T> Add<T> for Mint<M> where T: Into<usize>+Copy{
    type Output = Self;

    fn add(self, rhs: T) -> Self::Output {
        self+Mint::<M>::new(rhs.into())
    }
}

impl<const M: usize, T> AddAssign<T> for Mint<M> where T: Into<usize>+Copy{
    fn add_assign(&mut self, rhs: T) {
        *self = *self+Mint::<M>::new(rhs.into())
    }
}

impl<const M: usize, T> Sub<T> for Mint<M>
where
    T: Into<usize> + Copy,
{
    type Output = Self;
    fn sub(self, rhs: T) -> Self::Output {
        self - Mint::<M>::new(rhs.into())
    }
}

impl<const M: usize, T> SubAssign<T> for Mint<M>
where
    T: Into<usize> + Copy,
{
    fn sub_assign(&mut self, rhs: T) {
        *self = *self - Mint::<M>::new(rhs.into());
    }
}

impl<const M: usize, T> Mul<T> for Mint<M>
where
    T: Into<usize> + Copy,
{
    type Output = Self;
    fn mul(self, rhs: T) -> Self::Output {
        self * Mint::<M>::new(rhs.into())
    }
}

impl<const M: usize, T> MulAssign<T> for Mint<M>
where
    T: Into<usize> + Copy,
{
    fn mul_assign(&mut self, rhs: T) {
        *self = *self * Mint::<M>::new(rhs.into());
    }
}

impl<const M: usize, T> Div<T> for Mint<M>
where
    T: Into<usize> + Copy,
{
    type Output = Self;
    fn div(self, rhs: T) -> Self::Output {
        let rhs_m = Mint::<M>::new(rhs.into());
        if rhs_m.val == 0 {
            panic!("Mint: 0 division_error");
        }
        self / rhs_m
    }
}

impl<const M: usize, T> DivAssign<T> for Mint<M>
where
    T: Into<usize> + Copy,
{
    fn div_assign(&mut self, rhs: T) {
        let rhs_m = Mint::<M>::new(rhs.into());
        if rhs_m.val == 0 {
            panic!("Mint: 0 division_error");
        }
        *self = *self / rhs_m;
    }
}

static F: Once = Once::new();
static MX: usize = 10000010;
static mut FACT: Option<Vec<Mint<COMB_MOD>>> = None;
static mut INV: Option<Vec<Mint<COMB_MOD>>> = None;
const COMB_MOD: usize = 998244353;

pub fn init_fact(){
    F.call_once(||{
        let mut fact = vec![Mint::<COMB_MOD>::new(1); MX+1];
        let mut inv = vec![Mint::<COMB_MOD>::new(1); MX+1];
        let mut finv = vec![Mint::<COMB_MOD>::new(1); MX+1];
        let z = Mint::new(0);
        for i in 1..=MX{
            fact[i] = fact[i-1]*i;
        }
        inv[0] = Mint::new(0);
        for i in 2..=MX{
            inv[i] = z-inv[COMB_MOD%i]*(COMB_MOD/i);
            finv[i] = finv[i-1]*inv[i];
        }
        unsafe {
            FACT = Some(fact);
            INV = Some(finv);
        }
    });
}

#[allow(static_mut_refs)]
pub fn comb(a: usize, b: usize) ->Mint<COMB_MOD>{
    init_fact();
    if a < b{
        return Mint::new(0);
    } else {
        unsafe {
            let fa = FACT.as_ref().unwrap()[a];
            let inv_b = INV.as_ref().unwrap()[b];
            let inv_ab = INV.as_ref().unwrap()[a-b];
            fa*inv_b*inv_ab
        }
    }
}
