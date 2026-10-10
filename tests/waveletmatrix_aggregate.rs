include!("../src/DataStructure/waveletmatrix_prefix.rs");
include!("../src/DataStructure/waveletmatrix_monoid.rs");

struct Sum;
impl WaveletMonoid for Sum {
    type S = i64;
    const COMMUTATIVE: bool = true;
    fn identity() -> i64 {
        0
    }
    fn op(&self, a: &i64, b: &i64) -> i64 {
        a + b
    }
}
struct Concat;
impl WaveletMonoid for Concat {
    type S = Vec<usize>;
    fn identity() -> Self::S {
        vec![]
    }
    fn op(&self, a: &Self::S, b: &Self::S) -> Self::S {
        a.iter().chain(b).copied().collect()
    }
}
fn rng(s: &mut u64) -> u64 {
    *s ^= *s << 7;
    *s ^= *s >> 9;
    *s
}
fn verify(keys: &[usize], seed: &mut u64) {
    let n = keys.len();
    let mut weights: Vec<_> = (0..n).map(|_| (rng(seed) % 31) as i64 - 15).collect();
    let unsigned: Vec<_> = weights.iter().map(|&v| (v + 15) as u64).collect();
    let xor: Vec<_> = (0..n).map(|_| rng(seed)).collect();
    let mut payload: Vec<_> = (0..n).map(|i| vec![i]).collect();
    let sum = WaveletMatrixSum::new(keys, &weights, WaveletSum::default());
    let usum = WaveletMatrixSum::<u64>::new(keys, &unsigned, WaveletSum::default());
    let wx = WaveletMatrixXor::new(keys, &xor, WaveletXor::default());
    let mut ms = WaveletMatrixMonoid::new(keys, &weights, Sum);
    let mut mc = WaveletMatrixMonoid::new(keys, &payload, Concat);
    assert_eq!(sum.len(), n);
    assert_eq!(sum.is_empty(), n == 0);
    assert_eq!(mc.len(), n);
    assert_eq!(mc.is_empty(), n == 0);
    for step in 0..3 {
        for l in 0..=n {
            for r in l..=n {
                let mut order: Vec<_> = (l..r).collect();
                order.sort_by_key(|&i| (keys[i], i));
                for (k, &i) in order.iter().enumerate() {
                    assert_eq!(mc.kth_index(l, r, k), i);
                    assert_eq!(mc.kth_data(l, r, k), payload[i]);
                    assert_eq!(sum.matrix().kth_smallest(l, r, k), keys[i]);
                    assert_eq!(mc.matrix().kth_smallest(l, r, k), keys[i]);
                }
                for start in 0..=order.len() {
                    let mut ends = vec![start, order.len()];
                    if n <= 9 {
                        ends.extend(start..=order.len());
                    } else {
                        ends.push(start + (rng(seed) as usize % (order.len() - start + 1)));
                    }
                    for end in ends {
                        let ids = &order[start..end];
                        let expected: i64 = ids.iter().map(|&i| weights[i]).sum();
                        let concat: Vec<_> = ids.iter().flat_map(|&i| payload[i].clone()).collect();
                        assert_eq!(ms.prod_sorted(l, r, start, end), expected);
                        assert_eq!(mc.prod_sorted(l, r, start, end), concat);
                        if step == 0 {
                            assert_eq!(sum.prod_sorted(l, r, start, end), expected);
                            assert_eq!(
                                usum.prod_sorted(l, r, start, end),
                                ids.iter().map(|&i| unsigned[i]).sum::<u64>()
                            );
                            assert_eq!(
                                wx.prod_sorted(l, r, start, end),
                                ids.iter().fold(0, |a, &i| a ^ xor[i])
                            );
                        }
                    }
                    assert_eq!(
                        ms.prefix_prod(l, r, start),
                        order[..start].iter().map(|&i| weights[i]).sum::<i64>()
                    );
                    assert_eq!(
                        mc.prefix_prod(l, r, start),
                        order[..start]
                            .iter()
                            .flat_map(|&i| payload[i].clone())
                            .collect::<Vec<_>>()
                    );
                }
                assert_eq!(
                    mc.prod(l, r),
                    order
                        .iter()
                        .flat_map(|&i| payload[i].clone())
                        .collect::<Vec<_>>()
                );
                for upper in [0, 1, 2, 3, 7, 8, 16, usize::MAX] {
                    let ids: Vec<_> = order.iter().copied().filter(|&i| keys[i] < upper).collect();
                    assert_eq!(
                        ms.prod_less(l, r, upper),
                        ids.iter().map(|&i| weights[i]).sum::<i64>()
                    );
                    assert_eq!(
                        mc.prod_less(l, r, upper),
                        ids.iter()
                            .flat_map(|&i| payload[i].clone())
                            .collect::<Vec<_>>()
                    );
                    if step == 0 {
                        assert_eq!(sum.prod(l, r), weights[l..r].iter().sum::<i64>());
                        assert_eq!(
                            sum.prod_less(l, r, upper),
                            ids.iter().map(|&i| weights[i]).sum::<i64>()
                        );
                        assert_eq!(
                            wx.prod_less(l, r, upper),
                            ids.iter().fold(0, |a, &i| a ^ xor[i])
                        );
                    }
                    let lower = upper / 2;
                    let ids: Vec<_> = ids.into_iter().filter(|&i| keys[i] >= lower).collect();
                    assert_eq!(
                        mc.prod_between(l, r, lower, upper),
                        ids.iter()
                            .flat_map(|&i| payload[i].clone())
                            .collect::<Vec<_>>()
                    );
                    assert_eq!(
                        ms.prod_between(l, r, lower, upper),
                        ids.iter().map(|&i| weights[i]).sum::<i64>()
                    );
                    if step == 0 {
                        assert_eq!(
                            sum.prod_between(l, r, lower, upper),
                            ids.iter().map(|&i| weights[i]).sum::<i64>()
                        );
                        assert_eq!(
                            wx.prod_between(l, r, lower, upper),
                            ids.iter().fold(0, |a, &i| a ^ xor[i])
                        );
                    }
                }
                assert_eq!(mc.prod_between(l, r, 10, 3), Vec::<usize>::new());
                assert_eq!(sum.prod_between(l, r, 10, 3), 0);
            }
        }
        if n > 0 {
            for _ in 0..n {
                let i = rng(seed) as usize % n;
                weights[i] = (rng(seed) % 101) as i64 - 50;
                payload[i] = vec![i, step, n];
                ms.set(i, weights[i]);
                mc.set(i, payload[i].clone());
                assert_eq!(ms.get(i), weights[i]);
                assert_eq!(mc.get(i), payload[i]);
            }
        }
    }
}
#[test]
fn boundaries_and_noncommutative_order() {
    let mut seed = 371;
    for a in [
        vec![],
        vec![0],
        vec![0; 8],
        vec![7; 9],
        vec![2, 0, 3, 1, 2, 0, 3, 1],
        vec![usize::MAX, 0, usize::MAX - 1, 1, usize::MAX, 1 << 63],
    ] {
        verify(&a, &mut seed);
    }
    for n in [63, 64, 65] {
        let a: Vec<_> = (0..n).map(|i| i % 3).collect();
        verify(&a, &mut seed);
    }
}
#[test]
fn deterministic_random_reference() {
    let mut seed = 0xabc987;
    for _ in 0..80 {
        let n = rng(&mut seed) as usize % 10;
        let a: Vec<_> = (0..n).map(|_| rng(&mut seed) as usize % 32).collect();
        verify(&a, &mut seed);
    }
}
#[test]
fn invalid_arguments_panic() {
    let s = WaveletMatrixSum::new(&[1, 2], &[3, 4], WaveletSum::default());
    let m = WaveletMatrixMonoid::new(&[1, 2], &[3, 4], Sum);
    for f in [
        Box::new(|| {
            s.prod(2, 1);
        }) as Box<dyn Fn()>,
        Box::new(|| {
            s.prod_less(0, 3, 0);
        }),
        Box::new(|| {
            s.prefix_prod(0, 2, 3);
        }),
        Box::new(|| {
            s.prod_sorted(0, 2, 2, 1);
        }),
        Box::new(|| {
            m.prod(0, 3);
        }),
        Box::new(|| {
            m.kth_index(0, 0, 0);
        }),
        Box::new(|| {
            m.prod_sorted(0, 2, 0, 3);
        }),
    ] {
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).is_err());
    }
    assert!(std::panic::catch_unwind(|| WaveletMatrixMonoid::new(&[1], &[], Sum)).is_err());
    assert!(std::panic::catch_unwind(|| WaveletMatrixSum::<i64>::new(
        &[1],
        &[],
        WaveletSum::default()
    ))
    .is_err());
}
