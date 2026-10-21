#[path = "../src/DataStructure/intmap.rs"]
mod implementation;
use implementation::intmap::*;
use std::collections::HashMap;
use std::hash::Hasher;
use std::panic::{catch_unwind, AssertUnwindSafe};

fn next(x: &mut u64) -> u64 {
    *x ^= *x << 13;
    *x ^= *x >> 7;
    *x ^= *x << 17;
    *x
}

macro_rules! model_test {
    ($name:ident, $key:ty) => {
        #[test]
        fn $name() {
            let mut map = IntMap::<$key, i64>::new();
            let mut model = HashMap::new();
            let mut rng = 912378123u64;
            for step in 0..150_000 {
                let r = next(&mut rng);
                let key = match r % 8 {
                    0 => <$key>::MAX,
                    1 => 0,
                    2 => (r as $key).wrapping_shl(<$key>::BITS / 2),
                    _ => ((r >> 12) % 2048) as $key,
                };
                let value = (r >> 8) as i64;
                match r >> 60 {
                    0..=3 => assert_eq!(map.insert(key, value), model.insert(key, value)),
                    4..=6 => assert_eq!(map.remove(&key), model.remove(&key)),
                    7..=9 => {
                        let a = map.entry(key).and_modify(|v| *v ^= value).or_insert(value);
                        let b = model
                            .entry(key)
                            .and_modify(|v| *v ^= value)
                            .or_insert(value);
                        assert_eq!(a, b);
                    }
                    10 => assert_eq!(
                        map.get_mut(&key).map(|v| {
                            *v = value;
                            *v
                        }),
                        model.get_mut(&key).map(|v| {
                            *v = value;
                            *v
                        })
                    ),
                    11 => map.reserve(3),
                    _ => assert_eq!(map.get(&key), model.get(&key)),
                }
                assert_eq!(map.len(), model.len());
                assert_eq!(map.contains_key(&key), model.contains_key(&key));
                if step % 4096 == 0 {
                    assert_eq!(
                        map.iter().map(|(&k, &v)| (k, v)).collect::<HashMap<_, _>>(),
                        model
                    );
                    let copy = map.clone();
                    assert_eq!(copy.len(), model.len());
                    for (k, v) in &model {
                        assert_eq!(copy.get(k), Some(v));
                    }
                }
                if step % 30_000 == 29_999 {
                    map.clear();
                    model.clear();
                }
            }
        }
    };
}
model_test!(u32_model, u32);
model_test!(u64_model, u64);
model_test!(usize_model, usize);

#[test]
fn collisions_and_wraparound() {
    let mut map = U64Map::with_capacity(96);
    let keys: Vec<_> = (0..)
        .filter(|&k| {
            k != u64::MAX && {
                let mut h = IntHasher::default();
                h.write_u64(k);
                h.finish() & 255 == 254
            }
        })
        .take(96)
        .collect();
    for (i, &key) in keys.iter().enumerate() {
        map.insert(key, i);
    }
    for i in (0..keys.len()).step_by(3) {
        assert_eq!(map.remove(&keys[i]), Some(i));
    }
    for (i, key) in keys.iter().enumerate() {
        assert_eq!(map.get(key), if i % 3 == 0 { None } else { Some(&i) });
    }
    for i in (0..keys.len()).rev() {
        map.remove(&keys[i]);
    }
    assert!(map.is_empty());
    for (i, &key) in keys.iter().enumerate() {
        *map.entry(key).or_default() = i;
    }
    for (i, key) in keys.iter().enumerate() {
        assert_eq!(map[key], i);
    }
}

#[test]
fn api_and_boundaries() {
    let mut map: U32Map<String> = [
        (0, "zero".into()),
        (u32::MAX, "max".into()),
        (0, "ZERO".into()),
    ]
    .into_iter()
    .collect();
    assert_eq!(map.len(), 2);
    map[&0].push('!');
    for (_, v) in map.iter_mut() {
        v.push('?');
    }
    for v in map.values_mut() {
        v.push('.');
    }
    assert_eq!(map[&0], "ZERO!?.");
    assert_eq!(map[&u32::MAX], "max?.");
    assert_eq!(map.keys().count(), 2);
    assert_eq!(map.values().count(), 2);
    map.entry(0)
        .or_insert_with(|| panic!("occupied entry must be lazy"));
    map.entry(u32::MAX)
        .or_insert_with(|| panic!("occupied entry must be lazy"));
    let cap = map.capacity();
    map.clear();
    assert_eq!(map.capacity(), cap);
    assert!(map.is_empty());
    assert_eq!(map.remove(&0), None);
    let mut empty = UsizeMap::<()>::default();
    assert_eq!(empty.get(&1), None);
    assert_eq!(empty.remove(&1), None);
    empty.entry(usize::MAX).or_default();
    empty.entry(0).or_default();
    assert_eq!(empty.iter().count(), 2);
    empty.remove(&usize::MAX);
    empty.remove(&0);
    assert!(empty.is_empty());
    assert_eq!(U64Map::<u64>::new().capacity(), 0);
}

#[test]
fn drops_and_unwinding() {
    use std::cell::Cell;
    use std::rc::Rc;
    struct Value(Rc<Cell<usize>>);
    impl Drop for Value {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }
    let drops = Rc::new(Cell::new(0));
    let mut created = 0;
    {
        let mut map = U64Map::new();
        for key in (0..4000).chain([u64::MAX, 0, u64::MAX]) {
            map.insert(key, Value(drops.clone()));
            created += 1;
        }
        for key in (0..4000).step_by(3) {
            map.remove(&key);
        }
        assert!(catch_unwind(AssertUnwindSafe(|| {
            map.entry(99999).or_insert_with(|| panic!("factory"));
        }))
        .is_err());
        assert_eq!(map.get(&99999).map(|_| ()), None);
        map.reserve(20_000);
        assert_eq!(drops.get() + map.len(), created);
        map.clear();
        assert_eq!(drops.get(), created);
        map.insert(u64::MAX, Value(drops.clone()));
        created += 1;
        map.insert(0, Value(drops.clone()));
        created += 1;
    }
    assert_eq!(drops.get(), created);
}

#[test]
fn panicking_clone_and_drop() {
    use std::cell::Cell;
    use std::rc::Rc;
    #[derive(Debug)]
    struct Value {
        live: Rc<Cell<usize>>,
        clones: Rc<Cell<usize>>,
        panic_drop: bool,
    }
    impl Clone for Value {
        fn clone(&self) -> Self {
            let n = self.clones.get();
            self.clones.set(n + 1);
            if n == 7 {
                panic!("clone");
            }
            self.live.set(self.live.get() + 1);
            Self {
                live: self.live.clone(),
                clones: self.clones.clone(),
                panic_drop: false,
            }
        }
    }
    impl Drop for Value {
        fn drop(&mut self) {
            self.live.set(self.live.get() - 1);
            if self.panic_drop {
                panic!("drop");
            }
        }
    }
    let live = Rc::new(Cell::new(0));
    let clones = Rc::new(Cell::new(0));
    let mut map = U64Map::new();
    for key in (0..20).chain([u64::MAX]) {
        live.set(live.get() + 1);
        map.insert(
            key,
            Value {
                live: live.clone(),
                clones: clones.clone(),
                panic_drop: key == u64::MAX,
            },
        );
    }
    assert!(catch_unwind(AssertUnwindSafe(|| map.clone())).is_err());
    assert_eq!(live.get(), map.len());
    assert!(catch_unwind(AssertUnwindSafe(|| map.clear())).is_err());
    assert!(map.is_empty());
    let remaining = live.get();
    live.set(remaining + 1);
    map.insert(
        1234,
        Value {
            live: live.clone(),
            clones: clones.clone(),
            panic_drop: false,
        },
    );
    map.clear();
    assert_eq!(live.get(), remaining);
    drop(map);
    assert_eq!(live.get(), remaining);
}

#[test]
fn standard_map_api() {
    use std::collections::hash_map::Entry;
    let mut map: U32Map<u64> = (0..100).map(|x| (x, x as u64)).collect();
    match map.entry(3) {
        Entry::Occupied(mut e) => {
            *e.get_mut() += 10;
        }
        Entry::Vacant(_) => panic!("missing"),
    }
    assert_eq!(map.remove_entry(&3), Some((3, 13)));
    map.retain(|&k, _| k % 2 == 0);
    let copy = map.clone();
    assert_eq!(copy, map);
    for (_, v) in &mut map {
        *v += 1;
    }
    assert_ne!(copy, map);
    assert_eq!((&map).into_iter().count(), map.len());
    let count = map.len();
    assert_eq!(map.into_iter().count(), count);
    let mut copy = copy;
    assert_eq!(copy.drain().count(), count);
    assert!(copy.is_empty());
}
