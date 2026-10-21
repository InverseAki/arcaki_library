#[allow(dead_code)]
mod bitset {
    use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign};
    include!("../src/DataStructure/bitset.rs");
}
use bitset::{BitMatrix, BitSet};
use std::io::{self, Read, Write};

fn main() {
    let mut input = Vec::new();
    io::stdin().read_to_end(&mut input).unwrap();
    let mut tokens = input.split(|c| c.is_ascii_whitespace()).filter(|s| !s.is_empty());
    let mut number = || tokens.next().unwrap().iter().fold(0usize, |x, c| x * 10 + (c - b'0') as usize);
    let n = number();
    let m = number();
    let k = number();
    let a = BitMatrix::from_rows(m, (0..n).map(|_| BitSet::from_binary(tokens.next().unwrap())).collect());
    let b = BitMatrix::from_rows(k, (0..m).map(|_| BitSet::from_binary(tokens.next().unwrap())).collect());
    let c = a.prod_xor(&b);
    let mut output = vec![b'\n'; n * (k + 1)];
    for (row, line) in c.rows().iter().zip(output.chunks_mut(k + 1)) {
        row.write_binary(&mut line[..k]);
    }
    io::stdout().lock().write_all(&output).unwrap();
}
