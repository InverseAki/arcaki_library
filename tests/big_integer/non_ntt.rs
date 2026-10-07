use super::*;
fn check<const B: u32>() {
    let mut state = 20261004u64;
    let mut random = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        (state % B as u64) as u32
    };
    for n in [
        2, 3, 4, 5, 7, 8, 9, 15, 16, 31, 32, 33, 63, 64, 65, 127, 128, 129, 255,
    ] {
        for qn in [1, 2, 3, 4, 7, 8, 31, 32, 33] {
            for top in [1, B / 2, B - 1] {
                let mut b: Vec<_> = (0..n).map(|_| random()).collect();
                b[n - 1] = top;
                let mut q: Vec<_> = (0..qn).map(|_| random()).collect();
                q[qn - 1] = (B - 1).max(1);
                for r in [vec![], vec![1], sub::<B>(&b, &[1])] {
                    let a = add::<B>(&mul::<B>(&q, &b), &r);
                    let scale = B / (b[n - 1] + 1);
                    let (reference_q, reference_r) = school_div::<B>(
                        &mul_small::<B>(&a, scale),
                        &mul_small::<B>(&b, scale),
                    );
                    let (reference_r, rem) = div_small::<B>(&reference_r, scale);
                    assert_eq!(rem, 0);
                    let (wide_q, wide_r) = wide_school::divide::<B>(&a, &b);
                    assert_eq!(wide_q, reference_q);
                    assert_eq!(wide_r, reference_r);
                    assert_eq!(div_rem::<B>(&a, &b), (q.clone(), r));
                }
            }
        }
    }
    for p in [0, 1, 2, 31, 32, 63, 64, 65, 255] {
        for top in [1, 2] {
            let x = power(p, top);
            for a in [vec![], vec![1], x.clone(), sub::<B>(&x, &[1])] {
                assert_eq!(power_minus::<B>(p, top, a.clone()), sub::<B>(&x, &a));
            }
        }
    }
    for n in [1, 4, 64, 257] {
        let max = vec![B - 1; n];
        let pow = power(n, 1);
        assert_eq!(add::<B>(&max, &[1]), pow);
        assert_eq!(sub::<B>(&pow, &[1]), max);
        assert_eq!(add::<B>(&max, &[]), max);
    }
}
#[test]
fn wide_school_and_reused_arrays_decimal() {
    check::<10000>();
}
#[test]
fn wide_school_and_reused_arrays_hex() {
    check::<65536>();
}
