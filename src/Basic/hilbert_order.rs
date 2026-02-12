const ROT_DELTA: [u32; 4] = [3, 0, 0, 1];
#[inline]
pub fn hilbert_order(x: u32, y: u32, pow: u32, rot: u32) -> u64 {
    if pow == 0 { return 0; }
    let h: u32 = 1u32 << (pow - 1);
    let mut seg: u32 = if x < h {if y < h { 0 } else { 3 }} else {if y < h { 1 } else { 2 }};
    seg = (seg + rot) & 3;
    let nrot = (rot + ROT_DELTA[seg as usize]) & 3;
    let nx = x & (h - 1);
    let ny = y & (h - 1);
    let sub: u64 = 1u64 << (2 * pow - 2);
    let mut ord = (seg as u64) * sub;
    let add = hilbert_order(nx, ny, pow - 1, nrot);
    ord += if seg == 1 || seg == 2 { add } else { sub - 1 - add };
    ord
}
