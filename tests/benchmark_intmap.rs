#![allow(dead_code)]
#[path = "../src/Basic/fxhash.rs"]
mod fx;
#[path = "../src/DataStructure/intmap.rs"]
mod implementation;
use fx::fxhash::FxMap;
use implementation::intmap::*;
use std::hint::black_box;
use std::time::Instant;

macro_rules! benchmark {
    ($name:ident, $key:ty, $map:ty) => {
        fn $name(keys: &[$key], probes: &[$key], op: &str) -> (u128, u64) {
            let mut map: $map = Default::default();
            let n = keys.len();
            if op != "grow" {
                map.reserve(n);
            }
            if op != "build" && op != "grow" && op != "entry_miss" {
                for (i, &k) in keys.iter().enumerate() {
                    map.insert(k, i as u64);
                }
            }
            let start = Instant::now();
            let mut sum = 0u64;
            match op {
                "build" | "grow" => {
                    for (i, &k) in keys.iter().enumerate() {
                        map.insert(black_box(k), i as u64);
                    }
                }
                "hit" => {
                    for k in keys {
                        sum = sum.wrapping_add(map.get(black_box(k)).copied().unwrap_or(0));
                    }
                }
                "miss" => {
                    for k in probes {
                        sum = sum.wrapping_add(map.get(black_box(k)).copied().unwrap_or(0));
                    }
                }
                "update" => {
                    for k in keys {
                        *map.get_mut(black_box(k)).unwrap() += 1;
                    }
                }
                "entry_hit" => {
                    for &k in keys {
                        *map.entry(black_box(k)).or_insert(0) += 1;
                    }
                }
                "entry_miss" => {
                    for &k in keys {
                        *map.entry(black_box(k)).or_insert(0) += 1;
                    }
                }
                "remove" => {
                    for k in keys {
                        sum = sum.wrapping_add(map.remove(black_box(k)).unwrap_or(0));
                    }
                }
                "mixed" => {
                    for (i, &k) in keys.iter().enumerate() {
                        sum = sum.wrapping_add(map.remove(black_box(&k)).unwrap_or(0));
                        map.insert(black_box(probes[i]), i as u64);
                        sum = sum.wrapping_add(
                            map.get(black_box(&keys[(i + n / 2) % n]))
                                .copied()
                                .unwrap_or(0),
                        );
                    }
                }
                "iter" => {
                    for (_, &v) in map.iter() {
                        sum = sum.wrapping_add(v);
                    }
                }
                _ => unreachable!(),
            }
            black_box(&map);
            black_box(sum);
            let elapsed = start.elapsed().as_nanos();
            // Observe all values after timing to keep writes live and compare backends.
            sum ^= map.iter().fold(0u64, |s, (&k, &v)| {
                s.wrapping_add((k as u64).wrapping_mul(31) ^ v)
            });
            (elapsed, sum ^ map.len() as u64)
        }
    };
}
benchmark!(int32, u32, U32Map<u64>);
benchmark!(fx32, u32, FxMap<u32, u64>);
benchmark!(int64, u64, U64Map<u64>);
benchmark!(fx64, u64, FxMap<u64, u64>);
benchmark!(intsize, usize, UsizeMap<u64>);
benchmark!(fxsize, usize, FxMap<usize, u64>);

fn random(x: &mut u64) -> u64 {
    *x = x.wrapping_add(0x9e3779b97f4a7c15);
    let mut z = *x;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}

fn main() {
    println!("key,n,pattern,operation,int_ns,fx_ns,speedup");
    for n in [1024, 100_000, 1_000_000] {
        for pattern in ["sequential", "random", "shifted"] {
            for kind in ["u32", "u64", "usize"] {
                let mut rng = 0x123456789abcdef;
                let make_key = |i: u64| match pattern {
                    "sequential" => i,
                    "random" if kind == "u32" => {
                        let mut x = i as u32;
                        x = (x ^ (x >> 16)).wrapping_mul(0x7feb352d);
                        x = (x ^ (x >> 15)).wrapping_mul(0x846ca68b);
                        (x ^ (x >> 16)) as u64
                    }
                    "random" => {
                        let mut x = i.wrapping_add(0x9e3779b97f4a7c15);
                        x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
                        x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
                        x ^ (x >> 31)
                    }
                    _ if kind == "u32" => (i as u32).rotate_left(16) as u64,
                    _ => i << 32,
                };
                let mut keys: Vec<u64> = (0..n as u64).map(make_key).collect();
                let mut probes: Vec<u64> = (n as u64..2 * n as u64).map(make_key).collect();
                for i in (1..n).rev() {
                    keys.swap(i, random(&mut rng) as usize % (i + 1));
                    probes.swap(i, random(&mut rng) as usize % (i + 1));
                }
                macro_rules! run {
                    ($int:ident, $fx:ident, $key:ty) => {{
                        let keys: Vec<$key> = keys.iter().map(|&k| k as $key).collect();
                        let probes: Vec<$key> = probes.iter().map(|&k| k as $key).collect();
                        for op in [
                            "build",
                            "grow",
                            "hit",
                            "miss",
                            "update",
                            "entry_hit",
                            "entry_miss",
                            "remove",
                            "mixed",
                            "iter",
                        ] {
                            let mut a = Vec::new();
                            let mut b = Vec::new();
                            for rep in 0..7 {
                                let (x, y) = if rep % 2 == 0 {
                                    ($int(&keys, &probes, op), $fx(&keys, &probes, op))
                                } else {
                                    let y = $fx(&keys, &probes, op);
                                    let x = $int(&keys, &probes, op);
                                    (x, y)
                                };
                                assert_eq!(x.1, y.1, "{} {} {}", kind, pattern, op);
                                if rep != 0 {
                                    a.push(x.0);
                                    b.push(y.0);
                                }
                            }
                            a.sort_unstable();
                            b.sort_unstable();
                            let x = (a[2] + a[3]) / 2;
                            let y = (b[2] + b[3]) / 2;
                            println!(
                                "{kind},{n},{pattern},{op},{x},{y},{:.4}",
                                y as f64 / x as f64
                            );
                        }
                    }};
                }
                match kind {
                    "u32" => run!(int32, fx32, u32),
                    "u64" => run!(int64, fx64, u64),
                    _ => run!(intsize, fxsize, usize),
                }
            }
        }
    }
}
