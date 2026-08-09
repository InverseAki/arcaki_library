pub trait SegtreeMonoid {
    type S: Clone;

    fn identity() -> Self::S;

    fn len(len: usize) -> Self::S;

    fn op(a: &Self::S, b: &Self::S) -> Self::S;
}

pub trait LazySegtreeMonoid: SegtreeMonoid {
    type F: Clone;

    fn id_e() -> Self::F;

    fn mapping(f: &Self::F, x: &Self::S, len: usize) -> Self::S;

    fn composition(f: &Self::F, g: &Self::F) -> Self::F;
}

const NIL: u32 = u32::MAX;

#[derive(Debug)]
pub struct LazySegtreeNode<MM>
where
    MM: LazySegtreeMonoid,
{
    val: MM::S,
    lazy: MM::F,
    len: u32,
    left: u32,
    right: u32,
}

impl<MM> Clone for LazySegtreeNode<MM>
where
    MM: LazySegtreeMonoid,
{
    fn clone(&self) -> Self {
        Self {
            val: self.val.clone(),
            lazy: self.lazy.clone(),
            len: self.len,
            left: self.left,
            right: self.right,
        }
    }
}

impl<MM> LazySegtreeNode<MM>
where
    MM: LazySegtreeMonoid,
{
    pub fn identity() -> Self {
        Self {
            val: MM::identity(),
            lazy: MM::id_e(),
            len: 0,
            left: NIL,
            right: NIL,
        }
    }

    pub fn len(len: usize) -> Self {
        Self {
            val: MM::len(len),
            lazy: MM::id_e(),
            len: len as u32,
            left: NIL,
            right: NIL,
        }
    }

    fn empty_segment(len: usize) -> Self {
        Self {
            val: MM::identity(),
            lazy: MM::id_e(),
            len: len as u32,
            left: NIL,
            right: NIL,
        }
    }

    pub fn new(val: MM::S, lazy: MM::F, len: u32, left: u32, right: u32) -> Self {
        Self {
            val,
            lazy,
            len,
            left,
            right,
        }
    }
}

pub struct PersistentLazySegtree<MM>
where
    MM: LazySegtreeMonoid,
{
    /// 外から見える本来の長さ
    size: usize,

    /// 内部の 2 冪サイズ
    n: usize,

    root: Vec<u32>,
    data: Vec<LazySegtreeNode<MM>>,
}

impl<MM> PersistentLazySegtree<MM>
where
    MM: LazySegtreeMonoid,
{
    pub fn new(size: usize) -> Self {
        assert!(size > 0);

        let n = size.next_power_of_two();

        let mut data = Vec::new();
        let r = Self::build_default_dfs(0, n, size, &mut data);

        Self {
            size,
            n,
            root: vec![r],
            data,
        }
    }

    pub fn build(base: Vec<MM::S>) -> Self {
        assert!(!base.is_empty());

        let size = base.len();
        let n = size.next_power_of_two();

        let mut data = Vec::with_capacity(2 * n);
        let r = Self::build_dfs(0, n, &base, &mut data);

        Self {
            size,
            n,
            root: vec![r],
            data,
        }
    }

    fn build_default_dfs(
        l: usize,
        r: usize,
        size: usize,
        data: &mut Vec<LazySegtreeNode<MM>>,
    ) -> u32 {
        if size <= l {
            let idx = data.len() as u32;
            data.push(LazySegtreeNode::<MM>::empty_segment(r - l));
            return idx;
        }

        if r <= size {
            let idx = data.len() as u32;
            data.push(LazySegtreeNode::<MM>::len(r - l));
            return idx;
        }

        let m = (l + r) >> 1;
        let lc = Self::build_default_dfs(l, m, size, data);
        let rc = Self::build_default_dfs(m, r, size, data);

        let val = MM::op(&data[lc as usize].val, &data[rc as usize].val);

        let idx = data.len() as u32;
        data.push(LazySegtreeNode::<MM>::new(
            val,
            MM::id_e(),
            (r - l) as u32,
            lc,
            rc,
        ));

        idx
    }

    fn build_dfs(
        l: usize,
        r: usize,
        base: &[MM::S],
        data: &mut Vec<LazySegtreeNode<MM>>,
    ) -> u32 {
        if r - l == 1 {
            let idx = data.len() as u32;

            let val = if l < base.len() {
                base[l].clone()
            } else {
                MM::identity()
            };

            data.push(LazySegtreeNode {
                val,
                lazy: MM::id_e(),
                len: 1,
                left: NIL,
                right: NIL,
            });

            idx
        } else {
            let m = (l + r) >> 1;

            let lc = Self::build_dfs(l, m, base, data);
            let rc = Self::build_dfs(m, r, base, data);

            let val = MM::op(&data[lc as usize].val, &data[rc as usize].val);

            let idx = data.len() as u32;
            data.push(LazySegtreeNode::<MM>::new(
                val,
                MM::id_e(),
                (r - l) as u32,
                lc,
                rc,
            ));

            idx
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn versions(&self) -> usize {
        self.root.len()
    }

    pub fn root_index(&self, t: usize) -> u32 {
        assert!(t < self.root.len());
        self.root[t]
    }

    pub fn all_prod(&self, t: usize) -> MM::S {
        assert!(t < self.root.len());
        self.data[self.root[t] as usize].val.clone()
    }

    fn make_default_node(&mut self, len: usize) -> u32 {
        let idx = self.data.len() as u32;
        self.data.push(LazySegtreeNode::<MM>::len(len));
        idx
    }

    fn child_or_default(&mut self, child: u32, len: usize) -> u32 {
        if child == NIL {
            self.make_default_node(len)
        } else {
            child
        }
    }

    fn inner_apply(&mut self, p: u32, f: &MM::F) -> u32 {
        let mut node = self.data[p as usize].clone();

        node.val = MM::mapping(f, &node.val, node.len as usize);
        node.lazy = MM::composition(f, &node.lazy);

        let idx = self.data.len() as u32;
        self.data.push(node);

        idx
    }

    /// p を複製し、lazy を子へ押し下げた新しいノードを返す。
    ///
    /// 元の p は変更しない。
    fn push(&mut self, p: u32) -> u32 {
        let node = self.data[p as usize].clone();
        let len = node.len as usize;

        if len <= 1 {
            return p;
        }

        let half = len >> 1;

        let lc = self.child_or_default(node.left, half);
        let rc = self.child_or_default(node.right, half);

        let nl = self.inner_apply(lc, &node.lazy);
        let nr = self.inner_apply(rc, &node.lazy);

        let idx = self.data.len() as u32;

        self.data.push(LazySegtreeNode::<MM>::new(
            node.val,
            MM::id_e(),
            len as u32,
            nl,
            nr,
        ));

        idx
    }

    /// version t に対して [l, r) に f を作用させた新 version を追加する。
    pub fn apply_range(&mut self, t: usize, l: usize, r: usize, f: MM::F) -> usize {
        assert!(t < self.root.len());
        assert!(l <= r);
        assert!(r <= self.size);

        if l == r {
            self.root.push(self.root[t]);
            return self.root.len() - 1;
        }

        let new_root = self.apply_dfs(self.root[t], l, r, 0, self.n, &f);
        self.root.push(new_root);

        self.root.len() - 1
    }

    fn apply_dfs(
        &mut self,
        p: u32,
        ql: usize,
        qr: usize,
        pl: usize,
        pr: usize,
        f: &MM::F,
    ) -> u32 {
        if qr <= pl || pr <= ql {
            return p;
        }

        if ql <= pl && pr <= qr {
            return self.inner_apply(p, f);
        }

        let p = self.push(p);

        let m = (pl + pr) >> 1;

        let lc = self.data[p as usize].left;
        let rc = self.data[p as usize].right;

        let nl = self.apply_dfs(lc, ql, qr, pl, m, f);
        let nr = self.apply_dfs(rc, ql, qr, m, pr, f);

        let val = MM::op(&self.data[nl as usize].val, &self.data[nr as usize].val);

        let idx = self.data.len() as u32;

        self.data.push(LazySegtreeNode::<MM>::new(
            val,
            MM::id_e(),
            (pr - pl) as u32,
            nl,
            nr,
        ));

        idx
    }

    /// version t の i 番目を取得する。
    ///
    /// 探索中に push した結果を新 version として追加する。
    ///
    /// 返り値は `(値, 新 version 番号)`。
    pub fn get(&mut self, t: usize, i: usize) -> (MM::S, usize) {
        assert!(i < self.size);
        self.prod(t, i, i + 1)
    }

    /// version t の [l, r) の積を取得する。
    ///
    /// 探索中に必要な push を実体化した version を末尾に追加する。
    ///
    /// 返り値は `(区間積, 新 version 番号)`。
    pub fn prod(&mut self, t: usize, l: usize, r: usize) -> (MM::S, usize) {
        assert!(t < self.root.len());
        assert!(l <= r);
        assert!(r <= self.size);

        if l == r {
            self.root.push(self.root[t]);
            return (MM::identity(), self.root.len() - 1);
        }

        let (new_root, ans) = self.prod_dfs_mut(self.root[t], l, r, 0, self.n);

        self.root.push(new_root);

        let new_version = self.root.len() - 1;
        (ans, new_version)
    }

    fn prod_dfs_mut(
        &mut self,
        p: u32,
        ql: usize,
        qr: usize,
        pl: usize,
        pr: usize,
    ) -> (u32, MM::S) {
        if qr <= pl || pr <= ql {
            return (p, MM::identity());
        }

        if ql <= pl && pr <= qr {
            return (p, self.data[p as usize].val.clone());
        }

        let p = self.push(p);

        let m = (pl + pr) >> 1;

        let lc = self.data[p as usize].left;
        let rc = self.data[p as usize].right;

        let (nl, vl) = self.prod_dfs_mut(lc, ql, qr, pl, m);
        let (nr, vr) = self.prod_dfs_mut(rc, ql, qr, m, pr);

        let val = MM::op(&self.data[nl as usize].val, &self.data[nr as usize].val);

        let idx = self.data.len() as u32;

        self.data.push(LazySegtreeNode::<MM>::new(
            val,
            MM::id_e(),
            (pr - pl) as u32,
            nl,
            nr,
        ));

        let ans = MM::op(&vl, &vr);

        (idx, ans)
    }

    /// version t をベースにして、
    /// version k の [l, r) をコピーした新 version を追加する。
    pub fn range_copy(&mut self, t: usize, k: usize, l: usize, r: usize) -> usize {
        assert!(t < self.root.len());
        assert!(k < self.root.len());
        assert!(l <= r);
        assert!(r <= self.size);

        if l == r {
            self.root.push(self.root[t]);
            return self.root.len() - 1;
        }

        let new_root = self.copy_dfs(self.root[t], self.root[k], l, r, 0, self.n);
        self.root.push(new_root);

        self.root.len() - 1
    }

    fn copy_dfs(
        &mut self,
        dst: u32,
        src: u32,
        ql: usize,
        qr: usize,
        pl: usize,
        pr: usize,
    ) -> u32 {
        if qr <= pl || pr <= ql {
            return dst;
        }

        if ql <= pl && pr <= qr {
            return src;
        }

        let dst = self.push(dst);
        let src = self.push(src);

        let m = (pl + pr) >> 1;

        let dst_l = self.data[dst as usize].left;
        let dst_r = self.data[dst as usize].right;

        let src_l = self.data[src as usize].left;
        let src_r = self.data[src as usize].right;

        let nl = self.copy_dfs(dst_l, src_l, ql, qr, pl, m);
        let nr = self.copy_dfs(dst_r, src_r, ql, qr, m, pr);

        let val = MM::op(&self.data[nl as usize].val, &self.data[nr as usize].val);

        let idx = self.data.len() as u32;

        self.data.push(LazySegtreeNode::<MM>::new(
            val,
            MM::id_e(),
            (pr - pl) as u32,
            nl,
            nr,
        ));

        idx
    }
}
