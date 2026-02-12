pub mod fxhash {
    use std::hash::BuildHasherDefault;
    const K: u64 = 0x517c_c1b7_2722_0a95;
    #[derive(Default)]
    pub struct FxHasher {
        pub hash: u64,
    }
    impl FxHasher {
        #[inline(always)]
        fn mix_u64(mut h: u64, x: u64) -> u64 {
            h = h.rotate_left(5) ^ x;
            h = h.wrapping_mul(K);
            let x2 = x ^ (x >> 33) ^ (x << 11);
            h = h.rotate_left(5) ^ x2;
            h = h.wrapping_mul(K);
            h
        }

        #[inline(always)]
        fn write_u64_impl(&mut self, x: u64) {
            self.hash = Self::mix_u64(self.hash, x);
        }
    }

    impl std::hash::Hasher for FxHasher {
        #[inline(always)]
        fn finish(&self) -> u64 {
            self.hash
        }

        #[inline(always)]
        fn write(&mut self, bytes: &[u8]) {
            let mut h = self.hash;
            for &b in bytes {
                h = h.rotate_left(5) ^ (b as u64);
                h = h.wrapping_mul(K);
            }
            self.hash = h;
        }

        #[inline(always)]
        fn write_u64(&mut self, i: u64) { self.write_u64_impl(i); }
        #[inline(always)]
        fn write_u32(&mut self, i: u32) { self.write_u64_impl(i as u64); }
        #[inline(always)]
        fn write_u16(&mut self, i: u16) { self.write_u64_impl(i as u64); }
        #[inline(always)]
        fn write_u8 (&mut self, i: u8 ) { self.write_u64_impl(i as u64); }
        #[inline(always)]
        fn write_usize(&mut self, i: usize) { self.write_u64_impl(i as u64); }
        #[inline(always)]
        fn write_i64(&mut self, i: i64) { self.write_u64_impl(i as u64); }
        #[inline(always)]
        fn write_i32(&mut self, i: i32) { self.write_u64_impl(i as u64); }
        #[inline(always)]
        fn write_i16(&mut self, i: i16) { self.write_u64_impl(i as u64); }
        #[inline(always)]
        fn write_i8 (&mut self, i: i8 ) { self.write_u64_impl(i as u64); }
        #[inline(always)]
        fn write_isize(&mut self, i: isize) { self.write_u64_impl(i as u64); }
    }

    pub type FxBuildHasher = BuildHasherDefault<FxHasher>;
    pub type FxMap<K, V> = std::collections::HashMap<K, V, FxBuildHasher>;
    pub type FxSet<K> = std::collections::HashSet<K, FxBuildHasher>;
}
