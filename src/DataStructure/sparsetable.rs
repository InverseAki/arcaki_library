// Source: https://github.com/InverseAki/arcaki_library/blob/main/src/DataStructure/sparsetable.rs
#[derive(Clone, Debug)]
pub struct SparseTableMI<T: Copy> {
    data: SparseTableData<T>,
}

impl<T: Ord + Copy> SparseTableMI<T> {
    pub fn build(a: Vec<T>) -> Self {
        Self::build_by_key(a, |x| x)
    }

    #[inline]
    pub fn query(&self, l: usize, r: usize) -> T {
        self.query_by_key(l, r, |x| x)
    }
}

impl<T: Copy> SparseTableMI<T> {
    pub fn build_by_key<K: Ord, F: Fn(T) -> K>(a: Vec<T>, key: F) -> Self {
        Self {
            data: SparseTableData::build(a, |x, y| if key(x) <= key(y) { x } else { y }),
        }
    }

    #[inline]
    pub fn query_by_key<K: Ord, F: Fn(T) -> K>(&self, l: usize, r: usize, key: F) -> T {
        self.data
            .query(l, r, |x, y| if key(x) <= key(y) { x } else { y })
    }
}

#[derive(Clone, Debug)]
pub struct SparseTableMX<T: Ord + Copy> {
    data: SparseTableData<T>,
}

impl<T: Ord + Copy> SparseTableMX<T> {
    pub fn build(a: Vec<T>) -> Self {
        Self {
            data: SparseTableData::build(a, |x, y| x.max(y)),
        }
    }

    #[inline]
    pub fn query(&self, l: usize, r: usize) -> T {
        self.data.query(l, r, |x, y| x.max(y))
    }
}

#[derive(Clone, Debug)]
enum SparseTableData<T: Copy> {
    Direct {
        rows: Vec<Vec<T>>,
        n: usize,
    },
    Blocked {
        values: Vec<T>,
        prefix: Vec<T>,
        suffix: Vec<T>,
        table: Vec<T>,
        blocks: usize,
    },
}

impl<T: Copy> SparseTableData<T> {
    fn build(values: Vec<T>, select: impl Fn(T, T) -> T) -> Self {
        let n = values.len();
        let levels = (usize::BITS - n.leading_zeros()) as usize;
        let bytes = (0..levels)
            .try_fold(0usize, |sum, k| sum.checked_add(n - (1 << k) + 1))
            .and_then(|cells| cells.checked_mul(std::mem::size_of::<T>()));
        if bytes.unwrap_or(usize::MAX) > 16 * 1024 * 1024 {
            return Self::build_blocked(values, select);
        }
        let mut rows = Vec::with_capacity(levels.max(1));
        rows.push(values);
        for k in 1..levels {
            let w = 1 << (k - 1);
            let len = n - 2 * w + 1;
            let prev = &rows[k - 1];
            let cur = prev[..len]
                .iter()
                .zip(&prev[w..w + len])
                .map(|(&x, &y)| select(x, y))
                .collect();
            rows.push(cur);
        }
        Self::Direct { rows, n }
    }

    fn build_blocked(values: Vec<T>, select: impl Fn(T, T) -> T) -> Self {
        let n = values.len();
        let blocks = n.div_ceil(16);
        let levels = (usize::BITS - blocks.leading_zeros()) as usize;
        let cells = (0..levels)
            .try_fold(0usize, |sum, k| sum.checked_add(blocks - (1 << k) + 1))
            .expect("sparse table size overflow");
        let mut prefix = Vec::with_capacity(n);
        let mut suffix = values.clone();
        let mut table = Vec::with_capacity(cells);
        for (b, chunk) in values.chunks(16).enumerate() {
            let mut p = chunk[0];
            prefix.push(p);
            for &x in &chunk[1..] {
                p = select(p, x);
                prefix.push(p);
            }
            table.push(p);
            let mut s = chunk[chunk.len() - 1];
            for i in (0..chunk.len() - 1).rev() {
                s = select(chunk[i], s);
                suffix[b * 16 + i] = s;
            }
        }
        if blocks > 0 {
            table.resize(cells, table[0]);
        }
        let (mut prev, mut base, mut w) = (0, blocks, 1);
        while w <= blocks / 2 {
            let len = blocks - 2 * w + 1;
            let (src, dst) = table.split_at_mut(base);
            for (out, (&x, &y)) in dst[..len].iter_mut().zip(
                src[prev..prev + len]
                    .iter()
                    .zip(&src[prev + w..prev + w + len]),
            ) {
                *out = select(x, y);
            }
            prev = base;
            base += len;
            w *= 2;
        }
        Self::Blocked {
            values,
            prefix,
            suffix,
            table,
            blocks,
        }
    }

    #[inline(always)]
    fn query(&self, l: usize, r: usize, select: impl Fn(T, T) -> T) -> T {
        match self {
            Self::Direct { rows, n } => {
                assert!(l < r && r <= *n);
                let k = (usize::BITS - 1 - (r - l).leading_zeros()) as usize;
                // The validated interval fits both precomputed length-2^k ranges.
                unsafe {
                    let row = rows.get_unchecked(k);
                    select(*row.get_unchecked(l), *row.get_unchecked(r - (1 << k)))
                }
            }
            Self::Blocked {
                values,
                prefix,
                suffix,
                table,
                blocks,
            } => {
                assert!(l < r && r <= values.len());
                let left = l >> 4;
                let right = (r - 1) >> 4;
                // All accesses stay in the validated interval or in fully covered blocks.
                unsafe {
                    if left == right {
                        let values = values.get_unchecked(l..r);
                        let mut best = values[0];
                        for &x in &values[1..] {
                            best = select(best, x);
                        }
                        return best;
                    }
                    let x = *suffix.get_unchecked(l);
                    let y = *prefix.get_unchecked(r - 1);
                    if left + 1 == right {
                        return select(x, y);
                    }
                    let l = left + 1;
                    let r = right;
                    let k = (usize::BITS - 1 - (r - l).leading_zeros()) as usize;
                    let w = 1 << k;
                    let base = k.wrapping_mul(*blocks + 1).wrapping_sub(w - 1);
                    let middle = select(
                        *table.get_unchecked(base + l),
                        *table.get_unchecked(base + (r - w)),
                    );
                    select(x, select(middle, y))
                }
            }
        }
    }
}
