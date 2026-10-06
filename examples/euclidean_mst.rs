#[allow(dead_code)]
#[path = "../src/Gemetory/delaunay.rs"]
mod delaunay;
use std::io::{self, Read, Write};
fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut it = input.split_whitespace();
    let n: usize = it.next().unwrap().parse().unwrap();
    let points: Vec<(i64, i64)> = (0..n)
        .map(|_| {
            (
                it.next().unwrap().parse().unwrap(),
                it.next().unwrap().parse().unwrap(),
            )
        })
        .collect();
    let mut out = io::BufWriter::new(io::stdout().lock());
    for (a, b) in delaunay::euclidean_mst(&points) {
        writeln!(out, "{a} {b}").unwrap();
    }
}
