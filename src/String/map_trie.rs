pub struct Trie{
    node: Vec<FxMap<usize, usize>>,
    cnt: Vec<usize>,
}

impl Trie{
    pub fn new()->Self{
        Trie{
            node: vec![FxMap::default()],
            cnt: vec![0],
        }
    }

    pub fn add(&mut self, arr: &[usize])->usize{
        let mut p = 0;
        self.cnt[p] += 1;
        for &v in arr{
            let to = if let Some(&nex) = self.node[p].get(&v){nex}else{
                let l = self.node.len();
                self.node[p].insert(v, l);
                self.node.push(FxMap::default());
                self.cnt.push(0);
                l
            };
            p = to;
            self.cnt[p] += 1;
        }
        p
    }

    pub fn next(&self, p: usize, c: usize)->usize{
        *self.node[p].get(&c).unwrap_or(&!0)
    }

    pub fn nexts(&self, p: usize)->&FxMap<usize, usize>{
        &self.node[p]
    }

    pub fn cnt(&self, p: usize)->usize{
        self.cnt[p]
    }

    pub fn len(&self)->usize{
        self.node.len()
    }
}
