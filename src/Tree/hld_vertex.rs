pub struct HLDVertex {
    parent: Vec<u32>,
    head: Vec<u32>,
    depth: Vec<u32>,
    int: Vec<u32>,
    out: Vec<u32>,
    seq: Vec<u32>,
}

impl HLDVertex {
    pub fn new(r: usize, edge: &mut UnweightedGraph)->Self{
        HLDVertex::size_dfs(r, !0, edge);
        let mut int: Vec<u32> = vec![0; n];
        let mut seq: Vec<u32> = Vec::with_capacity(n);
        let mut out: Vec<u32> = vec![0; n];
        let mut parent: Vec<u32> = vec![!0; n];
        let mut head: Vec<u32> = vec![0; n];
        let mut depth: Vec<u32> = vec![0; n];
        HLDVertex::euler_tour(r, !0, r, &edge, &mut parent, &mut depth, &mut head, &mut int, &mut out, &mut seq);
        HLDVertex { parent, head, depth, int, out, seq,}
    }

    #[inline]
    fn size_dfs(p: usize, pre: usize, edge: &mut UnweightedGraph)->usize{
        let mut res = 1;
        let l = (&edge[p]).len();
        let mut mx = 0;
        let mut rv = 0;
        for i in 0..l{
            let nex = edge[p][i];
            if nex==pre{continue;}
            let nn = HLDVertex::size_dfs(nex, p, edge);
            if mx < nn {
                mx = nn;
                rv = i;
            }
            res += nn;
        }
        if rv!=0{edge.edge[p].swap(0, rv)};
        res
    }

    #[inline]
    fn euler_tour(p: usize, pre: usize, h: usize, edge: &UnweightedGraph, parent: &mut [u32], depth: &mut [u32], head: &mut [u32], int: &mut [u32], end: &mut [u32], seq: &mut Vec<u32>){
        int[p] = seq.len()as u32;
        seq.push(p as u32);
        head[p] = h as u32;
        if edge[p].is_empty() || edge[p][0]==pre{
            end[p] = seq.len()as u32;
            return;
        }
        let hc = edge[p][0];
        depth[hc] = depth[p]+1;
        parent[hc]=p as u32;
        HLDVertex::euler_tour(hc, p, h, edge, parent, depth, head, int, end, seq);
        for &nex in &edge[p][1..]{
            if nex==pre{continue;}
            depth[nex] = depth[p]+1;
            parent[nex]=p as u32;
            HLDVertex::euler_tour(nex, p, nex, edge, parent, depth, head, int, end, seq);
        }
        end[p] = seq.len() as u32;
    }

    #[inline]
    pub fn parent_up_k(&self, mut p: usize, mut k: usize)->usize{
        if (self.depth[p]as usize) < k {return !0;}
        while self.int[p]as usize-self.int[self.head[p]as usize]as usize+1 <= k{
            k -= self.int[p]as usize-self.int[self.head[p]as usize]as usize+1;
            p = self.parent[self.head[p]as usize]as usize;
        }
        self.seq[self.int[p]as usize-k]as usize
    }

    #[inline]
    pub fn parent_at_k(&self, p: usize, k: usize)->usize{
        if (self.depth[p]as usize) < k{return !0;}
        self.parent_up_k(p, self.depth[p]as usize-k)
    }

    #[inline]
    pub fn lca(&self, mut u: usize, mut v: usize)->usize{
        while self.head[u]!=self.head[v]{
            if self.int[self.head[u]as usize]>self.int[self.head[v]as usize]{
                swap(&mut u, &mut v);
            }
            v = self.parent[self.head[v]as usize]as usize;
        }
        if self.int[u]>self.int[v]{v} else {u}
    }

    #[inline]
    pub fn subtree(&self, p: usize)->(usize, usize){
        (self.int[p]as usize, self.out[p]as usize)
    }

    #[inline]
    pub fn distance(&self, u: usize, v: usize)->usize{
        let p = self.lca(u, v);
        (self.depth[u]+self.depth[v]-self.depth[p]*2)as usize
    }

    #[inline]
    pub fn jump_k(&self, u: usize, v: usize, k: usize)->usize{        
        let p = self.lca(u, v);
        let du = (self.depth[u]-self.depth[p])as usize;
        let dv = (self.depth[v]-self.depth[p])as usize;
        let d = du+dv;
        if d < k{return !0;}
        if k <= du {
            self.parent_up_k(u, k)
        } else {
            self.parent_up_k(v, d-k)
        }
    }

    pub fn all_path(&self, mut u: usize, mut v: usize)->Vec<usize>{
        let mut res = vec![u];
        let p = self.lca(u, v);
        while u!=p{
            u = self.parent[u]as usize;
            res.push(u);
        }
        let mut rev = Vec::new();
        while v!=p{
            rev.push(v);
            v = self.parent[v]as usize;
        }
        rev.reverse();
        res.extend(rev);
        res
    }

    // (left, right, rev: bool)
    #[inline]
    pub fn path(&self, mut u: usize, mut v: usize)->Vec<(usize, usize, bool)> {
        let mut res = Vec::new();
        let mut rev = Vec::new();
        while self.head[u]!=self.head[v]{
            let hu = self.head[u]as usize;
            let hv = self.head[v]as usize;
            if self.depth[hu] >= self.depth[hv] {
                res.push((self.int[hu]as usize, self.int[u]as usize+1, true));
                u=self.parent[hu]as usize;
            } else {
                rev.push((self.int[hv]as usize, self.int[v]as usize+1, false));
                v=self.parent[hv]as usize;
            }
        }
        if self.depth[u]<=self.depth[v]{
            rev.push((self.int[u]as usize, self.int[v]as usize+1, false));
        } else {
            res.push((self.int[v]as usize, self.int[u]as usize+1, true));
        }
        rev.reverse();
        res.extend(rev);
        res
    }

    #[inline]
    pub fn get_index(&self, p: usize)->usize{
        self.int[p]as usize
    }
}
