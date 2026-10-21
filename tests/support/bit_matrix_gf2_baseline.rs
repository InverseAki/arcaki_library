use super::BitMatrix;

pub fn prod_xor(a: &BitMatrix, b: &BitMatrix) -> BitMatrix {
    assert_eq!(a.width(), b.height());
    let transposed = b.transpose();
    let mut out = BitMatrix::new(a.height(), b.width());
    for (i, a) in a.rows().iter().enumerate() {
        for (j, b) in transposed.rows().iter().enumerate() {
            if a.and_count_ones(b) & 1 != 0 { out.set(i, j, true); }
        }
    }
    out
}
