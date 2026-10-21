#![allow(dead_code)]
#[path = "support/integer_sort_candidates.rs"]
mod candidates;
use candidates::*;
include!("../src/Basic/split_mix.rs");
include!("../src/Basic/integer_sort.rs");
use std::{hint::black_box, time::Instant};

fn data<T: BenchInt>(n: usize, kind: &str, seed: u64) -> Vec<T> {
    let mut rng = SplitMix64::new(seed);
    let mut a: Vec<_> = (0..n)
        .map(|i| {
            T::from_bits(match kind {
                "random" => rng.next_u64(),
                "low32" => rng.next_u64() & 0xffff_ffff,
                "low16" => rng.next_u64() & 65535,
                "duplicates" => rng.next_u64() & 15,
                "highbits" => (rng.next_u64() & 65535) << (T::BITS - 16),
                "offset" => (u64::MAX - 65535) | (rng.next_u64() & 65535),
                "signed_boundary" => (1u64 << 63)
                    .wrapping_sub(32768)
                    .wrapping_add(rng.next_u64() & 65535),
                "sorted" | "reverse" | "nearly" => i as u64,
                "equal" => 17,
                "organ" => i.min(n - 1 - i) as u64,
                "saw" => (i % 257) as u64,
                _ => unreachable!(),
            })
        })
        .collect();
    if kind == "reverse" {
        a.reverse();
    }
    if kind == "nearly" && n > 0 {
        for _ in 0..(n / 100).max(1) {
            a.swap(rng.gen_range(0..n), rng.gen_range(0..n));
        }
    }
    a
}

fn run<T: BenchInt>(
    ty: &str,
    n: usize,
    kind: &str,
    a: &[T],
    expected: &[T],
    name: &str,
    mut f: impl FnMut(&mut [T]),
    rounds: usize,
) {
    let mut work = a.to_vec();
    f(&mut work);
    assert_eq!(work, expected, "{ty}/{n}/{kind}/{name}");
    let mut times = Vec::new();
    let batch = (8192 / n.max(1)).max(1);
    for _ in 0..rounds {
        let mut elapsed = std::time::Duration::ZERO;
        for _ in 0..batch {
            work.copy_from_slice(a);
            let start = Instant::now();
            f(black_box(&mut work));
            elapsed += start.elapsed();
            black_box(&work);
        }
        times.push(elapsed.as_secs_f64() * 1e6 / batch as f64);
    }
    times.sort_by(f64::total_cmp);
    println!(
        "RESULT,{ty},{n},{kind},{name},{:.4}",
        times[times.len() / 2]
    );
}

fn suite<T: BenchInt + IntegerSortValue>(ty: &str) {
    let quick = std::env::var_os("SORT_FULL").is_none();
    let adaptive = std::env::var_os("SORT_ADAPTIVE").is_some();
    let selected = std::env::var_os("SORT_SELECTED").is_some();
    let block = std::env::var_os("SORT_BLOCK").is_some();
    let sizes = if quick {
        vec![64, 256, 1024, 4096, 65536, 1000000]
    } else {
        vec![
            16, 64, 128, 256, 512, 1024, 2048, 4096, 16384, 65536, 200000, 1000000, 4000000,
        ]
    };
    for n in sizes {
        for kind in [
            "random",
            "low32",
            "low16",
            "duplicates",
            "highbits",
            "offset",
            "signed_boundary",
            "sorted",
            "reverse",
            "nearly",
            "equal",
            "organ",
            "saw",
        ] {
            let a = data::<T>(n, kind, 20261021);
            let mut expected = a.clone();
            expected.sort_unstable();
            let mut methods: Vec<(&str, fn(&mut [T]))> = vec![
                ("std", <[T]>::sort_unstable),
                ("lsd8", lsd::<T, 8, false, false>),
                ("lsd11", lsd::<T, 11, false, false>),
                ("lsd16", lsd::<T, 16, false, false>),
                ("all8", lsd::<T, 8, true, false>),
                ("all11", lsd::<T, 11, true, false>),
                ("unchecked8", lsd::<T, 8, false, true>),
                ("msd8", msd::<T>),
            ];
            if adaptive {
                methods = vec![
                    ("std", <[T]>::sort_unstable),
                    ("adaptive8", candidates::adaptive::<T, 8, true>),
                    ("adaptive10", candidates::adaptive::<T, 10, true>),
                    ("adaptive11", candidates::adaptive::<T, 11, true>),
                    ("adaptive12", candidates::adaptive::<T, 12, true>),
                    ("nocount11", candidates::adaptive::<T, 11, false>),
                ];
            }
            if selected {
                methods = vec![
                    ("std", <[T]>::sort_unstable),
                    ("selected", integer_sort::<T>),
                ];
            }
            if block {
                if kind != "random" || n < 4096 {
                    continue;
                }
                methods = vec![
                    ("std", <[T]>::sort_unstable),
                    ("selected", integer_sort::<T>),
                    ("block8x8", blocked::<T, 256, 8>),
                    ("block8x32", blocked::<T, 256, 32>),
                    ("block11x8", blocked::<T, 2048, 8>),
                    ("block11x32", blocked::<T, 2048, 32>),
                ];
            }
            if !quick && !selected && !adaptive && !block {
                methods.extend([
                    ("lsd10", lsd::<T, 10, false, false> as fn(&mut [T])),
                    ("lsd12", lsd::<T, 12, false, false>),
                ]);
            }
            for (name, f) in methods {
                run(
                    ty,
                    n,
                    kind,
                    &a,
                    &expected,
                    name,
                    f,
                    if quick { 3 } else { 5 },
                );
            }
            if selected {
                let mut buffer = Vec::new();
                run(
                    ty,
                    n,
                    kind,
                    &a,
                    &expected,
                    "reused",
                    |a| integer_sort_with_buffer(a, &mut buffer),
                    if quick { 3 } else { 5 },
                );
            }
        }
    }
}

#[test]
#[ignore]
fn compare() {
    println!("HEADER,type,n,distribution,method,median_us");
    suite::<u64>("u64");
    suite::<u32>("u32");
    if std::env::var_os("SORT_ALL_TYPES").is_some() {
        suite::<usize>("usize");
        suite::<i64>("i64");
    }
}
