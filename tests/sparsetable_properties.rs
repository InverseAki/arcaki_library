#![allow(dead_code)]
use std::cmp::Ordering;
include!("../src/DataStructure/sparsetable.rs");

fn next(s: &mut u64) -> usize {
    *s ^= *s << 13;
    *s ^= *s >> 7;
    *s ^= *s << 17;
    *s as usize
}

fn check(values: Vec<i64>, ranges: impl IntoIterator<Item = (usize, usize)>) {
    let mi = SparseTableMI::build(values.clone());
    let blocked_mi = SparseTableMI {
        data: SparseTableData::build_blocked(values.clone(), |x, y| x.min(y)),
    };
    let blocked_mx = SparseTableMX {
        data: SparseTableData::build_blocked(values.clone(), |x, y| x.max(y)),
    };
    let blocked_key = SparseTableMI {
        data: SparseTableData::build_blocked((0..values.len()).collect(), |x, y| {
            if values[x] <= values[y] {
                x
            } else {
                y
            }
        }),
    };
    let mx = SparseTableMX::build(values.clone());
    let ids: Vec<usize> = (0..values.len()).collect();
    let keyed = SparseTableMI::build_by_key(ids, |i| values[i]);
    for (l, r) in ranges {
        let expected = values[l..r]
            .iter()
            .enumerate()
            .min_by_key(|&(_, x)| x)
            .unwrap()
            .0
            + l;
        assert_eq!(blocked_mi.query(l, r), values[expected]);
        assert_eq!(blocked_mx.query(l, r), *values[l..r].iter().max().unwrap());
        assert_eq!(blocked_key.query_by_key(l, r, |i| values[i]), expected);
        assert_eq!(mi.query(l, r), values[expected], "min [{l},{r})");
        assert_eq!(
            mx.query(l, r),
            *values[l..r].iter().max().unwrap(),
            "max [{l},{r})"
        );
        assert_eq!(
            keyed.query_by_key(l, r, |i| values[i]),
            expected,
            "key [{l},{r})"
        );
    }
}

#[test]
fn all_intervals_around_block_and_level_boundaries() {
    let mut state = 20261021;
    for n in [
        0, 1, 2, 3, 7, 8, 9, 15, 16, 17, 31, 32, 33, 47, 48, 49, 63, 64, 65, 127, 128, 129, 255,
        256, 257,
    ] {
        for pattern in 0..6 {
            let values = (0..n)
                .map(|i| match pattern {
                    0 => 7,
                    1 => i as i64,
                    2 => -(i as i64),
                    3 => [i64::MIN, i64::MAX, 0, 0][i % 4],
                    4 => (next(&mut state) % 5) as i64,
                    _ => next(&mut state) as i64,
                })
                .collect();
            check(
                values,
                (0..n).flat_map(|l| (l + 1..=n).map(move |r| (l, r))),
            );
        }
    }
}

#[test]
fn random_ranges_and_powers_of_two() {
    let mut state = 319_922;
    for n in [511, 512, 513, 1023, 1024, 1025, 4095, 4096, 4097, 65_537] {
        let values = (0..n)
            .map(|_| (next(&mut state) % 1001) as i64 - 500)
            .collect();
        let mut ranges = vec![(0, n)];
        for i in 0..n.min(1025) {
            ranges.push((i, i + 1));
        }
        for _ in 0..2000 {
            let l = next(&mut state) % n;
            let r = l + 1 + next(&mut state) % (n - l);
            ranges.push((l, r));
        }
        for k in 0..usize::BITS {
            let len = 1usize << k;
            if len > n {
                break;
            }
            for width in [len.saturating_sub(1).max(1), len, (len + 1).min(n)] {
                for l in [0, (n - width) / 2, n - width] {
                    ranges.push((l, l + width));
                }
            }
        }
        check(values, ranges);
    }
}

#[derive(Clone, Copy, Debug)]
struct Item {
    rank: u8,
    id: usize,
}
impl PartialEq for Item {
    fn eq(&self, rhs: &Self) -> bool {
        self.rank == rhs.rank
    }
}
impl Eq for Item {}
impl PartialOrd for Item {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}
impl Ord for Item {
    fn cmp(&self, rhs: &Self) -> Ordering {
        self.rank.cmp(&rhs.rank)
    }
}

#[test]
fn tie_selection_and_independent_clones() {
    for n in [1, 15, 16, 17, 31, 32, 33, 65, 129] {
        let a: Vec<_> = (0..n)
            .map(|id| Item {
                rank: (id % 3) as u8,
                id,
            })
            .collect();
        let mi = SparseTableMI::build(a.clone()).clone();
        let mx = SparseTableMX::build(a.clone()).clone();
        let blocked_mi = SparseTableMI {
            data: SparseTableData::build_blocked(a.clone(), |x, y| if x <= y { x } else { y }),
        }
        .clone();
        let blocked_mx = SparseTableMX {
            data: SparseTableData::build_blocked(a.clone(), |x, y| x.max(y)),
        }
        .clone();
        for l in 0..n {
            for r in l + 1..=n {
                assert_eq!(blocked_mi.query(l, r).id, a[l..r].iter().min().unwrap().id);
                assert_eq!(blocked_mx.query(l, r).id, a[l..r].iter().max().unwrap().id);
                assert_eq!(mi.query(l, r).id, a[l..r].iter().min().unwrap().id);
                assert_eq!(mx.query(l, r).id, a[l..r].iter().max().unwrap().id);
            }
        }
    }
}

#[test]
fn owned_buffer_non_ord_payload_non_copy_keys_and_zero_sized_values() {
    #[derive(Clone, Copy)]
    struct Token(usize);
    let input: Vec<_> = (0..513).map(Token).collect();
    let ptr = input.as_ptr();
    let st = SparseTableMI::build_by_key(input, |x| format!("{:02}", x.0 % 7));
    let stored = match &st.data {
        SparseTableData::Direct { rows, .. } => rows[0].as_ptr(),
        SparseTableData::Blocked { values, .. } => values.as_ptr(),
    };
    assert_eq!(stored, ptr);
    assert_eq!(
        st.query_by_key(15, 200, |x| format!("{:02}", x.0 % 7)).0,
        21
    );
    let st = SparseTableMI::build(vec![(); 513]);
    assert_eq!(st.query(13, 501), ());
    let st = SparseTableMX::build(vec![(); 513]);
    assert_eq!(st.query(13, 501), ());
}

#[test]
fn invalid_ranges_panic_in_all_query_apis() {
    for n in [0, 1, 17, 65] {
        let mi = [
            SparseTableMI::build(vec![0; n]),
            SparseTableMI {
                data: SparseTableData::build_blocked(vec![0; n], |x, y| x.min(y)),
            },
        ];
        let mx = [
            SparseTableMX::build(vec![0; n]),
            SparseTableMX {
                data: SparseTableData::build_blocked(vec![0; n], |x, y| x.max(y)),
            },
        ];
        for (l, r) in [
            (0, 0),
            (n, n),
            (0, n + 1),
            (n + 1, n),
            (usize::MAX, 0),
            (0, usize::MAX),
        ] {
            for table in &mi {
                assert!(std::panic::catch_unwind(|| table.query(l, r)).is_err());
                assert!(std::panic::catch_unwind(|| table.query_by_key(l, r, |x| x)).is_err());
            }
            for table in &mx {
                assert!(std::panic::catch_unwind(|| table.query(l, r)).is_err());
            }
        }
    }
}

#[test]
fn automatic_strategy_selection_and_large_clones() {
    for n in [100_000, 200_000] {
        let input: Vec<u64> = (0..n).map(|i| (i * 37 % 1009) as u64).collect();
        let original = SparseTableMI::build(input.clone());
        match (&original.data, n) {
            (SparseTableData::Direct { .. }, 100_000)
            | (SparseTableData::Blocked { .. }, 200_000) => {}
            _ => panic!("unexpected strategy"),
        }
        let cloned = original.clone();
        drop(original);
        for (l, r) in [(0, n), (15, n - 17), (0, 16), (16, 33), (n - 15, n)] {
            assert_eq!(cloned.query(l, r), *input[l..r].iter().min().unwrap());
        }
    }
}
