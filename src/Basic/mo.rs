pub trait MoState {
    type Data;
    type Query;
    type Ans;
    // 添え字とその場所でのデータ
    fn add_left(&mut self, idx: usize, data: &Self::Data);
    fn add_right(&mut self, idx: usize, data: &Self::Data);
    fn sub_left(&mut self, idx: usize, data: &Self::Data);
    fn sub_right(&mut self, idx: usize, data: &Self::Data);
    fn ans(&mut self, query_id: usize, q_data: &Self::Query) -> Self::Ans;
}

// 対称ならこっち
pub trait SymMoState {
    type Data;
    type Query;
    type Ans;
    fn add(&mut self, idx: usize, data: &Self::Data);
    fn sub(&mut self, idx: usize, data: &Self::Data);
    fn ans(&mut self, query_id: usize, q_data: &Self::Query) -> Self::Ans;
}

impl<M: SymMoState> MoState for M {
    type Data = <M as SymMoState>::Data;
    type Query = <M as SymMoState>::Query;
    type Ans = <M as SymMoState>::Ans;
    
    #[inline]
    fn add_left(&mut self, idx: usize, data: &Self::Data) { self.add(idx, data); }
    #[inline]
    fn add_right(&mut self, idx: usize, data: &Self::Data) { self.add(idx, data); }
    #[inline]
    fn sub_left(&mut self, idx: usize, data: &Self::Data) { self.sub(idx, data); }
    #[inline]
    fn sub_right(&mut self, idx: usize, data: &Self::Data) { self.sub(idx, data); }
    #[inline]
    fn ans(&mut self, query_id: usize, q_data: &Self::Query) -> Self::Ans {
        self.ans(query_id, q_data)
    }
}

pub struct MoQuery {
    pub l: usize,
    pub r: usize,
    pub id: usize,
}

pub struct MoSolver<M: MoState> {
    data: Vec<M::Data>,
    query: Vec<(MoQuery, M::Query)>,
}

impl<M> MoSolver<M> where M: MoState {
    pub fn new(data: Vec<M::Data>) -> Self {
        MoSolver { data, query: Vec::new() }
    }

    pub fn add_query(&mut self, l: usize, r: usize, qd: M::Query) {
        self.query.push((MoQuery { l, r, id: self.query.len() }, qd));
    }

    const ROT_DELTA: [u32; 4] = [3, 0, 0, 1];
    #[inline]
    fn hilbert_order(x: u32, y: u32, pow: u32, rot: u32) -> u64 {
        if pow == 0 { return 0; }
        let h: u32 = 1u32 << (pow - 1);
        let mut seg: u32 = if x < h { if y < h { 0 } else { 3 } } else { if y < h { 1 } else { 2 } };
        seg = (seg + rot) & 3;
        let nrot = (rot + MoSolver::<M>::ROT_DELTA[seg as usize]) & 3;
        let nx = x & (h - 1);
        let ny = y & (h - 1);
        let sub: u64 = 1u64 << (2 * pow - 2);
        let mut ord = (seg as u64) * sub;
        let add = MoSolver::<M>::hilbert_order(nx, ny, pow - 1, nrot);
        ord += if seg == 1 || seg == 2 { add } else { sub - 1 - add };
        ord
    }

    fn get_ord(&self) -> Vec<usize> {
        let mx = self.query.iter().map(|(q, _)| q.l.max(q.r)).max().unwrap_or(0);
        let pow = if mx == 0 {0} else {(usize::BITS - mx.leading_zeros()) as u32};
        let mut ord = (0..self.query.len()).map(|i| {
            let q = &self.query[i].0;
            (Self::hilbert_order(q.l as u32,q.r as u32,pow,0),i,)
        }).collect::<Vec<_>>();
        ord.sort_unstable_by_key(|&(key, _)| key);
        ord.into_iter().map(|(_, i)| i).collect()
    }
    
    // 状態をstateで管理する
    pub fn solve(&mut self, state: &mut M) -> Vec<M::Ans> {
        let q = self.query.len();
        let mut res: Vec<Option<M::Ans>> = (0..q).map(|_| None).collect();
        let ord = self.get_ord();
        let mut l = 0;
        let mut r = 0;
        for qi in ord {
            let (query, qd) = &self.query[qi];
            while l > query.l {
                l -= 1;
                state.add_left(l, &self.data[l]);
            }
            while r < query.r {
                state.add_right(r, &self.data[r]);
                r += 1;
            }
            while l < query.l {
                state.sub_left(l, &self.data[l]);
                l += 1;
            }
            while r > query.r {
                r -= 1;
                state.sub_right(r, &self.data[r]);
            }
            res[query.id] = Some(state.ans(query.id, qd));
        }
        res.into_iter().map(|x| x.unwrap()).collect()
    }
}
