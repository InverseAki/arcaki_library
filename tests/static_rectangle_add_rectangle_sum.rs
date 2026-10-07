#![allow(dead_code)]
include!("../src/Basic/static_rectangle_add_rectangle_sum.rs");
mod mint {
    include!("../src/NumberTheory/mint.rs");
}
type Rect = (i64, i64, i64, i64, i128);
type Query = (i64, i64, i64, i64);
const MOD: i128 = 998244353;

fn naive(rects: &[Rect], queries: &[Query]) -> Vec<i128> {
    queries
        .iter()
        .map(|&(lx, ly, rx, ry)| {
            rects
                .iter()
                .map(|&(a, b, c, d, w)| {
                    let dx = (i128::from(rx.min(c)) - i128::from(lx.max(a))).max(0);
                    let dy = (i128::from(ry.min(d)) - i128::from(ly.max(b))).max(0);
                    dx * dy * w
                })
                .sum()
        })
        .collect()
}

fn exact(rects: &[Rect], queries: &[Query]) -> Vec<i128> {
    let mut s = StaticRectangleAddRectangleSum::<i64>::build(rects.iter().copied());
    for &(lx, ly, rx, ry) in queries {
        s.push_query(lx, ly, rx, ry);
    }
    s.solve()
}

fn modular(rects: &[Rect], queries: &[Query]) -> Vec<i128> {
    type M = mint::Mint<998244353>;
    let mut s = StaticRectangleAddRectangleSum::<i64, i64, M>::build(
        rects
            .iter()
            .map(|&(lx, ly, rx, ry, w)| (lx, ly, rx, ry, M::new(w.rem_euclid(MOD) as usize))),
    );
    for &(lx, ly, rx, ry) in queries {
        s.push_query(lx, ly, rx, ry);
    }
    let cv = |v: &i64| M::new(i128::from(*v).rem_euclid(MOD) as usize);
    s.solve_with(M::new(0), cv, cv)
        .into_iter()
        .map(|v| v.val() as i128)
        .collect()
}

#[test]
fn exhaustive_overlap_and_boundaries() {
    let mut rects = Vec::new();
    for lx in -1..=1 {
        for rx in lx..=1 {
            for ly in -1..=1 {
                for ry in ly..=1 {
                    rects.push((lx, ly, rx, ry, 7));
                }
            }
        }
    }
    let mut queries = Vec::new();
    for lx in -2..=2 {
        for rx in lx..=2 {
            for ly in -2..=2 {
                for ry in ly..=2 {
                    queries.push((lx, ly, rx, ry));
                }
            }
        }
    }
    for &a in &rects {
        for &(lx, ly, rx, ry, _) in &rects {
            let rs = [a, (lx, ly, rx, ry, -11)];
            let expected = naive(&rs, &queries);
            assert_eq!(exact(&rs, &queries), expected);
            assert_eq!(
                modular(&rs, &queries),
                expected
                    .iter()
                    .map(|v| v.rem_euclid(MOD))
                    .collect::<Vec<_>>()
            );
        }
    }
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 7;
        self.0 ^= self.0 >> 9;
        self.0 ^= self.0 << 8;
        self.0
    }
    fn coord(&mut self, bound: u64) -> i64 {
        (self.next() % bound) as i64 - (bound / 2) as i64
    }
    fn query(&mut self, bound: u64) -> Query {
        let (a, b, c, d) = (
            self.coord(bound),
            self.coord(bound),
            self.coord(bound),
            self.coord(bound),
        );
        (a.min(c), b.min(d), a.max(c), b.max(d))
    }
}

fn workload(n: usize, q: usize, mode: &str, seed: u64) -> (Vec<Rect>, Vec<Query>) {
    let mut rng = Rng(seed);
    let bound = if mode == "ties" { 8 } else { 1_000_000_001 };
    let rects = (0..n)
        .map(|_| {
            let (lx, ly, rx, ry) = rng.query(bound);
            (lx, ly, rx, ry, rng.coord(2_000_000_001) as i128)
        })
        .collect();
    let queries = (0..q).map(|_| rng.query(bound)).collect();
    (rects, queries)
}

#[test]
fn random_exact_and_modular() {
    for seed in 1..=6000 {
        for mode in ["mixed", "ties"] {
            let (rects, queries) = workload(seed as usize % 37, seed as usize % 43, mode, seed);
            let expected = naive(&rects, &queries);
            assert_eq!(
                exact(&rects, &queries),
                expected,
                "seed={seed}, mode={mode}"
            );
            assert_eq!(
                modular(&rects, &queries),
                expected
                    .iter()
                    .map(|v| v.rem_euclid(MOD))
                    .collect::<Vec<_>>(),
                "modular seed={seed}, mode={mode}"
            );
        }
    }
}

#[test]
fn actual_lengths_large_weights_and_static_order() {
    let rects = [
        (0, 0, 1_000_000_000, 1_000_000_000, 998_244_352),
        (
            -1_000_000_000,
            -1_000_000_000,
            1_000_000_000,
            1_000_000_000,
            -998_244_351,
        ),
    ];
    let queries = [
        (-1_000_000_000, -1_000_000_000, 1_000_000_000, 1_000_000_000),
        (-17, 3, 1_000_000_000, 9),
        (0, 0, 3, 4),
        (0, 0, 0, 1),
    ];
    let expected = naive(&rects, &queries);
    assert_eq!(exact(&rects, &queries), expected);
    let mut s = StaticRectangleAddRectangleSum::<i64>::new();
    for &(lx, ly, rx, ry) in &queries {
        s.push_query(lx, ly, rx, ry);
    }
    for &(lx, ly, rx, ry, w) in &rects {
        s.push_add(lx, ly, rx, ry, w);
    }
    assert_eq!(s.solve(), expected);
    assert_eq!(
        exact(
            &[(
                1_000_000_000,
                2_000_000_000,
                1_000_000_007,
                2_000_000_011,
                3
            )],
            &[(1_000_000_002, 2_000_000_003, 1_000_000_010, 2_000_000_020)]
        ),
        vec![120]
    );
}

#[test]
fn coordinate_types_and_explicit_conversion() {
    let mut s = StaticRectangleAddRectangleSum::<i64, u64>::build([(-5, 0, 5, u64::MAX, 1)]);
    s.push_query(-1, u64::MAX - 10, 2, u64::MAX);
    assert_eq!(s.solve(), vec![30]);
    #[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
    struct Coord(i64);
    let mut s = StaticRectangleAddRectangleSum::<Coord, Coord>::build([(
        Coord(-5),
        Coord(2),
        Coord(7),
        Coord(10),
        3,
    )]);
    s.push_query(Coord(-2), Coord(3), Coord(1), Coord(7));
    assert_eq!(
        s.solve_with(0, |x| i128::from(x.0), |y| i128::from(y.0)),
        vec![36]
    );
    let mut s = StaticRectangleAddRectangleSum::<i32, i32, i64>::build([(0, 0, 3, 4, 5)]);
    s.push_query(1, 1, 5, 5);
    assert_eq!(s.solve(), vec![30]);
}

#[test]
fn empty_clone_and_reuse() {
    let mut s = StaticRectangleAddRectangleSum::<i64>::build([(0, 0, 3, 4, 5)]);
    s.push_query(1, 1, 5, 5);
    let mut c = s.clone();
    assert_eq!(c.solve(), vec![30]);
    assert_eq!(s.solve(), vec![30]);
    assert!(s.solve().is_empty());
    s.push_query(0, 0, 3, 4);
    assert_eq!(s.solve(), vec![0]);
    s.push_add(0, 0, 3, 4, 5);
    assert!(s.solve().is_empty());
    s.push_query(0, 0, 3, 4);
    assert_eq!(s.solve(), vec![0]);
    let mut s: StaticRectangleAddRectangleSum = Default::default();
    assert!(s.solve().is_empty());
    s.push_add(0, 0, 0, 1, 9);
    s.push_query(0, 0, 2, 2);
    assert_eq!(s.solve(), vec![0]);
    let mut s = StaticRectangleAddRectangleSum::<i64>::build([(0, 0, 1, 1, 3)]);
    s.push_query(0, 0, 0, 1);
    assert_eq!(s.solve(), vec![0]);
}

#[test]
#[should_panic(expected = "invalid rectangle")]
fn rejects_reversed_update() {
    StaticRectangleAddRectangleSum::<i64>::build([(1, 0, 0, 1, 3)]);
}
#[test]
#[should_panic(expected = "invalid rectangle")]
fn rejects_reversed_query() {
    StaticRectangleAddRectangleSum::<i64>::new().push_query(0, 1, 1, 0);
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let method = &args[1];
    let mode = &args[2];
    let n = args[3].parse().unwrap();
    let q = args[4].parse().unwrap();
    let (rects, queries) = workload(n, q, mode, 123456789);
    let start = std::time::Instant::now();
    let answers = match method.as_str() {
        "exact" => exact(&rects, &queries)
            .into_iter()
            .map(|v| v.rem_euclid(MOD))
            .collect::<Vec<_>>(),
        "modular" => modular(&rects, &queries),
        _ => panic!("unknown method"),
    };
    let elapsed = start.elapsed().as_secs_f64();
    let hash = answers.iter().fold(0u64, |h, &v| {
        h.wrapping_mul(1000000007).wrapping_add(v as u64)
    });
    println!("{elapsed:.6} {hash} {}", answers.len());
}
