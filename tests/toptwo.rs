include!("../src/DataStructure/toptwo.rs");

use std::cmp::Reverse;
use std::collections::BTreeMap;

type Candidate = (i64, usize);

fn reference(items: &[Candidate]) -> Vec<Candidate> {
    let mut maxima = BTreeMap::new();
    for &(value, key) in items {
        maxima
            .entry(key)
            .and_modify(|old: &mut i64| *old = (*old).max(value))
            .or_insert(value);
    }
    let mut result: Vec<_> = maxima
        .into_iter()
        .map(|(key, value)| (value, key))
        .collect();
    result.sort_by_key(|&(value, key)| (Reverse(value), key));
    result.truncate(2);
    result
}

fn check(top: &TopTwo, items: &[Candidate]) {
    let expected = reference(items);
    assert_eq!(top.iter().copied().collect::<Vec<_>>(), expected);
    assert_eq!(top.len(), expected.len());
    assert_eq!(top.is_empty(), expected.is_empty());
    assert_eq!(top.first().copied(), expected.first().copied());
    assert_eq!(top.second().copied(), expected.get(1).copied());
    for key in [0, 1, 2, 3, 4, 5, usize::MAX] {
        let filtered: Vec<_> = items.iter().copied().filter(|&(_, k)| k != key).collect();
        assert_eq!(
            top.best_excluding(&key).copied(),
            reference(&filtered).first().copied()
        );
        assert_eq!(top.best_rv(key), top.best_excluding(&key));
    }
}

#[test]
fn empty_single_and_full_integer_range() {
    let mut top = TopTwo::default();
    check(&top, &[]);
    let items = [
        (i64::MIN, usize::MAX),
        (i64::MIN, 0),
        (i64::MAX, usize::MAX),
    ];
    for i in 0..items.len() {
        top.push(items[i]);
        check(&top, &items[..=i]);
    }
    assert_eq!(top.into_iter().collect::<Vec<_>>(), reference(&items));
    top.clear();
    check(&top, &[]);
    top.push((0, 0));
    check(&top, &[(0, 0)]);
}

#[test]
fn duplicate_keys_promotions_and_ties() {
    let mut top = TopTwo::new();
    let items = [
        (10, 2),
        (8, 1),
        (7, 1),
        (11, 1),
        (9, 1),
        (11, 2),
        (11, 0),
        (10, 0),
        (12, 2),
        (20, 5),
        (25, 0),
        (25, 5),
    ];
    for i in 0..items.len() {
        top.push(items[i]);
        check(&top, &items[..=i]);
    }
}

#[test]
fn exhaustive_small_sequences_match_all_candidates() {
    for mut code in 0..9usize.pow(5) {
        let mut top = TopTwo::new();
        let mut items = Vec::new();
        for _ in 0..5 {
            let digit = code % 9;
            code /= 9;
            let item = (digit as i64 / 3 - 1, digit % 3);
            items.push(item);
            top.push(item);
        }
        check(&top, &items);
    }
}

#[test]
fn merge_is_a_commutative_idempotent_monoid() {
    let candidates: Vec<_> = (-1..=1).flat_map(|v| (0..3).map(move |k| (v, k))).collect();
    let mut states = vec![TopTwo::new()];
    for &a in &candidates {
        states.push([a].into_iter().collect());
        for &b in &candidates {
            let state: TopTwo = [a, b].into_iter().collect();
            if !states.contains(&state) {
                states.push(state);
            }
        }
    }
    let empty = TopTwo::new();
    for a in &states {
        assert_eq!(a.merged(&empty), *a);
        assert_eq!(empty.merged(a), *a);
        assert_eq!(a.merged(a), *a);
        for b in &states {
            assert_eq!(a.merged(b), b.merged(a));
            let union: Vec<_> = a.iter().chain(b.iter()).copied().collect();
            check(&a.merged(b), &union);
            for c in &states {
                assert_eq!(a.merged(b).merged(c), a.merged(&b.merged(c)));
            }
        }
    }
}

#[test]
fn partitioned_random_merges_match_full_history() {
    let mut seed = 123456789u64;
    for _ in 0..1000 {
        let mut groups = [TopTwo::new(); 3];
        let mut items = Vec::new();
        for i in 0..50 {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            let item = ((seed % 15) as i64 - 7, (seed / 15 % 6) as usize);
            groups[i % 3].push(item);
            items.push(item);
        }
        let mut result = groups[2];
        result.merge(groups[0]);
        result.merge(groups[1]);
        check(&result, &items);
    }
}

#[test]
fn reverse_values_and_structured_keys() {
    let top: TopTwo<Reverse<i32>, (&str, usize)> = [
        (Reverse(4), ("red", 0)),
        (Reverse(2), ("red", 0)),
        (Reverse(3), ("blue", 0)),
        (Reverse(1), ("blue", 1)),
    ]
    .into_iter()
    .collect();
    assert_eq!(top.first(), Some(&(Reverse(1), ("blue", 1))));
    assert_eq!(top.second(), Some(&(Reverse(2), ("red", 0))));
    assert_eq!(top.best_excluding(&("blue", 1)), top.second());
}

#[test]
fn owned_non_clone_values_and_keys() {
    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
    struct Owned(String);
    let mut top: TopTwo<Owned, Owned> = TopTwo::new();
    top.push((Owned("a".into()), Owned("x".into())));
    top.push((Owned("z".into()), Owned("x".into())));
    let rhs = [(Owned("b".into()), Owned("y".into()))]
        .into_iter()
        .collect();
    top.merge(rhs);
    assert_eq!(top.first().unwrap().0 .0, "z");
    assert_eq!(top.best_excluding(&Owned("x".into())).unwrap().0 .0, "b");
    let items: Vec<_> = top.into_iter().collect();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].1 .0, "x");
}
