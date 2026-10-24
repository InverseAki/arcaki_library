include!("common.rs");
use std::{hint::black_box, time::Instant};

fn run<S: Set>(n: usize, initial: &[usize], ops: &[usize], kind: &str) -> (f64, usize) {
    let mut s = S::new(n);
    for &p in initial { s.insert(p); }
    let start = Instant::now();
    let mut sum = 0usize;
    for &p in black_box(ops) {
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
    (start.elapsed().as_secs_f64() * 1e9 / ops.len() as f64, sum)
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let count = args[1].parse().unwrap();
    let repeats = args[2].parse().unwrap();
    let variants = VARIANTS;
    let n = 262145;
    let mut seed = 45678;
    let ops: Vec<_> = (0..count).map(|_| rand(&mut seed) % n).collect();
    println!("pattern,workload,implementation,repeat,ns_per_op,checksum");
    for pattern in ["empty", "single_middle", "word_end", "dense"] {
        let initial = match pattern {
            "empty" => vec![], "single_middle" => vec![n / 2],
            "word_end" => (63..n).step_by(64).collect(),
            _ => (0..n).step_by(2).collect(),
        };
        for kind in ["strict", "inclusive", "extrema", "include"] {
            let mut expected = None;
            for repeat in 0..repeats {
                for j in 0..variants {
                    let index = (j + repeat) % variants;
                    let (name, (ns, checksum)) = match index {
                        MATCH_ARMS
                        _ => unreachable!(),
                    };
                    if let Some(v) = expected { assert_eq!(checksum, v); } else { expected = Some(checksum); }
                    println!("{pattern},{kind},{name},{repeat},{ns:.5},{checksum}");
                }
            }
        }
    }
}
