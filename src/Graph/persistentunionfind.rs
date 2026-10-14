// Source: https://github.com/InverseAki/arcaki_library/blob/main/src/Graph/persistentunionfind.rs
#[derive(Clone, Debug)]
pub struct Node<T: Clone> {
    val: T,
    left: u32,
    right: u32,
}

#[derive(Debug)]
pub struct PersistentArray<T: Clone> {
    n: usize,
    data: Vec<Node<T>>,
    root: Vec<u32>,
    default: T,
}

impl<T: Clone> PersistentArray<T> {
    pub fn new(n: usize, default: T) -> Self {
        let n2 = n.next_power_of_two();
        let mut pa = Self {
            n: n2,
            data: Vec::with_capacity(2 * n2),
            root: Vec::new(),
            default,
        };
        let r = pa.init(0, n2);
        pa.root.push(r as u32);
        pa
    }

    pub fn build(a: &[T], default: T) -> Self {
        let n2 = a.len().next_power_of_two();
        let mut pa = Self {
            n: n2,
            data: Vec::with_capacity(2 * n2),
            root: Vec::new(),
            default,
        };
        let r = pa.init_from(a, 0, n2);
        pa.root.push(r as u32);
        pa
    }

    #[inline(always)]
    fn push(&mut self, node: Node<T>) -> usize {
        let idx = self.data.len();
        self.data.push(node);
        idx
    }

    fn init(&mut self, l: usize, r: usize) -> usize {
        if l + 1 == r {
            return self.push(Node { val: self.default.clone(), left: !0, right: !0 });
        }
        let m = (l + r) >> 1;
        let left = self.init(l, m);
        let right = self.init(m, r);
        self.push(Node { val: self.default.clone(), left: left as u32, right: right as u32 })
    }

    fn init_from(&mut self, a: &[T], l: usize, r: usize) -> usize {
        if l + 1 == r {
            let v = if l < a.len() { a[l].clone() } else { self.default.clone() };
            return self.push(Node { val: v, left: !0, right: !0 });
        }
        let m = (l + r) >> 1;
        let left = self.init_from(a, l, m);
        let right = self.init_from(a, m, r);
        self.push(Node { val: self.default.clone(), left: left as u32, right: right as u32 })
    }

    #[inline]
    pub fn versions(&self) -> usize {
        self.root.len()
    }

    #[inline]
    pub fn set(&mut self, t: usize, p: usize, x: T) {
        assert!(p < self.n);
        let nr = self.set_dfs(self.root[t] as usize, 0, self.n, p, &x);
        self.root.push(nr as u32);
    }

    fn set_dfs(&mut self, cur: usize, l: usize, r: usize, p: usize, x: &T) -> usize {
        if l + 1 == r {
            return self.push(Node { val: x.clone(), left: !0, right: !0 });
        }
        let m = (l + r) >> 1;
        let pre = &self.data[cur];
        let (cl, cr) = (pre.left, pre.right);

        if p < m {
            let nl = self.set_dfs(cl as usize, l, m, p, x) as u32;
            self.push(Node { val: self.default.clone(), left: nl, right: cr })
        } else {
            let nr = self.set_dfs(cr as usize, m, r, p, x) as u32;
            self.push(Node { val: self.default.clone(), left: cl, right: nr })
        }
    }

    #[inline]
    pub fn get(&self, t: usize, p: usize) -> T {
        assert!(p < self.n);
        self.get_dfs(self.root[t] as usize, 0, self.n, p)
    }

    fn get_dfs(&self, cur: usize, l: usize, r: usize, p: usize) -> T {
        if l + 1 == r {
            return self.data[cur].val.clone();
        }
        let m = (l + r) >> 1;
        let node = &self.data[cur];
        if p < m {
            self.get_dfs(node.left as usize, l, m, p)
        } else {
            self.get_dfs(node.right as usize, m, r, p)
        }
    }
}

pub struct PersistentUnionFind{
    parent: PersistentArray<i32>,
    hist: Vec<usize>,
}

impl PersistentUnionFind{
    pub fn new(n: usize) -> Self {
        let parent = PersistentArray::new(n, -1);
        Self {
            parent,
            hist: vec![0],
        }
    }

    pub fn find(&self, t: usize, p: usize)->usize{
        let t = self.hist[t];
        self._find(t, p)
    }

    fn _find(&self, t: usize, p: usize)->usize{
        if self.parent.get(t, p) < 0{return p;}
        self._find(t, self.parent.get(t, p)as usize)
    }

    pub fn union(&mut self, t: usize, u: usize, v: usize){
        let (mut pu, mut pv) = (self.find(t, u), self.find(t, v));
        if pu != pv{
            let t = self.hist[t];
            let (mut nu, mut nv) = (self.parent.get(t,pu), self.parent.get(t, pv));
            if nu > nv{
                swap(&mut pu, &mut pv);
                swap(&mut nu, &mut nv);
            }
            self.parent.set(t, pu, nu+nv);
            let p = self.parent.versions();
            self.parent.set(p-1, pv, pu as i32);
            self.hist.push(self.parent.versions()-1);
        } else {
            self.hist.push(self.hist[t]);
        }
    }

    pub fn same(&mut self, t: usize, u: usize, v: usize)->bool{
        self.hist.push(self.parent.versions()-1);
        self.find(t, u)==self.find(t, v)
    }
}
