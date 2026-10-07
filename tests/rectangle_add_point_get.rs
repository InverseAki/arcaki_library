#![allow(dead_code)]
include!("../src/Basic/rectangle_add_point_get.rs");

type Rect = (i32, i32, i32, i32, i64);
type Op = RectangleAddPointGetQuery;

fn register(s: &mut RectangleAddPointGet, ops: &[Op]) {
    for &op in ops {
        match op {
            Op::Add { lx, ly, rx, ry, w } => s.push_add(lx, ly, rx, ry, w),
            Op::Query { x, y } => s.push_query(x, y),
        }
    }
}

fn naive(initial: &[Rect], ops: &[Op]) -> Vec<i64> {
    let mut rectangles = initial.to_vec();
    let mut answers = Vec::new();
    for &op in ops {
        match op {
            Op::Add { lx, ly, rx, ry, w } => rectangles.push((lx, ly, rx, ry, w)),
            Op::Query { x, y } => answers.push(
                rectangles
                    .iter()
                    .filter(|&&(lx, ly, rx, ry, _)| lx <= x && x < rx && ly <= y && y < ry)
                    .map(|&(_, _, _, _, w)| w)
                    .sum(),
            ),
        }
    }
    answers
}

fn library(initial: &[Rect], ops: &[Op], build: bool) -> Vec<i64> {
    let mut s = if build {
        RectangleAddPointGet::build(initial.iter().copied())
    } else {
        RectangleAddPointGet::new()
    };
    if !build {
        for &(lx, ly, rx, ry, w) in initial {
            s.push_add(lx, ly, rx, ry, w);
        }
    }
    register(&mut s, ops);
    s.solve()
}

fn original(initial: &[Rect], ops: &[Op]) -> Vec<i64> {
    let mut data: Vec<Op> = initial
        .iter()
        .map(|&(lx, ly, rx, ry, w)| Op::Add { lx, ly, rx, ry, w })
        .collect();
    data.extend_from_slice(ops);
    let mut ans = vec![0; data.len()];
    let mut bit = RectangleAddPointGetBit::new(2 * initial.len() + ops.len());
    fn dfs(l: usize, r: usize, data: &[Op], bit: &mut RectangleAddPointGetBit, ans: &mut [i64]) {
        if r - l <= 1 {
            return;
        }
        let m = (l + r) / 2;
        dfs(l, m, data, bit, ans);
        dfs(m, r, data, bit, ans);
        let mut qs = Vec::new();
        let mut ys = Vec::new();
        for &op in &data[l..m] {
            if let Op::Add { lx, ly, rx, ry, w } = op {
                qs.extend([
                    (lx, false, ly, w),
                    (lx, false, ry, -w),
                    (rx, false, ly, -w),
                    (rx, false, ry, w),
                ]);
                ys.extend([ly, ry]);
            }
        }
        if ys.is_empty() {
            return;
        }
        let mut has_query = false;
        for (i, &op) in data.iter().enumerate().take(r).skip(m) {
            if let Op::Query { x, y } = op {
                has_query = true;
                qs.push((x, true, y, i as i64));
            }
        }
        if !has_query {
            return;
        }
        qs.sort_unstable();
        ys.sort_unstable();
        ys.dedup();
        for &(_, query, y, w) in &qs {
            if query {
                ans[w as usize] += bit.prefix(ys.partition_point(|&v| v <= y));
            } else {
                bit.add(ys.partition_point(|&v| v < y), w);
            }
        }
    }
    dfs(0, data.len(), &data, &mut bit, &mut ans);
    data.iter()
        .enumerate()
        .filter_map(|(i, op)| matches!(op, Op::Query { .. }).then_some(ans[i]))
        .collect()
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
    fn rect(&mut self, bound: u64) -> Rect {
        let (x1, x2) = (self.coord(bound), self.coord(bound));
        let (y1, y2) = (self.coord(bound), self.coord(bound));
        (
            x1.min(x2),
            y1.min(y2),
            x1.max(x2),
            y1.max(y2),
            self.coord(2001) as i64,
        )
    }
}

fn workload(n: usize, q: usize, mode: &str, seed: u64) -> (Vec<Rect>, Vec<Op>) {
    let mut rng = Rng(seed);
    let bound = if mode == "ties" { 8 } else { 1_000_000_001 };
    let initial = (0..n).map(|_| rng.rect(bound)).collect();
    let mut ops = Vec::with_capacity(q);
    for i in 0..q {
        let add = match mode {
            "static" => false,
            "only_add" => true,
            "add_heavy" => i % 10 != 9,
            "query_heavy" => i % 10 == 0,
            "query_first" => i >= q / 2,
            "add_first" => i < q / 2,
            _ => i % 2 == 0,
        };
        if add {
            let (lx, ly, rx, ry, w) = rng.rect(bound);
            ops.push(Op::Add { lx, ly, rx, ry, w });
        } else {
            ops.push(Op::Query {
                x: rng.coord(bound),
                y: rng.coord(bound),
            });
        }
    }
    (initial, ops)
}

#[test]
fn differential() {
    for seed in 1..=3000 {
        for mode in [
            "mixed",
            "static",
            "ties",
            "only_add",
            "add_heavy",
            "query_heavy",
            "query_first",
            "add_first",
        ] {
            let (initial, ops) = workload(seed as usize % 35, seed as usize % 70, mode, seed);
            let expected = naive(&initial, &ops);
            assert_eq!(
                library(&initial, &ops, true),
                expected,
                "build: seed={seed} mode={mode}"
            );
            assert_eq!(
                library(&initial, &ops, false),
                expected,
                "prepend: seed={seed} mode={mode}"
            );
            assert_eq!(
                original(&initial, &ops),
                expected,
                "original: seed={seed} mode={mode}"
            );
        }
    }
}

#[test]
fn exhaustive_boundary_time_and_overlap() {
    let mut rectangles = Vec::new();
    for lx in -1..=1 {
        for rx in lx..=1 {
            for ly in -1..=1 {
                for ry in ly..=1 {
                    rectangles.push((lx, ly, rx, ry, 7));
                }
            }
        }
    }
    for &a in &rectangles {
        for &b in &rectangles {
            let points: Vec<_> = (-2..=2)
                .flat_map(|x| (-2..=2).map(move |y| Op::Query { x, y }))
                .collect();
            let mut ops = points.clone();
            let (lx, ly, rx, ry, _) = b;
            ops.push(Op::Add {
                lx,
                ly,
                rx,
                ry,
                w: -11,
            });
            ops.extend(points);
            let expected = naive(&[a], &ops);
            assert_eq!(library(&[a], &ops, true), expected);
            assert_eq!(library(&[a], &ops, false), expected);
        }
    }
}

#[test]
fn extrema_heterogeneous_and_owned_types() {
    let mut s = RectangleAddPointGet::<i64>::build([(i64::MIN, i64::MIN, i64::MAX, i64::MAX, 7)]);
    s.push_query(i64::MIN, i64::MIN);
    s.push_query(i64::MAX, 0);
    s.push_query(0, i64::MAX);
    s.push_add(-1, -1, 1, 1, -3);
    s.push_query(0, 0);
    assert_eq!(s.solve(), vec![7, 0, 0, 4]);
    let mut s = RectangleAddPointGet::<i128, u64>::build([(i128::MIN, 0, i128::MAX, u64::MAX, 9)]);
    s.push_query(i128::MIN, u64::MAX - 1);
    s.push_query(0, u64::MAX);
    assert_eq!(s.solve(), vec![9, 0]);
    #[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
    struct Key(String);
    let k = |s: &str| Key(s.to_owned());
    let mut s = RectangleAddPointGet::<Key, String>::build([(
        k("a"),
        "apple".into(),
        k("z"),
        "pear".into(),
        5,
    )]);
    s.push_query(k("a"), "apple".into());
    s.push_add(k("a"), "a".into(), k("z"), "z".into(), -2);
    s.push_query(k("b"), "banana".into());
    s.push_query(k("z"), "apple".into());
    assert_eq!(s.solve(), vec![5, 3, 0]);
}

#[test]
fn bit_returns_to_zero_after_complete_sweep() {
    let mut s = RectangleAddPointGet::new();
    s.push_add(0, 0, 100, 100, 7);
    s.push_query(1, 1);
    s.push_add(-10, -10, 200, 200, -3);
    s.push_query(2, 2);
    s.push_query(-20, 2);
    s.push_query(3, 3);
    assert_eq!(s.solve(), vec![7, 4, 0, 4]);
    let lx = 0;
    let rx = 100;
    let mut seq = vec![];
    RectangleAddPointGet::<i32>::rectangle_events(&lx, 0, &rx, 2, 7, &mut seq);
    let mut bit = RectangleAddPointGetBit::new(2);
    RectangleAddPointGet::<i32>::sweep(&mut seq, &mut bit, &mut []);
    assert!(bit.data.iter().all(|&v| v == 0));
}

#[test]
fn empty_clone_and_reuse() {
    let mut s = RectangleAddPointGet::build([(0, 0, 2, 2, 3)]);
    s.push_query(1, 1);
    let mut c = s.clone();
    assert_eq!(c.solve(), vec![3]);
    assert_eq!(s.solve(), vec![3]);
    assert!(s.solve().is_empty());
    s.push_query(1, 1);
    assert_eq!(s.solve(), vec![0]);
    s.push_add(0, 0, 2, 2, -5);
    s.push_query(1, 1);
    assert_eq!(s.solve(), vec![-5]);
    let mut s: RectangleAddPointGet = RectangleAddPointGet::default();
    assert!(s.solve().is_empty());
    s.push_add(0, 0, 1, 1, 8);
    assert!(s.solve().is_empty());
    s.push_query(0, 0);
    assert_eq!(s.solve(), vec![0]);
    let mut s = RectangleAddPointGet::<u64>::build([]);
    s.push_add(0, 0, 0, 1, 5);
    s.push_add(0, 0, 1, 0, 5);
    s.push_query(0, 0);
    assert_eq!(s.solve(), vec![0]);
    let mut s = RectangleAddPointGet::build([(0, 0, 2, 2, 3)]);
    assert!(s.solve().is_empty());
    s.push_query(0, 0);
    assert_eq!(s.solve(), vec![0]);
}

#[test]
#[should_panic(expected = "invalid rectangle")]
fn invalid_dynamic_rectangle() {
    RectangleAddPointGet::new().push_add(1, 0, 0, 1, 1);
}

#[test]
#[should_panic(expected = "invalid rectangle")]
fn invalid_initial_rectangle() {
    RectangleAddPointGet::build([(0, 1, 1, 0, 1)]);
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let method = &args[1];
    let mode = &args[2];
    let n = args[3].parse().unwrap();
    let q = args[4].parse().unwrap();
    let (initial, ops) = workload(n, q, mode, 123456789);
    let start = std::time::Instant::now();
    let answers = match method.as_str() {
        "original" => original(&initial, &ops),
        "prepend" => library(&initial, &ops, false),
        "build" => library(&initial, &ops, true),
        _ => panic!("unknown method"),
    };
    let elapsed = start.elapsed().as_secs_f64();
    let hash = answers.iter().fold(0u64, |h, &v| {
        h.wrapping_mul(1000000007).wrapping_add(v as u64)
    });
    println!("{elapsed:.6} {hash} {}", answers.len());
}
