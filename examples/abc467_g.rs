include!("../src/Basic/waveletmatrix_offline.rs");

fn main() {
    use std::io::{Read, Write};
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    let mut it = input.split_whitespace();
    let mut next = || it.next().unwrap().parse::<usize>().unwrap();
    let n = next();
    let q = next();
    let initial: Vec<_> = (0..n).map(|_| next()).collect();
    let queries: Vec<_> = (0..q)
        .map(|_| {
            let c = next() - 1;
            let x = next();
            let l = next() - 1;
            let r = next();
            let target = next() as u128;
            (c, x, l, r, target)
        })
        .collect();
    let candidates: Vec<_> = queries.iter().map(|&(i, x, _, _, _)| (i, x)).collect();
    let mut wm = WaveletMatrixOffline::new(&initial, &candidates);
    let mut out = std::io::BufWriter::new(std::io::stdout().lock());
    for (i, x, l, r, target) in queries {
        wm.set(i, x);
        match wm.min_count_for_sum(l, r, target) {
            Some(count) => writeln!(out, "{count}").unwrap(),
            None => writeln!(out, "-1").unwrap(),
        }
    }
}
