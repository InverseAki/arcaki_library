include!("common.rs");
use std::{hint::black_box, time::Instant};

#[derive(Clone, Copy)]
struct Op { kind: u8, p: usize }

fn run<S: Set>(n: usize, initial: &[usize], ops: &[Op]) -> (f64, usize) {
    let mut set = S::new(n);
    for &p in initial { set.insert(p); }
    let start = Instant::now();
    let mut sum = 0usize;
    for op in black_box(ops) {
        let v = match op.kind {
            0 => { set.insert(op.p); 0 },
            1 => { set.remove(op.p); 0 },
            2 => set.prev(op.p),
            3 => set.next(op.p),
            4 => set.inprev(op.p),
            5 => set.innext(op.p),
            _ => set.include(op.p) as usize,
        };
        sum = sum.wrapping_add(v);
    }
    black_box(&set);
    black_box(sum);
    (start.elapsed().as_secs_f64() * 1e9 / ops.len() as f64, sum)
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let count: usize = args.get(1).map(|s| s.parse().unwrap()).unwrap_or(1_000_000);
    let repeats: usize = args.get(2).map(|s| s.parse().unwrap()).unwrap_or(7);
    let current_only = args.get(3).is_some_and(|s| s == "current");
    let variants = if current_only { 2 } else { 7 };
    println!("n,stride,workload,implementation,repeat,ns_per_op,checksum");
    for n in [64, 4096, 262_144, 16_777_216] {
        for stride in [2, 64, 4096] {
            if stride > n { continue; }
            let mut seed = 12345;
            let mut initial: Vec<usize> = (0..n).step_by(stride).map(|p| (p + rand(&mut seed) % stride).min(n - 1)).collect();
            initial.sort_unstable();
            for workload in ["strict", "inclusive", "update", "mixed"] {
                let mut state = vec![true; initial.len()];
                let ops: Vec<Op> = (0..count).map(|_| {
                    let r = rand(&mut seed);
                    if workload == "update" || (workload == "mixed" && r % 2 == 0) {
                        let index = rand(&mut seed) % initial.len();
                        let present = if rand(&mut seed) % 4 == 0 { state[index] } else { !state[index] };
                        state[index] = present;
                        Op { kind: if present { 0 } else { 1 }, p: initial[index] }
                    } else {
                        Op { kind: match workload { "strict" => 2 + (r % 2) as u8,
                            "inclusive" => 4 + (r % 2) as u8, _ => 2 + (rand(&mut seed) % 4) as u8 }, p: rand(&mut seed) % n }
                    }
                }).collect();
                let mut expected = None;
                for repeat in 0..repeats {
                    for j in 0..variants {
                        let index = if current_only {
                            if (j + repeat) % 2 == 0 { 0 } else { 7 }
                        } else { (j + repeat) % 7 };
                        let (name, (ns, checksum)) = match index {
                            0 => ("baseline", run::<baseline::Predecessor64>(n, &initial, &ops)),
                            1 => ("update_only", run::<update_only::Predecessor64>(n, &initial, &ops)),
                            2 => ("incremental", run::<candidates::Incremental>(n, &initial, &ops)),
                            3 => ("leaf_first", run::<candidates::LeafFirst>(n, &initial, &ops)),
                            4 => ("flat", run::<flat::Flat>(n, &initial, &ops)),
                            5 => ("shifted", run::<candidates::Shifted>(n, &initial, &ops)),
                            6 => ("flat_shifted", run::<flat::FlatShifted>(n, &initial, &ops)),
                            _ => ("current", run::<current::Predecessor64>(n, &initial, &ops)),
                        };
                        if let Some(v) = expected { assert_eq!(checksum, v); }
                        else { expected = Some(checksum); }
                        println!("{n},{stride},{workload},{name},{repeat},{ns:.5},{checksum}");
                    }
                }
            }
        }
    }
}
