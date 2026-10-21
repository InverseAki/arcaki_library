#![allow(dead_code)]
use std::{hint::black_box, time::Instant};
mod baseline {
    mod original {
        include!("support/sparsetable_nested_baseline.rs");
    }
    pub struct SparseTableMI<T: Copy>(original::SparseTableMI<T>);
    impl<T: Copy> SparseTableMI<T> {
        pub fn build_by_key<K: Ord>(a: Vec<T>, key: impl Fn(T) -> K) -> Self {
            Self(original::SparseTableMI::build_by_key(&a, key))
        }
        pub fn query_by_key<K: Ord>(&self, l: usize, r: usize, key: impl Fn(T) -> K) -> T {
            self.0.query_by_key(l, r, key)
        }
    }
    impl<T: Ord + Copy> SparseTableMI<T> {
        pub fn build(a: Vec<T>) -> Self {
            Self(original::SparseTableMI::build(&a))
        }
        pub fn query(&self, l: usize, r: usize) -> T {
            self.0.query(l, r)
        }
    }
    pub struct SparseTableMX<T: Ord + Copy>(original::SparseTableMX<T>);
    impl<T: Ord + Copy> SparseTableMX<T> {
        pub fn build(a: Vec<T>) -> Self {
            Self(original::SparseTableMX::build(&a))
        }
    }
}
mod current {
    include!("../src/DataStructure/sparsetable.rs");
}
fn next(s: &mut u64) -> usize {
    *s ^= *s << 13;
    *s ^= *s >> 7;
    *s ^= *s << 17;
    *s as usize
}
fn median(mut a: Vec<f64>) -> f64 {
    a.sort_by(f64::total_cmp);
    a[a.len() / 2]
}
#[test]
#[ignore]
fn benchmark_sparsetable() {
    let mut state = 20261021u64;
    for n in [1024usize, 100_000, 1_000_000] {
        let a: Vec<u64> = (0..n).map(|_| next(&mut state) as u64).collect();
        let ids: Vec<usize> = (0..n).collect();
        macro_rules! build {
            ($m:ident, $kind:ident) => {{
                let input = a.clone();
                let t = Instant::now();
                let table = $m::$kind::build(black_box(input));
                let elapsed = t.elapsed().as_secs_f64() * 1000.;
                black_box(&table);
                elapsed
            }};
        }
        for kind in 0..3 {
            let label = ["min", "max", "by_key"][kind];
            let mut times = [Vec::new(), Vec::new()];
            for repeat in 0..12 {
                for j in 0..2 {
                    let v = (repeat + j) % 2;
                    macro_rules! run {
                        ($m:ident) => {
                            match kind {
                                0 => build!($m, SparseTableMI),
                                1 => build!($m, SparseTableMX),
                                _ => {
                                    let input = ids.clone();
                                    let t = Instant::now();
                                    let table =
                                        $m::SparseTableMI::build_by_key(black_box(input), |i| a[i]);
                                    let e = t.elapsed().as_secs_f64() * 1000.;
                                    black_box(&table);
                                    e
                                }
                            }
                        };
                    }
                    let t = match v {
                        0 => run!(baseline),
                        _ => run!(current),
                    };
                    if repeat >= 3 {
                        times[v].push(t);
                    }
                }
            }
            println!(
                "build n={n} kind={label} baseline/current ms: {:.3} {:.3}",
                median(times[0].clone()),
                median(times[1].clone())
            );
        }
        let qs: Vec<_> = (0..2_000_000)
            .map(|_| {
                let l = next(&mut state) % n;
                let r = l + 1 + next(&mut state) % (n - l);
                (l, r)
            })
            .collect();
        let b = baseline::SparseTableMI::build(a.clone());
        let c = current::SparseTableMI::build(a.clone());
        let mut bt = Vec::new();
        let mut ct = Vec::new();
        for _ in 0..7 {
            let t = Instant::now();
            let mut sum = 0;
            for &(l, r) in black_box(&qs) {
                sum ^= b.query(l, r);
            }
            let expected = black_box(sum);
            bt.push(t.elapsed().as_secs_f64() * 1000.);
            let t = Instant::now();
            let mut sum = 0;
            for &(l, r) in black_box(&qs) {
                sum ^= c.query(l, r);
            }
            black_box(sum);
            ct.push(t.elapsed().as_secs_f64() * 1000.);
            assert_eq!(sum, expected);
        }
        println!(
            "query n={n} baseline/current ms: {:.3} {:.3}",
            median(bt),
            median(ct)
        );
    }
}
