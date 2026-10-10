#![allow(dead_code)]
#[path = "../src/DataStructure/sortable_sequence.rs"]
mod sortable_sequence;
use sortable_sequence::{KeyedAvlMonoid, SortableSequence};
use std::io::{self, Read, Write};
const MOD: u64 = 998_244_353;
struct Affine;
impl KeyedAvlMonoid for Affine {
    type S = (u64, u64);
    fn identity() -> Self::S {
        (1, 0)
    }
    fn op(f: &Self::S, g: &Self::S) -> Self::S {
        (f.0 * g.0 % MOD, (f.1 * g.0 + g.1) % MOD)
    }
}
fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut it = input.split_whitespace().map(|s| s.parse::<u64>().unwrap());
    let n = it.next().unwrap() as usize;
    let q = it.next().unwrap() as usize;
    let values = (0..n)
        .map(|_| (it.next().unwrap(), (it.next().unwrap(), it.next().unwrap())))
        .collect();
    let mut seq = SortableSequence::<u64, Affine>::from_vec(values);
    let mut out = io::BufWriter::new(io::stdout().lock());
    for _ in 0..q {
        match it.next().unwrap() {
            0 => {
                let i = it.next().unwrap() as usize;
                let p = it.next().unwrap();
                let a = it.next().unwrap();
                let b = it.next().unwrap();
                seq.set(i, p, (a, b));
            }
            1 => {
                let l = it.next().unwrap() as usize;
                let r = it.next().unwrap() as usize;
                let x = it.next().unwrap();
                let (a, b) = seq.prod(l, r);
                writeln!(out, "{}", (a * x + b) % MOD).unwrap();
            }
            t @ (2 | 3) => {
                let l = it.next().unwrap() as usize;
                let r = it.next().unwrap() as usize;
                seq.sort(l, r, t == 3);
            }
            _ => unreachable!(),
        }
    }
}
