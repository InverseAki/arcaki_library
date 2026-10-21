include!("../src/Basic/integer_sort.rs");
use std::io::{self, Read, Write};

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut tokens = input.split_ascii_whitespace();
    let n: usize = tokens.next().unwrap().parse().unwrap();
    let mut values: Vec<u64> = (0..n)
        .map(|_| tokens.next().unwrap().parse().unwrap())
        .collect();
    integer_sort(&mut values);
    let mut output = io::BufWriter::new(io::stdout().lock());
    for (i, value) in values.iter().enumerate() {
        if i != 0 {
            write!(output, " ").unwrap();
        }
        write!(output, "{value}").unwrap();
    }
    writeln!(output).unwrap();
}
