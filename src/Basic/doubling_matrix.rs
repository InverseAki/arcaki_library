// 実装はmatrix.rsへ統合。旧ファイルの入口とAddMulMonoid（定数MOD:i64）を維持。
include!("matrix.rs");

pub struct AddMulMonoid;
impl MatrixMonoid for AddMulMonoid {
    type S = i64;
    #[inline(always)]
    fn zero() -> i64 {
        assert!(MOD > 0);
        0
    }
    #[inline(always)]
    fn one() -> i64 {
        1 % MOD
    }
    #[inline(always)]
    fn sum(a: &i64, b: &i64) -> i64 {
        if MOD <= i64::MAX / 2 {
            let v = *a + *b;
            if v >= MOD {
                v - MOD
            } else {
                v
            }
        } else {
            ((*a as i128 + *b as i128) % MOD as i128) as i64
        }
    }
    #[inline(always)]
    fn mul(a: &i64, b: &i64) -> i64 {
        if MOD <= i32::MAX as i64 {
            *a * *b % MOD
        } else {
            (*a as i128 * *b as i128 % MOD as i128) as i64
        }
    }
    fn multiply_kernel(n: usize, a: &[i64], b: &[i64], out: &mut [i64]) {
        if MOD <= u32::MAX as i64 && n >= 16 {
            let a: Vec<u32> = a
                .iter()
                .map(|&x| {
                    assert!(x >= 0 && x < MOD);
                    x as u32
                })
                .collect();
            let b: Vec<u32> = b
                .iter()
                .map(|&x| {
                    assert!(x >= 0 && x < MOD);
                    x as u32
                })
                .collect();
            let mut c = vec![0u32; n * n];
            matrix_mod_u32_kernel::<{ MOD as u32 }>(n, &a, &b, &mut c, MOD as u32);
            for (x, y) in out.iter_mut().zip(c) {
                *x = y as i64;
            }
        } else {
            matrix_generic_kernel::<Self>(n, a, b, out);
        }
    }
}
