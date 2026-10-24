mod current {
    include!("../src/OfflineQuery/mo.rs");
    pub fn order(x: u32, y: u32, p: u32, r: u32) -> u64 {
        MoSolver::<super::Sym>::hilbert_order(x, y, p, r)
    }
}
#[allow(dead_code)]
mod baseline {
    include!("support/mo_baseline.rs");
    pub fn order(x: u32, y: u32, p: u32, r: u32) -> u64 {
        MoSolver::<super::OldSym>::hilbert_order(x, y, p, r)
    }
}
use current::*;
use std::{cell::Cell, collections::VecDeque, rc::Rc};

struct Sym(i64);
impl SymMoState for Sym {
    type Data = i64;
    type Query = i64;
    type Ans = i64;
    fn add(&mut self, i: usize, v: &i64) {
        self.0 += *v + i as i64;
    }
    fn sub(&mut self, i: usize, v: &i64) {
        self.0 -= *v + i as i64;
    }
    fn ans(&mut self, id: usize, q: &i64) -> i64 {
        self.0 + *q + id as i64
    }
}
struct OldSym;
impl baseline::SymMoState for OldSym {
    type Data = ();
    type Query = ();
    type Ans = ();
    fn add(&mut self, _: usize, _: &()) {}
    fn sub(&mut self, _: usize, _: &()) {}
    fn ans(&mut self, _: usize, _: &()) {}
}
fn random(s: &mut u64) -> u32 {
    *s ^= *s << 13;
    *s ^= *s >> 7;
    *s ^= *s << 17;
    *s as u32
}
#[test]
fn hilbert_exactly_matches_recursive_order() {
    for p in 0..=7 {
        for x in 0..1 << p {
            for y in 0..1 << p {
                for r in 0..4 {
                    assert_eq!(current::order(x, y, p, r), baseline::order(x, y, p, r));
                }
            }
        }
    }
    let mut s = 123456789;
    for p in 1..=32 {
        let mask = u32::MAX >> (32 - p);
        for _ in 0..10000 {
            let (x, y, r) = (
                random(&mut s) & mask,
                random(&mut s) & mask,
                random(&mut s) % 4,
            );
            assert_eq!(current::order(x, y, p, r), baseline::order(x, y, p, r));
        }
    }
}

struct Item(i64);
#[derive(Debug, PartialEq)]
struct Answer {
    values: Vec<(usize, i64)>,
    id: usize,
    tag: String,
}
struct Directional {
    values: VecDeque<(usize, i64)>,
}
impl MoState for Directional {
    type Data = Item;
    type Query = (usize, usize, String);
    type Ans = Answer;
    fn add_left(&mut self, i: usize, v: &Item) {
        if let Some(&(j, _)) = self.values.front() {
            assert_eq!(i + 1, j);
        }
        self.values.push_front((i, v.0));
    }
    fn add_right(&mut self, i: usize, v: &Item) {
        if let Some(&(j, _)) = self.values.back() {
            assert_eq!(j + 1, i);
        }
        self.values.push_back((i, v.0));
    }
    fn sub_left(&mut self, i: usize, v: &Item) {
        assert_eq!(self.values.pop_front(), Some((i, v.0)));
    }
    fn sub_right(&mut self, i: usize, v: &Item) {
        assert_eq!(self.values.pop_back(), Some((i, v.0)));
    }
    fn ans(&mut self, id: usize, q: &Self::Query) -> Answer {
        assert_eq!(
            self.values.iter().map(|x| x.0).collect::<Vec<_>>(),
            (q.0..q.1).collect::<Vec<_>>()
        );
        Answer {
            values: self.values.iter().copied().collect(),
            id,
            tag: q.2.clone(),
        }
    }
}
fn check(n: usize, ranges: &[(usize, usize)]) {
    let data: Vec<_> = (0..n).map(|i| i as i64 * 7 - 19).collect();
    let mut directional = MoSolver::<Directional>::new(data.iter().map(|&v| Item(v)).collect());
    let mut symmetric = MoSolver::<Sym>::new(data.clone());
    for (id, &(l, r)) in ranges.iter().enumerate() {
        directional.add_query(l, r, (l, r, id.to_string()));
        symmetric.add_query(l, r, -(id as i64) * 13);
    }
    for _ in 0..2 {
        let a = directional.solve(&mut Directional {
            values: VecDeque::new(),
        });
        let b = symmetric.solve(&mut Sym(0));
        for (id, &(l, r)) in ranges.iter().enumerate() {
            assert_eq!(
                a[id],
                Answer {
                    values: (l..r).map(|i| (i, data[i])).collect(),
                    id,
                    tag: id.to_string()
                }
            );
            assert_eq!(
                b[id],
                (l..r).map(|i| data[i] + i as i64).sum::<i64>() - id as i64 * 12
            );
        }
    }
    directional.add_query(0, n, (0, n, "extra".into()));
    assert_eq!(
        directional
            .solve(&mut Directional {
                values: VecDeque::new()
            })
            .last()
            .unwrap()
            .tag,
        "extra"
    );
}
#[test]
fn exhaustive_directional_and_symmetric_ranges() {
    for n in 0..=24 {
        let mut ranges: Vec<_> = (0..=n).flat_map(|l| (l..=n).map(move |r| (l, r))).collect();
        ranges.reverse();
        ranges.extend([(n, n), (0, n), (0, 0), (0, n)]);
        check(n, &ranges);
    }
}
#[test]
fn random_and_disjoint_ranges() {
    let mut seed = 987654321;
    for n in [1, 17, 63, 256] {
        let mut ranges = vec![(0, 1), (n - 1, n), (0, 0), (n, n)];
        for _ in 0..2000 {
            let (x, y) = (
                random(&mut seed) as usize % (n + 1),
                random(&mut seed) as usize % (n + 1),
            );
            ranges.push((x.min(y), x.max(y)));
        }
        check(n, &ranges);
    }
}
#[test]
fn empty_queries_and_invalid_ranges() {
    use std::panic::{catch_unwind, AssertUnwindSafe};
    let mut solver = MoSolver::<Sym>::new(vec![0; 3]);
    assert!(solver.solve(&mut Sym(99)).is_empty());
    for (l, r) in [(0, 4), (4, 4)] {
        let mut invalid = MoSolver::<Sym>::new(vec![0; 3]);
        invalid.add_query(l, r, 0);
        assert!(catch_unwind(AssertUnwindSafe(|| invalid.solve(&mut Sym(0)))).is_err());
    }
    solver.add_query(0, 3, 0);
    assert_eq!(solver.solve(&mut Sym(0)), [3]);
}
struct Tracked(Rc<Cell<usize>>);
impl Drop for Tracked {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}
struct Panics {
    made: usize,
    panic_at: usize,
    drops: Rc<Cell<usize>>,
    in_add: bool,
}
impl SymMoState for Panics {
    type Data = ();
    type Query = ();
    type Ans = Tracked;
    fn add(&mut self, _: usize, _: &()) {
        if self.in_add && self.made == self.panic_at {
            panic!("add");
        }
    }
    fn sub(&mut self, _: usize, _: &()) {}
    fn ans(&mut self, _: usize, _: &()) -> Tracked {
        if !self.in_add && self.made == self.panic_at {
            panic!("ans");
        }
        self.made += 1;
        Tracked(self.drops.clone())
    }
}
#[test]
fn answers_drop_on_success_and_callback_panic() {
    use std::panic::{catch_unwind, AssertUnwindSafe};
    for in_add in [false, true] {
        for panic_at in [0, 1, 3, usize::MAX] {
            let drops = Rc::new(Cell::new(0));
            let mut solver = MoSolver::<Panics>::new(vec![(); 32]);
            for i in 0..32 {
                solver.add_query(i, i + 1, ());
            }
            let mut state = Panics {
                made: 0,
                panic_at,
                drops: drops.clone(),
                in_add,
            };
            let result = catch_unwind(AssertUnwindSafe(|| solver.solve(&mut state)));
            if panic_at == usize::MAX {
                assert!(result.is_ok());
            } else {
                assert!(result.is_err());
            }
            drop(result);
            assert_eq!(drops.get(), state.made);
        }
    }
}
#[test]
fn public_query_fields_and_zst_answers() {
    let q = MoQuery { l: 0, r: 0, id: 7 };
    assert_eq!((q.l, q.r, q.id), (0, 0, 7));
    struct Z;
    impl SymMoState for Z {
        type Data = ();
        type Query = ();
        type Ans = ();
        fn add(&mut self, _: usize, _: &()) {}
        fn sub(&mut self, _: usize, _: &()) {}
        fn ans(&mut self, _: usize, _: &()) {}
    }
    let mut solver = MoSolver::<Z>::new(vec![(); 8]);
    for _ in 0..100 {
        solver.add_query(8, 8, ());
    }
    assert_eq!(solver.solve(&mut Z).len(), 100);
}
