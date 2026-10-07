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
