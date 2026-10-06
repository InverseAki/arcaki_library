#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Matrix {
    n: usize,
    g: Vec<MI>,
}

impl Matrix {
    #[inline]
    pub fn new(n: usize, g: Vec<MI>) -> Self {
        assert_eq!(g.len(), n * n);
        Self { n, g }
    }

    #[inline]
    pub fn zeros(n: usize) -> Self {
        Self { n, g: vec![MI::new(0); n * n] }
    }

    #[inline]
    pub fn identity(n: usize) -> Self {
        let mut res = Self::zeros(n);
        for i in 0..n {
            res[(i, i)] = MI::new(1);
        }
        res
    }

    #[inline]
    pub fn mul(&self, rhs: &Self) -> Self {
        let n = self.n;
        let mut res = vec![MI::new(0); n * n];
        for i in 0..n {
            for k in 0..n {
                let a = self[(i, k)];
                for j in 0..n {
                    res[i * n + j] += a * rhs[(k, j)];
                }
            }
        }
        Self { n, g: res }
    }

    pub fn inv(&self) -> Self {
        let n = self.n;
        let mut a = self.clone();
        let mut b = Self::identity(n);

        for col in 0..n {
            let mut pivot = col;
            while pivot < n && a[(pivot, col)].val() == 0 {
                pivot += 1;
            }
            assert!(pivot < n, "matrix is not invertible");

            if pivot != col {
                for j in 0..n {
                    a.g.swap(col * n + j, pivot * n + j);
                    b.g.swap(col * n + j, pivot * n + j);
                }
            }

            let inv_pivot = a[(col, col)].inv();
            for j in 0..n {
                a[(col, j)] *= inv_pivot;
                b[(col, j)] *= inv_pivot;
            }

            for row in 0..n {
                if row == col { continue; }
                let factor = a[(row, col)];
                if factor.val() == 0 { continue; }
                for j in 0..n {
                    a[(row, j)] -= factor * a[(col, j)];
                    b[(row, j)] -= factor * b[(col, j)];
                }
            }
        }

        b
    }
}

impl std::ops::Index<(usize, usize)> for Matrix {
    type Output = MI;
    fn index(&self, (i, j): (usize, usize)) -> &Self::Output {
        &self.g[i * self.n + j]
    }
}

impl std::ops::IndexMut<(usize, usize)> for Matrix {
    fn index_mut(&mut self, (i, j): (usize, usize)) -> &mut Self::Output {
        &mut self.g[i * self.n + j]
    }
}