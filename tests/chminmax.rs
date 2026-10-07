#[macro_use]
#[path = "../src/Basic/chminmax.rs"]
mod chminmax;
use chminmax::{Chmax, Chmin};

#[test]
fn same_dp_and_nested_dp() {
    let mut dp = vec![0, 7, 3];
    assert!(chmax!(dp[0], dp[1] + 2));
    assert_eq!(dp, [9, 7, 3]);
    assert!(!chmax!(dp[0], dp[0],));
    assert!(chmin!(dp[0], dp[2] + 1));
    assert_eq!(dp[0], 4);
    let mut dp = vec![vec![10, 4], vec![2, 8]];
    assert!(chmin!(dp[0][0], dp[1][0] + dp[0][1],));
    assert!(chmax!(dp[1][1], dp[0][0] + dp[0][1]));
    assert_eq!(dp, [vec![6, 4], vec![2, 10]]);
}

#[test]
fn both_expressions_once_candidate_first_even_without_update() {
    let mut dp = vec![10, 1];
    let mut events = Vec::new();
    assert!(!chmax!(
        dp[{
            events.push("lhs");
            0
        }],
        {
            events.push("rhs");
            dp[1]
        }
    ));
    assert_eq!(events, ["rhs", "lhs"]);
    events.clear();
    assert!(chmin!(
        dp[{
            events.push("lhs");
            0
        }],
        {
            events.push("rhs");
            dp[1]
        }
    ));
    assert_eq!(events, ["rhs", "lhs"]);
    assert_eq!(dp, [1, 1]);
}

#[test]
fn owned_non_copy_values_and_hygiene() {
    let mut dp = vec![String::from("a"), String::from("z")];
    let candidate = 3;
    let target = 5;
    assert!(chmax!(dp[0], dp[1].clone()));
    assert!(!chmin!(dp[1], dp[0].clone()));
    assert!(chmin!(dp[1], String::from("b")));
    assert_eq!(dp, ["z", "b"]);
    assert_eq!((candidate, target), (3, 5));
    assert!(dp[0].chmin(String::from("c")));
    assert!(dp[1].chmax(String::from("d")));
    assert!(!dp[0].chmin(String::from("c")));
}

#[test]
fn arbitrary_mutable_places_and_tuples() {
    struct State {
        best: (i32, i32),
    }
    let mut state = State { best: (1, 9) };
    assert!(chmax!(state.best, (2, 0)));
    let best = &mut state.best;
    assert!(chmin!(*best, (1, 10)));
    assert_eq!(state.best, (1, 10));
}

#[test]
fn equality_and_incomparable_do_not_update() {
    let mut x = 4.0_f64;
    assert!(!chmax!(x, f64::NAN));
    assert!(!chmin!(x, f64::NAN));
    assert!(!x.chmax(f64::NAN));
    assert!(!x.chmin(f64::NAN));
    assert_eq!(x, 4.0);
    x = f64::NAN;
    assert!(!chmax!(x, 5.0));
    assert!(!chmin!(x, 5.0));
    assert!(x.is_nan());
    let mut zero = -0.0_f64;
    assert!(!chmax!(zero, 0.0));
    assert!(!chmin!(zero, 0.0));
    assert!(zero.is_sign_negative());
}

#[test]
fn exhaustive_integer_update_and_return_value() {
    for lhs in -10..=10 {
        for rhs in -10..=10 {
            let mut a = lhs;
            let mut b = lhs;
            assert_eq!(chmax!(a, rhs), lhs < rhs);
            assert_eq!(b.chmax(rhs), lhs < rhs);
            assert_eq!((a, b), (lhs.max(rhs), lhs.max(rhs)));
            a = lhs;
            b = lhs;
            assert_eq!(chmin!(a, rhs), lhs > rhs);
            assert_eq!(b.chmin(rhs), lhs > rhs);
            assert_eq!((a, b), (lhs.min(rhs), lhs.min(rhs)));
        }
    }
}
