#![allow(dead_code)]
include!("../src/Basic/mo.rs");
include!("../src/Basic/rollbackmo.rs");

use std::collections::VecDeque;

// Clone / Default を要求されない Data / Query / Ans / Snapshot。
struct Data(i64);
struct QueryData {
    l: usize,
    r: usize,
    salt: usize,
}
#[derive(Debug, PartialEq, Eq)]
struct Answer {
    id: usize,
    salt: usize,
    values: Vec<(usize, i64)>,
}
struct Mark(usize);

enum Undo {
    AddedLeft,
    AddedRight,
    DeletedLeft((usize, i64)),
    DeletedRight((usize, i64)),
    Aux(usize),
}

struct IntervalState {
    values: VecDeque<(usize, i64)>,
    history: Vec<Undo>,
    aux: usize,
    operations: usize,
    answers: usize,
}

impl IntervalState {
    fn empty() -> Self {
        Self {
            values: VecDeque::new(),
            history: vec![Undo::Aux(777)],
            aux: 888,
            operations: 0,
            answers: 0,
        }
    }
    fn full(n: usize) -> Self {
        let mut state = Self::empty();
        state.values = (0..n).map(|i| (i, value(i))).collect();
        state
    }
    fn add_left_impl(&mut self, idx: usize, data: &Data) {
        if let Some(&(front, _)) = self.values.front() {
            assert_eq!(idx + 1, front);
        }
        self.values.push_front((idx, data.0));
        self.history.push(Undo::AddedLeft);
        self.operations += 1;
    }
    fn add_right_impl(&mut self, idx: usize, data: &Data) {
        if let Some(&(back, _)) = self.values.back() {
            assert_eq!(idx, back + 1);
        }
        self.values.push_back((idx, data.0));
        self.history.push(Undo::AddedRight);
        self.operations += 1;
    }
    fn sub_left_impl(&mut self, idx: usize, data: &Data) {
        let item = self.values.pop_front().expect("sub_left on empty interval");
        assert_eq!(item, (idx, data.0));
        self.history.push(Undo::DeletedLeft(item));
        self.operations += 1;
    }
    fn sub_right_impl(&mut self, idx: usize, data: &Data) {
        let item = self.values.pop_back().expect("sub_right on empty interval");
        assert_eq!(item, (idx, data.0));
        self.history.push(Undo::DeletedRight(item));
        self.operations += 1;
    }
    fn restore(&mut self, mark: Mark) {
        assert!(mark.0 <= self.history.len());
        while self.history.len() > mark.0 {
            match self.history.pop().unwrap() {
                Undo::AddedLeft => {
                    self.values.pop_front().unwrap();
                }
                Undo::AddedRight => {
                    self.values.pop_back().unwrap();
                }
                Undo::DeletedLeft(item) => self.values.push_front(item),
                Undo::DeletedRight(item) => self.values.push_back(item),
                Undo::Aux(old) => self.aux = old,
            }
        }
    }
    fn answer(&mut self, id: usize, query: &QueryData) -> Answer {
        let values: Vec<_> = self.values.iter().copied().collect();
        assert_eq!(
            values,
            (query.l..query.r)
                .map(|i| (i, value(i)))
                .collect::<Vec<_>>()
        );
        // ans の変更も query snapshot で復元されることを検証。
        self.history.push(Undo::Aux(self.aux));
        self.aux += 1;
        self.answers += 1;
        Answer {
            id,
            salt: query.salt,
            values,
        }
    }
    fn assert_baseline(&self, n: usize, full: bool) {
        assert_eq!(self.aux, 888);
        assert_eq!(self.history.len(), 1);
        assert!(matches!(self.history[0], Undo::Aux(777)));
        let expected: Vec<_> = if full {
            (0..n).map(|i| (i, value(i))).collect()
        } else {
            vec![]
        };
        assert_eq!(self.values.iter().copied().collect::<Vec<_>>(), expected);
    }
}

impl RollbackMoState for IntervalState {
    type Data = Data;
    type Query = QueryData;
    type Ans = Answer;
    type Snapshot = Mark;
    fn add_left(&mut self, idx: usize, data: &Data) {
        self.add_left_impl(idx, data);
    }
    fn add_right(&mut self, idx: usize, data: &Data) {
        self.add_right_impl(idx, data);
    }
    fn snapshot(&self) -> Mark {
        Mark(self.history.len())
    }
    fn rollback(&mut self, mark: Mark) {
        self.restore(mark);
    }
    fn ans(&mut self, id: usize, q: &QueryData) -> Answer {
        self.answer(id, q)
    }
}

impl RollbackMoDeleteState for IntervalState {
    type Data = Data;
    type Query = QueryData;
    type Ans = Answer;
    type Snapshot = Mark;
    fn sub_left(&mut self, idx: usize, data: &Data) {
        self.sub_left_impl(idx, data);
    }
    fn sub_right(&mut self, idx: usize, data: &Data) {
        self.sub_right_impl(idx, data);
    }
    fn snapshot(&self) -> Mark {
        Mark(self.history.len())
    }
    fn rollback(&mut self, mark: Mark) {
        self.restore(mark);
    }
    fn ans(&mut self, id: usize, q: &QueryData) -> Answer {
        self.answer(id, q)
    }
}

fn value(i: usize) -> i64 {
    (i * 7 % 11) as i64 - 5
}
fn data(n: usize) -> Vec<Data> {
    (0..n).map(|i| Data(value(i))).collect()
}

fn check_intervals(n: usize, ranges: &[(usize, usize)], block: Option<usize>) {
    let mut add = RollbackMoSolver::<IntervalState>::new(data(n));
    let mut sub = RollbackMoDeleteSolver::<IntervalState>::new(data(n));
    let mut expected = Vec::new();
    for (id, &(l, r)) in ranges.iter().enumerate() {
        let salt = id * 13 + 7;
        add.add_query(l, r, QueryData { l, r, salt });
        sub.add_query(l, r, QueryData { l, r, salt });
        expected.push(Answer {
            id,
            salt,
            values: (l..r).map(|i| (i, value(i))).collect(),
        });
    }
    let mut add_state = IntervalState::empty();
    let mut sub_state = IntervalState::full(n);
    // solver / state の再利用、以前の履歴の保持も確認。
    for _ in 0..2 {
        let actual_add = match block {
            Some(b) => add.solve_with_block_size(&mut add_state, b),
            None => add.solve(&mut add_state),
        };
        let actual_sub = match block {
            Some(b) => sub.solve_with_block_size(&mut sub_state, b),
            None => sub.solve(&mut sub_state),
        };
        assert_eq!(actual_add, expected);
        assert_eq!(actual_sub, expected);
        add_state.assert_baseline(n, false);
        sub_state.assert_baseline(n, true);
    }
    assert_eq!(add_state.answers, ranges.len() * 2);
    assert_eq!(sub_state.answers, ranges.len() * 2);
}

#[test]
fn exhaustive_intervals_and_block_boundaries() {
    for n in 0..=16 {
        let mut ranges: Vec<_> = (0..=n).flat_map(|l| (l..=n).map(move |r| (l, r))).collect();
        ranges.reverse();
        ranges.extend([(0, n), (n, n), (0, 0)]);
        for block in [
            None,
            Some(1),
            Some(2),
            Some(3),
            Some(5),
            Some(n + 1),
            Some(usize::MAX),
        ] {
            check_intervals(n, &ranges, block);
        }
    }
}

#[test]
fn random_intervals_with_duplicate_queries() {
    let mut seed = 12478965u64;
    for n in [1, 7, 31, 64, 99] {
        let mut ranges = Vec::new();
        for _ in 0..1000 {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            let l = seed as usize % (n + 1);
            let r = (seed >> 32) as usize % (n + 1);
            ranges.push((l.min(r), l.max(r)));
        }
        for block in [None, Some(1), Some(7), Some(11)] {
            check_intervals(n, &ranges, block);
        }
    }
}

#[test]
fn no_queries_do_not_touch_state() {
    for n in [0, 1, 7] {
        check_intervals(n, &[], None);
        let mut add = RollbackMoSolver::<IntervalState>::new(data(n));
        let mut sub = RollbackMoDeleteSolver::<IntervalState>::new(data(n));
        let mut add_state = IntervalState::empty();
        let mut sub_state = IntervalState::full(n);
        assert!(add.solve_with_block_size(&mut add_state, 1).is_empty());
        assert!(sub.solve_with_block_size(&mut sub_state, 1).is_empty());
        assert_eq!(add_state.operations, 0);
        assert_eq!(sub_state.operations, 0);
    }
}

#[test]
fn invalid_ranges_and_zero_block_size_are_rejected() {
    use std::panic::{catch_unwind, AssertUnwindSafe};
    let mut add = RollbackMoSolver::<IntervalState>::new(data(3));
    let mut sub = RollbackMoDeleteSolver::<IntervalState>::new(data(3));
    for (l, r) in [(2, 1), (0, 4), (3, usize::MAX)] {
        assert!(catch_unwind(AssertUnwindSafe(|| add.add_query(
            l,
            r,
            QueryData { l, r, salt: 0 }
        )))
        .is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| sub.add_query(
            l,
            r,
            QueryData { l, r, salt: 0 }
        )))
        .is_err());
    }
    let mut a = IntervalState::empty();
    let mut s = IntervalState::full(3);
    assert!(catch_unwind(AssertUnwindSafe(|| add.solve_with_block_size(&mut a, 0))).is_err());
    assert!(catch_unwind(AssertUnwindSafe(|| sub.solve_with_block_size(&mut s, 0))).is_err());
    a.assert_baseline(3, false);
    s.assert_baseline(3, true);
}

struct SumState {
    sum: i64,
    operations: usize,
}
impl SymRollbackMoState for SumState {
    type Data = i64;
    type Query = i64;
    type Ans = i64;
    type Snapshot = i64;
    fn add(&mut self, idx: usize, data: &i64) {
        self.sum += *data + idx as i64;
        self.operations += 1;
    }
    fn snapshot(&self) -> i64 {
        self.sum
    }
    fn rollback(&mut self, snapshot: i64) {
        self.sum = snapshot;
    }
    fn ans(&mut self, id: usize, qd: &i64) -> i64 {
        self.sum + *qd + id as i64
    }
}
impl SymRollbackMoDeleteState for SumState {
    type Data = i64;
    type Query = i64;
    type Ans = i64;
    type Snapshot = i64;
    fn sub(&mut self, idx: usize, data: &i64) {
        self.sum -= *data + idx as i64;
        self.operations += 1;
    }
    fn snapshot(&self) -> i64 {
        self.sum
    }
    fn rollback(&mut self, snapshot: i64) {
        self.sum = snapshot;
    }
    fn ans(&mut self, id: usize, qd: &i64) -> i64 {
        self.sum + *qd + id as i64
    }
}

#[test]
fn symmetric_states_and_incremental_query_registration() {
    let a = [3, -2, 9, 0, -5, 7, 7];
    let full = a.iter().enumerate().map(|(i, v)| *v + i as i64).sum();
    let mut add = RollbackMoSolver::<SumState>::new(a.to_vec());
    let mut sub = RollbackMoDeleteSolver::<SumState>::new(a.to_vec());
    let mut add_state = SumState {
        sum: 0,
        operations: 0,
    };
    let mut sub_state = SumState {
        sum: full,
        operations: 0,
    };
    let mut expected = Vec::new();
    for (id, (l, r)) in [(0, 7), (1, 4), (7, 7), (2, 3), (0, 0)]
        .into_iter()
        .enumerate()
    {
        let qd = id as i64 * -9;
        add.add_query(l, r, qd);
        sub.add_query(l, r, qd);
        expected.push((l..r).map(|i| a[i] + i as i64).sum::<i64>() + qd + id as i64);
        assert_eq!(add.solve(&mut add_state), expected);
        assert_eq!(sub.solve(&mut sub_state), expected);
        assert_eq!(add_state.sum, 0);
        assert_eq!(sub_state.sum, full);
    }
}

#[test]
fn structured_queries_respect_operation_bound() {
    let (n, q, b) = (4096usize, 32768usize, 64usize);
    let mut add = RollbackMoSolver::<SumState>::new(vec![0; n]);
    let mut sub = RollbackMoDeleteSolver::<SumState>::new(vec![0; n]);
    let mut expected = Vec::new();
    for id in 0..q {
        // 全左ブロックを使い、短区間・全長・空区間・交互の右端を混ぜる。
        let l = id * 37 % (n + 1);
        let r = match id % 4 {
            0 => l,
            1 => n,
            2 => (l + 1).min(n),
            _ => l + (id * 97 % (n - l + 1)),
        };
        add.add_query(l, r, 0);
        sub.add_query(l, r, 0);
        let sum = (r * r.saturating_sub(1) - l * l.saturating_sub(1)) / 2;
        expected.push(sum as i64 + id as i64);
    }
    let mut a = SumState {
        sum: 0,
        operations: 0,
    };
    let full = (n * (n - 1) / 2) as i64;
    let mut s = SumState {
        sum: full,
        operations: 0,
    };
    assert_eq!(add.solve_with_block_size(&mut a, b), expected);
    assert_eq!(sub.solve_with_block_size(&mut s, b), expected);
    let bound = (n / b + 1) * n + q * b;
    assert!(a.operations <= bound, "add: {} > {bound}", a.operations);
    assert!(s.operations <= bound, "delete: {} > {bound}", s.operations);
    assert_eq!(a.sum, 0);
    assert_eq!(s.sum, full);
    println!(
        "N={n}, Q={q}, B={b}: add={}, delete={}, bound={bound}",
        a.operations, s.operations
    );
}

struct LegacyMax;
struct LegacyState {
    max: i64,
    history: Vec<i64>,
}
impl RollbackMoMonoid for LegacyMax {
    type S = i64;
    type T = LegacyState;
    type U = ();
    type V = ();
    type X = i64;
    fn init_t(_: usize, _: usize, _: &Vec<i64>, _: &()) -> LegacyState {
        LegacyState {
            max: i64::MIN,
            history: Vec::new(),
        }
    }
    fn increase(t: &mut LegacyState, s: &i64) {
        t.history.push(t.max);
        t.max = t.max.max(*s);
    }
    fn snapshot(t: &mut LegacyState) {
        t.history.clear();
    }
    fn rollback(t: &mut LegacyState) {
        while let Some(old) = t.history.pop() {
            t.max = old;
        }
    }
    fn get(t: &LegacyState, _: &()) -> i64 {
        t.max
    }
}

#[test]
fn legacy_api_remains_usable() {
    for n in 0..=20 {
        let a: Vec<_> = (0..n).map(value).collect();
        let mut queries = Vec::new();
        let mut expected = Vec::new();
        for l in 0..=n {
            for r in l..=n {
                queries.push((l, r, ()));
                expected.push(a[l..r].iter().copied().max().unwrap_or(i64::MIN));
            }
        }
        assert_eq!(solve_rollback_mo::<LegacyMax>(a, &(), queries), expected);
    }
}
