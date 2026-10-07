include!("common.rs");
use std::{hint::black_box, time::Instant};

macro_rules! time_build {
    ($name:expr, $build:expr) => {{
        let start = Instant::now();
        let set = $build;
        black_box(&set);
        let elapsed = start.elapsed().as_secs_f64() * 1e9;
        black_box(set);
        ($name, elapsed)
    }};
}

fn main() {
    println!("words,pattern,constructor,implementation,repeat,ns_per_build");
    for words in [1, 65, 4097, 262145] {
        for pattern in ["empty", "dense", "sparse"] {
            let base: Vec<u64> = (0..words).map(|i| match pattern {
                "empty" => 0, "dense" => !0, _ => if i % 64 == 63 { 1 << 63 } else { 0 },
            }).collect();
            let string = if words <= 4097 {
                (0..words * 64).map(|p| if base[p >> 6] >> (p & 63) & 1 != 0 { '1' } else { '0' }).collect()
            } else { String::new() };
            for constructor in ["new", "from_vec_u64", "from_vec_usize", "from_01_string"] {
                if constructor == "from_01_string" && words > 4097 { continue; }
                for repeat in 0..21 {
                    for j in 0..2 {
                        let old = (repeat + j) % 2 == 0;
                        let (name, ns) = match constructor {
                            "new" => if old {
                                time_build!("baseline", baseline::Predecessor64::new(words * 64))
                            } else { time_build!("current", current::Predecessor64::new(words * 64)) },
                            "from_vec_u64" => {
                                let input = base.clone();
                                if old { time_build!("baseline", baseline::Predecessor64::from_vec_u64(black_box(input))) }
                                else { time_build!("current", current::Predecessor64::from_vec_u64(black_box(input))) }
                            },
                            "from_vec_usize" => {
                                let input: Vec<_> = base.iter().map(|&v| v as usize).collect();
                                if old { time_build!("baseline", baseline::Predecessor64::from_vec_usize(black_box(input))) }
                                else { time_build!("current", current::Predecessor64::from_vec_usize(black_box(input))) }
                            },
                            _ => {
                                let input = string.clone();
                                if old { time_build!("baseline", baseline::Predecessor64::from_01_string(black_box(input))) }
                                else { time_build!("current", current::Predecessor64::from_01_string(black_box(input))) }
                            },
                        };
                        println!("{words},{pattern},{constructor},{name},{repeat},{ns:.5}");
                    }
                }
            }
        }
    }
}
