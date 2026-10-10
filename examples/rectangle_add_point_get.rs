#[allow(dead_code)]
mod rectangle_add_point_get {
    include!("../src/OfflineQuery/rectangle_add_point_get.rs");
}

use rectangle_add_point_get::RectangleAddPointGet;
use std::io::{self, BufWriter, Read, Write};

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut tokens = input.split_whitespace();
    fn read<T: std::str::FromStr>(tokens: &mut std::str::SplitWhitespace<'_>) -> T {
        tokens.next().unwrap().parse().ok().unwrap()
    }
    let n: usize = read(&mut tokens);
    let q: usize = read(&mut tokens);
    let initial = (0..n)
        .map(|_| {
            (
                read::<i32>(&mut tokens),
                read::<i32>(&mut tokens),
                read::<i32>(&mut tokens),
                read::<i32>(&mut tokens),
                read::<i64>(&mut tokens),
            )
        })
        .collect::<Vec<_>>();
    let mut solver = RectangleAddPointGet::build(initial);
    for _ in 0..q {
        if read::<u8>(&mut tokens) == 0 {
            solver.push_add(
                read(&mut tokens),
                read(&mut tokens),
                read(&mut tokens),
                read(&mut tokens),
                read(&mut tokens),
            );
        } else {
            solver.push_query(read(&mut tokens), read(&mut tokens));
        }
    }
    let mut output = BufWriter::new(io::stdout().lock());
    for answer in solver.solve() {
        writeln!(output, "{answer}").unwrap();
    }
}
