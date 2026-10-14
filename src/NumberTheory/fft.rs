// Source: https://github.com/InverseAki/arcaki_library/blob/main/src/NumberTheory/fft.rs
#![allow(unused)]
use std::{ops::*, f64::consts::PI};

#[derive(Copy, Clone, Debug)]
struct Complex{
    re: f64, ie: f64,
}

impl Complex{
    pub fn new(re: f64, ie: f64)->Self{
        Complex{re, ie}
    }

    pub fn polar(r: f64, t: f64)->Self{
        Complex{re: r*t.cos(), ie: r*t.sin()}
    }

    pub fn re(&self)->f64{
        self.re
    }

    pub fn f_into(f: f64)->Self{
        Complex{re: f, ie: 0.}
    }
}

impl Add for Complex{
    type Output = Complex;
    fn add(self, rhs: Self) -> Self::Output {
        Self{
            re: self.re+rhs.re,
            ie: self.ie+rhs.ie,
        }
    }
}

impl AddAssign for Complex{
    fn add_assign(&mut self, rhs: Self) {
        *self = Self{
            re: self.re+rhs.re,
            ie: self.ie+rhs.ie,
        }
    }
}

impl Sub for Complex{
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self{
            re: self.re-rhs.re,
            ie: self.ie-rhs.ie,
        }
    }
}

impl SubAssign for Complex{
    fn sub_assign(&mut self, rhs: Self) {
        *self = Self{
            re: self.re-rhs.re,
            ie: self.ie-rhs.ie,
        }
    }
}

impl Mul for Complex{
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self{
            re: self.re*rhs.re-self.ie*rhs.ie,
            ie: self.re*rhs.ie+self.ie*rhs.re,
        }
    }
}

impl MulAssign for Complex{
    fn mul_assign(&mut self, rhs: Self) {
        *self = Self{
            re: self.re*rhs.re-self.ie*rhs.ie,
            ie: self.re*rhs.ie+self.ie*rhs.re,
        }
    }
}

pub trait IntoComplex{
    fn into_complex(self)->Complex;
}

impl IntoComplex for f64{
    fn into_complex(self) -> Complex {
        Complex::new(self, 0.)
    }
}

pub fn dft(vc: &mut [Complex], inv: f64){
    let k = vc.len();
    if k <= 1{return;}
    let mut va = Vec::with_capacity(k/2);
    let mut vb = Vec::with_capacity(k/2);
    for i in 0..k/2{
        va.push(vc[2*i]); vb.push(vc[2*i+1]);
    }
    dft(&mut va, inv); dft(&mut vb, inv);
    let t = inv*2.0*PI/(k as f64);
    let z = Complex::polar(1., t);
    let mut x = Complex::new(1., 0.);
    for i in 0..k{
        vc[i] = va[i%(k/2)]+x*vb[i%(k/2)];
        x *= z;
    }
}

pub fn fft_f64(a: &Vec<f64>, b: &Vec<f64>)->Vec<f64>{
    let k = (a.len()+b.len()).next_power_of_two();
    let mut va = vec![Complex::new(0., 0.); k];
    let mut vb = vec![Complex::new(0., 0.); k];
    for (i, &v) in a.iter().enumerate(){
        va[i] = v.into_complex();
    }
    for (i, &v) in b.iter().enumerate(){
        vb[i] = v.into_complex();
    }
    dft(&mut va, 1.); dft(&mut vb, 1.);
    for i in 0..k{
        va[i] = va[i]*vb[i];
    }
    dft(&mut va, -1.);
    va.iter().map(|z| z.re/(k as f64)).collect()
}
