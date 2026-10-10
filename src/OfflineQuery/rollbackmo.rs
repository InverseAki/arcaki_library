pub trait RollbackMoState {
    type Data;
    type Query;
    type Ans;
    type Snapshot;
    fn add_left(&mut self, idx: usize, data: &Self::Data);
    fn add_right(&mut self, idx: usize, data: &Self::Data);
    fn snapshot(&self) -> Self::Snapshot;
    fn rollback(&mut self, snapshot: Self::Snapshot);
    fn ans(&mut self, query_id: usize, q_data: &Self::Query) -> Self::Ans;
}

pub trait SymRollbackMoState {
    type Data;
    type Query;
    type Ans;
    type Snapshot;
    fn add(&mut self, idx: usize, data: &Self::Data);
    fn snapshot(&self) -> Self::Snapshot;
    fn rollback(&mut self, snapshot: Self::Snapshot);
    fn ans(&mut self, query_id: usize, q_data: &Self::Query) -> Self::Ans;
}

impl<M: SymRollbackMoState> RollbackMoState for M {
    type Data = M::Data;
    type Query = M::Query;
    type Ans = M::Ans;
    type Snapshot = M::Snapshot;
    #[inline]
    fn add_left(&mut self, idx: usize, data: &Self::Data) {
        SymRollbackMoState::add(self, idx, data);
    }
    #[inline]
    fn add_right(&mut self, idx: usize, data: &Self::Data) {
        SymRollbackMoState::add(self, idx, data);
    }
    #[inline]
    fn snapshot(&self) -> Self::Snapshot {
        SymRollbackMoState::snapshot(self)
    }
    #[inline]
    fn rollback(&mut self, snapshot: Self::Snapshot) {
        SymRollbackMoState::rollback(self, snapshot);
    }
    #[inline]
    fn ans(&mut self, query_id: usize, q_data: &Self::Query) -> Self::Ans {
        SymRollbackMoState::ans(self, query_id, q_data)
    }
}

pub trait RollbackMoDeleteState {
    type Data;
    type Query;
    type Ans;
    type Snapshot;
    fn sub_left(&mut self, idx: usize, data: &Self::Data);
    fn sub_right(&mut self, idx: usize, data: &Self::Data);
    fn snapshot(&self) -> Self::Snapshot;
    fn rollback(&mut self, snapshot: Self::Snapshot);
    fn ans(&mut self, query_id: usize, q_data: &Self::Query) -> Self::Ans;
}

pub trait SymRollbackMoDeleteState {
    type Data;
    type Query;
    type Ans;
    type Snapshot;
    fn sub(&mut self, idx: usize, data: &Self::Data);
    fn snapshot(&self) -> Self::Snapshot;
    fn rollback(&mut self, snapshot: Self::Snapshot);
    fn ans(&mut self, query_id: usize, q_data: &Self::Query) -> Self::Ans;
}

impl<M: SymRollbackMoDeleteState> RollbackMoDeleteState for M {
    type Data = M::Data;
    type Query = M::Query;
    type Ans = M::Ans;
    type Snapshot = M::Snapshot;
    #[inline]
    fn sub_left(&mut self, idx: usize, data: &Self::Data) {
        SymRollbackMoDeleteState::sub(self, idx, data);
    }
    #[inline]
    fn sub_right(&mut self, idx: usize, data: &Self::Data) {
        SymRollbackMoDeleteState::sub(self, idx, data);
    }
    #[inline]
    fn snapshot(&self) -> Self::Snapshot {
        SymRollbackMoDeleteState::snapshot(self)
    }
    #[inline]
    fn rollback(&mut self, snapshot: Self::Snapshot) {
        SymRollbackMoDeleteState::rollback(self, snapshot);
    }
    #[inline]
    fn ans(&mut self, query_id: usize, q_data: &Self::Query) -> Self::Ans {
        SymRollbackMoDeleteState::ans(self, query_id, q_data)
    }
}

mod rollback_mo_detail {
    pub struct Query {
        pub l: usize,
        pub r: usize,
        pub id: usize,
    }

    pub struct Queries<D, Q> {
        pub data: Vec<D>,
        pub query: Vec<(Query, Q)>,
    }

    impl<D, Q> Queries<D, Q> {
        pub fn new(data: Vec<D>) -> Self {
            Self {
                data,
                query: Vec::new(),
            }
        }

        pub fn add_query(&mut self, l: usize, r: usize, qd: Q) {
            assert!(
                l <= r && r <= self.data.len(),
                "invalid range for Rollback Mo"
            );
            self.query.push((
                Query {
                    l,
                    r,
                    id: self.query.len(),
                },
                qd,
            ));
        }

        pub fn block_size(&self) -> usize {
            let n = self.data.len().max(1);
            let q = self.query.len().max(1);
            ((n as f64 / (q as f64).sqrt()).ceil() as usize).clamp(1, n)
        }

        pub fn order(&self, block_size: usize, deleting: bool) -> Vec<usize> {
            assert!(block_size > 0, "block_size must be positive");
            let mut ord: Vec<_> = (0..self.query.len()).collect();
            ord.sort_unstable_by(|&a, &b| {
                let a = &self.query[a].0;
                let b = &self.query[b].0;
                (a.l / block_size)
                    .cmp(&(b.l / block_size))
                    .then_with(|| {
                        if deleting {
                            b.r.cmp(&a.r)
                        } else {
                            a.r.cmp(&b.r)
                        }
                    })
                    .then_with(|| a.id.cmp(&b.id))
            });
            ord
        }
    }
}

pub struct RollbackMoSolver<M: RollbackMoState> {
    queries: rollback_mo_detail::Queries<M::Data, M::Query>,
}

impl<M: RollbackMoState> RollbackMoSolver<M> {
    pub fn new(data: Vec<M::Data>) -> Self {
        Self {
            queries: rollback_mo_detail::Queries::new(data),
        }
    }

    pub fn add_query(&mut self, l: usize, r: usize, qd: M::Query) {
        self.queries.add_query(l, r, qd);
    }

    pub fn solve(&mut self, state: &mut M) -> Vec<M::Ans> {
        self.solve_with_block_size(state, self.queries.block_size())
    }

    pub fn solve_with_block_size(&mut self, state: &mut M, block_size: usize) -> Vec<M::Ans> {
        let queries = &self.queries;
        let ord = queries.order(block_size, false);
        let mut res: Vec<Option<M::Ans>> = (0..ord.len()).map(|_| None).collect();
        let mut begin = 0;
        while begin < ord.len() {
            let block = queries.query[ord[begin]].0.l / block_size;
            let start = block * block_size;
            let border = start + block_size.min(queries.data.len() - start);
            let mut end = begin + 1;
            while end < ord.len() && queries.query[ord[end]].0.l / block_size == block {
                end += 1;
            }
            let block_snapshot = state.snapshot();
            let mut right = border;
            for &qi in &ord[begin..end] {
                let (query, qd) = &queries.query[qi];
                if query.r <= border {
                    let snapshot = state.snapshot();
                    for idx in query.l..query.r {
                        state.add_right(idx, &queries.data[idx]);
                    }
                    res[query.id] = Some(state.ans(query.id, qd));
                    state.rollback(snapshot);
                } else {
                    while right < query.r {
                        state.add_right(right, &queries.data[right]);
                        right += 1;
                    }
                    let snapshot = state.snapshot();
                    for idx in (query.l..border).rev() {
                        state.add_left(idx, &queries.data[idx]);
                    }
                    res[query.id] = Some(state.ans(query.id, qd));
                    state.rollback(snapshot);
                }
            }
            state.rollback(block_snapshot);
            begin = end;
        }
        res.into_iter().map(|x| x.unwrap()).collect()
    }
}

pub struct RollbackMoDeleteSolver<M: RollbackMoDeleteState> {
    queries: rollback_mo_detail::Queries<M::Data, M::Query>,
}

impl<M: RollbackMoDeleteState> RollbackMoDeleteSolver<M> {
    pub fn new(data: Vec<M::Data>) -> Self {
        Self {
            queries: rollback_mo_detail::Queries::new(data),
        }
    }

    pub fn add_query(&mut self, l: usize, r: usize, qd: M::Query) {
        self.queries.add_query(l, r, qd);
    }

    pub fn solve(&mut self, state: &mut M) -> Vec<M::Ans> {
        self.solve_with_block_size(state, self.queries.block_size())
    }

    pub fn solve_with_block_size(&mut self, state: &mut M, block_size: usize) -> Vec<M::Ans> {
        let queries = &self.queries;
        let ord = queries.order(block_size, true);
        let mut res: Vec<Option<M::Ans>> = (0..ord.len()).map(|_| None).collect();
        let mut begin = 0;
        while begin < ord.len() {
            let block = queries.query[ord[begin]].0.l / block_size;
            let start = block * block_size;
            let mut end = begin + 1;
            while end < ord.len() && queries.query[ord[end]].0.l / block_size == block {
                end += 1;
            }
            let block_snapshot = state.snapshot();
            for idx in 0..start {
                state.sub_left(idx, &queries.data[idx]);
            }
            let mut right = queries.data.len();
            for &qi in &ord[begin..end] {
                let (query, qd) = &queries.query[qi];
                while right > query.r {
                    right -= 1;
                    state.sub_right(right, &queries.data[right]);
                }
                let snapshot = state.snapshot();
                for idx in start..query.l {
                    state.sub_left(idx, &queries.data[idx]);
                }
                res[query.id] = Some(state.ans(query.id, qd));
                state.rollback(snapshot);
            }
            state.rollback(block_snapshot);
            begin = end;
        }
        res.into_iter().map(|x| x.unwrap()).collect()
    }
}

pub trait RollbackMoMonoid {
    type S: Clone;
    type T;
    type U;
    type V: Clone;
    type X: Clone + Default;
    fn init_t(n: usize, q: usize, a: &Vec<Self::S>, b: &Self::U) -> Self::T;
    fn increase(t: &mut Self::T, s: &Self::S);
    fn snapshot(t: &mut Self::T);
    fn rollback(t: &mut Self::T);
    fn get(t: &Self::T, x: &Self::V) -> Self::X;
}

pub fn solve_rollback_mo<M>(a: Vec<M::S>, x: &M::U, query: Vec<(usize, usize, M::V)>) -> Vec<M::X>
where
    M: RollbackMoMonoid,
{
    let (n, q) = (a.len(), query.len());
    let b = ((n as f64).sqrt() as usize).max(1);
    let mut ans = vec![M::X::default(); q];
    let mut qs = vec![Vec::new(); (n + b - 1) / b];
    let mut t = M::init_t(n, q, &a, x);
    for (idx, (l, r, z)) in query.iter().enumerate() {
        let (bl, br) = ((*l + b - 1) / b, *r / b);
        if bl >= br {
            for i in *l..*r {
                M::increase(&mut t, &a[i]);
            }
            ans[idx] = M::get(&t, z);
            M::rollback(&mut t);
        } else {
            qs[bl].push((*r, *l, z.clone(), idx));
        }
    }
    for i in 0..qs.len() {
        qs[i].sort_by(|u, v| u.0.cmp(&v.0));
        let st = (i + 1) * b;
        let mut right = st;
        t = M::init_t(n, q, &a, x);
        for (r, l, z, idx) in &qs[i] {
            while right < *r {
                M::increase(&mut t, &a[right]);
                right += 1;
            }
            M::snapshot(&mut t);
            for i in (*l..st).rev() {
                M::increase(&mut t, &a[i]);
            }
            ans[*idx] = M::get(&mut t, z);
            M::rollback(&mut t);
        }
    }
    ans
}
