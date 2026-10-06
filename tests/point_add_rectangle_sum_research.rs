#![allow(dead_code)]
mod baseline {
    const I: i32 = i32::MAX;
    include!("../src/Basic/point_add_rectangle_sum.rs");
}
#[path = "support/point_add_rectangle_sum_presort.rs"]
mod presort;
mod compressed {
    const SHARED: bool = false;
    const SIMPLE_TIES: bool = false;
    include!("support/point_add_rectangle_sum_compressed.rs");
}
mod shared {
    const SHARED: bool = true;
    const SIMPLE_TIES: bool = false;
    include!("support/point_add_rectangle_sum_compressed.rs");
}
mod simple_ties {
    const SHARED: bool = false;
    const SIMPLE_TIES: bool = true;
    include!("support/point_add_rectangle_sum_compressed.rs");
}
use baseline::PointAddRectangleSumQuery as Op;

fn compressed(ops: &[Op], method: &str) -> Vec<i64> {
    macro_rules! run {
        ($module:ident) => {{
            let mut solver = $module::PointAddRectangleSum::new();
            for &op in ops {
                match op {
                    Op::Add { x, y, w } => solver.push_add(x, y, w),
                    Op::Query { lx, ly, rx, ry } => solver.push_query(lx, ly, rx, ry),
                }
            }
            solver.solve()
        }};
    }
    match method {
        "shared" => run!(shared),
        "simple_ties" => run!(simple_ties),
        _ => run!(compressed),
    }
}

fn baseline(ops: &[Op]) -> Vec<i64> {
    let mut solver = baseline::PointAddRectangleSum::new();
    for &op in ops {
        match op {
            Op::Add { x, y, w } => solver.push_add(x, y, w),
            Op::Query { lx, ly, rx, ry } => solver.push_query(lx, ly, rx, ry),
        }
    }
    solver.solve()
}

fn naive(ops: &[Op]) -> Vec<i64> {
    let mut points = Vec::new();
    let mut answers = Vec::new();
    for &op in ops {
        match op {
            Op::Add { x, y, w } => points.push((x, y, w)),
            Op::Query { lx, ly, rx, ry } => answers.push(points.iter()
                .filter(|&&(x, y, _)| lx <= x && x < rx && ly <= y && y < ry)
                .map(|&(_, _, w)| w).sum()),
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
    fn coord(&mut self, bound: u64) -> i32 { (self.next() % bound) as i32 - (bound / 2) as i32 }
}

fn workload(n: usize, mode: &str, seed: u64) -> Vec<Op> {
    let mut rng = Rng(seed);
    let mut result = Vec::with_capacity(n);
    let bound = if mode == "ties" { 8 } else { 1_000_000_001 };
    for i in 0..n {
        let add = match mode {
            "initial" => i < n / 2,
            "add_heavy" => i % 10 != 9,
            "query_heavy" => i % 10 == 0,
            "query_first" => i >= n / 2,
            "only_add" => true,
            "only_query" => false,
            _ => i % 2 == 0,
        };
        if add {
            let x = if mode == "sorted" { i as i32 } else { rng.coord(bound) };
            result.push(Op::Add { x, y: rng.coord(bound), w: rng.coord(2001) as i64 });
        } else {
            let (x1, x2) = (rng.coord(bound), rng.coord(bound));
            let (y1, y2) = (rng.coord(bound), rng.coord(bound));
            result.push(Op::Query { lx: x1.min(x2), rx: x1.max(x2), ly: y1.min(y2), ry: y1.max(y2) });
        }
    }
    result
}

fn check(ops: &[Op]) {
    let expected = naive(ops);
    assert_eq!(baseline(ops), expected, "baseline: {ops:?}");
    assert_eq!(presort::solve::<false>(ops), expected, "presort: {ops:?}");
    assert_eq!(presort::solve::<true>(ops), expected, "add-only: {ops:?}");
    assert_eq!(compressed(ops, "compressed"), expected, "compressed: {ops:?}");
    assert_eq!(compressed(ops, "shared"), expected, "shared: {ops:?}");
    assert_eq!(compressed(ops, "simple_ties"), expected, "simple ties: {ops:?}");
    assert_eq!(presort::solve_merge(ops), expected, "merge: {ops:?}");
}

#[test]
fn boundaries() {
    check(&[]);
    check(&[
        Op::Query { lx: i32::MIN, ly: i32::MIN, rx: i32::MAX, ry: i32::MAX },
        Op::Add { x: i32::MIN, y: i32::MIN, w: 7 },
        Op::Add { x: i32::MAX, y: i32::MAX, w: -9 },
        Op::Add { x: 0, y: 0, w: -3 },
        Op::Query { lx: i32::MIN, ly: i32::MIN, rx: i32::MAX, ry: i32::MAX },
        Op::Query { lx: 0, ly: 0, rx: 1, ry: 1 },
        Op::Query { lx: -1, ly: -1, rx: 0, ry: 0 },
        Op::Query { lx: 0, ly: 0, rx: 0, ry: 1 },
        Op::Query { lx: 0, ly: 0, rx: 1, ry: 0 },
    ]);
    let mut solver = baseline::PointAddRectangleSum::new();
    assert!(solver.solve().is_empty());
    assert!(solver.solve().is_empty());
    for _ in 0..3 {
        solver.push_add(1, 1, 7);
        solver.push_query(1, 1, 2, 2);
        assert_eq!(solver.solve(), vec![7]);
    }
}

#[test]
fn differential() {
    for seed in 1..=2000 {
        for mode in ["mixed", "ties", "initial", "add_heavy", "query_heavy", "query_first", "only_add", "only_query", "sorted"] {
            check(&workload(seed as usize % 80 + 1, mode, seed));
        }
    }
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let method = &args[1];
    let mode = &args[2];
    let n: usize = args[3].parse().unwrap();
    let ops = workload(n, mode, 123456789);
    let start = std::time::Instant::now();
    let answers = match method.as_str() {
        "baseline" => baseline(&ops),
        "presort" => presort::solve::<false>(&ops),
        "add_only" => presort::solve::<true>(&ops),
        "compressed" | "shared" | "simple_ties" => compressed(&ops, method),
        "merge" => presort::solve_merge(&ops),
        _ => panic!("unknown method"),
    };
    let elapsed = start.elapsed().as_secs_f64();
    let hash = answers.iter().fold(0u64, |h, &v| h.wrapping_mul(1000000007).wrapping_add(v as u64));
    println!("{elapsed:.6} {hash} {}", answers.len());
}
