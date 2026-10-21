#![allow(dead_code)]
use std::{hint::black_box, time::Instant};
mod current {
    include!("../src/DataStructure/sparsetable.rs");
}
mod candidates {
    include!("support/sparsetable_candidates.rs");
}
mod baseline {
    include!("support/sparsetable_nested_baseline.rs");
}
fn next(s: &mut u64) -> usize {
    *s ^= *s << 13;
    *s ^= *s >> 7;
    *s ^= *s << 17;
    *s as usize
}
fn measure<T: Copy, S>(
    a: &[T],
    qs: &[(usize, usize)],
    build: impl FnOnce(Vec<T>) -> S,
    query: impl Fn(&S, usize, usize) -> T,
    number: impl Fn(T) -> u64,
) -> (f64, f64, u64) {
    let a = a.to_vec();
    let start = Instant::now();
    let table = build(black_box(a));
    let built = start.elapsed().as_secs_f64() * 1000.;
    let start = Instant::now();
    let table = black_box(&table);
    let mut sum = 0u64;
    for &(l, r) in black_box(qs) {
        sum = sum.wrapping_add(number(query(table, l, r)));
    }
    black_box(sum);
    (built, start.elapsed().as_secs_f64() * 1000., sum)
}
macro_rules! benchmark_candidates {
($name:ident, $value:ty) => {
#[test]
#[ignore]
fn $name() {
    type Value = $value;
    let n = std::env::var("ST_N")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(500_000);
    let q = std::env::var("ST_Q")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1_000_000);
    let rounds = std::env::var("ST_ROUNDS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(9);
    let mut seed = 20261021u64;
    let values: Vec<Value> = (0..n).map(|_| (next(&mut seed) >> 8) as Value).collect();
    let ids: Vec<usize> = (0..n).collect();
    for keyed in [false, true] {
        for distribution in ["wide", "short", "power"] {
            let qs: Vec<_> = (0..q)
                .map(|_| {
                    let l = next(&mut seed) % n;
                    let len = match distribution {
                        "short" => 1 + next(&mut seed) % 64,
                        "power" => {
                            1 << (next(&mut seed) % (usize::BITS - n.leading_zeros()) as usize)
                        }
                        _ => 1 + next(&mut seed) % n,
                    };
                    (l, l + len.min(n - l))
                })
                .collect();
            let mut expected = None;
            for round in 0..rounds + 2 {
                for shift in 0..17 {
                    let case = (shift + round) % 17;
                    macro_rules! flat {
                        ($a:expr, $select:expr, $num:expr, $formula:literal, $init:literal, $unchecked:literal) => {
                            measure(
                                $a,
                                &qs,
                                |a| candidates::Flat::<_, $formula, $init>::build(a, $select),
                                |t, l, r| t.query::<$unchecked>(l, r, $select),
                                $num,
                            )
                        };
                    }
                    macro_rules! run {
                        ($a:expr, $select:expr, $num:expr, $key:expr) => {
                            match case {
                                0 => measure(
                                    $a,
                                    &qs,
                                    |a| baseline::SparseTableMI::build_by_key(&a, $key),
                                    |t, l, r| t.query_by_key(l, r, $key),
                                    $num,
                                ),
                                1 => measure(
                                    $a,
                                    &qs,
                                    |a| candidates::Nested::build(a, $select),
                                    |t, l, r| t.query::<false>(l, r, $select),
                                    $num,
                                ),
                                2 => measure(
                                    $a,
                                    &qs,
                                    |a| candidates::Nested::build(a, $select),
                                    |t, l, r| t.query::<true>(l, r, $select),
                                    $num,
                                ),
                                3 => flat!($a, $select, $num, false, false, false),
                                4 => flat!($a, $select, $num, false, false, true),
                                5 => flat!($a, $select, $num, true, false, false),
                                6 => flat!($a, $select, $num, true, false, true),
                                7 => flat!($a, $select, $num, false, true, true),
                                8 => measure(
                                    $a,
                                    &qs,
                                    |a| {
                                        candidates::Block::<_, 16>::build(
                                            a,
                                            |x| $key(x) as u64,
                                            $select,
                                        )
                                    },
                                    |t, l, r| t.query(l, r, $select),
                                    $num,
                                ),
                                9 => measure(
                                    $a,
                                    &qs,
                                    |a| {
                                        candidates::Block::<_, 32>::build(
                                            a,
                                            |x| $key(x) as u64,
                                            $select,
                                        )
                                    },
                                    |t, l, r| t.query(l, r, $select),
                                    $num,
                                ),
                                10 => measure(
                                    $a,
                                    &qs,
                                    |a| {
                                        candidates::Block::<_, 64>::build(
                                            a,
                                            |x| $key(x) as u64,
                                            $select,
                                        )
                                    },
                                    |t, l, r| t.query(l, r, $select),
                                    $num,
                                ),
                                11 => measure(
                                    $a,
                                    &qs,
                                    |a| candidates::BoxRows::build(a, $select),
                                    |t, l, r| t.query(l, r, $select),
                                    $num,
                                ),
                                12 => measure(
                                    $a,
                                    &qs,
                                    |a| candidates::Stride::build(a, $select),
                                    |t, l, r| t.query(l, r, $select),
                                    $num,
                                ),
                                13 => measure(
                                    $a,
                                    &qs,
                                    |a| candidates::PrefixSuffix::<_, 4>::build(a, $select),
                                    |t, l, r| t.query(l, r, $select),
                                    $num,
                                ),
                                14 => measure(
                                    $a,
                                    &qs,
                                    |a| candidates::PrefixSuffix::<_, 8>::build(a, $select),
                                    |t, l, r| t.query(l, r, $select),
                                    $num,
                                ),
                                15 => measure(
                                    $a,
                                    &qs,
                                    |a| candidates::PrefixSuffix::<_, 16>::build(a, $select),
                                    |t, l, r| t.query(l, r, $select),
                                    $num,
                                ),
                                _ => measure(
                                    $a,
                                    &qs,
                                    |a| current::SparseTableMI::build_by_key(a, $key),
                                    |t, l, r| t.query_by_key(l, r, $key),
                                    $num,
                                ),
                            }
                        };
                    }
                    let (build, query, sum) = if keyed {
                        run!(
                            &ids,
                            |x: usize, y: usize| if values[x] <= values[y] { x } else { y },
                            |x: usize| x as u64,
                            |x: usize| values[x]
                        )
                    } else {
                        run!(
                            &values,
                            |x: Value, y: Value| x.min(y),
                            |x: Value| x as u64,
                            |x: Value| x
                        )
                    };
                    assert_eq!(*expected.get_or_insert(sum), sum);
                    if round >= 2 {
                        println!("RESULT {} {n} {q} {keyed} {distribution} {case} {round} {build:.6} {query:.6}", stringify!($value));
                    }
                }
            }
        }
    }
}

};
}
benchmark_candidates!(benchmark_candidates_u32, u32);
benchmark_candidates!(benchmark_candidates_u64, u64);
