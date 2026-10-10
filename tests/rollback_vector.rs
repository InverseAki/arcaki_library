#![allow(dead_code)]
include!("../src/DataStructure/rollbackvector.rs");

#[test]
fn nested_rollbacks_and_non_clone_values() {
    #[derive(Debug, PartialEq)]
    struct Value(i32);
    let mut v = RollbackVector::from_vec(vec![Value(1), Value(2)]);
    let base = v.history_len();
    v.set(0, Value(3));
    let mark = v.history_len();
    v.set(0, Value(4));
    v.set(0, Value(4));
    v.set(1, Value(5));
    v.rollback(v.history_len());
    assert_eq!(v.history_len(), 4);
    v.rollback(mark);
    assert_eq!(v.as_slice(), &[Value(3), Value(2)]);
    v.set(1, Value(6));
    v.rollback(base);
    assert_eq!(v.as_slice(), &[Value(1), Value(2)]);
    assert_eq!(v.history_len(), 0);
    assert_eq!(v.len(), 2);
    assert_eq!(v[0], Value(1));

    let mut empty = RollbackVector::<Value>::from_vec(vec![]);
    empty.rollback(0);
    assert!(empty.is_empty());
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        empty.rollback(1);
    }))
    .is_err());
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        empty.set(0, Value(0));
    }))
    .is_err());
    assert_eq!(empty.history_len(), 0);
}

#[test]
fn stack_of_saved_states_against_vec() {
    let mut seed = 123456789u64;
    for n in [1, 3, 16] {
        let mut v = RollbackVector::new(n, 0u64);
        let mut a = vec![0u64; n];
        let mut marks = Vec::new();
        for _ in 0..5000 {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            match seed % 4 {
                0 => marks.push((v.history_len(), a.clone())),
                1 if !marks.is_empty() => {
                    let (len, old) = marks.pop().unwrap();
                    v.rollback(len);
                    a = old;
                }
                _ => {
                    let i = (seed as usize) % n;
                    v.set(i, seed);
                    a[i] = seed;
                }
            }
            assert_eq!(v.as_slice(), a.as_slice());
        }
        v.rollback(0);
        assert_eq!(v.as_slice(), vec![0; n]);
    }
}
