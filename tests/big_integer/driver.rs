#[path = "../../src/NumberTheory/big_integer.rs"]
mod big_integer;
use big_integer::{BigInt, HexBigInt, RadixBigInt};
use std::io::{Read, Write};
fn solve<const B: u32>(
    op: &str,
    tokens: &mut std::str::SplitWhitespace<'_>,
    t: usize,
    out: &mut String,
) {
    for _ in 0..t {
        let a: RadixBigInt<B> = tokens.next().unwrap().parse().unwrap();
        let b: RadixBigInt<B> = tokens.next().unwrap().parse().unwrap();
        match op {
            "add" => (&a + &b).append_to(out),
            "sub" => (&a - &b).append_to(out),
            "mul" => (&a * &b).append_to(out),
            "div" => {
                let (q, r) = a.div_rem(&b);
                q.append_to(out);
                out.push(' ');
                r.append_to(out);
            }
            _ => panic!("unknown operation"),
        }
        out.push('\n');
    }
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    let mut tokens = input.split_whitespace();
    let t: usize = tokens.next().unwrap().parse().unwrap();
    let mut out = String::with_capacity(input.len() * 2);
    if args.get(2).map_or(false, |s| s == "hex") {
        solve::<65536>(&args[1], &mut tokens, t, &mut out)
    } else {
        solve::<10000>(&args[1], &mut tokens, t, &mut out)
    }
    std::io::stdout().lock().write_all(out.as_bytes()).unwrap();
}
