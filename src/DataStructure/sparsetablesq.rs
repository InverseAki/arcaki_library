const INF: i64 = 1<<60;

pub fn bit_length(x: usize)->usize{
    64-x.leading_zeros() as usize
}

pub struct SparseTable2D{
    table: Vec<Vec<Vec<Vec<i64>>>>,
}

impl SparseTable2D{
    pub fn new(grid: Vec<Vec<i64>>)->Self{
        let h = grid.len();
        let w = grid[0].len();
        let bit1 = bit_length(h);
        let bit2 = bit_length(w);
        let mut table = vec![vec![vec![vec![-INF; w]; h]; bit2]; bit1];
        table[0][0] = grid.clone();
        for bit in 0..bit2-1{
            for i in 0..h{
                for j in 0..=w-(1<<(bit+1)) {
                    table[0][bit+1][i][j] = table[0][bit][i][j].max(table[0][bit][i][j+(1<<bit)]);
                }
            }
        }
        for b1 in 0..bit1-1{
            for b2 in 0..bit2{
                for i in 0..=h-(1<<(b1+1)){
                    for j in 0..=w-(1<<b2){
                        table[b1+1][b2][i][j] = table[b1][b2][i][j].max(table[b1][b2][i+(1<<b1)][j]);
                    }
                }
            }
        }
        SparseTable2D{
            table,
        }
    }

    pub fn query(&self, s: usize, t: usize, u: usize, v: usize)->i64{
        let p = bit_length(u-s)-1;
        let q = bit_length(v-t)-1;
        let x = u-(1<<p);
        let y = v-(1<<q);
        let st = &self.table[p][q];
        st[s][t].max(st[s][y]).max(st[x][t]).max(st[x][y])
    }
}