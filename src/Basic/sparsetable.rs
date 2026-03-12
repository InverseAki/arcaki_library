#[derive(Clone, Debug)]
pub struct SparseTableMI<T: Ord + Copy> {
    table: Vec<Vec<T>>,
}

impl<T: Ord + Copy> SparseTableMI<T> {
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
                cur.push(prev[i].min(prev[i+w]));
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
        self.table[k][l].min(self.table[k][r - w])
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
