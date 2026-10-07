use std::{mem::swap, collections::HashMap};

pub struct RollbackUnionFind{
    parent: Vec<i32>,
    hist: Vec<(usize, usize, i32, i32)>,
}

impl RollbackUnionFind{
    pub fn new(n: usize)->Self{
        RollbackUnionFind{
            parent: vec![-1; n],
            hist: Vec::new(),
        }
    }

    pub fn find(&self, u: usize)->usize{
        let mut res = u;
        while self.parent[res] >= 0{
            res = self.parent[res] as usize;
        }
        res
    }

    pub fn merge(&mut self, u: usize, v: usize)->bool{
        let (mut pu, mut pv) = (self.find(u), self.find(v));
        if pu==pv{
            self.hist.push((!0, !0, -1, -1));
            return false;
        }
        if self.parent[pu] > self.parent[pv]{
            swap(&mut pu, &mut pv);
        }
        self.hist.push((pu, pv, self.parent[pu], self.parent[pv]));
        self.parent[pu] += self.parent[pv];
        self.parent[pv] = pu as i32;
        true
    }

    pub fn same(&self, u: usize, v: usize) -> bool{
        self.find(u)==self.find(v)
    }

    pub fn size(&mut self, p: usize)->usize{
        (-self.parent[self.find(p)]) as usize
    }

    pub fn rollback(&mut self){
        if let Some((u, v, p1, p2)) = self.hist.pop(){
            if u < self.parent.len(){
                self.parent[u] = p1;
                self.parent[v] = p2;
            }
        }
    }
}

pub fn offline_dynamic_connectivity(n: usize, query: &Vec<(usize, usize, usize)>)->Vec<bool>{
    let mut uf = RollbackUnionFind::new(n);
    let q = query.len().next_power_of_two();
    let mut segtree = vec![Vec::new(); 2*q];
    let mut dic = HashMap::new();
    let mut num = HashMap::new();
    let mut edge = Vec::new();
    let mut question = vec![None; q];
    let mut cnt = 0;
    for (i, &(t, u, v)) in query.iter().enumerate(){
        match t{
            0 => {
                *num.entry((u, v)).or_insert(0) += 1;
                if dic.contains_key(&(u, v)){continue;}
                dic.insert((u, v), i);
            },
            1 => {
                *num.entry((u, v)).or_insert(0) -= 1;
                if num[&(u, v)] > 0{continue;}
                let z = dic[&(u, v)];
                edge.push((u, v, z, i));
                dic.remove(&(u, v));
                num.remove(&(u, v));
            },
            _ => {
                question[i] = Some((u, v));
                cnt += 1;
            }
        }
    }
    for (&(u, v), &l) in &dic{
        edge.push((u, v, l, q));
    }
    for &(u, v, mut l, mut r) in &edge{
        l += q; r += q;
        while l < r{
            if l&1==1{
                segtree[l].push((u, v));
                l += 1;
            }
            if r&1==1{
                r -= 1;
                segtree[r].push((u, v));
            }
            l >>= 1; r >>= 1;
        }
    }
    let mut ans = Vec::with_capacity(cnt);
    let mut vec = Vec::with_capacity(q/4);
    vec.push((1, true));
    while !vec.is_empty(){
        let (p, f) = vec.pop().unwrap();
        if f {
            for &(u, v) in &segtree[p]{
                uf.merge(u, v);
            }
            vec.push((p, false));
            if p < q{
                vec.push((2*p+1, true));
                vec.push((2*p, true));
            } else if let Some((u, v)) = question[p-q]{
                ans.push(uf.same(u, v));
            }
        } else {
            for _ in 0..segtree[p].len(){
                uf.rollback();
            }
        }
    }
    ans
}
