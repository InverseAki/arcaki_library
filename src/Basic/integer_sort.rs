// Source: https://github.com/InverseAki/arcaki_library/blob/main/src/Basic/integer_sort.rs
mod integer_sort_private {
    pub trait Sealed {}
}

pub trait IntegerSortValue: Copy + Ord + integer_sort_private::Sealed {
    fn sort_key(self) -> u64;
    fn from_sort_key(key: u64) -> Self;
}

macro_rules! integer_sort_unsigned {
    ($($t:ty),*) => {$(
        impl integer_sort_private::Sealed for $t {}
        impl IntegerSortValue for $t {
            #[inline]
            fn sort_key(self) -> u64 { self as u64 }
            #[inline]
            fn from_sort_key(key: u64) -> Self { key as Self }
        }
    )*};
}
integer_sort_unsigned!(u32, u64, usize);

impl integer_sort_private::Sealed for i64 {}
impl IntegerSortValue for i64 {
    #[inline]
    fn sort_key(self) -> u64 {
        (self as u64) ^ (1 << 63)
    }
    #[inline]
    fn from_sort_key(key: u64) -> Self {
        (key ^ (1 << 63)) as Self
    }
}

pub fn integer_sort<T: IntegerSortValue>(values: &mut [T]) {
    integer_sort_with_buffer(values, &mut Vec::new());
}

pub fn integer_sort_with_buffer<T: IntegerSortValue>(values: &mut [T], buffer: &mut Vec<T>) {
    let n = values.len();
    if n <= 128 {
        values.sort_unstable();
        return;
    }
    if values.is_sorted() {
        return;
    }
    if values.windows(2).all(|w| w[0] >= w[1]) {
        values.reverse();
        return;
    }
    let first = values[0].sort_key();
    let mut low = first;
    let mut high = first;
    let mut varying = 0;
    for &value in values.iter() {
        let key = value.sort_key();
        low = low.min(key);
        high = high.max(key);
        varying |= key ^ first;
    }
    let offset = varying.trailing_zeros();
    let range = (high - low) >> offset;
    if range <= 65535 && range < (n / 8) as u64 {
        let mut counts = vec![0usize; range as usize + 1];
        for &value in values.iter() {
            counts[((value.sort_key() - low) >> offset) as usize] += 1;
        }
        let mut pos = 0;
        for (i, count) in counts.into_iter().enumerate() {
            if count != 0 {
                values[pos..pos + count].fill(T::from_sort_key(low + ((i as u64) << offset)));
                pos += count;
            }
        }
        return;
    }
    let mut width = 64 - varying.leading_zeros() - offset;
    let mut base = 0;
    let normalized_width = 64 - range.leading_zeros();
    if normalized_width < width {
        width = normalized_width;
        base = low;
        varying = (u64::MAX >> range.leading_zeros()) << offset;
    }
    if (n < 1024 && width > 16) || (n < 8192 && width > 32) {
        values.sort_unstable();
        return;
    }
    buffer.resize(n, values[0]);
    if width == 9 {
        integer_sort_radix::<T, 512>(values, &mut buffer[..n], varying, base);
    } else if width == 10 {
        integer_sort_radix::<T, 1024>(values, &mut buffer[..n], varying, base);
    } else if width == 11 {
        integer_sort_radix::<T, 2048>(values, &mut buffer[..n], varying, base);
    } else if n < 2048 || width <= 16 {
        integer_sort_radix::<T, 256>(values, &mut buffer[..n], varying, base);
    } else {
        integer_sort_radix::<T, 2048>(values, &mut buffer[..n], varying, base);
    }
}

fn integer_sort_radix<T: IntegerSortValue, const BINS: usize>(
    values: &mut [T],
    buffer: &mut [T],
    mut varying: u64,
    base: u64,
) {
    let bits = BINS.trailing_zeros();
    let mask = BINS - 1;
    let mut shift = varying.trailing_zeros();
    varying >>= shift;
    let mut source = values;
    let mut target = buffer;
    let mut in_buffer = false;
    while varying != 0 {
        if varying & mask as u64 != 0 {
            if source.is_sorted() {
                break;
            }
            let mut counts = [0usize; BINS];
            for &value in source.iter() {
                counts[((value.sort_key() - base) >> shift) as usize & mask] += 1;
            }
            let mut sum = 0;
            for count in counts.iter_mut() {
                let len = *count;
                *count = sum;
                sum += len;
            }
            for &value in source.iter() {
                let pos = &mut counts[((value.sort_key() - base) >> shift) as usize & mask];
                target[*pos] = value;
                *pos += 1;
            }
            std::mem::swap(&mut source, &mut target);
            in_buffer = !in_buffer;
        }
        varying >>= bits;
        shift += bits;
    }
    if in_buffer {
        target.copy_from_slice(source);
    }
}
