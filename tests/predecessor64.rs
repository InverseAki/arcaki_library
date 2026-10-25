#![allow(dead_code)]
include!("predecessor64_performance/model.rs");

type TestedSet = current::Predecessor64;
include!("predecessor64_performance/constructor_checks.rs");

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
