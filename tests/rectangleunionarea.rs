#[path = "../src/Basic/rectangleunionarea.rs"]
mod rectangleunionarea;
use rectangleunionarea::area_of_union_rectangles;

fn brute(rec: &[(i64, i64, i64, i64)]) -> i64 {
    let mut xs: Vec<_> = rec.iter().flat_map(|&(l, _, r, _)| [l, r]).collect();
    let mut ys: Vec<_> = rec.iter().flat_map(|&(_, d, _, u)| [d, u]).collect();
    xs.sort_unstable();
    xs.dedup();
    ys.sort_unstable();
    ys.dedup();
    let mut ans = 0;
    for x in xs.windows(2) {
        for y in ys.windows(2) {
            if rec
                .iter()
                .any(|&(l, d, r, u)| l <= x[0] && x[1] <= r && d <= y[0] && y[1] <= u)
            {
                ans += (x[1] - x[0]) * (y[1] - y[0]);
            }
        }
    }
    ans
}

#[test]
fn exhaustive_subsets() {
    let mut rectangles = Vec::new();
    for l in 0..2 {
        for r in l + 1..=2 {
            for d in 0..2 {
                for u in d + 1..=2 {
                    rectangles.push((l, d, r, u));
                }
            }
        }
    }
    for mask in 0..1usize << rectangles.len() {
        let rec: Vec<_> = rectangles
            .iter()
            .enumerate()
            .filter(|&(i, _)| mask >> i & 1 != 0)
            .map(|(_, &r)| r)
            .collect();
        assert_eq!(area_of_union_rectangles(&rec), brute(&rec), "{rec:?}");
    }
}

#[test]
fn exhaustive_pairs_including_degenerate() {
    let mut rectangles = Vec::new();
    for l in -1..=2 {
        for r in l..=2 {
            for d in -1..=2 {
                for u in d..=2 {
                    rectangles.push((l, d, r, u));
                }
            }
        }
    }
    for &a in &rectangles {
        for &b in &rectangles {
            assert_eq!(
                area_of_union_rectangles(&[a, b]),
                brute(&[a, b]),
                "{a:?} {b:?}"
            );
        }
    }
}

#[test]
fn random_differential() {
    let mut seed = 0x0123456789abcdefu64;
    let mut next = || {
        seed ^= seed << 7;
        seed ^= seed >> 9;
        seed
    };
    for _ in 0..10000 {
        let n = (next() % 25) as usize;
        let mut rec = Vec::new();
        for _ in 0..n {
            let a = (next() % 17) as i64 - 8;
            let b = (next() % 17) as i64 - 8;
            let c = (next() % 17) as i64 - 8;
            let d = (next() % 17) as i64 - 8;
            rec.push((a.min(b), c.min(d), a.max(b), c.max(d)));
        }
        assert_eq!(area_of_union_rectangles(&rec), brute(&rec), "{rec:?}");
    }
}

#[test]
fn structured_cases() {
    let duplicate = vec![(-100, -100, 100, 100); 10000];
    assert_eq!(area_of_union_rectangles(&duplicate), 40000);
    let nested: Vec<_> = (1..=1000).map(|i| (-i, -i, i, i)).collect();
    assert_eq!(area_of_union_rectangles(&nested), 4000000);
    let strips: Vec<_> = (0..1001).map(|i| (0, i, 10, i + 1)).collect();
    assert_eq!(area_of_union_rectangles(&strips), 10010);
    let touching: Vec<_> = (0..1001).map(|i| (i, i, i + 1, i + 1)).collect();
    assert_eq!(area_of_union_rectangles(&touching), 1001);
    let hole = [(0, 0, 4, 1), (0, 3, 4, 4), (0, 1, 1, 3), (3, 1, 4, 3)];
    assert_eq!(area_of_union_rectangles(&hole), 12);
    assert_eq!(area_of_union_rectangles::<i64>(&[]), 0);
    assert_eq!(area_of_union_rectangles(&[(i64::MIN, 0, i64::MAX, 0)]), 0);
}

#[test]
fn integer_types_and_wide_area() {
    macro_rules! check {
        ($($t:ty),*) => {$(
            let rec: [($t,$t,$t,$t);2] = [(0,0,3,2),(1,1,4,3)];
            assert_eq!(area_of_union_rectangles(&rec), 10);
        )*};
    }
    check!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
    let a = 1_000_000_000_000_000_000i128;
    assert_eq!(area_of_union_rectangles(&[(-a, -a, a, a)]), 4 * a * a);
    let a = u64::MAX - 10;
    assert_eq!(area_of_union_rectangles(&[(a, a, a + 10, a + 10)]), 100);
}

// Hash・外部クレート・AddAssign を要求しない独自型でも使える。
#[derive(Copy, Clone, Default, Eq, PartialEq, Ord, PartialOrd, Debug)]
struct Scalar(i128);
macro_rules! op {
    ($trait:ident, $method:ident, $symbol:tt) => {
        impl std::ops::$trait for Scalar {
            type Output = Self;
            fn $method(self, rhs: Self) -> Self {Self(self.0 $symbol rhs.0)}
        }
    };
}
op!(Add, add, +);
op!(Sub, sub, -);
op!(Mul, mul, *);

#[test]
fn custom_scalar() {
    let rec = [(Scalar(-2), Scalar(-3), Scalar(4), Scalar(5))];
    assert_eq!(area_of_union_rectangles(&rec), Scalar(48));
}

#[test]
#[should_panic(expected = "rectangle endpoints must be ordered")]
fn reversed_x() {
    area_of_union_rectangles(&[(2i64, 0, 1, 3)]);
}

#[test]
#[should_panic(expected = "rectangle endpoints must be ordered")]
fn reversed_y() {
    area_of_union_rectangles(&[(0i64, 3, 2, 1)]);
}
