/// 固定長の配列。setの旧値を保存し、指定した履歴長まで巻き戻す。
/// setは償却O(1)、rollbackは取り消す更新数に比例。領域O(n + 履歴長)。
/// 保存した履歴長は同じ配列の現在の履歴上でのみ使用する。
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

    /// 同じ値の代入も1件として記録する。T: Cloneは不要。
    pub fn set(&mut self, i: usize, value: T) {
        let old = std::mem::replace(&mut self.data[i], value);
        self.hist.push((i, old));
    }

    pub fn history_len(&self) -> usize {
        self.hist.len()
    }

    /// 現在の値を基準にして履歴を捨てる。
    pub fn clear_history(&mut self) {
        self.hist.clear();
    }

    /// 更新前に保存したhistory_len()まで戻す。
    /// lenが現在の履歴長を超える場合はpanicする。
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
