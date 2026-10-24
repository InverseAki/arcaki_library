use std::{hint::black_box, time::Instant};

macro_rules! candidate {
    ($name:ident) => {
        mod $name {
            include!(concat!(
                env!("MO_CANDIDATE_DIR"),
                "/",
                stringify!($name),
                ".rs"
            ));
            struct State<const P: usize, const A: usize> {
                counts: Vec<u32>,
                distinct: u64,
            }
            impl<const P: usize, const A: usize> SymMoState for State<P, A> {
                type Data = usize;
                type Query = [u64; P];
                type Ans = [u64; A];
                fn add(&mut self, _: usize, value: &usize) {
                    self.distinct += (self.counts[*value] == 0) as u64;
                    self.counts[*value] += 1;
                }
                fn sub(&mut self, _: usize, value: &usize) {
                    self.counts[*value] -= 1;
                    self.distinct -= (self.counts[*value] == 0) as u64;
                }
                fn ans(&mut self, id: usize, q: &Self::Query) -> Self::Ans {
                    [self.distinct ^ id as u64 ^ q.first().copied().unwrap_or(0); A]
                }
            }
            pub fn run<const P: usize, const A: usize>(
                n: usize,
                ranges: &[(usize, usize)],
            ) -> (u128, u64) {
                let start = std::time::Instant::now();
                let mut solver =
                    MoSolver::<State<P, A>>::new((0..n).map(|i| i * 11939 % 8192).collect());
                for (id, &(l, r)) in ranges.iter().enumerate() {
                    solver.add_query(l, r, [id as u64; P]);
                }
                let mut state = State {
                    counts: vec![0; 8192],
                    distinct: 0,
                };
                let answers = solver.solve(&mut state);
                let checksum = answers.iter().map(|a| a[0]).sum();
                std::hint::black_box((&solver, &answers, &state));
                (start.elapsed().as_nanos(), checksum)
            }
        }
    };
}
candidate!(baseline);
candidate!(iterative);
candidate!(slices);
candidate!(unchecked);
candidate!(flat);
candidate!(uninit);
candidate!(adopted);
fn random(seed: &mut u64) -> usize {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed as usize
}
fn main() {
    let mut seed = 0x123456789abcdefu64;
    let names = [
        "baseline",
        "iterative",
        "slices",
        "unchecked",
        "flat",
        "uninit",
        "adopted",
    ];
    for (case, n, q) in [
        ("random", 200000, 200000),
        ("short", 200000, 200000),
        ("clustered", 200000, 200000),
        ("duplicates", 200000, 200000),
        ("few", 200000, 2000),
        ("tiny", 128, 256),
        ("query_heavy", 2000, 200000),
    ] {
        let ranges: Vec<_> = (0..q)
            .map(|id| {
                let x = random(&mut seed) % (n + 1);
                let y = random(&mut seed) % (n + 1);
                match case {
                    "short" => (x, (x + y % 64).min(n)),
                    "clustered" => {
                        let l = (id % 8) * (n / 8) + x % 128;
                        (l, (l + y % 2048).min(n))
                    }
                    "duplicates" => {
                        let l = x % 16 * (n / 32);
                        (l, l + n / 2)
                    }
                    _ => (x.min(y), x.max(y)),
                }
            })
            .collect();
        for payload in [0, 32] {
            let mut expected = None;
            for rep in 0..7 {
                for j in 0..names.len() {
                    let name = names[(j + rep) % names.len()];
                    macro_rules! run {
                        ($m:ident) => {
                            if payload == 0 {
                                $m::run::<0, 1>(n, &ranges)
                            } else {
                                $m::run::<32, 16>(n, &ranges)
                            }
                        };
                    }
                    let (ns, checksum) = match name {
                        "baseline" => run!(baseline),
                        "iterative" => run!(iterative),
                        "slices" => run!(slices),
                        "unchecked" => run!(unchecked),
                        "flat" => run!(flat),
                        "uninit" => run!(uninit),
                        _ => run!(adopted),
                    };
                    assert_eq!(*expected.get_or_insert(checksum), checksum);
                    println!("{case},{n},{q},{payload},{rep},{name},{ns},{checksum}");
                }
            }
        }
    }
    black_box(Instant::now());
}
