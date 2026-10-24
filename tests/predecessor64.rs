#![allow(dead_code)]
include!("predecessor64_performance/model.rs");

fn check_constructor(mut s: current::Predecessor64, n: usize, mut oracle: BTreeSet<usize>) {
    let clone = s.clone();
    let original = oracle.clone();
    let mut seed = n as u64;
    for p in (0..n.min(256)).chain(n.saturating_sub(128)..n) {
        check(&s, &oracle, p);
        check(&clone, &original, p);
    }
    for _ in 0..1000 {
        let p = rand(&mut seed) % n;
        check(&s, &oracle, p);
        check(&clone, &original, p);
        if rand(&mut seed) % 2 == 0 { s.insert(p); oracle.insert(p); }
        else { s.remove(p); oracle.remove(&p); }
        check(&s, &oracle, p);
    }
}

#[test]
fn constructors_words_and_clone() {
    for len in [0, 1, 2, 63, 64, 65, 4095, 4096, 4097] {
        for pattern in 0..5 {
            let mut seed = (len * 5 + pattern) as u64;
            let base: Vec<u64> = (0..len).map(|i| match pattern {
                0 => 0,
                1 => !0,
                2 => if i % 64 == 63 { 1 << 63 } else { 0 },
                3 => if i == 0 || i + 1 == len { 1 } else { 0 },
                _ => rand(&mut seed) as u64,
            }).collect();
            let n = if len == 0 { 1 } else { len * 64 };
            let oracle: BTreeSet<_> = base.iter().enumerate().flat_map(|(i, &v)| {
                (0..64).filter(move |&j| v >> j & 1 != 0).map(move |j| i * 64 + j)
            }).collect();
            let mut spare = Vec::with_capacity(len + len / 64 + 32);
            spare.extend_from_slice(&base);
            check_constructor(current::Predecessor64::from_vec_u64(spare), n, oracle.clone());
            check_constructor(current::Predecessor64::from_vec_u64(base.clone()), n, oracle.clone());
            let words: Vec<_> = base.into_iter().map(|v| v as usize).collect();
            check_constructor(current::Predecessor64::from_vec_usize(words), n, oracle);
        }
    }
}

#[test]
fn constructors_strings_preserve_padding_and_byte_indices() {
    for len in [0usize, 1, 63, 64, 65, 4095, 4096, 4097] {
        for pattern in 0..4 {
            let s: String = (0..len).map(|i| match pattern {
                0 => '0', 1 => '1', 2 => if i % 64 == 63 { '1' } else { '0' },
                _ => if i == 0 || i + 1 == len { '1' } else { '0' },
            }).collect();
            let n = if len == 0 { 1 } else { len.div_ceil(64) * 64 };
            let oracle = s.bytes().enumerate().filter(|&(_, c)| c == b'1').map(|(i, _)| i).collect();
            check_constructor(current::Predecessor64::from_01_string(s), n, oracle);
        }
    }
    let s = "１x1é0".to_string();
    let oracle = s.bytes().enumerate().filter(|&(_, c)| c == b'1').map(|(i, _)| i).collect();
    check_constructor(current::Predecessor64::from_01_string(s), 64, oracle);
}

#[test]
fn zero_domain_and_outside_queries() {
    for n in [0usize, 1, 63, 64, 65, 4095, 4096, 4097] {
        let limit = n.max(1);
        let mut s = current::Predecessor64::new(n);
        assert!(s.is_empty());
        assert_eq!(s.min(), !0);
        assert_eq!(s.max(), !0);
        for p in [limit, limit + 1, usize::MAX] {
            assert!(!s.include(p));
            assert_eq!(s.prev(p), !0);
            assert_eq!(s.inprev(p), !0);
            assert_eq!(s.next(p), !0);
            assert_eq!(s.innext(p), !0);
        }
        s.insert(0);
        s.insert(limit - 1);
        for p in [limit, limit + 1, usize::MAX] {
            assert!(!s.include(p));
            assert_eq!(s.prev(p), limit - 1);
            assert_eq!(s.inprev(p), limit - 1);
            assert_eq!(s.next(p), !0);
            assert_eq!(s.innext(p), !0);
            for insert in [true, false] {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    if insert {
                        s.insert(p);
                    } else {
                        s.remove(p);
                    }
                }));
                assert!(result.is_err());
                assert!(s.include(0));
                assert!(s.include(limit - 1));
                assert_eq!(s.min(), 0);
                assert_eq!(s.max(), limit - 1);
            }
        }
    }
}

#[test]
fn string_packing_all_eight_bit_patterns_and_tails() {
    for bits in 0..256usize {
        for len in 0usize..=72 {
            let text: String = (0..len)
                .map(|p| if bits >> (p & 7) & 1 == 0 { '0' } else { '1' })
                .collect();
            let s = current::Predecessor64::from_01_string(text.clone());
            for p in 0..len.max(1).div_ceil(64) * 64 {
                assert_eq!(s.include(p), p < len && text.as_bytes()[p] == b'1');
            }
        }
    }
    for len in 0..=256 {
        let mut text = String::from("é１1🦀");
        text.extend((0..len).map(|i| char::from_u32((i % 128) as u32).unwrap()));
        let s = current::Predecessor64::from_01_string(text.clone());
        for p in 0..text.len().div_ceil(64) * 64 {
            assert_eq!(s.include(p), p < text.len() && text.as_bytes()[p] == b'1');
        }
    }
}

mod layout_invariants {
    include!("../src/DataStructure/predecessor64.rs");

    fn check(s: &Predecessor64) {
        assert!(s.mx > 0);
        assert!((1..=11).contains(&s.levels));
        assert_eq!(s.offset[0], 0);
        assert_eq!(s.data.len(), s.offset[s.levels - 1] + 1);
        let mut len = s.mx.div_ceil(64);
        for i in 1..s.levels {
            assert_eq!(s.offset[i], s.offset[i - 1] + len);
            for (j, chunk) in s.data[s.offset[i - 1]..s.offset[i]].chunks(64).enumerate() {
                let expected = chunk
                    .iter()
                    .enumerate()
                    .fold(0, |bits, (k, &word)| bits | (u64::from(word != 0) << k));
                assert_eq!(s.data[s.offset[i] + j], expected);
            }
            len = len.div_ceil(64);
        }
        assert_eq!(len, 1);
    }

    #[test]
    fn full_usize_layout_without_allocation() {
        let mut domains = vec![0usize, 1, usize::MAX];
        for exponent in (6..usize::BITS).step_by(6) {
            let n = 1usize << exponent;
            domains.extend([n - 1, n, n + 1]);
        }
        for n in domains {
            let mut len = n.max(1).div_ceil(64);
            let (offset, levels, total) = Predecessor64::layout(len);
            let mut sum = 0;
            for &position in &offset[..levels] {
                assert_eq!(position, sum);
                sum += len;
                len = len.div_ceil(64);
            }
            assert_eq!(len, 1);
            assert_eq!(sum, total);
            assert_eq!(offset[levels - 1] + 1, total);
        }
    }

    #[test]
    fn summaries_remain_exact_after_updates_and_construction() {
        for n in [0usize, 1, 63, 64, 65, 4097, 262145, 16777217] {
            let mut s = Predecessor64::new(n);
            let mut seed = 78901;
            check(&s);
            for i in 0..10000 {
                let p = super::rand(&mut seed) % n.max(1);
                if i % 3 == 0 {
                    s.remove(p);
                } else {
                    s.insert(p);
                }
                if i % 127 == 0 {
                    check(&s);
                }
            }
            check(&s);
            let words = s.data[..n.max(1).div_ceil(64)].to_vec();
            check(&Predecessor64::from_vec_u64(words.clone()));
            check(&Predecessor64::from_vec_usize(
                words.into_iter().map(|v| v as usize).collect(),
            ));
        }
    }
}
