pub trait BenchInt: Copy + Ord + std::fmt::Debug {
    const BITS: usize;
    fn key(self) -> u64;
    fn from_bits(x: u64) -> Self;
    fn from_key(x: u64) -> Self;
}
macro_rules! bench_int {
    ($t:ty, $xor:expr) => {
        impl BenchInt for $t {
            const BITS: usize = <$t>::BITS as usize;
            fn key(self) -> u64 {
                (self as u64) ^ $xor
            }
            fn from_bits(x: u64) -> Self {
                x as Self
            }
            fn from_key(x: u64) -> Self {
                (x ^ $xor) as Self
            }
        }
    };
}
bench_int!(u64, 0);
bench_int!(u32, 0);
bench_int!(usize, 0);
bench_int!(i64, 1 << 63);

pub fn lsd<T: BenchInt, const B: usize, const ALL: bool, const UNCHECKED: bool>(a: &mut [T]) {
    if a.len() < 2 {
        return;
    }
    let base = a[0].key();
    let mut varying = 0;
    for &x in a.iter() {
        varying |= x.key() ^ base;
    }
    if varying == 0 {
        return;
    }
    let bins = 1usize << B;
    let mask = bins - 1;
    let shifts: Vec<_> = (0..T::BITS)
        .step_by(B)
        .filter(|&s| (varying >> s) & mask as u64 != 0)
        .collect();
    let mut counts = vec![0usize; bins * if ALL { shifts.len() } else { 1 }];
    if ALL {
        for &x in a.iter() {
            let k = x.key();
            for (row, &s) in counts.chunks_exact_mut(bins).zip(&shifts) {
                row[(k >> s) as usize & mask] += 1;
            }
        }
    }
    let mut buf = vec![a[0]; a.len()];
    let mut src: &mut [T] = a;
    let mut dst: &mut [T] = &mut buf;
    for (pass, &s) in shifts.iter().enumerate() {
        let row = if ALL {
            &mut counts[pass * bins..(pass + 1) * bins]
        } else {
            &mut counts[..]
        };
        if !ALL {
            row.fill(0);
            for &x in src.iter() {
                row[(x.key() >> s) as usize & mask] += 1;
            }
        }
        let mut sum = 0;
        for c in row.iter_mut() {
            let n = *c;
            *c = sum;
            sum += n;
        }
        for &x in src.iter() {
            let c = &mut row[(x.key() >> s) as usize & mask];
            if UNCHECKED {
                unsafe {
                    *dst.get_unchecked_mut(*c) = x;
                }
            } else {
                dst[*c] = x;
            }
            *c += 1;
        }
        std::mem::swap(&mut src, &mut dst);
    }
    if shifts.len() & 1 != 0 {
        dst.copy_from_slice(src);
    }
}

pub fn msd<T: BenchInt>(a: &mut [T]) {
    fn rec<T: BenchInt>(a: &mut [T], shift: usize) {
        if a.len() < 128 {
            a.sort_unstable();
            return;
        }
        let mut count = [0usize; 256];
        for &x in a.iter() {
            count[(x.key() >> shift) as usize & 255] += 1;
        }
        let mut start = [0usize; 256];
        let mut end = [0usize; 256];
        let mut total = 0;
        for i in 0..256 {
            start[i] = total;
            total += count[i];
            end[i] = total;
        }
        let mut next = start;
        for b in 0..256 {
            while next[b] < end[b] {
                let k = (a[next[b]].key() >> shift) as usize & 255;
                if k == b {
                    next[b] += 1;
                } else {
                    a.swap(next[b], next[k]);
                    next[k] += 1;
                }
            }
        }
        if shift > 0 {
            for b in 0..256 {
                rec(&mut a[start[b]..end[b]], shift - 8);
            }
        }
    }
    rec(a, T::BITS - 8);
}

pub fn adaptive<T: BenchInt, const B: usize, const COUNT: bool>(a: &mut [T]) {
    if a.len() <= 128 {
        a.sort_unstable();
        return;
    }
    if a.is_sorted() {
        return;
    }
    if a.windows(2).all(|w| w[0] >= w[1]) {
        a.reverse();
        return;
    }
    if COUNT {
        let mut lo = u64::MAX;
        let mut hi = 0;
        for &x in a.iter() {
            lo = lo.min(x.key());
            hi = hi.max(x.key());
        }
        let range = hi - lo;
        if range <= 65535 && range < a.len() as u64 {
            let mut counts = vec![0usize; range as usize + 1];
            for &x in a.iter() {
                counts[(x.key() - lo) as usize] += 1;
            }
            let mut pos = 0;
            for (i, c) in counts.into_iter().enumerate() {
                a[pos..pos + c].fill(T::from_key(lo + i as u64));
                pos += c;
            }
            return;
        }
    }
    lsd::<T, B, false, false>(a);
}

pub fn blocked<T: BenchInt, const BINS: usize, const BLOCK: usize>(a: &mut [T]) {
    if a.len() < 2 {
        return;
    }
    let mut buffer = vec![a[0]; a.len()];
    let mut src = a;
    let mut dst = &mut buffer[..];
    let mut passes = 0;
    for shift in (0..T::BITS).step_by(BINS.trailing_zeros() as usize) {
        let mut counts = [0usize; BINS];
        for &x in src.iter() {
            counts[(x.key() >> shift) as usize & (BINS - 1)] += 1;
        }
        let mut total = 0;
        for c in &mut counts {
            let n = *c;
            *c = total;
            total += n;
        }
        let mut blocks = [[src[0]; BLOCK]; BINS];
        let mut used = [0usize; BINS];
        for &x in src.iter() {
            let k = (x.key() >> shift) as usize & (BINS - 1);
            blocks[k][used[k]] = x;
            used[k] += 1;
            if used[k] == BLOCK {
                dst[counts[k]..counts[k] + BLOCK].copy_from_slice(&blocks[k]);
                counts[k] += BLOCK;
                used[k] = 0;
            }
        }
        for k in 0..BINS {
            dst[counts[k]..counts[k] + used[k]].copy_from_slice(&blocks[k][..used[k]]);
        }
        std::mem::swap(&mut src, &mut dst);
        passes += 1;
    }
    if passes & 1 != 0 {
        dst.copy_from_slice(src);
    }
}
