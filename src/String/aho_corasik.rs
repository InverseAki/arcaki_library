const SIGMA: usize = 26;

pub struct AhoCorasik<const SIGMA: usize>{
    trie: Vec<[u32; SIGMA]>,
    last: Vec<usize>,
    link: Vec<u32>,
}

impl<const SIGMA: usize> AhoCorasik<SIGMA>{
    pub fn new()->Self{
        AhoCorasik { trie: vec![[!0;SIGMA]], last: vec![0], link: vec![0] }
    }

    // add→小bitなら集合で、いらないなら適当に0/1でも入れといて
    pub fn add(&mut self, arr: &[usize], id: usize){
        let mut p = 0;
        for &c in arr{
            if self.trie[p][c]==!0{
                let l = self.trie.len();
                self.trie[p][c] = l as u32;
                self.link.push(0);
                self.last.push(0);
                self.trie.push([!0;SIGMA]);
            }
            p = self.trie[p][c]as usize;
        }
        self.last[p] |= id;
    }

    pub fn build(&mut self){
        let mut stack = VecDeque::new();
        for i in 0..SIGMA{
            if self.trie[0][i]==!0{
                self.trie[0][i] = 0;
            } else {
                self.link[self.trie[0][i]as usize] = 0;
                stack.push_back(self.trie[0][i]as usize);
            }
        }
        while let Some(p) = stack.pop_front(){
            self.last[p] |= self.last[self.link[p]as usize];
            for i in 0..SIGMA{
                let v = self.trie[p][i];
                if v==!0{
                    self.trie[p][i] = self.trie[self.link[p]as usize][i];
                } else {
                    self.link[v as usize] = self.trie[self.link[p]as usize][i];
                    stack.push_back(v as usize);
                }
            }
        }
    }

    pub fn len(&self)->usize{
        self.trie.len()
    }

    pub fn next(&self, v: usize, c: usize)->usize{
        self.trie[v][c]as usize
    }

    pub fn matched(&self, v: usize)->usize{
        self.last[v]
    }
}
