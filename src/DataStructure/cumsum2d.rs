pub struct Cumsum2D<T> where T: Copy+Add<Output=T>+Sub<Output=T>+Default{
    ac: Vec<Vec<T>>,
}

impl<T> Cumsum2D<T> where T: Copy+Add<Output=T>+Sub<Output=T>+Default{
    pub fn new(data: &Vec<Vec<T>>)->Self where <T as Add>::Output: Add<T>{
        let h = data.len();
        let w = data[0].len();
        let mut ac = vec![vec![T::default(); w+1]; h+1];
        for i in 0..h{
            for j in 0..w{
                ac[i+1][j+1] = data[i][j]+ac[i+1][j]+ac[i][j+1]-ac[i][j];
            }
        }
        Cumsum2D{
            ac
        }
    }

    pub fn query(&self, lx: usize, ly: usize, rx: usize, ry: usize)->T{
        let ac = &self.ac;
        ac[rx][ry]+ac[lx][ly]-ac[lx][ry]-ac[rx][ly]
    }
}
