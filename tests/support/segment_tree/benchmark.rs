#![allow(dead_code)]
use std::{hint::black_box, time::Instant};
fn next(s: &mut u64) -> usize {
    *s ^= *s << 13;
    *s ^= *s >> 7;
    *s ^= *s << 17;
    *s as usize
}
fn range(s: &mut u64, n: usize, pattern: &str, i: usize) -> (usize, usize) {
    match pattern {
        "point" => {
            let l = next(s) % n;
            (l, l + 1)
        }
        "short" | "slice" => {
            let l = next(s) % n;
            (l, (l + 1 + next(s) % 16).min(n))
        }
        "aligned" => {
            let w = 1 << (next(s) % (usize::BITS as usize - n.leading_zeros() as usize));
            let l = (next(s) % (n / w)) * w;
            (l, l + w)
        }
        "full" => (0, n),
        "sequential" => {
            let l = i % n;
            (l, (l + 17).min(n))
        }
        _ => {
            let a = next(s) % (n + 1);
            let b = next(s) % (n + 1);
            (a.min(b), a.max(b))
        }
    }
}
macro_rules! seg_bench {
    ($name:ident, $module:ident, $($instance:expr),*) => {
        fn $name(n:usize,q:usize,mode:&str,pattern:&str)->(u128,u64) {
            use $module::*;
            struct Sum;
            impl SegtreeMonoid for Sum { type S=u64; fn identity()->u64{0} seg_bench!(@op $($instance),*); }
            let input=vec![1u64;n];
            let mut s=43u64; let mut check=0;
            let mut tree=Segtree::<Sum>::from(input.clone(), $($instance),*);
            let now=Instant::now();
            if mode=="build" {for _ in 0..q {check^=black_box(Segtree::<Sum>::from(black_box(input.clone()), $($instance),*)).all_prod();}}
            else {for i in 0..q {
                let (l,r)=range(&mut s,n,pattern,i);
                match mode {
                    "prod" => check^=black_box(tree.prod(l,r)),
                    "set" => tree.set(l.min(n-1),black_box((i%17) as u64)),
                    "push" => tree.push(l.min(n-1),black_box(1)),
                    "search" => {check^=tree.max_right(l,|x|*x<=31) as u64;check^=tree.min_left(r,|x|*x<=31) as u64;},
                    _ => {if i%2==0 {tree.set(l.min(n-1),black_box((i%17) as u64));} else {check^=tree.prod(l,r);}}
                }
            }}
            let elapsed=now.elapsed().as_nanos(); (elapsed,black_box(check^tree.all_prod()))
        }
    };
    (@op) => {fn op(a:&u64,b:&u64)->u64{a+b}};
    (@op $instance:expr) => {fn op(&self,a:&u64,b:&u64)->u64{a+b}};
}
seg_bench!(plain_bench, plain,);
seg_bench!(state_bench, state, Sum);
macro_rules! lazy_bench {
    ($name:ident,$affine:expr) => {
        fn $name(n: usize, q: usize, mode: &str, pattern: &str) -> (u128, u64) {
            use lazy::*;
            const P: u64 = 998244353;
            struct Sum;
            impl SegtreeMonoid for Sum {
                type S = (u64, u64);
                fn identity() -> Self::S {
                    (0, 0)
                }
                fn op(a: &Self::S, b: &Self::S) -> Self::S {
                    if $affine {
                        ((a.0 + b.0) % P, a.1 + b.1)
                    } else {
                        (a.0 + b.0, a.1 + b.1)
                    }
                }
            }
            struct Action;
            impl LazySegtreeMonoid for Action {
                type M = Sum;
                type F = (u64, u64);
                fn identity() -> Self::F {
                    (1, 0)
                }
                fn map(f: &Self::F, x: &(u64, u64)) -> (u64, u64) {
                    if $affine {
                        ((f.0 * x.0 + f.1 * x.1) % P, x.1)
                    } else {
                        (x.0 + f.1 * x.1, x.1)
                    }
                }
                fn composition(f: &Self::F, g: &Self::F) -> Self::F {
                    if $affine {
                        ((f.0 * g.0) % P, (f.0 * g.1 + f.1) % P)
                    } else {
                        (1, f.1 + g.1)
                    }
                }
            }
            let input = vec![(1, 1); n];
            let mut tree = LazySegtree::<Action>::build(&input);
            let mut s = 43u64;
            let mut check = 0;
            let now = Instant::now();
            if mode == "build" {
                for _ in 0..q {
                    check ^= black_box(LazySegtree::<Action>::build(black_box(&input)))
                        .all_prod()
                        .0;
                }
            } else {
                for i in 0..q {
                    let (l, r) = range(&mut s, n, pattern, i);
                    match mode {
                        "prod" => check ^= black_box(tree.prod(l, r).0),
                        "set" => tree.set(l.min(n - 1), black_box(((i % 17) as u64, 1))),
                        "get" => check ^= tree.get(l.min(n - 1)).0,
                        "apply" => {
                            tree.apply_range(l, r, black_box((if $affine { 3 } else { 1 }, 1)))
                        }
                        "search" => {
                            check ^= tree.max_right(l, |x| x.0 <= 31) as u64;
                            check ^= tree.min_left(r, |x| x.0 <= 31) as u64;
                        }
                        "slice" => {
                            tree.apply_range(0, n, (1, 1));
                            check ^= black_box(tree.get_slice(l, r))
                                .iter()
                                .map(|x| x.0)
                                .sum::<u64>();
                        }
                        "updated_search" => {
                            if i % 3 == 0 {
                                tree.apply_range(l, r, (1, 1));
                            } else {
                                check ^= tree.max_right(l, |x| x.0 <= 10000) as u64;
                                check ^= tree.min_left(r, |x| x.0 <= 10000) as u64;
                            }
                        }
                        _ => {
                            if i % 2 == 0 {
                                tree.apply_range(l, r, black_box((if $affine { 3 } else { 1 }, 1)));
                            } else {
                                check ^= tree.prod(l, r).0;
                            }
                        }
                    }
                }
            }
            let elapsed = now.elapsed().as_nanos();
            (elapsed, black_box(check ^ tree.all_prod().0))
        }
    };
}
lazy_bench!(add_bench, false);
lazy_bench!(affine_bench, true);
fn matrix_bench(n: usize, q: usize, mode: &str, pattern: &str) -> (u128, u64) {
    use plain::*;
    #[derive(Clone)]
    struct Value {
        sum: u64,
        m: [u64; 4],
    }
    struct Matrix;
    impl SegtreeMonoid for Matrix {
        type S = Value;
        fn identity() -> Value {
            Value {
                sum: 0,
                m: [1, 0, 0, 1],
            }
        }
        fn op(a: &Value, b: &Value) -> Value {
            let mut m = [0; 4];
            for i in 0..2 {
                for j in 0..2 {
                    m[i * 2 + j] = (a.m[i * 2] * b.m[j] + a.m[i * 2 + 1] * b.m[2 + j]) % 998244353;
                }
            }
            Value {
                sum: a.sum + b.sum,
                m,
            }
        }
    }
    let input = vec![
        Value {
            sum: 1,
            m: [1, 1, 1, 0]
        };
        n
    ];
    let mut t = Segtree::<Matrix>::from(input.clone());
    let mut s = 43;
    let mut check = 0;
    let now = Instant::now();
    if mode == "build" {
        for _ in 0..q {
            check ^= black_box(Segtree::<Matrix>::from(black_box(input.clone())))
                .all_prod()
                .m[0];
        }
    } else {
        for i in 0..q {
            let (l, r) = range(&mut s, n, pattern, i);
            match mode {
                "set" => t.set(
                    l.min(n - 1),
                    black_box(Value {
                        sum: 1,
                        m: [1, i as u64 % 17, 1, 0],
                    }),
                ),
                "push" => t.push(
                    l.min(n - 1),
                    black_box(Value {
                        sum: 1,
                        m: [1, 1, 0, 1],
                    }),
                ),
                "search" => {
                    check ^= t.max_right(l, |x| black_box(x).sum <= 31) as u64;
                    check ^= t.min_left(r, |x| black_box(x).sum <= 31) as u64;
                }
                "mixed" if i % 2 == 0 => t.set(
                    l.min(n - 1),
                    black_box(Value {
                        sum: 1,
                        m: [1, i as u64 % 17, 1, 0],
                    }),
                ),
                _ => check ^= black_box(t.prod(l, r)).m[0],
            }
        }
    }
    (
        now.elapsed().as_nanos(),
        black_box(check ^ t.all_prod().m[0]),
    )
}

fn boxed_bench(n: usize, q: usize, _mode: &str, _pattern: &str) -> (u128, u64) {
    use plain::*;
    struct Sum;
    impl SegtreeMonoid for Sum {
        type S = Box<u64>;
        fn identity() -> Self::S {
            Box::new(0)
        }
        fn op(a: &Self::S, b: &Self::S) -> Self::S {
            Box::new(**a + **b)
        }
    }
    let input = vec![Box::new(1); n];
    let mut check = 0;
    let now = Instant::now();
    for _ in 0..q {
        check ^= *black_box(Segtree::<Sum>::from(black_box(input.clone()))).all_prod();
    }
    (now.elapsed().as_nanos(), black_box(check))
}

fn main() {
    for n in [1024, 65536, 262144] {
        let q = if n >= 262144 { 30000 } else { 50000 };
        for (name, fun) in [
            (
                "plain",
                plain_bench as fn(usize, usize, &str, &str) -> (u128, u64),
            ),
            ("state", state_bench),
            ("matrix", matrix_bench),
            ("boxed", boxed_bench),
            ("add", add_bench),
            ("affine", affine_bench),
        ] {
            let lazy = name == "add" || name == "affine";
            for mode in [
                "build",
                "set",
                "push",
                "get",
                "prod",
                "apply",
                "mixed",
                "search",
                "updated_search",
                "slice",
            ] {
                if name == "boxed" && mode != "build" {
                    continue;
                }
                if (!lazy && ["get", "apply", "updated_search", "slice"].contains(&mode))
                    || (lazy && mode == "push")
                    || (name == "affine" && mode == "updated_search")
                {
                    continue;
                }
                for pattern in ["random", "short", "aligned", "full", "sequential"] {
                    if ["build", "set", "get", "push"].contains(&mode) && pattern != "random" {
                        continue;
                    }
                    if mode == "slice" && pattern != "short" && pattern != "full" {
                        continue;
                    }
                    let count = if mode == "build" {
                        (1000000 / n).max(4)
                    } else if mode == "slice" {
                        if pattern == "full" {
                            20
                        } else {
                            50
                        }
                    } else {
                        q
                    };
                    let (ns, check) = fun(n, count, mode, pattern);
                    println!("{name},{n},{mode},{pattern},{count},{ns},{check}");
                }
            }
        }
    }
}
