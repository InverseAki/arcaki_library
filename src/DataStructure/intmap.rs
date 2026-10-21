// Source: https://github.com/InverseAki/arcaki_library/blob/main/src/DataStructure/intmap.rs
pub mod intmap {
    use std::collections::HashMap;
    use std::hash::{BuildHasherDefault, Hasher};
    mod sealed {
        pub trait Sealed {}
        impl Sealed for u32 {}
        impl Sealed for u64 {}
        impl Sealed for usize {}
    }
    pub trait IntKey: sealed::Sealed + Copy + Eq + std::hash::Hash {}
    impl IntKey for u32 {}
    impl IntKey for u64 {}
    impl IntKey for usize {}

    #[derive(Default)]
    pub struct IntHasher(u64);
    impl IntHasher {
        #[inline(always)]
        fn integer(&mut self, x: u64) {
            let p = ((x ^ 0xa076_1d64_78bd_642f) as u128) * 0xe703_7ed1_a0b4_28dbu128;
            self.0 = p as u64 ^ (p >> 64) as u64;
        }
    }
    impl Hasher for IntHasher {
        #[inline(always)]
        fn finish(&self) -> u64 {
            self.0
        }
        fn write(&mut self, _: &[u8]) {
            panic!("IntHasher supports only u32, u64 and usize keys");
        }
        #[inline(always)]
        fn write_u32(&mut self, x: u32) {
            self.integer(x as u64);
        }
        #[inline(always)]
        fn write_u64(&mut self, x: u64) {
            self.integer(x);
        }
        #[inline(always)]
        fn write_usize(&mut self, x: usize) {
            self.integer(x as u64);
        }
    }

    pub struct IntMap<K: IntKey, V>(HashMap<K, V, BuildHasherDefault<IntHasher>>);
    pub type U32Map<V> = IntMap<u32, V>;
    pub type U64Map<V> = IntMap<u64, V>;
    pub type UsizeMap<V> = IntMap<usize, V>;
    impl<K: IntKey, V> IntMap<K, V> {
        pub fn new() -> Self {
            Self(HashMap::default())
        }
        pub fn with_capacity(capacity: usize) -> Self {
            Self(HashMap::with_capacity_and_hasher(
                capacity,
                Default::default(),
            ))
        }
    }
    impl<K: IntKey, V> Default for IntMap<K, V> {
        fn default() -> Self {
            Self::new()
        }
    }
    impl<K: IntKey, V> std::ops::Deref for IntMap<K, V> {
        type Target = HashMap<K, V, BuildHasherDefault<IntHasher>>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<K: IntKey, V> std::ops::DerefMut for IntMap<K, V> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<K: IntKey, V: Clone> Clone for IntMap<K, V> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<K: IntKey + std::fmt::Debug, V: std::fmt::Debug> std::fmt::Debug for IntMap<K, V> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.0.fmt(f)
        }
    }
    impl<K: IntKey, V> Extend<(K, V)> for IntMap<K, V> {
        fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iter: I) {
            self.0.extend(iter);
        }
    }
    impl<K: IntKey, V> FromIterator<(K, V)> for IntMap<K, V> {
        fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
            Self(iter.into_iter().collect())
        }
    }
    impl<K: IntKey, V> std::ops::Index<&K> for IntMap<K, V> {
        type Output = V;
        fn index(&self, key: &K) -> &V {
            &self.0[key]
        }
    }
    impl<K: IntKey, V> std::ops::IndexMut<&K> for IntMap<K, V> {
        fn index_mut(&mut self, key: &K) -> &mut V {
            self.0.get_mut(key).expect("missing key")
        }
    }
    impl<K: IntKey, V: PartialEq> PartialEq for IntMap<K, V> {
        fn eq(&self, other: &Self) -> bool {
            self.0 == other.0
        }
    }
    impl<K: IntKey, V: Eq> Eq for IntMap<K, V> {}
    impl<K: IntKey, V> IntoIterator for IntMap<K, V> {
        type Item = (K, V);
        type IntoIter = std::collections::hash_map::IntoIter<K, V>;
        fn into_iter(self) -> Self::IntoIter {
            self.0.into_iter()
        }
    }
    impl<'a, K: IntKey, V> IntoIterator for &'a IntMap<K, V> {
        type Item = (&'a K, &'a V);
        type IntoIter = std::collections::hash_map::Iter<'a, K, V>;
        fn into_iter(self) -> Self::IntoIter {
            self.0.iter()
        }
    }
    impl<'a, K: IntKey, V> IntoIterator for &'a mut IntMap<K, V> {
        type Item = (&'a K, &'a mut V);
        type IntoIter = std::collections::hash_map::IterMut<'a, K, V>;
        fn into_iter(self) -> Self::IntoIter {
            self.0.iter_mut()
        }
    }
}
