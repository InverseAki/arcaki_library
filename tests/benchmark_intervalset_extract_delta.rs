#![allow(dead_code)]
use std::{hint::black_box, time::{Duration, Instant}};
mod baseline {
    use std::collections::BTreeMap;
    include!("support/intervalset_extract_before_delta.rs");
}
mod candidate {
    use std::collections::BTreeMap;
    include!("../src/DataStructure/intervalset.rs");
}
macro_rules! normalize {
    (baseline, $iter:expr, $add:expr) => { $iter };
    (candidate, $iter:expr, $add:expr) => { $iter.map(|(l, r)| (l, r, $add)) };
}
macro_rules! run {
    ($module:ident, $case:expr, $records:expr) => {{
        let n = 65536i64;
        let mut sum = 0i64;
        let mut elapsed = Duration::ZERO;
        let total = Instant::now();
        if let Some(k) = $case.strip_prefix("bulk_") {
            let k: i64 = k.parse().unwrap();
            let mut initial = $module::IntervalSet::new();
            for i in 0..n { initial.insert(4*i, 4*i+2); }
            for _ in 0..16 {
                let mut s = initial.clone();
                let start = Instant::now();
                for i in (0..n).step_by(k as usize) {
                    if $records {
                        for (l, r, add) in normalize!($module, s.insert_with_data(4*i+1, 4*(i+k)-1), true) {
                            sum = sum.wrapping_add((r-l) * if add { 1 } else { -1 });
                        }
                        for (l, r, add) in normalize!($module, s.remove_with_data(4*i+1, 4*(i+k)-1), false) {
                            sum = sum.wrapping_add((r-l) * if add { 1 } else { -1 });
                        }
                    } else {
                        s.insert(4*i+1, 4*(i+k)-1);
                        s.remove(4*i+1, 4*(i+k)-1);
                    }
                }
                elapsed += start.elapsed();
                sum = sum.wrapping_add(s.contains(0) as i64);
                black_box(&s);
            }
        } else {
            let mut s = $module::IntervalSet::new();
            for i in 0..n { s.insert(4*i, 4*i+3); }
            let start = Instant::now();
            let mut seed = 123456789u64;
            for q in 0..1000000 {
                seed ^= seed << 7;
                seed ^= seed >> 9;
                let i = (seed % n as u64) as i64;
                let (l, r, insert) = match $case {
                    "shorten" => (4*i+1, 4*i+3, q % 2 != 0),
                    "noop" => (4*i+1, 4*i+2, true),
                    "random" => (4*i, 4*i + ((seed >> 32) % 64 + 1) as i64, seed & 1 != 0),
                    _ => panic!("unknown workload"),
                };
                if $records {
                    if insert {
                        for (a, b, add) in normalize!($module, s.insert_with_data(l, r), true) { sum = sum.wrapping_add((b-a) * if add { 1 } else { -1 }); }
                    } else {
                        for (a, b, add) in normalize!($module, s.remove_with_data(l, r), false) { sum = sum.wrapping_add((b-a) * if add { 1 } else { -1 }); }
                    }
                } else if insert { s.insert(l, r); }
                else { s.remove(l, r); }
            }
            elapsed += start.elapsed();
            for i in 0..4*n { sum = sum.wrapping_add(s.contains(i) as i64); }
            black_box(&s);
        }
        (elapsed.as_secs_f64()*1000., total.elapsed().as_secs_f64()*1000., black_box(sum))
    }};
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let case = args[2].as_str();
    let records = args[3] == "records";
    let (update_ms, total_ms, checksum) = match args[1].as_str() {
        "baseline" => run!(baseline, case, records),
        "candidate" => run!(candidate, case, records),
        _ => panic!(),
    };
    println!("{update_ms:.3} {total_ms:.3} {checksum}");
}
