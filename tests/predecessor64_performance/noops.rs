include!("common.rs");
use std::{hint::black_box, time::Instant};

fn run<S: Set>(n: usize, initial: &[usize], positions: &[usize], insert: bool) -> f64 {
    let mut s = S::new(n);
    for &p in initial { s.insert(p); }
    let start = Instant::now();
    if insert {
        for &p in black_box(positions) { s.insert(p); }
    } else {
        for &p in black_box(positions) { s.remove(p); }
    }
    black_box(&s);
    start.elapsed().as_secs_f64() * 1e9 / positions.len() as f64
}

fn main() {
    let current_only = std::env::args().nth(1).is_some_and(|s| s == "current");
    let variants = if current_only { 2 } else { 7 };
    let n = 262144;
    println!("stride,workload,implementation,repeat,ns_per_op");
    for stride in [2, 64, 4096] {
        let initial: Vec<_> = (0..n).step_by(stride).collect();
        for insert in [true, false] {
            let mut seed = 98765;
            let positions: Vec<_> = (0..1_000_000).map(|_| {
                if insert { initial[rand(&mut seed) % initial.len()] }
                else { let p = rand(&mut seed) % n; if p % stride == 0 { p + 1 } else { p } }
            }).collect();
            let workload = if insert { "insert_present" } else { "remove_absent" };
            for repeat in 0..7 {
                for j in 0..variants {
                    let index = if current_only { if (j + repeat) % 2 == 0 { 0 } else { 7 } }
                        else { (j + repeat) % 7 };
                    let (name, ns) = match index {
                        0 => ("baseline", run::<baseline::Predecessor64>(n, &initial, &positions, insert)),
                        1 => ("update_only", run::<update_only::Predecessor64>(n, &initial, &positions, insert)),
                        2 => ("incremental", run::<candidates::Incremental>(n, &initial, &positions, insert)),
                        3 => ("leaf_first", run::<candidates::LeafFirst>(n, &initial, &positions, insert)),
                        4 => ("flat", run::<flat::Flat>(n, &initial, &positions, insert)),
                        5 => ("shifted", run::<candidates::Shifted>(n, &initial, &positions, insert)),
                        6 => ("flat_shifted", run::<flat::FlatShifted>(n, &initial, &positions, insert)),
                        _ => ("current", run::<current::Predecessor64>(n, &initial, &positions, insert)),
                    };
                    println!("{stride},{workload},{name},{repeat},{ns:.5}");
                }
            }
        }
    }
}
