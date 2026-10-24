include!("common.rs");
use std::{hint::black_box, time::Instant};
trait Build: Set {
    fn from_words(v: Vec<u64>) -> Self;
    fn from_usize(v: Vec<usize>) -> Self;
    fn from_string(v: String) -> Self;
}
BUILD_IMPLS
fn build<S: Build>(kind: &str, base: &[u64], string: &str) -> (f64, usize) {
    let words = base.to_vec();
    let integers = base.iter().map(|&v| v as usize).collect();
    let string = string.to_string();
    let start = Instant::now();
    let s = match kind {
        "words" => S::from_words(black_box(words)),
        "usize" => S::from_usize(black_box(integers)),
        _ => S::from_string(black_box(string)),
    };
    black_box(&s);
    let elapsed = start.elapsed().as_secs_f64() * 1e9;
    let checksum = s.min().wrapping_add(s.max());
    (elapsed, checksum)
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let repeats: usize = args[2].parse::<usize>().unwrap() * 3;
    let variants = VARIANTS;
    println!("words,pattern,constructor,implementation,repeat,ns_per_op,checksum");
    for words in [1, 64, 65, 4096, 4097, 156250, 262144, 262145] {
        for pattern in ["empty", "dense", "sparse", "random"] {
            let mut seed = 7890;
            let base: Vec<_> = (0..words).map(|i| match pattern {
                "empty" => 0, "dense" => !0, "sparse" => if i % 4096 == 63 { 1 << 63 } else { 0 },
                _ => if rand(&mut seed) & 1 == 0 { rand(&mut seed) as u64 } else { 0 },
            }).collect();
            let string: String = if words <= 156250 {
                (0..words*64).map(|p| if base[p>>6] >> (p & 63) & 1 == 0 { '0' } else { '1' }).collect()
            } else { String::new() };
            for constructor in ["words", "usize", "string"] {
                if constructor == "string" && words > 156250 { continue; }
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
                        println!("{words},{pattern},{constructor},{name},{repeat},{ns:.5},{checksum}");
                    }
                }
            }
        }
    }
}
