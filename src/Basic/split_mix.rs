#[derive(Clone, Debug)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    pub const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    #[inline]
    pub fn next(&mut self) -> u64 {
        self.next_u64()
    }

    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }

    #[inline]
    pub fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    #[inline]
    pub fn next_u128(&mut self) -> u128 {
        ((self.next_u64() as u128) << 64) | self.next_u64() as u128
    }

    #[inline]
    pub fn gen_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / 9007199254740992.0)
    }

    #[inline]
    pub fn gen_f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 * (1.0 / 16777216.0)
    }

    pub fn gen_bool(&mut self, p: f64) -> bool {
        assert!((0.0..=1.0).contains(&p), "probability must be in [0, 1]");
        self.gen_f64() < p
    }

    pub fn gen_range<T: SplitMixInteger>(&mut self, range: impl std::ops::RangeBounds<T>) -> T {
        use std::ops::Bound;
        let low = match range.start_bound() {
            Bound::Unbounded => 0,
            Bound::Included(&x) => x.to_ordered(),
            Bound::Excluded(&x) => {
                let x = x.to_ordered();
                assert!(x < T::MAX_ORDERED, "empty random range");
                x + 1
            }
        };
        let high = match range.end_bound() {
            Bound::Unbounded => T::MAX_ORDERED,
            Bound::Included(&x) => x.to_ordered(),
            Bound::Excluded(&x) => x.to_ordered().checked_sub(1).expect("empty random range"),
        };
        assert!(low <= high, "empty random range");
        let width = high.wrapping_sub(low).wrapping_add(1);
        T::from_ordered(low + self.sample_below(width))
    }

    #[inline]
    fn sample_below(&mut self, width: u128) -> u128 {
        if width == 0 {
            return self.next_u128();
        }
        if width == 1u128 << 64 {
            return self.next_u64() as u128;
        }
        if width <= u64::MAX as u128 {
            let width = width as u64;
            let threshold = width.wrapping_neg() % width;
            loop {
                let x = self.next_u64();
                if x >= threshold {
                    return (x % width) as u128;
                }
            }
        }
        let threshold = width.wrapping_neg() % width;
        loop {
            let x = self.next_u128();
            if x >= threshold {
                return x % width;
            }
        }
    }

    pub fn shuffle<T>(&mut self, values: &mut [T]) {
        for i in (1..values.len()).rev() {
            let j = self.gen_range(0..=i);
            values.swap(i, j);
        }
    }

    pub fn choose<'a, T>(&mut self, values: &'a [T]) -> Option<&'a T> {
        if values.is_empty() {
            None
        } else {
            Some(&values[self.gen_range(0..values.len())])
        }
    }

    pub fn choose_mut<'a, T>(&mut self, values: &'a mut [T]) -> Option<&'a mut T> {
        if values.is_empty() {
            None
        } else {
            let i = self.gen_range(0..values.len());
            Some(&mut values[i])
        }
    }
}

mod split_mix_private {
    pub trait Sealed {}
}

pub trait SplitMixInteger: Copy + split_mix_private::Sealed {
    #[doc(hidden)]
    const MAX_ORDERED: u128;
    #[doc(hidden)]
    fn to_ordered(self) -> u128;
    #[doc(hidden)]
    fn from_ordered(value: u128) -> Self;
}

macro_rules! split_mix_unsigned {
    ($($t:ty),*) => {$ (
        impl split_mix_private::Sealed for $t {}
        impl SplitMixInteger for $t {
            const MAX_ORDERED: u128 = <$t>::MAX as u128;
            #[inline]
            fn to_ordered(self) -> u128 { self as u128 }
            #[inline]
            fn from_ordered(value: u128) -> Self { value as Self }
        }
    )*};
}
macro_rules! split_mix_signed {
    ($(($t:ty, $u:ty)),*) => {$ (
        impl split_mix_private::Sealed for $t {}
        impl SplitMixInteger for $t {
            const MAX_ORDERED: u128 = <$u>::MAX as u128;
            #[inline]
            fn to_ordered(self) -> u128 {
                ((self as $u) ^ (1 as $u << (<$t>::BITS - 1))) as u128
            }
            #[inline]
            fn from_ordered(value: u128) -> Self {
                ((value as $u) ^ (1 as $u << (<$t>::BITS - 1))) as Self
            }
        }
    )*};
}
split_mix_unsigned!(u8, u16, u32, u64, u128, usize);
split_mix_signed!(
    (i8, u8),
    (i16, u16),
    (i32, u32),
    (i64, u64),
    (i128, u128),
    (isize, usize)
);
