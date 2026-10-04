type MI = StaticModInt<Mod1000000007>;
use convolution_mod1000000007 as convolution;
#[test]
fn random_and_boundary_lengths() {
    let mut seed = 12345u64;
    for (n, m) in [
        (0, 0),
        (0, 8),
        (8, 0),
        (1, 1),
        (60, 200),
        (61, 61),
        (64, 65),
        (127, 130),
        (256, 257),
        (513, 321),
    ] {
        for _ in 0..8 {
            let mut make = |n| {
                (0..n)
                    .map(|_| {
                        seed ^= seed << 13;
                        seed ^= seed >> 7;
                        seed ^= seed << 17;
                        MI::new(seed % 1000000007)
                    })
                    .collect::<Vec<_>>()
            };
            let a = make(n);
            let b = make(m);
            let mut expected = if n == 0 || m == 0 {
                vec![]
            } else {
                vec![0u128; n + m - 1]
            };
            for (i, x) in a.iter().enumerate() {
                for (j, y) in b.iter().enumerate() {
                    expected[i + j] += x.val() as u128 * y.val() as u128;
                }
            }
            let expected: Vec<u32> = expected
                .into_iter()
                .map(|x| (x % 1000000007) as u32)
                .collect();
            assert_eq!(
                convolution(&a, &b)
                    .iter()
                    .map(|x| x.val())
                    .collect::<Vec<_>>(),
                expected
            );
        }
    }
}
#[test]
fn large_coefficients_exceed_i64_and_structured_inputs() {
    let n = 4097;
    let m = 5000;
    let a = vec![MI::new(-1); n];
    let b = vec![MI::new(-1); m];
    let c = convolution(&a, &b);
    for (i, x) in c.iter().enumerate() {
        assert_eq!(x.val(), (i + 1).min(n).min(m).min(n + m - 1 - i) as u32);
    }
    let zeros = vec![MI::new(0); 97];
    assert!(convolution(&a, &zeros).iter().all(|x| x.val() == 0));
    let mut impulse = vec![MI::new(0); 100];
    impulse[37] = MI::new(1);
    let c = convolution(&a, &impulse);
    for (i, x) in c.iter().enumerate() {
        assert_eq!(
            x.val(),
            if (37..37 + n).contains(&i) {
                1000000006
            } else {
                0
            }
        );
    }
}
#[test]
fn existing_merge_helpers_use_new_backend() {
    let mut factors = vec![vec![MI::new(1), MI::new(1)]; 128];
    let product = convolution_merge(&mut factors);
    let mut expected = vec![1u64];
    for _ in 0..128 {
        let mut next = vec![0; expected.len() + 1];
        for (i, &v) in expected.iter().enumerate() {
            next[i] = (next[i] + v) % 1000000007;
            next[i + 1] = (next[i + 1] + v) % 1000000007;
        }
        expected = next;
    }
    assert_eq!(
        product.iter().map(|x| x.val() as u64).collect::<Vec<_>>(),
        expected
    );
    let mut factors = vec![vec![MI::new(1), MI::new(1)]; 128];
    assert_eq!(
        convolution_merge_with_mx(&mut factors, 70)
            .iter()
            .map(|x| x.val() as u64)
            .collect::<Vec<_>>(),
        expected[..70]
    );
}
