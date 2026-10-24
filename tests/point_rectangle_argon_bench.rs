#![allow(dead_code)]
mod baseline { include!("support/point_rectangle_argon/baseline.rs"); }
use baseline::*;
mod candidate { include!("support/point_rectangle_argon/config.rs"); }

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
            "blocks" => i % 256 < 128,
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
    for (i, op) in ops.iter_mut().enumerate() {
        match op {
            Op::Query { lx,ly,rx,ry } => match mode {
                "narrow" => { *rx = *lx + 100; *ry = *ly + 100; },
                "empty" => *rx = *lx,
                "outside" => { *ly=1_000_000_002; *ry=1_000_000_003; },
                "full" => { *lx=i32::MIN; *ly=i32::MIN; *rx=i32::MAX; *ry=i32::MAX; },
                "sorted" => { *lx=0; *rx=i as i32; },
                _ => {},
            },
            Op::Add{x,w,..} => { if mode == "zero" { *w=0; } if mode == "sorted" { *x=i as i32; } },
        }
    }
    (initial, ops)
}

fn run(initial: &[(i32,i32,i64)], ops: &[Op], method: &str) -> Vec<i64> {
    macro_rules! go { ($m:ident) => {{
        let mut s = $m::PointAddRectangleSum::build(initial.iter().copied());
        for &op in ops { match op {
            Op::Add{x,y,w} => s.push_add(x,y,w),
            Op::Query{lx,ly,rx,ry} => s.push_query(lx,ly,rx,ry),
        }}
        s.solve()
    }}; }
    if method == "baseline" { go!(baseline) } else { go!(candidate) }
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
