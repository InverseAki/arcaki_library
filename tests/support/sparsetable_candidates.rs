use std::mem::MaybeUninit;

#[derive(Clone, Debug)]
pub struct Nested<T> {
    pub table: Vec<Vec<T>>,
}
impl<T: Copy> Nested<T> {
    pub fn build(a: Vec<T>, select: impl Fn(T, T) -> T) -> Self {
        let n = a.len();
        let mut table = vec![a];
        let mut w = 1;
        while w <= n / 2 {
            let len = n - 2 * w + 1;
            let prev = table.last().unwrap();
            table.push(
                prev[..len]
                    .iter()
                    .zip(&prev[w..w + len])
                    .map(|(&x, &y)| select(x, y))
                    .collect(),
            );
            w *= 2;
        }
        Self { table }
    }
    #[inline(always)]
    pub fn query<const UNCHECKED: bool>(
        &self,
        l: usize,
        r: usize,
        select: impl Fn(T, T) -> T,
    ) -> T {
        assert!(l < r && r <= self.table[0].len());
        let k = (usize::BITS - 1 - (r - l).leading_zeros()) as usize;
        let w = 1 << k;
        if UNCHECKED {
            unsafe {
                let row = self.table.get_unchecked(k);
                select(*row.get_unchecked(l), *row.get_unchecked(r - w))
            }
        } else {
            select(self.table[k][l], self.table[k][r - w])
        }
    }
}

#[derive(Clone, Debug)]
pub struct Flat<T, const FORMULA: bool, const INITIALIZED: bool> {
    pub data: Vec<T>,
    pub offsets: Vec<usize>,
    pub n: usize,
}
impl<T: Copy, const FORMULA: bool, const INITIALIZED: bool> Flat<T, FORMULA, INITIALIZED> {
    pub fn build(mut data: Vec<T>, select: impl Fn(T, T) -> T) -> Self {
        let n = data.len();
        let levels = (usize::BITS - n.leading_zeros()) as usize;
        let mut offsets = Vec::with_capacity(levels);
        let mut total = 0usize;
        for k in 0..levels {
            offsets.push(total);
            total = total.checked_add(n - (1 << k) + 1).unwrap();
        }
        data.reserve_exact(total - n);
        if n > 0 && INITIALIZED {
            data.resize(total, data[0]);
        }
        for k in 1..levels {
            let w = 1 << (k - 1);
            let len = n - 2 * w + 1;
            let base = offsets[k];
            let prev = offsets[k - 1];
            if INITIALIZED {
                let (src, dst) = data.split_at_mut(base);
                for (out, (&x, &y)) in dst[..len].iter_mut().zip(
                    src[prev..prev + len]
                        .iter()
                        .zip(&src[prev + w..prev + w + len]),
                ) {
                    *out = select(x, y);
                }
            } else {
                unsafe {
                    let ptr = data.as_mut_ptr();
                    let src = std::slice::from_raw_parts(ptr.add(prev), len + w);
                    let dst =
                        std::slice::from_raw_parts_mut(ptr.add(base).cast::<MaybeUninit<T>>(), len);
                    for (out, (&x, &y)) in dst.iter_mut().zip(src[..len].iter().zip(&src[w..])) {
                        out.write(select(x, y));
                    }
                    data.set_len(base + len);
                }
            }
        }
        Self { data, offsets, n }
    }
    #[inline(always)]
    pub fn query<const UNCHECKED: bool>(
        &self,
        l: usize,
        r: usize,
        select: impl Fn(T, T) -> T,
    ) -> T {
        assert!(l < r && r <= self.n);
        let k = (usize::BITS - 1 - (r - l).leading_zeros()) as usize;
        let w = 1 << k;
        let base = if FORMULA {
            k * (self.n + 1) + 1 - w
        } else if UNCHECKED {
            unsafe { *self.offsets.get_unchecked(k) }
        } else {
            self.offsets[k]
        };
        if UNCHECKED {
            unsafe {
                select(
                    *self.data.get_unchecked(base + l),
                    *self.data.get_unchecked(base + r - w),
                )
            }
        } else {
            select(self.data[base + l], self.data[base + r - w])
        }
    }
}

#[derive(Clone, Debug)]
pub struct Block<T, const B: usize> {
    pub values: Vec<T>,
    pub masks: Vec<u64>,
    pub blocks: Flat<T, true, false>,
}
impl<T: Copy, const B: usize> Block<T, B> {
    pub fn build(values: Vec<T>, key: impl Fn(T) -> u64, select: impl Fn(T, T) -> T) -> Self {
        let n = values.len();
        let mut masks = Vec::with_capacity(n);
        let mut mins = Vec::with_capacity(n.div_ceil(B));
        let mut mask = 0u64;
        for i in 0..n {
            let local = i % B;
            if local == 0 {
                mask = 0;
            }
            while mask != 0 {
                let j = 63 - mask.leading_zeros() as usize;
                if key(values[i - local + j]) <= key(values[i]) {
                    break;
                }
                mask ^= 1 << j;
            }
            mask |= 1 << local;
            masks.push(mask);
            if local == B - 1 || i + 1 == n {
                mins.push(values[i - local + mask.trailing_zeros() as usize]);
            }
        }
        let blocks = Flat::build(mins, select);
        Self {
            values,
            masks,
            blocks,
        }
    }
    #[inline(always)]
    pub fn query(&self, l: usize, r: usize, select: impl Fn(T, T) -> T) -> T {
        assert!(l < r && r <= self.values.len());
        let lb = l / B;
        let rb = (r - 1) / B;
        unsafe {
            if lb == rb {
                let mask = *self.masks.get_unchecked(r - 1) & (!0u64 << (l % B));
                return *self
                    .values
                    .get_unchecked(lb * B + mask.trailing_zeros() as usize);
            }
            let left = *self.masks.get_unchecked((lb + 1) * B - 1) & (!0u64 << (l % B));
            let right = *self.masks.get_unchecked(r - 1);
            let x = *self
                .values
                .get_unchecked(lb * B + left.trailing_zeros() as usize);
            let y = *self
                .values
                .get_unchecked(rb * B + right.trailing_zeros() as usize);
            if lb + 1 < rb {
                select(x, select(self.blocks.query::<true>(lb + 1, rb, &select), y))
            } else {
                select(x, y)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct BoxRows<T> {
    pub rows: Vec<Box<[T]>>,
    pub n: usize,
}
impl<T: Copy> BoxRows<T> {
    pub fn build(a: Vec<T>, select: impl Fn(T, T) -> T) -> Self {
        let n = a.len();
        let mut rows = Vec::with_capacity((usize::BITS - n.leading_zeros()) as usize);
        rows.push(a.into_boxed_slice());
        let mut w = 1;
        while w <= n / 2 {
            let len = n - 2 * w + 1;
            let prev = rows.last().unwrap();
            rows.push(
                prev[..len]
                    .iter()
                    .zip(&prev[w..w + len])
                    .map(|(&x, &y)| select(x, y))
                    .collect(),
            );
            w *= 2;
        }
        Self { rows, n }
    }
    #[inline(always)]
    pub fn query(&self, l: usize, r: usize, select: impl Fn(T, T) -> T) -> T {
        assert!(l < r && r <= self.n);
        let k = (usize::BITS - 1 - (r - l).leading_zeros()) as usize;
        unsafe {
            let row = self.rows.get_unchecked(k);
            select(*row.get_unchecked(l), *row.get_unchecked(r - (1 << k)))
        }
    }
}

#[derive(Clone, Debug)]
pub struct Stride<T> {
    pub data: Vec<T>,
    pub n: usize,
    pub stride: usize,
}
impl<T: Copy> Stride<T> {
    pub fn build(mut data: Vec<T>, select: impl Fn(T, T) -> T) -> Self {
        let n = data.len();
        let levels = (usize::BITS - n.leading_zeros()) as usize;
        let stride = n;
        let total = n * levels;
        data.reserve_exact(total - n);
        for k in 1..levels {
            let w = 1 << (k - 1);
            let len = n - 2 * w + 1;
            unsafe {
                let ptr = data.as_mut_ptr();
                let src = std::slice::from_raw_parts(ptr.add((k - 1) * stride), len + w);
                let dst = std::slice::from_raw_parts_mut(
                    ptr.add(k * stride).cast::<MaybeUninit<T>>(),
                    stride,
                );
                for (out, (&x, &y)) in dst[..len].iter_mut().zip(src[..len].iter().zip(&src[w..])) {
                    out.write(select(x, y));
                }
                for out in &mut dst[len..] {
                    out.write(src[0]);
                }
                data.set_len((k + 1) * stride);
            }
        }
        Self { data, n, stride }
    }
    #[inline(always)]
    pub fn query(&self, l: usize, r: usize, select: impl Fn(T, T) -> T) -> T {
        assert!(l < r && r <= self.n);
        let k = (usize::BITS - 1 - (r - l).leading_zeros()) as usize;
        let base = k * self.stride;
        unsafe {
            select(
                *self.data.get_unchecked(base + l),
                *self.data.get_unchecked(base + r - (1 << k)),
            )
        }
    }
}

#[derive(Clone, Debug)]
pub struct PrefixSuffix<T, const B: usize> {
    values: Vec<T>,
    prefix: Vec<T>,
    suffix: Vec<T>,
    blocks: Flat<T, true, false>,
}
impl<T: Copy, const B: usize> PrefixSuffix<T, B> {
    pub fn build(values: Vec<T>, select: impl Fn(T, T) -> T) -> Self {
        let n = values.len();
        let mut prefix = Vec::with_capacity(n);
        let mut suffix = values.clone();
        let mut mins = Vec::with_capacity(n.div_ceil(B));
        for (bi, chunk) in values.chunks(B).enumerate() {
            let mut p = chunk[0];
            for &x in chunk {
                p = select(p, x);
                prefix.push(p);
            }
            let mut s = chunk[chunk.len() - 1];
            for (i, &x) in chunk.iter().enumerate().rev() {
                s = select(x, s);
                suffix[bi * B + i] = s;
            }
            mins.push(p);
        }
        let blocks = Flat::build(mins, select);
        Self {
            values,
            prefix,
            suffix,
            blocks,
        }
    }
    #[inline(always)]
    pub fn query(&self, l: usize, r: usize, select: impl Fn(T, T) -> T) -> T {
        assert!(l < r && r <= self.values.len());
        let lb = l / B;
        let rb = (r - 1) / B;
        unsafe {
            if lb == rb {
                let mut x = *self.values.get_unchecked(l);
                for i in l + 1..r {
                    x = select(x, *self.values.get_unchecked(i));
                }
                return x;
            }
            let x = *self.suffix.get_unchecked(l);
            let y = *self.prefix.get_unchecked(r - 1);
            if lb + 1 < rb {
                select(x, select(self.blocks.query::<true>(lb + 1, rb, &select), y))
            } else {
                select(x, y)
            }
        }
    }
}
