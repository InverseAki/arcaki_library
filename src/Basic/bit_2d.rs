pub struct BIT2D {
    h: usize,
    xs: Vec<i32>,
    flat: Vec<i32>,
    xp: Vec<usize>,
    data: Vec<i64>,
}

impl BIT2D {
    pub fn new(add: &[(i32, i32)])->Self{
        let mut xs = Vec::new();
        for &(x, _) in add{xs.push(x);}
        xs.sort_unstable();xs.dedup();
        let h = xs.len();
        let mut ys: Vec<Vec<i32>> = vec![Vec::new(); h+1];
        let mut flat = Vec::new();
        let mut xp = vec![0,0];
        for &(x,y)in add{ys[xs.partition_point(|&z| z<x)+1].push(y);}
        for i in 1..=h {
            ys[i].sort_unstable();ys[i].dedup();
            let j = i+(i&(!i+1));
            if j <= h{
                let z = ys[i].clone();
                ys[j].extend(z);
            }
            flat.extend(take(&mut ys[i]));
            xp.push(flat.len());
        }
        let data = vec![0;flat.len()+1];
        BIT2D { h, xs, flat, xp, data }
    }

    #[inline(always)]
    fn y_add(&mut self, p: usize, v: i32, w: i64){
        let l = self.xp[p];
        let n = self.xp[p+1]-l;
        let mut p = self.flat[l..l+n].partition_point(|&y| y<v)+1;
        while p <= n {
            self.data[p+l] += w;
            p += p&(!p+1);
        }
    }

    #[inline]
    pub fn add(&mut self, u: i32, v: i32, w: i64) {
        let mut u = self.xs.partition_point(|&y| y<u)+1;
        while u <= self.h {
            self.y_add(u, v, w);
            u += u & (!u + 1);
        }
    }

    #[inline(always)]
    fn y_g(&self, p: usize, v: i32, res: &mut i64){
        let l = self.xp[p];
        let n = self.xp[p+1]-l;
        let mut p = self.flat[l..l+n].partition_point(|&y| y<v);
        while p > 0 {
            *res += self.data[p+l];
            p -= p&(!p+1);
        }
    }

    #[inline(always)]
    fn g(&self, u: i32, v: i32)->i64 {
        let mut res = 0;
        let mut u = self.xs.partition_point(|&y| y<u);
        while u > 0 {
            self.y_g(u, v, &mut res);
            u -= u&(!u+1);
        }
        res
    }

    #[inline]
    pub fn prod(&self, lx: i32, ly: i32, rx: i32, ry: i32)->i64 {
        self.g(rx, ry)+self.g(lx, ly)-self.g(lx, ry)-self.g(rx, ly)
    }
}
