// MIを同じスコープに用意する。旧Matrix APIを維持する互換入口。
include!("matrix.rs");

pub struct MintMatrixMonoid;
impl MatrixMonoid for MintMatrixMonoid {
    type S = MI;
    #[inline(always)]
    fn zero() -> MI {
        MI::new(0)
    }
    #[inline(always)]
    fn one() -> MI {
        MI::new(1)
    }
    #[inline(always)]
    fn sum(a: &MI, b: &MI) -> MI {
        *a + *b
    }
    #[inline(always)]
    fn mul(a: &MI, b: &MI) -> MI {
        *a * *b
    }
    #[inline(always)]
    fn mul_add(acc: &mut MI, a: &MI, b: &MI) {
        *acc += *a * *b;
    }
    fn multiply_kernel(n: usize, a: &[MI], b: &[MI], out: &mut [MI]) {
        // modulus()を要求せず、旧MIのnew/val/減算から固定modを取得する。
        let modulus = (MI::new(0) - MI::new(1)).val() as u64 + 1;
        if modulus <= u32::MAX as u64 && n >= 16 {
            let a: Vec<u32> = a.iter().map(|x| x.val() as u32).collect();
            let b: Vec<u32> = b.iter().map(|x| x.val() as u32).collect();
            let mut c = vec![0u32; n * n];
            matrix_mod_u32_kernel::<0>(n, &a, &b, &mut c, modulus as u32);
            for (x, y) in out.iter_mut().zip(c) {
                // 旧Mintのusize引数と、同梱MIのInto<i128>引数の両方に対応。
                *x = if modulus <= i32::MAX as u64 {
                    MI::new(matrix_mint_input(y, 0))
                } else {
                    MI::new(matrix_mint_input(y >> 16, 0)) * MI::new(65536)
                        + MI::new(matrix_mint_input(y & 65535, 0))
                };
            }
        } else {
            matrix_generic_kernel::<Self>(n, a, b, out);
        }
    }
}
impl MatrixField for MintMatrixMonoid {
    #[inline(always)]
    fn is_zero(x: &MI) -> bool {
        x.val() == 0
    }
    #[inline(always)]
    fn sub(a: &MI, b: &MI) -> MI {
        *a - *b
    }
    #[inline(always)]
    fn inverse(x: &MI) -> Option<MI> {
        if x.val() == 0 {
            None
        } else {
            Some(x.inv())
        }
    }
}
pub type Matrix = SquareMatrix<MintMatrixMonoid>;

#[doc(hidden)]
fn matrix_mint_input<T: std::convert::TryFrom<u32>>(x: u32, _: T) -> T {
    T::try_from(x).ok().expect("matrix mint input overflow")
}
