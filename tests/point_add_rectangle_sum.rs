#![allow(dead_code)]
include!("../src/OfflineQuery/point_add_rectangle_sum.rs");
mod old {
    const I: i32 = i32::MAX;
    include!("support/point_add_rectangle_sum_baseline.rs");
}
mod compressed {
    const SHARED: bool = false;
    const SIMPLE_TIES: bool = true;
    include!("support/point_add_rectangle_sum_compressed.rs");
}

type Op = PointAddRectangleSumQuery;

fn register(s: &mut PointAddRectangleSum, ops: &[Op]) {
    for &op in ops {
        match op {
            Op::Add { x, y, w } => s.push_add(x, y, w),
            Op::Query { lx, ly, rx, ry } => s.push_query(lx, ly, rx, ry),
        }
    }
}

fn naive(initial: &[(i32, i32, i64)], ops: &[Op]) -> Vec<i64> {
    let mut points = initial.to_vec();
    let mut answers = Vec::new();
    for &op in ops {
        match op {
            Op::Add { x, y, w } => points.push((x, y, w)),
            Op::Query { lx, ly, rx, ry } => answers.push(
                points
                    .iter()
                    .filter(|&&(x, y, _)| lx <= x && x < rx && ly <= y && y < ry)
                    .map(|&(_, _, w)| w)
                    .sum(),
            ),
        }
    }
    answers
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 7;
        self.0 ^= self.0 >> 9;
        self.0 ^= self.0 << 8;
        self.0
    }
    fn coord(&mut self, bound: u64) -> i32 {
        (self.next() % bound) as i32 - (bound / 2) as i32
    }
}

fn workload(n: usize, q: usize, mode: &str, seed: u64) -> (Vec<(i32, i32, i64)>, Vec<Op>) {
    let mut rng = Rng(seed);
    let bound = if mode == "ties" { 8 } else { 1_000_000_001 };
    let initial = (0..n)
        .map(|_| (rng.coord(bound), rng.coord(bound), rng.coord(2001) as i64))
        .collect();
    let mut ops = Vec::with_capacity(q);
    for i in 0..q {
        let add = match mode {
            "static" => false,
            "add_heavy" => i % 10 != 9,
            "query_heavy" => i % 10 == 0,
            "only_add" => true,
            "query_first" => i >= q / 2,
            "add_first" => i < q / 2,
            _ => i % 2 == 0,
        };
        if add {
            ops.push(Op::Add {
                x: rng.coord(bound),
                y: rng.coord(bound),
                w: rng.coord(2001) as i64,
            });
        } else {
            let (x1, x2) = (rng.coord(bound), rng.coord(bound));
            let (y1, y2) = (rng.coord(bound), rng.coord(bound));
            ops.push(Op::Query {
                lx: x1.min(x2),
                rx: x1.max(x2),
                ly: y1.min(y2),
                ry: y1.max(y2),
            });
        }
    }
    (initial, ops)
}

fn run(initial: &[(i32, i32, i64)], ops: &[Op], method: &str) -> Vec<i64> {
    match method {
        "build" => {
            let mut s = PointAddRectangleSum::build(initial.iter().copied());
            register(&mut s, ops);
            s.solve()
        }
        "prepend" => {
            let mut s = PointAddRectangleSum::new();
            for &(x, y, w) in initial {
                s.push_add(x, y, w);
            }
            register(&mut s, ops);
            s.solve()
        }
        _ => {
            macro_rules! legacy {
                ($module:ident) => {{
                    let mut s = $module::PointAddRectangleSum::new();
                    for &(x, y, w) in initial {
                        s.push_add(x, y, w);
                    }
                    for &op in ops {
                        match op {
                            Op::Add { x, y, w } => s.push_add(x, y, w),
                            Op::Query { lx, ly, rx, ry } => s.push_query(lx, ly, rx, ry),
                        }
                    }
                    s.solve()
                }};
            }
            if method == "old" {
                legacy!(old)
            } else if method == "compressed" {
                legacy!(compressed)
            } else {
                panic!("unknown method");
            }
        }
    }
}

#[test]
fn differential_build_and_prepend() {
    for seed in 1..=3000 {
        for mode in [
            "mixed",
            "static",
            "ties",
            "add_heavy",
            "query_heavy",
            "only_add",
            "query_first",
            "add_first",
        ] {
            let (initial, ops) = workload(seed as usize % 35, seed as usize % 70, mode, seed);
            let expected = naive(&initial, &ops);
            for method in ["build", "prepend", "old", "compressed"] {
                assert_eq!(
                    run(&initial, &ops, method),
                    expected,
                    "seed={seed}, mode={mode}, method={method}"
                );
            }
        }
    }
}

#[test]
fn boundaries_initial_and_dynamic() {
    let initial = [
        (i32::MIN, i32::MIN, 7),
        (0, 0, 5),
        (0, 0, -2),
        (i32::MAX, i32::MAX, 9),
    ];
    let ops = [
        Op::Query {
            lx: i32::MIN,
            ly: i32::MIN,
            rx: i32::MAX,
            ry: i32::MAX,
        },
        Op::Query {
            lx: 0,
            ly: 0,
            rx: 1,
            ry: 1,
        },
        Op::Add { x: 0, y: 0, w: -10 },
        Op::Query {
            lx: 0,
            ly: 0,
            rx: 1,
            ry: 1,
        },
        Op::Query {
            lx: -1,
            ly: -1,
            rx: 0,
            ry: 0,
        },
        Op::Query {
            lx: 0,
            ly: 0,
            rx: 0,
            ry: 1,
        },
        Op::Query {
            lx: 0,
            ly: 0,
            rx: 1,
            ry: 0,
        },
    ];
    assert_eq!(run(&initial, &ops, "build"), vec![10, 3, -7, 0, 0, 0]);
    assert_eq!(run(&initial, &ops, "prepend"), vec![10, 3, -7, 0, 0, 0]);
}

#[test]
fn i64_u64_i128_and_independent_axes() {
    let mut s = PointAddRectangleSum::<i64>::build([(i64::MIN, i64::MIN, 3), (i64::MAX, 0, 9)]);
    s.push_query(i64::MIN, i64::MIN, i64::MAX, i64::MAX);
    s.push_add(-1, -1, 7);
    s.push_query(-1, -1, 0, 0);
    assert_eq!(s.solve(), vec![3, 7]);
    let mut s = PointAddRectangleSum::<u64>::build([(u64::MAX - 1, 0, 5)]);
    s.push_query(0, 0, u64::MAX, u64::MAX);
    assert_eq!(s.solve(), vec![5]);
    let mut s = PointAddRectangleSum::<i128, u64>::new();
    s.push_add(i128::MIN, u64::MAX - 1, 8);
    s.push_query(i128::MIN, 0, i128::MAX, u64::MAX);
    assert_eq!(s.solve(), vec![8]);
}

#[test]
fn clone_only_ordered_coordinates() {
    #[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
    struct Key(String);
    let k = |s: &str| Key(s.to_owned());
    let mut s = PointAddRectangleSum::<Key, String>::build([(k("a"), "apple".to_owned(), 5)]);
    s.push_query(k("a"), "a".to_owned(), k("b"), "z".to_owned());
    s.push_add(k("a"), "pear".to_owned(), -2);
    s.push_query(k("a"), "a".to_owned(), k("b"), "z".to_owned());
    assert_eq!(s.solve(), vec![5, 3]);
    #[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
    struct NonClone(String);
    let mut s = PointAddRectangleSum::build([(NonClone("a".into()), 3, 7)]);
    s.push_add(NonClone("b".into()), 4, -2);
    s.push_query(NonClone("a".into()), 0, NonClone("c".into()), 10);
    assert_eq!(s.solve(), vec![5]);
}

#[test]
fn empty_default_clone_and_reuse() {
    let mut s = PointAddRectangleSum::build([(1, 1, 7)]);
    s.push_query(0, 0, 2, 2);
    let mut c = s.clone();
    assert_eq!(c.solve(), vec![7]);
    assert_eq!(s.solve(), vec![7]);
    assert!(s.solve().is_empty());
    s.push_query(0, 0, 2, 2);
    assert_eq!(s.solve(), vec![0]);
    s.push_add(1, 1, -3);
    s.push_query(0, 0, 2, 2);
    assert_eq!(s.solve(), vec![-3]);
    let mut s: PointAddRectangleSum = PointAddRectangleSum::default();
    assert!(s.solve().is_empty());
    s.push_add(1, 1, 3);
    assert!(s.solve().is_empty());
    s.push_query(0, 0, 2, 2);
    assert_eq!(s.solve(), vec![0]);
    let mut s = PointAddRectangleSum::<i64>::build([]);
    s.push_query(0, 0, 2, 2);
    assert_eq!(s.solve(), vec![0]);
    let mut s = PointAddRectangleSum::build([(0, 0, 3)]);
    assert!(s.solve().is_empty());
    s.push_query(0, 0, 2, 2);
    assert_eq!(s.solve(), vec![0]);
}

#[test]
#[should_panic(expected = "invalid rectangle")]
fn reversed_rectangle_is_rejected() {
    let mut s = PointAddRectangleSum::new();
    s.push_query(2, 0, 1, 1);
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let method = &args[1];
    let mode = &args[2];
    let n: usize = args[3].parse().unwrap();
    let q: usize = args[4].parse().unwrap();
    let (initial, ops) = workload(n, q, mode, 123456789);
    let start = std::time::Instant::now();
    let ans = run(&initial, &ops, method);
    let elapsed = start.elapsed().as_secs_f64();
    let hash = ans.iter().fold(0u64, |h, &v| {
        h.wrapping_mul(1000000007).wrapping_add(v as u64)
    });
    println!("{elapsed:.6} {hash} {}", ans.len());
}
