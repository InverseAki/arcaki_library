include!("common.rs");
use std::{hint::black_box, time::Instant};

fn run<S: Set>(n: usize, initial: &[usize], queries: &[usize], kind: &str) -> (f64, usize) {
    let mut s = S::new(n);
    for &p in initial { s.insert(p); }
    let start = Instant::now();
    let mut sum = 0usize;
    for &p in black_box(queries) {
        let s = black_box(&s);
        let v = match kind {
            "strict" => if p & 1 == 0 { s.prev(p) } else { s.next(p) },
            "inclusive" => if p & 1 == 0 { s.inprev(p) } else { s.innext(p) },
            "extrema" => if p & 1 == 0 { s.min() } else { s.max() },
            _ => s.include(p) as usize,
        };
        sum = sum.wrapping_add(v);
    }
    black_box(sum);
    (start.elapsed().as_secs_f64() * 1e9 / queries.len() as f64, sum)
}

fn main() {
    let current_only = std::env::args().nth(1).is_some_and(|s| s == "current");
    let variants = if current_only { 2 } else { 7 };
    let n = 262145;
    let mut seed = 45678;
    let queries: Vec<_> = (0..1_000_000).map(|_| rand(&mut seed) % n).collect();
    println!("pattern,workload,implementation,repeat,ns_per_op,checksum");
    for pattern in ["empty", "single_middle", "word_end", "dense"] {
        let initial = match pattern {
            "empty" => vec![], "single_middle" => vec![n / 2],
            "word_end" => (63..n).step_by(64).collect(),
            _ => (0..n).step_by(2).collect(),
        };
        for kind in ["strict", "inclusive", "extrema", "include"] {
            let mut expected = None;
            for repeat in 0..7 {
                for j in 0..variants {
                    let index = if current_only { if (j + repeat) % 2 == 0 { 0 } else { 7 } }
                        else { (j + repeat) % 7 };
                    let (name, (ns, checksum)) = match index {
                        0 => ("baseline", run::<baseline::Predecessor64>(n, &initial, &queries, kind)),
                        1 => ("update_only", run::<update_only::Predecessor64>(n, &initial, &queries, kind)),
                        2 => ("incremental", run::<candidates::Incremental>(n, &initial, &queries, kind)),
                        3 => ("leaf_first", run::<candidates::LeafFirst>(n, &initial, &queries, kind)),
                        4 => ("flat", run::<flat::Flat>(n, &initial, &queries, kind)),
                        5 => ("shifted", run::<candidates::Shifted>(n, &initial, &queries, kind)),
                        6 => ("flat_shifted", run::<flat::FlatShifted>(n, &initial, &queries, kind)),
                        _ => ("current", run::<current::Predecessor64>(n, &initial, &queries, kind)),
                    };
                    if let Some(v) = expected { assert_eq!(checksum, v); } else { expected = Some(checksum); }
                    println!("{pattern},{kind},{name},{repeat},{ns:.5},{checksum}");
                }
            }
        }
    }
}
