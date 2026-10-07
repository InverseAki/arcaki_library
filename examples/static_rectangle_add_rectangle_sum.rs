#[allow(dead_code)]
mod static_rectangle_add_rectangle_sum {
    include!("../src/Basic/static_rectangle_add_rectangle_sum.rs");
}
#[allow(dead_code)]
mod mint {
    include!("../src/NumberTheory/mint.rs");
}
use static_rectangle_add_rectangle_sum::StaticRectangleAddRectangleSum;
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
    type M = mint::Mint<998244353>;
    let initial = (0..n)
        .map(|_| {
            (
                read::<i64>(&mut tokens),
                read::<i64>(&mut tokens),
                read::<i64>(&mut tokens),
                read::<i64>(&mut tokens),
                M::new(read(&mut tokens)),
            )
        })
        .collect::<Vec<_>>();
    let mut solver = StaticRectangleAddRectangleSum::<i64, i64, M>::build(initial);
    for _ in 0..q {
        solver.push_query(
            read(&mut tokens),
            read(&mut tokens),
            read(&mut tokens),
            read(&mut tokens),
        );
    }
    let cv = |x: &i64| M::new(*x as usize);
    let mut output = BufWriter::new(io::stdout().lock());
    for answer in solver.solve_with(M::new(0), cv, cv) {
        writeln!(output, "{answer}").unwrap();
    }
}
