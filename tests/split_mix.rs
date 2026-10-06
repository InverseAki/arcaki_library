include!("../src/Basic/split_mix.rs");

#[test]
fn known_sequence_and_compatibility() {
    let expected = [
        0xe220a8397b1dcdaf,
        0x6e789e6aa1b965f4,
        0x06c45d188009454f,
        0xf88bb8a8724c81ec,
        0x1b39896a51a8749b,
    ];
    let mut rng = SplitMix64::new(0);
    let mut other = rng.clone();
    for x in expected {
        assert_eq!(rng.next(), x);
        assert_eq!(other.next_u64(), x);
    }
    assert_eq!(rng.clone().next(), rng.next());
}

#[test]
fn exhaustive_small_integer_intervals() {
    let mut rng = SplitMix64::new(123);
    for low in i8::MIN as i16..=i8::MAX as i16 {
        for high in low..=i8::MAX as i16 {
            for _ in 0..3 {
                let x = rng.gen_range(low as i8..=high as i8) as i16;
                assert!(low <= x && x <= high);
                let x = rng.gen_range(low..high + 1);
                assert!(low <= x && x <= high);
            }
        }
    }
    for low in 0..=u8::MAX {
        for high in low..=u8::MAX {
            let x = rng.gen_range(low..=high);
            assert!(low <= x && x <= high);
        }
    }
}

#[test]
fn every_type_extremes_and_bounds() {
    use std::ops::Bound::{Excluded, Included, Unbounded};
    let mut rng = SplitMix64::new(u64::MAX);
    macro_rules! check {
        ($($t:ty),*) => {$ (
            for _ in 0..100 {
                assert_eq!(rng.gen_range(<$t>::MIN..=<$t>::MIN), <$t>::MIN);
                assert_eq!(rng.gen_range(<$t>::MAX..=<$t>::MAX), <$t>::MAX);
                assert_eq!(rng.gen_range(..=<$t>::MIN), <$t>::MIN);
                assert_eq!(rng.gen_range(<$t>::MAX..), <$t>::MAX);
                assert_eq!(rng.gen_range((Excluded(<$t>::MAX - 1), Unbounded)), <$t>::MAX);
                assert_eq!(rng.gen_range((Unbounded, Excluded(<$t>::MIN + 1))), <$t>::MIN);
                let _: $t = rng.gen_range(..);
                let _: $t = rng.gen_range(<$t>::MIN..=<$t>::MAX);
                let x: $t = rng.gen_range((Included(<$t>::MIN), Excluded(<$t>::MAX)));
                assert!(x < <$t>::MAX);
            }
        )*};
    }
    check!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);
    let mut raw = rng.clone();
    assert_eq!(rng.gen_range(0u64..=u64::MAX), raw.next_u64());
    let mut raw = rng.clone();
    assert_eq!(rng.gen_range(0u128..=u128::MAX), raw.next_u128());
    for _ in 0..1000 {
        let x = rng.gen_range(i128::MIN..=0);
        assert!(x <= 0);
        let x = rng.gen_range(u128::MAX - 100..=u128::MAX);
        assert!(x >= u128::MAX - 100);
        let x = rng.gen_range(-10i64..=10);
        assert!((-10..=10).contains(&x));
    }
}

#[test]
fn rejected_samples_match_reference() {
    let mut rejected = 0;
    for width in [
        1,
        3,
        17,
        (1u128 << 63) + 1,
        u64::MAX as u128,
        1u128 << 64,
        (1u128 << 64) + 1,
        (1u128 << 127) + 1,
        u128::MAX,
    ] {
        let mut rng = SplitMix64::new(42);
        let mut reference = rng.clone();
        for _ in 0..1000 {
            let max = if width <= 1u128 << 64 {
                u64::MAX as u128
            } else {
                u128::MAX
            };
            // 許容領域のサイズが width の倍数になる先頭位置。
            let threshold = (max % width + 1) % width;
            let expected = loop {
                let x = if width <= 1u128 << 64 {
                    reference.next_u64() as u128
                } else {
                    reference.next_u128()
                };
                if x >= threshold {
                    break x % width;
                }
                rejected += 1;
            };
            assert_eq!(rng.gen_range(0u128..width), expected);
            assert_eq!(rng.state, reference.state);
        }
    }
    assert!(rejected > 1000);
    // 8 bit の乱数空間で全幅を列挙し、棄却後の各剰余の個数が等しいことも確認。
    for width in 1usize..=256 {
        let threshold = 256 % width;
        let mut counts = vec![0; width];
        for x in threshold..256 {
            counts[x % width] += 1;
        }
        assert!(counts.iter().all(|&n| n == counts[0]));
    }
}

#[test]
fn invalid_ranges_and_probabilities_panic() {
    use std::ops::Bound::{Excluded, Unbounded};
    let panics = |f: fn(&mut SplitMix64)| {
        assert!(std::panic::catch_unwind(|| f(&mut SplitMix64::new(0))).is_err());
    };
    panics(|r| {
        r.gen_range(1..1);
    });
    panics(|r| {
        r.gen_range(3..=2);
    });
    panics(|r| {
        r.gen_range(..0u64);
    });
    panics(|r| {
        r.gen_range(..i128::MIN);
    });
    panics(|r| {
        r.gen_range((Excluded(u128::MAX), Unbounded));
    });
    panics(|r| {
        r.gen_bool(f64::NAN);
    });
    panics(|r| {
        r.gen_bool(f64::INFINITY);
    });
    panics(|r| {
        r.gen_bool(-0.1);
    });
    panics(|r| {
        r.gen_bool(1.1);
    });
}

#[test]
fn floats_bool_and_slice_operations() {
    let mut rng = SplitMix64::new(10);
    for _ in 0..10000 {
        assert!((0.0..1.0).contains(&rng.gen_f64()));
        assert!((0.0..1.0).contains(&rng.gen_f32()));
        assert!(!rng.gen_bool(0.0));
        assert!(rng.gen_bool(1.0));
    }
    let mut raw = rng.clone();
    assert_eq!(rng.next_u32(), (raw.next_u64() >> 32) as u32);
    let mut empty: [String; 0] = [];
    rng.shuffle(&mut empty);
    assert!(rng.choose(&empty).is_none());
    assert!(rng.choose_mut(&mut empty).is_none());
    let mut single = [String::from("abc")];
    rng.shuffle(&mut single);
    assert_eq!(rng.choose(&single).unwrap(), "abc");
    rng.choose_mut(&mut single).unwrap().push('d');
    assert_eq!(single[0], "abcd");
    for n in 0..100 {
        let mut a: Vec<_> = (0..n).collect();
        let mut b = a.clone();
        let mut copied_rng = rng.clone();
        rng.shuffle(&mut a);
        copied_rng.shuffle(&mut b);
        assert_eq!(a, b);
        a.sort_unstable();
        assert_eq!(a, (0..n).collect::<Vec<_>>());
    }
    let mut seen = std::collections::HashSet::new();
    for _ in 0..1000 {
        let mut a = [0, 1, 2];
        rng.shuffle(&mut a);
        seen.insert(a);
    }
    assert_eq!(seen.len(), 6);
}
