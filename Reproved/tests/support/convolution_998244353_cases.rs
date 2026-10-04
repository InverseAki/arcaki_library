type MI = StaticModInt<Mod998244353>;
use convolution_mod998244353 as convolution;
fn oracle(a: &[MI], b: &[MI]) -> Vec<u32> {
    if a.is_empty() || b.is_empty() {
        return vec![];
    }
    let mut c = vec![0u128; a.len() + b.len() - 1];
    for (i, x) in a.iter().enumerate() {
        for (j, y) in b.iter().enumerate() {
            c[i + j] += x.val() as u128 * y.val() as u128;
        }
    }
    c.into_iter().map(|x| (x % 998244353) as u32).collect()
}
#[test]
fn direct_and_blocked_match_naive() {
    let mut seed = 321u64;
    for (n, m) in [
        (0, 0),
        (0, 5),
        (8, 0),
        (1, 1),
        (60, 81),
        (61, 61),
        (64, 65),
        (129, 127),
        (513, 450),
    ] {
        for _ in 0..5 {
            let mut make = |n| {
                (0..n)
                    .map(|_| {
                        seed ^= seed << 13;
                        seed ^= seed >> 7;
                        seed ^= seed << 17;
                        MI::new(seed % 998244353)
                    })
                    .collect::<Vec<_>>()
            };
            let a = make(n);
            let b = make(m);
            let expected = oracle(&a, &b);
            assert_eq!(
                convolution(&a, &b)
                    .iter()
                    .map(|x| x.val())
                    .collect::<Vec<_>>(),
                expected
            );
            if n > 0 && m > 0 {
                assert_eq!(
                    convolution_998244353_detail::blocked(&a, &b, 64)
                        .iter()
                        .map(|x| x.val())
                        .collect::<Vec<_>>(),
                    expected
                );
            }
        }
    }
}
#[test]
fn length_bounds() {
    use convolution_998244353_detail::check_lengths;
    check_lengths(1 << 24, 1 << 24);
    assert!(std::panic::catch_unwind(|| check_lengths((1 << 24) + 1, 1)).is_err());
    assert!(std::panic::catch_unwind(|| check_lengths(1, (1 << 24) + 1)).is_err());
}
#[test]
fn large_coefficients_and_merge() {
    let a = vec![MI::new(-1); 1025];
    let b = vec![MI::new(-1); 900];
    let c = convolution_998244353_detail::blocked(&a, &b, 64);
    for (i, x) in c.iter().enumerate() {
        assert_eq!(
            x.val(),
            (i + 1).min(a.len()).min(b.len()).min(c.len() - i) as u32
        );
    }
    let mut factors = vec![vec![MI::new(1), MI::new(1)]; 128];
    let c = convolution_merge(&mut factors);
    assert_eq!(c.len(), 129);
    assert_eq!(c[0].val(), 1);
    assert_eq!(c[64].val(), c[128 - 64].val());
    let mut expected = vec![MI::new(1)];
    for _ in 0..128 {
        expected = oracle(&expected, &[MI::new(1), MI::new(1)])
            .into_iter()
            .map(MI::raw)
            .collect();
    }
    assert_eq!(
        c.iter().map(|x| x.val()).collect::<Vec<_>>(),
        expected.iter().map(|x| x.val()).collect::<Vec<_>>()
    );
}
#[test]
#[ignore = "maximum-size test: about 544 MiB and a long release-mode run"]
fn maximum_input_lengths() {
    let n = 1usize << 24;
    let a = vec![MI::new(-1); n];
    let b = vec![MI::new(-1); n];
    let c = convolution(&a, &b);
    assert_eq!(c.len(), 2 * n - 1);
    for (i, x) in c.iter().enumerate() {
        assert_eq!(
            x.val(),
            (i + 1).min(2 * n - 1 - i) as u32,
            "coefficient {i}"
        );
    }
}

#[test]
fn block_boundaries_and_streaming() {
    let mut seed = 8137u64;
    for block in [8usize, 16, 32, 64, 128] {
        for (n, m) in [
            (block - 1, block + 1),
            (block * 4, block * 4),
            (block * 4 + 1, block * 2 + 7),
            (block, 5 * block + 2),
            (5 * block + 2, block),
            (1, 5 * block + 1),
        ] {
            let mut make = |n| {
                (0..n)
                    .map(|_| {
                        seed ^= seed << 13;
                        seed ^= seed >> 7;
                        seed ^= seed << 17;
                        MI::new(seed % 998244353)
                    })
                    .collect::<Vec<_>>()
            };
            let a = make(n);
            let b = make(m);
            let got = convolution_998244353_detail::blocked(&a, &b, block);
            assert_eq!(
                got.iter().map(|x| x.val()).collect::<Vec<_>>(),
                oracle(&a, &b),
                "block={block},n={n},m={m}"
            );
        }
    }
}

#[test]
fn owned_matches_borrowed_and_naive() {
    let mut seed = 181u64;
    for (n, m) in [
        (0, 0),
        (0, 13),
        (1, 91),
        (32, 33),
        (33, 33),
        (61, 65),
        (255, 511),
        (512, 513),
    ] {
        let mut make = |n| {
            (0..n)
                .map(|_| {
                    seed ^= seed << 13;
                    seed ^= seed >> 7;
                    seed ^= seed << 17;
                    MI::new(seed % 998244353)
                })
                .collect::<Vec<_>>()
        };
        let a = make(n);
        let b = make(m);
        let expected = oracle(&a, &b);
        if n > 0 && m > 0 {
            let c = convolution_998244353_detail::blocked_owned(a.clone(), b.clone(), 64);
            assert_eq!(c.iter().map(|x| x.val()).collect::<Vec<_>>(), expected);
        }
        let c = convolution_mod998244353_owned(a, b);
        assert_eq!(c.iter().map(|x| x.val()).collect::<Vec<_>>(), expected);
    }
}
#[test]
#[ignore = "maximum-size owned API memory/correctness check"]
fn maximum_owned_input_lengths() {
    let n = 1usize << 24;
    let a = vec![MI::new(-1); n];
    let b = vec![MI::new(-1); n];
    let c = convolution_mod998244353_owned(a, b);
    assert_eq!(c.len(), 2 * n - 1);
    for (i, x) in c.iter().enumerate() {
        assert_eq!(x.val(), (i + 1).min(2 * n - 1 - i) as u32);
    }
}
