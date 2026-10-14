// Source: https://github.com/InverseAki/arcaki_library/blob/main/src/SegmentTree/segmenttree2d.rs
pub trait SegtreeMonoid{
    type S: Clone;
    fn identity()->Self::S;
    fn op(a: &Self::S, b: &Self::S)->Self::S;
}

pub struct SegmentTree2d<M> where M: SegtreeMonoid{
    h: usize,
    w: usize,
    logh: usize,
    logw: usize,
    data: Vec<Vec<M::S>>
}

impl<M> SegmentTree2d<M> where M: SegtreeMonoid{
    pub fn new(mut h: usize, mut w: usize) -> Self{
        h = h.next_power_of_two();
        w = w.next_power_of_two();
        let logh = 64-h.saturating_sub(1).leading_zeros()as usize;
        let logw = 64-w.saturating_sub(1).leading_zeros()as usize;
        let data = vec![vec![M::identity(); 2*w]; 2*h];
        SegmentTree2d{
            h, w, logh, logw, data,
        }
    }

    pub fn build(grid: &Vec<Vec<M::S>>)->Self{
        let h = grid.len().next_power_of_two();
        let w = grid[0].len().next_power_of_two();
        let logh = 64-h.saturating_sub(1).leading_zeros()as usize;
        let logw = 64-w.saturating_sub(1).leading_zeros()as usize;
        let mut data = vec![vec![M::identity(); 2*w]; 2*h];
        for (i, vc) in grid.iter().enumerate(){
            for (j, v) in vc.iter().enumerate(){
                data[i+h][j+w] = v.clone();
            }
        }
        for i in h..2*h{
            for j in (1..w).rev(){
                data[i][j] = M::op(&data[i][2*j], &data[i][2*j+1]);
            }
        }
        for i in (1..h).rev(){
            for j in 1..2*w{
                data[i][j] = M::op(&data[2*i][j], &data[2*i+1][j]);
            }
        }
        SegmentTree2d{
            h, w, logh, logw, data,
        }
    }

    pub fn get(&self, u: usize, v: usize)->M::S{
        self.data[self.h+u][self.w+v].clone()
    }

    fn get_sect(&self, mut l: usize, mut r: usize)->Vec<usize>{
        let mut res = Vec::new();
        while l < r{
            if l&1==1{
                res.push(l);
                l += 1;
            }
            if r&1==1{
                r -= 1;
                res.push(r);
            }
            l >>= 1;
            r >>= 1;
        }
        res
    }

    pub fn prod(&self, lx: usize, ly: usize, rx: usize, ry: usize)->M::S{
        let mut res = M::identity();
        let xs = self.get_sect(lx+self.h, rx+self.h);
        let ys = self.get_sect(ly+self.w, ry+self.w);
        for &x in &xs{
            for &y in &ys{
                res = M::op(&res, &self.data[x][y]);
            }
        }
        res
    }

    pub fn prod_all(&self)->M::S{
        self.data[1][1].clone()
    }

    pub fn prod_x_line_all(&self, mut p: usize)->M::S{
        p += self.h;
        self.data[p][1].clone()
    }

    pub fn prod_y_line_all(&self, mut p: usize)->M::S{
        p += self.w;
        self.data[1][p].clone()
    }

    pub fn prod_x_axis_all_sec(&self, mut l: usize, mut r: usize)->M::S{
        let mut res = M::identity();
        l += self.h; r += self.h;
        for x in self.get_sect(l, r){
            res = M::op(&res, &self.data[x][1]);
        }
        res
    }

    pub fn prod_y_axis_all_sec(&self, mut l: usize, mut r: usize)->M::S{
        let mut res = M::identity();
        l += self.w; r += self.w;
        for y in self.get_sect(l, r){
            res = M::op(&res, &self.data[1][y]);
        }
        res
    }

    pub fn set(&mut self, mut u: usize, mut v: usize, x: M::S){
        u += self.h;
        v += self.w;
        self.data[u][v] = x;
        self.update(u, v);
    }

    pub fn push(&mut self, mut u: usize, mut v: usize, x: M::S){
        u += self.h;
        v += self.w;
        self.data[u][v] = M::op(&self.data[u][v], &x);
        self.update(u, v);
    }

    fn update(&mut self, mut u: usize, mut v: usize){
        let mut ys = Vec::new();
        for _ in 1..=self.logw{
            v >>= 1;
            ys.push(v);
            self.data[u][v] = M::op(&self.data[u][v*2], &self.data[u][v*2+1]);
        }
        for _ in 1..=self.logh{
            u >>= 1;
            for &y in &ys{
                self.data[u][y] = M::op(&self.data[u*2][y], &self.data[u*2+1][y]);
            }
        }
    }
}
