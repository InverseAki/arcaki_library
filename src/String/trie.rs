const SIGMA: usize = 26;

pub struct Trie <const SIGMA: usize>{
    node: Vec<[usize;SIGMA]>,
    cnt: Vec<usize>,
}

impl<const SIGMA: usize> Trie<SIGMA>{
    pub fn new()->Self{
        Trie{
            node: vec![[!0;SIGMA]],
            cnt: vec![0],
        }
    }

    pub fn add(&mut self, arr: &[usize])->usize{
        let mut p = 0;
        self.cnt[p] += 1;
        for &v in arr{
            if self.node[p][v]==!0{
                let l = self.node.len();
                self.node[p][v] = l;
                self.node.push([!0; SIGMA]);
                self.cnt.push(0);
            }
            p = self.node[p][v];
            self.cnt[p] += 1;
        }
        p
    }

    pub fn next(&self, p: usize, c: usize)->usize{
        self.node[p][c]
    }

    pub fn nexts(&self, p: usize)->&[usize;SIGMA]{
        &self.node[p]
    }

    pub fn cnt(&self, p: usize)->usize{
        self.cnt[p]
    }

    pub fn len(&self)->usize{
        self.node.len()
    }
}
