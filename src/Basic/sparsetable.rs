#[derive(Clone, Debug)]
pub struct SparseTableMI<T: Copy> {
    table: Vec<Vec<T>>,
}

impl<T: Ord + Copy> SparseTableMI<T> {
    pub fn build(a: &Vec<T>) -> Self {
        Self::build_by_key(a, |x| x)
    }

    #[inline]
    pub fn query(&self, l: usize, r: usize) -> T {
        self.query_by_key(l, r, |x| x)
    }
}

impl<T: Copy> SparseTableMI<T> {
    pub fn build_by_key<K: Ord, F: Fn(T) -> K>(a: &[T], key: F) -> Self {
        let n = a.len();
        let mut table: Vec<Vec<T>> = Vec::new();
        table.push(a.to_vec());
        let mut k = 1;
        while 1<<k <= n {
            let prev = &table[k-1];
            let len = n-(1<<k)+1;
            let mut cur = Vec::with_capacity(len);
            let w = 1<<(k-1);
            for i in 0..len {
                let (x, y) = (prev[i], prev[i+w]);
                cur.push(if key(x) <= key(y) { x } else { y });
            }
            table.push(cur);
            k += 1;
        }
        Self { table }
    }

    #[inline]
    pub fn query_by_key<K: Ord, F: Fn(T) -> K>(&self, l: usize, r: usize, key: F) -> T {
        assert!(l < r && r <= self.table[0].len());
        let s = r-l;
        let k = (usize::BITS-1-s.leading_zeros())as usize;
        let w = 1<<k;
        let (x, y) = (self.table[k][l], self.table[k][r - w]);
        if key(x) <= key(y) { x } else { y }
    }
}

#[derive(Clone, Debug)]
pub struct SparseTableMX<T: Ord + Copy> {
    table: Vec<Vec<T>>,
}

impl<T: Ord + Copy> SparseTableMX<T> {
    pub fn build(a: &Vec<T>) -> Self {
        let n = a.len();
        let mut table: Vec<Vec<T>> = Vec::new();
        table.push(a.clone());
        let mut k = 1;
        while 1<<k <= n {
            let prev = &table[k-1];
            let len = n-(1<<k)+1;
            let mut cur = Vec::with_capacity(len);
            let w = 1<<(k-1);
            for i in 0..len {
                cur.push(prev[i].max(prev[i+w]));
            }
            table.push(cur);
            k += 1;
        }
        Self { table }
    }

    pub fn query(&self, l: usize, r: usize) -> T {
        let s = r-l;
        let k = (usize::BITS-1-s.leading_zeros())as usize;
        let w = 1<<k;
        self.table[k][l].max(self.table[k][r-w])
    }
}
