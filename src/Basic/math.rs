/// 標準整数型の共通部分。外部クレート不要、型ごとに静的に最適化される。
pub trait MathInteger:
    Copy
    + Ord
    + std::ops::Rem<Output = Self>
    + std::ops::Div<Output = Self>
    + std::ops::Sub<Output = Self>
    + std::ops::Mul<Output = Self>
{
    const ZERO: Self;
    const ONE: Self;
    fn euclid_div(self, rhs: Self) -> Self;
    fn euclid_rem(self, rhs: Self) -> Self;
}
/// 拡張GCDの係数を表現できる符号付き整数。
pub trait SignedMathInteger: MathInteger {}
macro_rules! impl_math_signed {
    ($($t:ty),*) => {$ (
        impl MathInteger for $t {
            const ZERO: Self = 0;
            const ONE: Self = 1;
            #[inline] fn euclid_div(self, rhs: Self) -> Self { self.div_euclid(rhs) }
            #[inline] fn euclid_rem(self, rhs: Self) -> Self {
                // MIN/-1 の商は表現不能だが、余りは0。
                if rhs == -1 { 0 } else { self.rem_euclid(rhs) }
            }
        }
        impl SignedMathInteger for $t {}
    )*};
}
macro_rules! impl_math_unsigned {
    ($($t:ty),*) => {$ (
        impl MathInteger for $t {
            const ZERO: Self = 0;
            const ONE: Self = 1;
            #[inline] fn euclid_div(self, rhs: Self) -> Self { self / rhs }
            #[inline] fn euclid_rem(self, rhs: Self) -> Self { self % rhs }
        }
    )*};
}
impl_math_signed!(i8, i16, i32, i64, i128, isize);
impl_math_unsigned!(u8, u16, u32, u64, u128, usize);

/// 非負の整数の最大公約数。gcd(0,0)=0。全標準整数型に対応。
#[inline]
pub fn gcd<T: MathInteger>(mut a: T, mut b: T) -> T {
    assert!(a >= T::ZERO && b >= T::ZERO);
    while b != T::ZERO {
        (a, b) = (b, a % b);
    }
    a
}
/// a=b*q+r, 0<=r<|b| の商。負の除数では数学的なfloorと異なる。
/// 除数0、符号付きMIN/-1の商はpanic。
#[inline]
pub fn floor<T: MathInteger>(a: T, b: T) -> T {
    a.euclid_div(b)
}
/// a=b*q+r, 0<=r<|b| の余り。除数0はpanic。MIN%-1は0。
#[inline]
pub fn modulo<T: MathInteger>(a: T, b: T) -> T {
    a.euclid_rem(b)
}
/// 非負入力に対して (g,x,y): a*x+b*y=g=gcd(a,b)。全符号付き整数型。
/// (0,0)=(0,1,0)。係数の計算は入力と同じ型。標準の更新式の係数は
/// |x|<=b/g, |y|<=a/g に収まり、非負入力なら型の範囲を超えない。
pub fn extended_gcd<T: SignedMathInteger>(a: T, b: T) -> (T, T, T) {
    assert!(a >= T::ZERO && b >= T::ZERO);
    let (mut r, mut s) = (a, b);
    let (mut x, mut nx, mut y, mut ny) = (T::ONE, T::ZERO, T::ZERO, T::ONE);
    while s != T::ZERO {
        let q = r / s;
        (r, s) = (s, r - q * s);
        (x, nx) = (nx, x - q * nx);
        (y, ny) = (ny, y - q * ny);
    }
    (r, x, y)
}

// 以下は既存API。階乗・組合せは同じスコープの定数MODを使う。
pub fn factorial_i64(n: usize) -> (Vec<i64>, Vec<i64>) {
    let mut res = vec![1; n + 1];
    let mut inv = vec![1; n + 1];
    for i in 0..n {
        res[i + 1] = (res[i] * (i + 1) as i64) % MOD;
    }
    inv[n] = mod_inverse(res[n], MOD);
    for i in (0..n).rev() {
        inv[i] = inv[i + 1] * (i + 1) as i64 % MOD;
    }
    (res, inv)
}
/// 正の法mでaの逆元を返す。互いに素であることが前提。
pub fn mod_inverse(a: i64, m: i64) -> i64 {
    assert!(m > 0);
    let (_, x, _) = extended_gcd(a.rem_euclid(m), m);
    x.rem_euclid(m)
}
pub fn comb(a: i64, b: i64, f: &Vec<(i64, i64)>) -> i64 {
    if a < b {
        return 0;
    } else if b == 0 || a == b {
        return 1;
    } else {
        let x = f[a as usize].0;
        let y = f[(a - b) as usize].1;
        let z = f[b as usize].1;
        return ((x * y) % MOD) * z % MOD;
    }
}
pub fn factorial(x: i64) -> Vec<(i64, i64)> {
    let mut f = vec![(1i64, 1i64), (1, 1)];
    let mut z = 1i64;
    let mut inv = vec![0; x as usize + 10];
    inv[1] = 1;
    for i in 2..x + 1 {
        z = (z * i) % MOD;
        let w = (MOD - inv[(MOD % i) as usize] * (MOD / i) % MOD) % MOD;
        inv[i as usize] = w;
        f.push((z, (f[i as usize - 1].1 * w) % MOD));
    }
    return f;
}
pub fn fast_mod_pow(mut x: i64, p: usize, m: i64) -> i64 {
    x %= m;
    let mut res = 1;
    let mut t = x;
    let mut z = p;
    while z > 0 {
        if z % 2 == 1 {
            res = (res * t) % m;
        }
        t = (t * t) % m;
        z /= 2;
    }
    res
}
