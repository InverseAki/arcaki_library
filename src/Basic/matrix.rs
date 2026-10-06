// 抽象化した正方行列。値は行優先の連続配列。セル(i,j)はi*n+j。
// MatrixMonoidは加法・乗法の半環（zeroは乗法の吸収元）を用意する。
// 区間積と異なり積算の順序はk=0,1,...を維持する。スカラー乗法は非可換でもよい。
pub trait MatrixMonoid {
    type S: Clone;
    fn sum(a: &Self::S, b: &Self::S) -> Self::S;
    fn zero() -> Self::S;
    fn mul(a: &Self::S, b: &Self::S) -> Self::S;
    fn one() -> Self::S;
    /// 省略してよい加法単位元だけtrue。既存実装は追加実装不要。
    #[inline(always)]
    fn is_additive_zero(_: &Self::S) -> bool {
        false
    }
    #[inline(always)]
    fn mul_add(acc: &mut Self::S, a: &Self::S, b: &Self::S) {
        *acc = Self::sum(acc, &Self::mul(a, b));
    }
    /// outはn*n個のzeroで初期化済み。特殊型はこのカーネルを上書きできる。
    #[inline]
    fn multiply_kernel(n: usize, a: &[Self::S], b: &[Self::S], out: &mut [Self::S]) {
        matrix_generic_kernel::<Self>(n, a, b, out);
    }
}

/// 逆行列を求めるための体の演算。半環だけの型は実装しなくてよい。
pub trait MatrixField: MatrixMonoid {
    fn is_zero(x: &Self::S) -> bool;
    fn sub(a: &Self::S, b: &Self::S) -> Self::S;
    fn inverse(x: &Self::S) -> Option<Self::S>;
    fn validate_field() {}
}

#[doc(hidden)]
pub trait MatrixData<T> {
    fn into_matrix_data(self) -> Vec<T>;
}
impl<T> MatrixData<T> for Vec<T> {
    fn into_matrix_data(self) -> Vec<T> {
        self
    }
}
impl<T: Clone> MatrixData<T> for &[T] {
    fn into_matrix_data(self) -> Vec<T> {
        self.to_vec()
    }
}
impl<T: Clone> MatrixData<T> for &Vec<T> {
    fn into_matrix_data(self) -> Vec<T> {
        self.clone()
    }
}
impl<T, const N: usize> MatrixData<T> for [T; N] {
    fn into_matrix_data(self) -> Vec<T> {
        Vec::from(self)
    }
}
impl<T: Clone, const N: usize> MatrixData<T> for &[T; N] {
    fn into_matrix_data(self) -> Vec<T> {
        self.to_vec()
    }
}

pub struct SquareMatrix<M: MatrixMonoid> {
    n: usize,
    g: Vec<M::S>,
}
/// 旧DoublingMatrixの型名・メソッドを維持する。
pub type DoublingMatrix<M> = SquareMatrix<M>;

impl<M: MatrixMonoid> Clone for SquareMatrix<M> {
    fn clone(&self) -> Self {
        Self {
            n: self.n,
            g: self.g.clone(),
        }
    }
}
impl<M: MatrixMonoid> std::fmt::Debug for SquareMatrix<M>
where
    M::S: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SquareMatrix")
            .field("n", &self.n)
            .field("g", &self.g)
            .finish()
    }
}
impl<M: MatrixMonoid> PartialEq for SquareMatrix<M>
where
    M::S: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.n == other.n && self.g == other.g
    }
}
impl<M: MatrixMonoid> Eq for SquareMatrix<M> where M::S: Eq {}

impl<M: MatrixMonoid> SquareMatrix<M> {
    /// Vecは移動し、&Vec/&[T]は複製する。要素数はn*nでなければならない。
    pub fn new<D: MatrixData<M::S>>(n: usize, data: D) -> Self {
        let cells = n.checked_mul(n).expect("matrix size overflow");
        let g = data.into_matrix_data();
        assert_eq!(g.len(), cells, "matrix data length mismatch");
        Self { n, g }
    }
    pub fn zeros(n: usize) -> Self {
        let cells = n.checked_mul(n).expect("matrix size overflow");
        Self {
            n,
            g: vec![M::zero(); cells],
        }
    }
    pub fn identity(n: usize) -> Self {
        let mut res = Self::zeros(n);
        for i in 0..n {
            res.g[i * n + i] = M::one();
        }
        res
    }
    #[inline]
    pub fn e(n: usize) -> Self {
        Self::identity(n)
    }
    #[inline]
    pub fn size(&self) -> usize {
        self.n
    }
    #[inline]
    pub fn as_slice(&self) -> &[M::S] {
        &self.g
    }
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [M::S] {
        &mut self.g
    }
    #[inline]
    pub fn into_vec(self) -> Vec<M::S> {
        self.g
    }
    #[inline]
    pub fn get(&self, i: usize, j: usize) -> &M::S {
        &self[(i, j)]
    }
    #[inline]
    pub fn set(&mut self, i: usize, j: usize, x: M::S) {
        self[(i, j)] = x;
    }
    #[inline]
    pub fn row(&self, i: usize) -> &[M::S] {
        assert!(i < self.n, "row out of range");
        &self.g[i * self.n..(i + 1) * self.n]
    }
    #[inline]
    pub fn row_mut(&mut self, i: usize) -> &mut [M::S] {
        assert!(i < self.n, "row out of range");
        &mut self.g[i * self.n..(i + 1) * self.n]
    }
    pub fn mul(&self, rhs: &Self) -> Self {
        assert_eq!(self.n, rhs.n, "matrix dimensions differ");
        let mut out = Self::zeros(self.n);
        M::multiply_kernel(self.n, &self.g, &rhs.g, &mut out.g);
        out
    }
    #[inline]
    pub fn prod(&self, rhs: &Self) -> Self {
        self.mul(rhs)
    }
    /// 出力領域を再利用する。outも同じ次元であること。
    pub fn mul_into(&self, rhs: &Self, out: &mut Self) {
        assert_eq!(self.n, rhs.n, "matrix dimensions differ");
        assert_eq!(self.n, out.n, "output dimensions differ");
        out.g.fill(M::zero());
        M::multiply_kernel(self.n, &self.g, &rhs.g, &mut out.g);
    }
    pub fn pow(&self, mut exponent: usize) -> Self {
        if exponent == 0 {
            return Self::identity(self.n);
        }
        if exponent == 1 {
            return self.clone();
        }
        let mut base = self.clone();
        let mut result: Option<Self> = None;
        let mut scratch = Self::zeros(self.n);
        while exponent > 0 {
            if exponent & 1 != 0 {
                if let Some(r) = &mut result {
                    r.mul_into(&base, &mut scratch);
                    std::mem::swap(r, &mut scratch);
                } else {
                    result = Some(base.clone());
                }
            }
            exponent >>= 1;
            if exponent > 0 {
                base.mul_into(&base, &mut scratch);
                std::mem::swap(&mut base, &mut scratch);
            }
        }
        result.unwrap()
    }
}
impl<M: MatrixField> SquareMatrix<M> {
    pub fn try_inv(&self) -> Option<Self> {
        M::validate_field();
        let n = self.n;
        let mut a = self.clone();
        let mut b = Self::identity(n);
        for col in 0..n {
            let pivot = (col..n).find(|&i| !M::is_zero(&a.g[i * n + col]))?;
            if pivot != col {
                for j in 0..n {
                    a.g.swap(col * n + j, pivot * n + j);
                    b.g.swap(col * n + j, pivot * n + j);
                }
            }
            let inv_pivot = M::inverse(&a.g[col * n + col])?;
            for j in col + 1..n {
                a.g[col * n + j] = M::mul(&a.g[col * n + j], &inv_pivot);
            }
            a.g[col * n + col] = M::one();
            for j in 0..n {
                b.g[col * n + j] = M::mul(&b.g[col * n + j], &inv_pivot);
            }
            for row in 0..n {
                if row == col {
                    continue;
                }
                let factor = a.g[row * n + col].clone();
                if M::is_zero(&factor) {
                    continue;
                }
                for j in col + 1..n {
                    a.g[row * n + j] =
                        M::sub(&a.g[row * n + j], &M::mul(&factor, &a.g[col * n + j]));
                }
                a.g[row * n + col] = M::zero();
                for j in 0..n {
                    b.g[row * n + j] =
                        M::sub(&b.g[row * n + j], &M::mul(&factor, &b.g[col * n + j]));
                }
            }
        }
        Some(b)
    }
    pub fn inv(&self) -> Self {
        self.try_inv().expect("matrix is not invertible")
    }
}
impl<M: MatrixMonoid> std::ops::Index<(usize, usize)> for SquareMatrix<M> {
    type Output = M::S;
    #[inline]
    fn index(&self, (i, j): (usize, usize)) -> &M::S {
        assert!(i < self.n && j < self.n, "matrix index out of range");
        &self.g[i * self.n + j]
    }
}
impl<M: MatrixMonoid> std::ops::IndexMut<(usize, usize)> for SquareMatrix<M> {
    #[inline]
    fn index_mut(&mut self, (i, j): (usize, usize)) -> &mut M::S {
        assert!(i < self.n && j < self.n, "matrix index out of range");
        &mut self.g[i * self.n + j]
    }
}
impl<'a, 'b, M: MatrixMonoid> std::ops::Mul<&'b SquareMatrix<M>> for &'a SquareMatrix<M> {
    type Output = SquareMatrix<M>;
    fn mul(self, rhs: &'b SquareMatrix<M>) -> Self::Output {
        SquareMatrix::mul(self, rhs)
    }
}

#[doc(hidden)]
pub fn matrix_generic_kernel<M: MatrixMonoid + ?Sized>(
    n: usize,
    a: &[M::S],
    b: &[M::S],
    out: &mut [M::S],
) {
    if n < 64 {
        for (a_row, out_row) in a.chunks_exact(n.max(1)).zip(out.chunks_exact_mut(n.max(1))) {
            for (k, x) in a_row.iter().enumerate() {
                if M::is_additive_zero(x) {
                    continue;
                }
                let b_row = &b[k * n..(k + 1) * n];
                for (acc, y) in out_row.iter_mut().zip(b_row) {
                    M::mul_add(acc, x, y);
                }
            }
        }
        return;
    }
    // kの順序を変えず、右辺と出力の近い領域をまとめて使う。
    for ii in (0..n).step_by(32) {
        for kk in (0..n).step_by(32) {
            for jj in (0..n).step_by(64) {
                let width = (n - jj).min(64);
                for i in ii..(ii + 32).min(n) {
                    let out_row = &mut out[i * n + jj..i * n + jj + width];
                    for k in kk..(kk + 32).min(n) {
                        let x = &a[i * n + k];
                        if M::is_additive_zero(x) {
                            continue;
                        }
                        let b_row = &b[k * n + jj..k * n + jj + width];
                        for (acc, y) in out_row.iter_mut().zip(b_row) {
                            M::mul_add(acc, x, y);
                        }
                    }
                }
            }
        }
    }
}

/// 法1..=u32::MAX、セルは0<=x<P。加算・乗算をmod Pで行う。
/// 逆行列のAPIを使うときはPが素数であることを検査する。
pub struct ModMatrixMonoid<const P: u32>;
impl<const P: u32> MatrixMonoid for ModMatrixMonoid<P> {
    type S = u32;
    #[inline(always)]
    fn zero() -> u32 {
        assert!(P > 0);
        0
    }
    #[inline(always)]
    fn one() -> u32 {
        assert!(P > 0);
        1 % P
    }
    #[inline(always)]
    fn sum(a: &u32, b: &u32) -> u32 {
        ((*a as u64 + *b as u64) % P as u64) as u32
    }
    #[inline(always)]
    fn mul(a: &u32, b: &u32) -> u32 {
        (*a as u64 * *b as u64 % P as u64) as u32
    }
    #[inline]
    fn multiply_kernel(n: usize, a: &[u32], b: &[u32], out: &mut [u32]) {
        matrix_mod_u32_kernel::<P>(n, a, b, out, P);
    }
}
impl<const P: u32> MatrixField for ModMatrixMonoid<P> {
    fn validate_field() {
        assert!(matrix_is_prime(P), "inverse requires a prime modulus");
    }
    #[inline(always)]
    fn is_zero(x: &u32) -> bool {
        *x == 0
    }
    #[inline(always)]
    fn sub(a: &u32, b: &u32) -> u32 {
        ((*a as u64 + P as u64 - *b as u64) % P as u64) as u32
    }
    fn inverse(x: &u32) -> Option<u32> {
        if *x == 0 {
            return None;
        }
        let (mut a, mut e, mut r) = (*x as u64, P as u64 - 2, 1u64);
        while e > 0 {
            if e & 1 != 0 {
                r = r * a % P as u64;
            }
            a = a * a % P as u64;
            e >>= 1;
        }
        Some(r as u32)
    }
}
fn matrix_is_prime(p: u32) -> bool {
    if p < 2 {
        return false;
    }
    if p % 2 == 0 {
        return p == 2;
    }
    let mut d = 3;
    while d <= p / d {
        if p % d == 0 {
            return false;
        }
        d += 2;
    }
    true
}

/// min-plus。INFは加法単位元。有限値とその演算結果は絶対値<INFの範囲で使う。
pub struct MinPlusMonoid;
impl MinPlusMonoid {
    pub const INF: i64 = 1 << 60;
}
impl MatrixMonoid for MinPlusMonoid {
    type S = i64;
    #[inline(always)]
    fn zero() -> i64 {
        Self::INF
    }
    #[inline(always)]
    fn one() -> i64 {
        0
    }
    #[inline(always)]
    fn sum(a: &i64, b: &i64) -> i64 {
        (*a).min(*b)
    }
    #[inline(always)]
    fn mul(a: &i64, b: &i64) -> i64 {
        if *a == Self::INF || *b == Self::INF {
            Self::INF
        } else {
            a + b
        }
    }
    #[inline(always)]
    fn is_additive_zero(x: &i64) -> bool {
        *x == Self::INF
    }
    fn multiply_kernel(n: usize, a: &[i64], b: &[i64], out: &mut [i64]) {
        // 右辺にINFがなければ内側の吸収元判定を全て省ける。
        if b.iter().all(|&x| x != Self::INF) {
            for (a_row, out_row) in a.chunks_exact(n.max(1)).zip(out.chunks_exact_mut(n.max(1))) {
                for (k, &x) in a_row.iter().enumerate() {
                    if x == Self::INF {
                        continue;
                    }
                    let b_row = &b[k * n..(k + 1) * n];
                    for (acc, &y) in out_row.iter_mut().zip(b_row) {
                        *acc = (*acc).min(x + y);
                    }
                }
            }
        } else {
            matrix_generic_kernel::<Self>(n, a, b, out);
        }
    }
}
pub struct BoolMatrixMonoid;
impl MatrixMonoid for BoolMatrixMonoid {
    type S = bool;
    #[inline(always)]
    fn zero() -> bool {
        false
    }
    #[inline(always)]
    fn one() -> bool {
        true
    }
    #[inline(always)]
    fn sum(a: &bool, b: &bool) -> bool {
        *a || *b
    }
    #[inline(always)]
    fn mul(a: &bool, b: &bool) -> bool {
        *a && *b
    }
    #[inline(always)]
    fn is_additive_zero(x: &bool) -> bool {
        !*x
    }
}

// P=0なら実行時のmodulusを使う。入力は正規化済みのu32。
#[doc(hidden)]
pub fn matrix_mod_u32_kernel<const P: u32>(
    n: usize,
    a: &[u32],
    b: &[u32],
    out: &mut [u32],
    modulus: u32,
) {
    let modulus = if P == 0 { modulus } else { P };
    assert!(modulus > 0);
    let mut zeros = 0;
    for &x in a {
        assert!(x < modulus, "matrix residue out of range");
        zeros += usize::from(x == 0);
    }
    let mut b_zero = true;
    for &x in b {
        assert!(x < modulus, "matrix residue out of range");
        b_zero &= x == 0;
    }
    if n == 0 {
        return;
    }
    if zeros == a.len() || b_zero {
        out.fill(0);
        return;
    }
    if n <= 4 {
        for i in 0..n {
            for k in 0..n {
                let x = a[i * n + k] as u64;
                for j in 0..n {
                    let index = i * n + j;
                    out[index] = ((out[index] as u64 + (x * b[k * n + j] as u64 % modulus as u64))
                        % modulus as u64) as u32;
                }
            }
        }
        return;
    }
    let bound = modulus as u64 - 1;
    // 既存の剰余も含めてu64に収まる個数だけ積算する。法が大きいと1になる。
    let batch = ((u64::MAX - bound) / (bound * bound)).min(64) as usize;
    if zeros > a.len() / 4 {
        matrix_mod_u32_blocked::<P, true>(n, a, b, out, modulus, batch);
    } else {
        matrix_mod_u32_blocked::<P, false>(n, a, b, out, modulus, batch);
    }
}
fn matrix_mod_u32_blocked<const P: u32, const SKIP_ZERO: bool>(
    n: usize,
    a: &[u32],
    b: &[u32],
    out: &mut [u32],
    modulus: u32,
    batch: usize,
) {
    let modulus = if P == 0 { modulus } else { P };
    let rows = n.min(if n <= 64 { 16 } else { 8 });
    let stride = n.min(128);
    let mut acc = vec![0u64; rows * stride];
    for ii in (0..n).step_by(rows) {
        let height = (n - ii).min(rows);
        for jj in (0..n).step_by(stride) {
            let width = (n - jj).min(stride);
            acc.fill(0);
            for kk in (0..n).step_by(batch) {
                for i in 0..height {
                    let out_row = &mut acc[i * stride..i * stride + width];
                    let a_row = &a[(ii + i) * n + kk..(ii + i) * n + (kk + batch).min(n)];
                    for (offset, &x) in a_row.iter().enumerate() {
                        if SKIP_ZERO && x == 0 {
                            continue;
                        }
                        let k = kk + offset;
                        let b_row = &b[k * n + jj..k * n + jj + width];
                        let x = x as u64;
                        for (s, &y) in out_row.iter_mut().zip(b_row) {
                            *s += x * y as u64;
                        }
                    }
                }
                for i in 0..height {
                    for s in &mut acc[i * stride..i * stride + width] {
                        *s %= modulus as u64;
                    }
                }
            }
            for i in 0..height {
                for (x, &s) in out[(ii + i) * n + jj..(ii + i) * n + jj + width]
                    .iter_mut()
                    .zip(&acc[i * stride..i * stride + width])
                {
                    *x = s as u32;
                }
            }
        }
    }
}
