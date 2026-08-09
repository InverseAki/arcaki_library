pub struct CSR{
    n: usize,
    ac: Vec<usize>,
    edge: Vec<usize>,
}

impl CSR{
    pub fn new(n: usize, es: &[(usize, usize)])->Self{
        let mut ac = vec![0; n+1];
        for &(u, _) in es{
            ac[u+1] += 1;
        }
        for i in 0..n{
            ac[i+1] += ac[i];
        }
        let mut cnt = ac.clone();
        let mut edge = vec![0; ac[n]];
        for &(u, v) in es{
            edge[cnt[u]] = v;
            cnt[u] += 1;
        }
        CSR { n, ac, edge }
    }

    pub fn undirected_new(n: usize, es: &[(usize, usize)])->Self{
        let mut e = Vec::with_capacity(es.len()<<1);
        for &(u, v) in es{
            e.push((u, v));
            e.push((v, u));
        }
        Self::new(n, &e)
    }

    pub fn len(&self)->usize{
        self.n
    }

    pub fn adj(&self, idx: usize)->&[usize]{
        &self.edge[self.ac[idx]..self.ac[idx+1]]
    }
}

impl Index<usize> for CSR{
    type Output = [usize];

    fn index(&self, index: usize) -> &Self::Output {
        &self.edge[self.ac[index]..self.ac[index+1]]
    }
}

pub struct UnweightedGraph{
    n: usize,
    edge: CSR,
}

impl UnweightedGraph{
    pub fn new(n: usize, edge: &[(usize, usize)])->Self{
        UnweightedGraph{n, edge: CSR::undirected_new(n, &edge)}
    }

    pub fn bfs(&self, p: usize)->Vec<usize>{
        let mut dist = vec![!0; self.n];
        dist[p] = 0;
        let mut stack = VecDeque::new();
        stack.push_back(p);
        while let Some(p) = stack.pop_front(){
            for &nex in &self.edge[p]{
                if dist[nex]==!0{
                    dist[nex] = dist[p]+1;
                    stack.push_back(nex);
                }
            }
        }
        dist
    }

    pub fn farthest_point(&self, p: usize)->(usize, usize){
        let d = self.bfs(p);
        let (mut res, mut mx) = (p, 0);
        for i in 0..self.n{
            if d[i]!=!0 && d[i] > mx{
                mx = d[i];
                res = i;
            }
        }
        (mx, res)
    }

    pub fn n(&self)->usize{self.n}

    pub fn build_path(&self, u: usize, v: usize)->Option<Vec<usize>>{
        let dist = self.bfs(u);
        if dist[v]==!0{return None}
        let mut res = Vec::new();
        res.push(v);
        let mut p = v;
        while p != u{
            let mut nx = 0;
            for &nex in &self.edge[p]{
                if dist[nex]!=!0 && dist[nex]+1==dist[p]{
                    nx = nex;
                    break;
                }
            }
            p = nx;
            res.push(p);
        }
        res.reverse();
        Some(res)
    }

    pub fn path(&self, u: usize, v: usize)->Vec<usize>{
        let dist = self.bfs(u);
        let mut res = Vec::new();
        res.push(v);
        let mut p = v;
        while p != u{
            let mut nx = 0;
            for &nex in &self.edge[p]{
                if dist[nex]!=!0 && dist[nex]+1==dist[p]{
                    nx = nex;
                    break;
                }
            }
            p = nx;
            res.push(p);
        }
        res.reverse();
        res
    }
}
