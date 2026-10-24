#![allow(dead_code)]
include!("../src/OfflineQuery/point_add_rectangle_sum.rs");

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 7;
        self.0 ^= self.0 >> 9;
        self.0 ^= self.0 << 8;
        self.0
    }
}

#[test]
fn adversarial_blocks_and_wrapping_weights() {
    for seed in 1..=10000 {
        let mut rng = Rng(seed);
        let mut s = PointAddRectangleSum::new();
        let mut points = Vec::new();
        let mut expected = Vec::new();
        let block = 1 + seed as usize % 65;
        for i in 0..(seed as usize % 200 + 1) {
            let x = (rng.next() % 9) as i32 - 4;
            let y = (rng.next() % 9) as i32 - 4;
            if i % (block * 2) < block {
                let w = match rng.next() % 5 {
                    0 => 0,
                    1 => i64::MIN,
                    2 => i64::MAX,
                    _ => rng.next() as i64,
                };
                points.push((x, y, w));
                s.push_add(x, y, w);
            } else {
                let rx = x + (rng.next() % 7) as i32;
                let ry = y + (rng.next() % 7) as i32;
                s.push_query(x, y, rx, ry);
                expected.push(
                    points
                        .iter()
                        .filter(|&&(px, py, _)| x <= px && px < rx && y <= py && py < ry)
                        .fold(0i64, |sum, p| sum.wrapping_add(p.2)),
                );
            }
        }
        assert_eq!(s.solve(), expected, "seed={seed}");
        assert!(s.solve().is_empty());
    }
}

#[test]
fn cancellation_and_minimum_weight_with_initial() {
    let mut s = PointAddRectangleSum::build([(0, 0, i64::MIN), (0, 0, i64::MAX), (1, 1, 1)]);
    s.push_query(-1, -1, 2, 2);
    s.push_add(0, 0, i64::MIN);
    s.push_query(0, 0, 1, 1);
    s.push_add(0, 0, i64::MIN);
    s.push_query(0, 0, 1, 1);
    s.push_add(0, 0, 1);
    s.push_query(0, 0, 1, 1);
    assert_eq!(s.solve(), [0, i64::MAX, -1, 0]);
}

#[test]
fn public_bit_keeps_generic_semantics() {
    let mut bit = BIT::new(10, 0.0f64);
    bit.add(0, 1e20);
    bit.add(2, 3.5);
    bit.add(9, -7.0);
    for l in 0..=10 {
        for r in 0..=10 {
            assert_eq!(bit.prod(l, r), bit.g(r) - bit.g(l));
        }
    }
    assert!(std::panic::catch_unwind(|| bit.g(11)).is_err());
}

#[test]
fn private_bit_all_intervals_and_rollback() {
    for n in 1..=130 {
        let mut bit = PointAddRectangleSumBit::new(n);
        let values: Vec<_> = (0..n)
            .map(|i| ((i * 1009) as i64).wrapping_mul(i64::MAX))
            .collect();
        for (i, &v) in values.iter().enumerate() {
            bit.add(i, v);
        }
        for l in 0..=n {
            for r in l..=n {
                let expected = values[l..r].iter().fold(0i64, |s, &v| s.wrapping_add(v));
                assert_eq!(bit.prod(l, r), expected, "n={n}, l={l}, r={r}");
            }
        }
        for (i, &v) in values.iter().enumerate() {
            bit.add(i, v.wrapping_neg());
        }
        assert!(bit.vec.iter().all(|&v| v == 0));
    }
}

#[test]
fn packed_event_size_and_query_limit() {
    assert_eq!(std::mem::size_of::<PointAddRectangleSumEvent>(), 16);
    let mut solver: PointAddRectangleSum = PointAddRectangleSum::new();
    solver.queries = i32::MAX as usize + 1;
    assert!(std::panic::catch_unwind(move || solver.solve()).is_err());
}
