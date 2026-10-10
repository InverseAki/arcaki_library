include!("../src/DataStructure/waveletmatrix_offline.rs");

fn rng(seed: &mut u64) -> u64 {
    *seed ^= *seed << 7;
    *seed ^= *seed >> 9;
    *seed
}
fn naive_min(sorted: &[usize], target: u128) -> Option<usize> {
    if target == 0 {
        return Some(0);
    }
    let mut sum = 0;
    for (i, &value) in sorted.iter().rev().enumerate() {
        sum += value as u128;
        if sum >= target {
            return Some(i + 1);
        }
    }
    None
}
fn verify_range(wm: &WaveletMatrixOffline, a: &[usize], l: usize, r: usize) {
    let mut sorted = a[l..r].to_vec();
    sorted.sort_unstable();
    let sum: u128 = sorted.iter().map(|&x| x as u128).sum();
    assert_eq!(wm.range_sum(l, r), sum);
    for k in 0..sorted.len() {
        assert_eq!(wm.kth_smallest(l, r, k), sorted[k]);
        assert_eq!(wm.kth_largest(l, r, k), sorted[sorted.len() - 1 - k]);
    }
    for k in 0..=sorted.len() {
        let small: u128 = sorted[..k].iter().map(|&x| x as u128).sum();
        let large: u128 = sorted[sorted.len() - k..].iter().map(|&x| x as u128).sum();
        assert_eq!(wm.sum_smallest(l, r, k), small);
        assert_eq!(wm.sum_largest(l, r, k), large);
        for end in k..=sorted.len() {
            assert_eq!(
                wm.sum_sorted(l, r, k, end),
                sorted[k..end].iter().map(|&x| x as u128).sum::<u128>()
            );
        }
        for target in [
            small,
            large,
            large.saturating_sub(1),
            large + 1,
            0,
            sum,
            sum + 1,
            u128::MAX,
        ] {
            assert_eq!(
                wm.min_count_for_sum(l, r, target),
                naive_min(&sorted, target),
                "a={a:?}, [{l},{r}), target={target}"
            );
        }
    }
    let mut bounds = vec![0, 1, 2, 3, 4, 8, 16, usize::MAX];
    bounds.extend(sorted.iter().copied());
    bounds.sort_unstable();
    bounds.dedup();
    for &upper in &bounds {
        assert_eq!(
            wm.range_freq(l, r, upper),
            sorted.iter().filter(|&&x| x < upper).count()
        );
        assert_eq!(
            wm.sum_less(l, r, upper),
            sorted
                .iter()
                .filter(|&&x| x < upper)
                .map(|&x| x as u128)
                .sum::<u128>()
        );
        assert_eq!(
            wm.rank_range(upper, l, r),
            sorted.iter().filter(|&&x| x == upper).count()
        );
        assert_eq!(
            wm.prev_value(l, r, upper),
            sorted.iter().copied().filter(|&x| x < upper).last()
        );
        assert_eq!(
            wm.next_value(l, r, upper),
            sorted.iter().copied().find(|&x| x >= upper)
        );
        for &lower in &[0, 1, upper / 2, upper, usize::MAX] {
            let chosen: Vec<_> = sorted
                .iter()
                .copied()
                .filter(|&x| lower <= x && x < upper)
                .collect();
            assert_eq!(wm.range_freq_between(l, r, lower, upper), chosen.len());
            assert_eq!(
                wm.sum_between(l, r, lower, upper),
                chosen.iter().map(|&x| x as u128).sum::<u128>()
            );
        }
    }
}
fn verify_all(wm: &WaveletMatrixOffline, a: &[usize]) {
    assert_eq!(wm.len(), a.len());
    assert_eq!(wm.is_empty(), a.is_empty());
    for (i, &x) in a.iter().enumerate() {
        assert_eq!(wm.get(i), x);
    }
    for l in 0..=a.len() {
        for r in l..=a.len() {
            verify_range(wm, a, l, r);
        }
    }
}

#[test]
fn exhaustive_states_and_subset_optimum() {
    let choices = [0, 1, 3];
    let initial = vec![0; 4];
    let updates: Vec<_> = (0..4).flat_map(|i| choices.map(|x| (i, x))).collect();
    let mut wm = WaveletMatrixOffline::new(&initial, &updates);
    for state in 0..81 {
        let mut code = state;
        let mut a = initial.clone();
        for i in 0..4 {
            a[i] = choices[code % 3];
            code /= 3;
            wm.set(i, a[i]);
        }
        verify_all(&wm, &a);
        for l in 0..=4 {
            for r in l..=4 {
                for target in 0..=13 {
                    let expected = (0usize..1usize << (r - l))
                        .filter_map(|mask| {
                            let sum: u128 = (l..r)
                                .filter(|&i| mask >> (i - l) & 1 != 0)
                                .map(|i| a[i] as u128)
                                .sum();
                            (sum >= target).then_some(mask.count_ones() as usize)
                        })
                        .min();
                    assert_eq!(wm.min_count_for_sum(l, r, target), expected);
                }
            }
        }
    }
}

#[test]
fn deterministic_random_updates() {
    let mut seed = 0xfeda713;
    for _ in 0..100 {
        let n = (rng(&mut seed) % 12 + 1) as usize;
        let mut a: Vec<_> = (0..n).map(|_| (rng(&mut seed) % 16) as usize).collect();
        let updates: Vec<_> = (0..60)
            .map(|_| {
                (
                    (rng(&mut seed) as usize) % n,
                    (rng(&mut seed) % 24) as usize,
                )
            })
            .collect();
        let mut wm = WaveletMatrixOffline::new(&a, &updates);
        for (i, x) in updates {
            wm.set(i, x);
            a[i] = x;
            assert_eq!(wm.get(i), x);
            for _ in 0..3 {
                let p = rng(&mut seed) as usize % (n + 1);
                let q = rng(&mut seed) as usize % (n + 1);
                verify_range(&wm, &a, p.min(q), p.max(q));
            }
        }
        verify_all(&wm, &a);
    }
}

#[test]
fn structured_candidates_and_large_sums() {
    let empty = WaveletMatrixOffline::new(&[], &[]);
    assert_eq!(empty.candidate_len(), 0);
    verify_all(&empty, &[]);
    for p in [1, 2, 3, 63, 64, 65, 127, 128, 129] {
        let updates: Vec<_> = (0..p)
            .map(|x| (0, x))
            .chain([(0, usize::MAX), (0, usize::MAX)])
            .collect();
        let mut wm = WaveletMatrixOffline::new(&[0], &updates);
        assert_eq!(wm.candidate_len(), p + 1);
        for &(i, x) in &updates {
            wm.set(i, x);
            verify_all(&wm, &[x]);
        }
    }
    for initial in [
        vec![0; 65],
        vec![7; 65],
        (0..65).collect(),
        (0..65).rev().collect(),
    ] {
        let updates: Vec<_> = (0..67).map(|x| (31, x)).collect();
        let mut a = initial.clone();
        let mut wm = WaveletMatrixOffline::new(&a, &updates);
        for &(i, x) in &[(31, 66), (31, 0), (31, 32), (31, 7), (31, 7)] {
            wm.set(i, x);
            a[i] = x;
            for &(l, r) in &[(0, 65), (0, 31), (31, 32), (32, 65), (25, 42), (65, 65)] {
                verify_range(&wm, &a, l, r);
            }
        }
    }
    let mut a = vec![usize::MAX, usize::MAX, 0, 1, usize::MAX - 1];
    let mut wm = WaveletMatrixOffline::new(&a, &[(2, usize::MAX), (0, 0)]);
    verify_all(&wm, &a);
    wm.set(2, usize::MAX);
    a[2] = usize::MAX;
    wm.set(0, 0);
    a[0] = 0;
    verify_all(&wm, &a);
}

#[test]
fn invalid_operations_preserve_state() {
    let mut wm = WaveletMatrixOffline::new(&[1, 2], &[(0, 3), (0, 3)]);
    assert_eq!(wm.candidate_len(), 3);
    for (i, x) in [(0, 2), (1, 3), (2, 1)] {
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| wm.set(i, x))).is_err());
        verify_all(&wm, &[1, 2]);
    }
    wm.set(0, 3);
    wm.set(0, 1);
    verify_all(&wm, &[1, 2]);
    for f in [
        Box::new(|| {
            wm.range_sum(2, 1);
        }) as Box<dyn Fn()>,
        Box::new(|| {
            wm.min_count_for_sum(0, 3, 0);
        }),
        Box::new(|| {
            wm.kth_smallest(0, 0, 0);
        }),
        Box::new(|| {
            wm.kth_largest(1, 0, 0);
        }),
        Box::new(|| {
            wm.sum_smallest(0, 2, 3);
        }),
        Box::new(|| {
            wm.sum_largest(0, 2, 3);
        }),
        Box::new(|| {
            wm.sum_sorted(0, 2, 2, 1);
        }),
        Box::new(|| {
            wm.sum_between(0, 3, 9, 2);
        }),
        Box::new(|| {
            wm.range_freq_between(0, 3, 9, 2);
        }),
        Box::new(|| {
            wm.get(2);
        }),
    ] {
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).is_err());
    }
    assert!(std::panic::catch_unwind(|| WaveletMatrixOffline::new(&[1], &[(1, 2)])).is_err());
    assert!(std::panic::catch_unwind(|| WaveletMatrixOffline::new(&[], &[(0, 2)])).is_err());
}
