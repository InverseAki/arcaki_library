#![allow(dead_code)]
#[path = "../src/Basic/barrett.rs"]
mod barrett;
#[path = "../src/Basic/couter.rs"]
mod counter;
#[path = "../src/Basic/hashcounter.rs"]
mod hashcounter;
mod math {
    const MOD: i64 = 998244353;
    include!("../src/Basic/math.rs");
}
mod combinations {
    use super::barrett::Barrett32;
    include!("../src/NumberTheory/barrett_combination.rs");
}
use combinations::BarrettCombination;
use counter::Counter;
use hashcounter::HashCounter;

#[test]
fn generic_integer_boundaries() {
    macro_rules! unsigned { ($($t:ty),*) => {$(
        assert_eq!(math::gcd(<$t>::MAX, <$t>::MAX-1),1);
        assert_eq!(math::gcd(0 as $t, 0),0);
        assert_eq!(math::floor(17 as $t,5),3);
        assert_eq!(math::modulo(17 as $t,5),2);
    )*}; }
    unsigned!(u8, u16, u32, u64, u128, usize);
    macro_rules! signed { ($($t:ty),*) => {$(
        assert_eq!(math::gcd(<$t>::MAX, <$t>::MAX-1),1);
        assert_eq!(math::modulo(<$t>::MIN,-1),0);
        assert_eq!(math::floor(-17 as $t,5),-4);
        assert_eq!(math::floor(17 as $t,-5),-3);
        assert_eq!(math::modulo(-17 as $t,-5),3);
        assert_eq!(math::extended_gcd(<$t>::MAX,<$t>::MAX-1),(1,1,-1));
        assert_eq!(math::extended_gcd(0 as $t,0),(0,1,0));
    )*}; }
    signed!(i8, i16, i32, i64, i128, isize);
    for a in 0..=127i8 {
        for b in 0..=127i8 {
            let (g, x, y) = math::extended_gcd(a, b);
            assert_eq!(a as i32 * x as i32 + b as i32 * y as i32, g as i32);
            assert_eq!(g, math::gcd(a, b));
        }
    }
    let mut s = 1789u128;
    for _ in 0..10000 {
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
        let a = (s >> 1) as i128;
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
        let b = (s >> 1) as i128;
        let (g, x, y) = math::extended_gcd(a, b);
        assert_eq!(g, math::gcd(a, b));
        // 等式の積がi128を超える場合も、mod 2^128なら検証できる。
        assert_eq!(a.wrapping_mul(x).wrapping_add(b.wrapping_mul(y)), g);
    }
}

#[test]
fn runtime_combinations_pascal_and_independent_moduli() {
    for m in [2, 3, 5, 7, 11, 97, 1000000007, 998244353, u32::MAX] {
        let n = if m == u32::MAX {
            2
        } else {
            60.min(m as usize - 1)
        };
        let c = BarrettCombination::new(n, m);
        assert_eq!(c.max_n(), n);
        assert_eq!(c.modulus(), m);
        let mut row = vec![1u64];
        for a in 0..=n {
            for k in 0..=a {
                assert_eq!(c.c(a, k) as u64, row[k]);
                let perm = (0..k).fold(1u64, |x, i| x * (a - i) as u64 % m as u64);
                assert_eq!(c.p(a, k) as u64, perm);
            }
            assert_eq!(c.c(a, a + 1), 0);
            let mut next = vec![1; row.len() + 1];
            for k in 1..row.len() {
                next[k] = (row[k - 1] + row[k]) % m as u64;
            }
            row = next;
        }
        let bt = barrett::Barrett32::new(m);
        for i in 1..=n {
            assert_eq!(bt.mul(c.inv(i), i as u32), 1);
            assert_eq!(bt.mul(c.f(i), c.fi(i)), 1);
        }
        assert_eq!(c.h(0, 0), 1);
        assert_eq!(c.h(0, 3), 0);
        assert_eq!(c.h(1, n), 1);
        assert_eq!(bt.mod_pow(7, 0), 1);
        for e in 0..40 {
            let expected = (0..e).fold(1u64, |v, _| v * 7 % m as u64) as u32;
            assert_eq!(bt.mod_pow(7, e), expected);
            assert_eq!(bt.pow(7, e), expected);
        }
    }
    assert!(BarrettCombination::try_new(2, 4).is_none());
    assert!(BarrettCombination::try_new(7, 7).is_none());
    assert!(BarrettCombination::try_new(0, 1).is_none());
    let a = BarrettCombination::with_barrett(4, barrett::Barrett32::new(25));
    let b = BarrettCombination::new(4, 7);
    assert_eq!((a.c(4, 2), b.c(4, 2)), (6, 6));
    assert_eq!((a.p(4, 3), b.p(4, 3)), (24, 3));
}

macro_rules! check_counter {
    ($new:expr) => {{
        let mut c = $new;
        let mut model = [0usize; 16];
        let mut seed = 891u64;
        for step in 0..20000 {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            let x = (seed % 16) as usize;
            let y = ((seed >> 8) % 16) as usize;
            let n = ((seed >> 16) % 8) as usize;
            match (seed >> 24) % 10 {
                0 => {
                    c.add(x, n);
                    model[x] += n;
                }
                1 => {
                    c.sub(x, n);
                    model[x] = model[x].saturating_sub(n);
                }
                2 => {
                    c.one_add(x);
                    model[x] += 1;
                }
                3 => {
                    c.one_sub(x);
                    model[x] = model[x].saturating_sub(1);
                }
                4 => {
                    c.del(x);
                    model[x] = 0;
                }
                5 => {
                    c.one_update(x, y);
                    model[x] = model[x].saturating_sub(1);
                    model[y] += 1;
                }
                6 if model[x] > 0 => {
                    let k = n.min(model[x]);
                    c.sub_ex(x, k);
                    model[x] -= k;
                }
                7 if model[x] > 0 => {
                    c.add_ex(x, n);
                    model[x] += n;
                }
                8 => {
                    let mut rhs = $new;
                    for i in 0..n {
                        rhs.add(i, i + 1);
                        model[i] += i + 1;
                    }
                    c.merge(&mut rhs);
                    assert_eq!(rhs.total(), 0);
                    assert!(rhs.is_empty());
                }
                _ => {}
            }
            assert_eq!(c.total(), model.iter().sum::<usize>(), "step {step}");
            assert_eq!(c.len(), model.iter().filter(|&&v| v > 0).count());
            assert_eq!(c.is_empty(), c.len() == 0);
            for i in 0..16 {
                assert_eq!(c.cnt(i), model[i]);
                assert_eq!(c.include(i), model[i] > 0);
            }
        }
        c.clear();
        assert_eq!(c.total(), 0);
        assert!(c.is_empty());
        c.add(0, usize::MAX);
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| c.add(1, 1))).is_err());
        assert_eq!(c.total(), usize::MAX);
        assert_eq!(c.len(), 1);
        c.clear();
        c.add(2, 3);
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| c.sub_ex(2, 4))).is_err());
        assert_eq!(c.total(), 3);
        assert_eq!(c.cnt(2), 3);
        c.add(1, 0);
        assert!(!c.include(1));
        c.sub(1, 100);
        assert_eq!(c.total(), 3);
    }};
}
#[test]
fn ordered_counter_random_model() {
    check_counter!(Counter::<usize>::new());
    let mut c = Counter::new();
    c.add(4, 2);
    c.add(1, 3);
    c.add(8, 0);
    assert_eq!((c.mi(), c.mx()), (Some(1), Some(4)));
    assert_eq!(
        c.range(2..=5).map(|(&k, &v)| (k, v)).collect::<Vec<_>>(),
        vec![(4, 2)]
    );
}
#[test]
fn hash_counter_random_model_and_custom_hasher() {
    check_counter!(HashCounter::<usize>::new());
    check_counter!(HashCounter::<
        usize,
        std::hash::BuildHasherDefault<std::collections::hash_map::DefaultHasher>,
    >::default());
    // OrdもCopyも持たないキーで使用可能。
    #[derive(Hash, PartialEq, Eq)]
    struct Key(String);
    let mut h = HashCounter::new();
    h.add(Key("x".into()), 2);
    assert_eq!(h.cnt(Key("x".into())), 2);
}
