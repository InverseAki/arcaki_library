#[allow(dead_code)]
#[path = "../src/DataStructure/intmap.rs"]
mod implementation;
use implementation::intmap::U64Map;
use std::io::{self, Read, Write};

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut tokens = input.split_ascii_whitespace();
    let q: usize = tokens.next().unwrap().parse().unwrap();
    let mut map = U64Map::<u64>::new();
    let mut output = io::BufWriter::new(io::stdout().lock());
    for _ in 0..q {
        let op = tokens.next().unwrap();
        let key: u64 = tokens.next().unwrap().parse().unwrap();
        if op == "0" {
            let value = tokens.next().unwrap().parse().unwrap();
            map.insert(key, value);
        } else {
            writeln!(output, "{}", map.get(&key).copied().unwrap_or(0)).unwrap();
        }
    }
}
