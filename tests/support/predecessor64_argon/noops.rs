include!("common.rs");
use std::{hint::black_box, time::Instant};
fn run<S: Set>(n: usize, initial: &[usize], ops: &[usize], kind: &str) -> (f64, usize) {
    let mut s = S::new(n);
    for &p in initial { s.insert(p); }
    let start = Instant::now();
    if kind == "insert_present" {
        for &p in black_box(ops) { s.insert(p); }
    } else {
        for &p in black_box(ops) { s.remove(p); }
    }
    black_box(&s);
    let ns = start.elapsed().as_secs_f64() * 1e9 / ops.len() as f64;
    (ns, s.min().wrapping_add(s.max()))
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let count = args[1].parse().unwrap();
    let repeats = args[2].parse().unwrap();
    let variants = VARIANTS;
    let n = 262144;
    println!("stride,workload,implementation,repeat,ns_per_op,checksum");
    for stride in [2, 64, 4096] {
        let initial: Vec<_> = (0..n).step_by(stride).collect();
        for kind in ["insert_present", "remove_absent"] {
            let mut seed = 98765;
            let ops: Vec<_> = (0..count).map(|_| {
                if kind == "insert_present" { initial[rand(&mut seed) % initial.len()] }
                else { let p = rand(&mut seed) % n; if p % stride == 0 { p + 1 } else { p } }
            }).collect();
            let mut expected = None;
            for repeat in 0..repeats {
                for j in 0..variants {
                    let index = (j + repeat) % variants;
                    let (name, (ns, checksum)) = match index {
                        MATCH_ARMS
                        _ => unreachable!(),
                    };
                    if let Some(value) = expected { assert_eq!(checksum, value); }
                    else { expected = Some(checksum); }
                    println!("{stride},{kind},{name},{repeat},{ns:.5},{checksum}");
                }
            }
        }
    }
}
