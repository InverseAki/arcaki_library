#![allow(dead_code)]
include!("../src/DataStructure/veb_tree.rs");

fn values(node: &VebNode, n: usize, data: &[u64]) -> Vec<usize> {
    let mut result = Vec::new();
    if node.shift == 0 {
        for i in 0..n {
            if node.bits >> i & 1 != 0 {
                result.push(i);
            }
        }
    } else if node.shift == 6 || node.shift == 12 {
        let count = n.div_ceil(64);
        assert!(node.leaf + count <= data.len());
        let mut expected = 0;
        for group in 0..n.div_ceil(4096) {
            let mut mask = 0;
            for i in group * 64..((group + 1) * 64).min(count) {
                let mut word = data[node.leaf + i];
                mask |= ((word != 0) as u64) << (i & 63);
                while word != 0 {
                    result.push(i * 64 + word.trailing_zeros() as usize);
                    word &= word - 1;
                }
            }
            if node.shift == 6 {
                expected = mask;
            } else {
                assert_eq!(data[node.upper + group], mask);
                expected |= ((mask != 0) as u64) << group;
            }
        }
        assert_eq!(node.bits, expected);
        if node.min != usize::MAX {
            assert!(!result.contains(&node.min));
            result.push(node.min);
        }
    } else {
        let size = 1 << node.shift;
        assert_eq!(node.clusters.len(), n.div_ceil(size));
        let mut occupied = Vec::new();
        for (i, child) in node.clusters.iter().enumerate() {
            let child_values = values(child, (n - i * size).min(size), data);
            if !child_values.is_empty() {
                occupied.push(i);
            }
            result.extend(child_values.into_iter().map(|p| i * size + p));
        }
        assert_eq!(
            values(node.summary.as_ref().unwrap(), node.clusters.len(), data),
            occupied
        );
        if node.min != usize::MAX {
            assert!(!result.contains(&node.min));
            result.push(node.min);
        }
    }
    result.sort_unstable();
    assert!(result.iter().all(|&p| p < n));
    assert_eq!(node.min, result.first().copied().unwrap_or(usize::MAX));
    assert_eq!(node.max, result.last().copied().unwrap_or(0));
    result
}

fn check(s: &VebTree) {
    assert_eq!(s.middle, s.mx.div_ceil(64));
    let count = s.mx.div_ceil(4096);
    let mut occupied = Vec::new();
    for i in 0..count {
        let start = i * 64;
        let end = (start + 64).min(s.middle);
        let expected = s.data[start..end]
            .iter()
            .enumerate()
            .fold(0, |bits, (j, &word)| bits | (((word != 0) as u64) << j));
        assert_eq!(s.data[s.middle + i], expected);
        if expected != 0 {
            occupied.push(i);
        }
    }
    assert_eq!(values(&s.summary, count, &s.data), occupied);
}

#[test]
fn packed_arena_and_cached_extrema() {
    for n in [1usize, 65, 4097, 262145, 16777217, 268435457] {
        let mut s = VebTree::new(n);
        check(&s);
        let mut state = 98173usize;
        for i in 0..10000 {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let p = state % n;
            s.insert(p);
            if i % 3 == 0 {
                s.remove(s.min());
            }
            if i % 7 == 0 && !s.is_empty() {
                s.remove(s.max());
            }
        }
        check(&s);
        let copy = VebTree::from_vec_u64(s.data[..s.middle].to_vec());
        check(&copy);
        while !s.is_empty() {
            s.remove(s.min());
        }
        check(&s);
    }
}

#[test]
fn recursive_nodes_beyond_the_packed_base() {
    use std::collections::BTreeSet;
    use std::ops::Bound::{Excluded, Unbounded};
    for n in [262145usize, 16777217] {
        let mut total = if n == 262145 { n } else { 0 };
        let mut node = VebNode::new(n, &mut total);
        let mut data = vec![0u64; total];
        let mut expected = BTreeSet::new();
        if n == 262145 {
            for p in (0..n).step_by(97) {
                data[p] = 1;
                expected.insert(p);
            }
            node.build(0, n, &mut data);
            assert_eq!(
                values(&node, n, &data),
                expected.iter().copied().collect::<Vec<_>>()
            );
        }
        let mut state = 71usize;
        for i in 0..20000 {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let p = state % n;
            if expected.insert(p) {
                node.insert(p, &mut data);
            }
            if i % 3 == 0 {
                let p = *expected.first().unwrap();
                expected.remove(&p);
                node.remove(p, &mut data);
            }
            assert_eq!(
                node.prev(p, &data),
                expected
                    .range(..p)
                    .next_back()
                    .copied()
                    .unwrap_or(usize::MAX)
            );
            assert_eq!(
                node.next(p, &data),
                expected
                    .range((Excluded(p), Unbounded))
                    .next()
                    .copied()
                    .unwrap_or(usize::MAX)
            );
        }
        assert_eq!(
            values(&node, n, &data),
            expected.iter().copied().collect::<Vec<_>>()
        );
        for p in expected {
            node.remove(p, &mut data);
        }
        assert!(values(&node, n, &data).is_empty());
    }
}
