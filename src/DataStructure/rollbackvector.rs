pub struct RollbackVector<T> {
    data: Vec<T>,
    hist: Vec<(usize, T)>,
}

impl<T> RollbackVector<T> {
    pub fn from_vec(data: Vec<T>) -> Self {
        Self {
            data,
            hist: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn get(&self, i: usize) -> &T {
        &self.data[i]
    }

    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    pub fn set(&mut self, i: usize, value: T) {
        let old = std::mem::replace(&mut self.data[i], value);
        self.hist.push((i, old));
    }

    pub fn history_len(&self) -> usize {
        self.hist.len()
    }

    pub fn clear_history(&mut self) {
        self.hist.clear();
    }

    pub fn rollback(&mut self, len: usize) {
        assert!(len <= self.hist.len());
        while self.hist.len() > len {
            let (i, old) = self.hist.pop().unwrap();
            self.data[i] = old;
        }
    }
}

impl<T: Clone> RollbackVector<T> {
    pub fn new(n: usize, value: T) -> Self {
        Self::from_vec(vec![value; n])
    }
}

impl<T> std::ops::Index<usize> for RollbackVector<T> {
    type Output = T;

    fn index(&self, i: usize) -> &T {
        self.get(i)
    }
}
