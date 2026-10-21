include!("../src/Basic/integer_sort.rs");
include!("../src/Basic/split_mix.rs");

fn check<T: IntegerSortValue + std::fmt::Debug>(input: &[T]) {
    let mut expected = input.to_vec();
    expected.sort_unstable();
    let mut actual = input.to_vec();
    integer_sort(&mut actual);
    assert_eq!(actual, expected);
    let mut buffer = input.iter().take(7).copied().collect();
    actual.copy_from_slice(input);
    integer_sort_with_buffer(&mut actual, &mut buffer);
    assert_eq!(actual, expected);
}

fn exhaustive<T: IntegerSortValue + std::fmt::Debug>(alphabet: &[T]) {
    for n in 0..=7u32 {
        for mut code in 0..alphabet.len().pow(n) {
            let input: Vec<_> = (0..n)
                .map(|_| {
                    let x = alphabet[code % alphabet.len()];
                    code /= alphabet.len();
                    x
                })
                .collect();
            check(&input);
        }
    }
}

#[test]
fn exhaustive_small_and_extremes() {
    exhaustive(&[0u64, 1, 1 << 63, u64::MAX]);
    exhaustive(&[0u32, 1, 1 << 31, u32::MAX]);
    exhaustive(&[0usize, 1, usize::MAX / 2, usize::MAX]);
    exhaustive(&[i64::MIN, -1, 0, i64::MAX]);
}

fn differential<T: IntegerSortValue + std::fmt::Debug>() {
    let mut rng = SplitMix64::new(0x123456789abcdef);
    for n in [
        0, 1, 2, 16, 127, 128, 129, 255, 256, 511, 512, 1023, 1024, 1025, 2047, 2048, 2049, 4096,
        8191, 8192, 8193, 16384,
    ] {
        for width in [
            1, 4, 8, 9, 10, 11, 12, 16, 17, 22, 23, 31, 32, 33, 44, 63, 64,
        ] {
            for shift in [0, 7, 23, 32, 48, 63] {
                let mask = u64::MAX >> (64 - width);
                let input: Vec<_> = (0..n)
                    .map(|_| {
                        T::from_sort_key(
                            ((rng.next_u64() & mask) << shift)
                                ^ (0xaaaa5555cccc3333 & !(mask << shift)),
                        )
                    })
                    .collect();
                check(&input);
            }
        }
    }
}

#[test]
fn random_u64() {
    differential::<u64>();
}
#[test]
fn random_u32() {
    differential::<u32>();
}
#[test]
fn random_usize() {
    differential::<usize>();
}
#[test]
fn random_i64() {
    differential::<i64>();
}

#[test]
fn missing_digits_and_pass_parity() {
    let mut rng = SplitMix64::new(314159265);
    for mask in [
        0,
        1,
        1 << 63,
        0xff,
        0x1ff,
        0x3ff,
        0x7ff,
        0xff00ff,
        0xff0000ff000000ff,
        0xffff000000000000,
        u64::MAX,
    ] {
        for fixed in [0, 1, 0x123456789abcdef0, u64::MAX] {
            let input: Vec<_> = (0..20000)
                .map(|_| (rng.next_u64() & mask) | (fixed & !mask))
                .collect();
            check(&input);
            check(&input.iter().map(|&x| x as i64).collect::<Vec<_>>());
        }
    }
}

#[test]
fn dense_ranges_crossing_boundaries() {
    let mut rng = SplitMix64::new(2718281828);
    for base in [
        0,
        255,
        1023,
        u32::MAX as u64 - 255,
        (1 << 63) - 255,
        u64::MAX - 65535,
    ] {
        for range in [15, 255, 256, 257, 4095, 8191, 8192, 65535] {
            if base > u64::MAX - range {
                continue;
            }
            let n = ((range + 1) * 8 + 8).max(129) as usize;
            let input: Vec<_> = (0..n)
                .map(|_| base + rng.next_u64() % (range + 1))
                .collect();
            check(&input);
            check(&input.iter().map(|&x| x as i64).collect::<Vec<_>>());
        }
    }
}

#[test]
fn structured_and_monotone() {
    for n in [129, 1024, 8192, 65536] {
        let sorted: Vec<_> = (0..n).map(|i| (i / 3) as u64).collect();
        check(&sorted);
        check(&sorted.iter().rev().copied().collect::<Vec<_>>());
        check(&vec![u64::MAX; n]);
        check(
            &(0..n)
                .map(|i| i.min(n - 1 - i) as i64 - n as i64 / 4)
                .collect::<Vec<_>>(),
        );
        check(&(0..n).map(|i| (i % 257) as u32).collect::<Vec<_>>());
        let mut nearly = sorted.clone();
        nearly.swap(n / 3, n / 2);
        check(&nearly);
        let mut buffer = Vec::new();
        let mut values = sorted.clone();
        integer_sort_with_buffer(&mut values, &mut buffer);
        assert_eq!(buffer.capacity(), 0);
        values.reverse();
        integer_sort_with_buffer(&mut values, &mut buffer);
        assert_eq!(buffer.capacity(), 0);
    }
}

#[test]
fn buffer_reuse_and_subslice() {
    let mut rng = SplitMix64::new(42);
    let mut buffer = vec![0u64; 20000];
    let pointer = buffer.as_ptr();
    let capacity = buffer.capacity();
    for n in [16384, 8192, 129, 0, 655, 20000, 1024, 16384] {
        let mut input: Vec<_> = (0..n + 2).map(|_| rng.next_u64()).collect();
        let first = input[0];
        let last = input[n + 1];
        let mut expected = input[1..n + 1].to_vec();
        expected.sort_unstable();
        integer_sort_with_buffer(&mut input[1..n + 1], &mut buffer);
        assert_eq!(&input[1..n + 1], expected);
        assert_eq!((input[0], input[n + 1]), (first, last));
        assert_eq!(buffer.as_ptr(), pointer);
        assert_eq!(buffer.capacity(), capacity);
    }
}

#[test]
fn normalized_keys_and_early_completion() {
    let mut rng = SplitMix64::new(987654321);
    for n in [129, 1024, 4096, 8192, 16384] {
        for base in [
            255u64,
            1023,
            (1 << 32) - 2048,
            (1 << 63) - 2048,
            u64::MAX - 65536,
        ] {
            for width in [11, 16, 22, 32] {
                let mask = (1u64 << width) - 1;
                if base > u64::MAX - mask {
                    continue;
                }
                let input: Vec<_> = (0..n).map(|_| base + (rng.next_u64() & mask)).collect();
                check(&input);
                check(&input.iter().map(|&x| x as i64).collect::<Vec<_>>());
                check(&input.iter().map(|&x| x as u32).collect::<Vec<_>>());
            }
        }
        let extremes: Vec<_> = (0..n)
            .map(|_| {
                (1u64 << 63)
                    .wrapping_sub(32768)
                    .wrapping_add(rng.next_u64() & 65535) as i64
            })
            .collect();
        check(&extremes);
    }
}
