// Source: https://github.com/InverseAki/arcaki_library/blob/main/src/OfflineQuery/mo.rs
pub trait MoState {
    type Data;
    type Query;
    type Ans;
    fn add_left(&mut self, idx: usize, data: &Self::Data);
    fn add_right(&mut self, idx: usize, data: &Self::Data);
    fn sub_left(&mut self, idx: usize, data: &Self::Data);
    fn sub_right(&mut self, idx: usize, data: &Self::Data);
    fn ans(&mut self, query_id: usize, q_data: &Self::Query) -> Self::Ans;
}

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
    fn add_left(&mut self, idx: usize, data: &Self::Data) {
        self.add(idx, data);
    }
    #[inline]
    fn add_right(&mut self, idx: usize, data: &Self::Data) {
        self.add(idx, data);
    }
    #[inline]
    fn sub_left(&mut self, idx: usize, data: &Self::Data) {
        self.sub(idx, data);
    }
    #[inline]
    fn sub_right(&mut self, idx: usize, data: &Self::Data) {
        self.sub(idx, data);
    }
    #[inline]
    fn ans(&mut self, query_id: usize, q_data: &Self::Query) -> Self::Ans {
        SymMoState::ans(self, query_id, q_data)
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

impl<M> MoSolver<M>
where
    M: MoState,
{
    pub fn new(data: Vec<M::Data>) -> Self {
        MoSolver {
            data,
            query: Vec::new(),
        }
    }

    pub fn add_query(&mut self, l: usize, r: usize, qd: M::Query) {
        self.query.push((
            MoQuery {
                l,
                r,
                id: self.query.len(),
            },
            qd,
        ));
    }

    #[inline]
    fn hilbert_order(x: u32, y: u32, pow: u32, mut rot: u32) -> u64 {
        let mut ord = 0;
        let mut flip = 0;
        for p in (0..pow).rev() {
            let rx = (x >> p) & 1;
            let ry = (y >> p) & 1;
            let seg = ((ry * 3 ^ rx) + rot) & 3;
            rot = (rot + [3, 0, 0, 1][seg as usize]) & 3;
            ord = (ord << 2) | ((seg ^ flip) as u64);
            if seg == 0 || seg == 3 {
                flip ^= 3;
            }
        }
        ord
    }

    fn get_ord(&self) -> Vec<usize> {
        let mx = self
            .query
            .iter()
            .map(|(q, _)| q.l.max(q.r))
            .max()
            .unwrap_or(0);
        let pow = if mx == 0 {
            0
        } else {
            (usize::BITS - mx.leading_zeros()) as u32
        };
        let mut ord = (0..self.query.len())
            .map(|i| {
                let q = &self.query[i].0;
                (Self::hilbert_order(q.l as u32, q.r as u32, pow, 0), i)
            })
            .collect::<Vec<_>>();
        ord.sort_unstable_by_key(|&(key, _)| key);
        ord.into_iter().map(|(_, i)| i).collect()
    }

    pub fn solve(&mut self, state: &mut M) -> Vec<M::Ans> {
        let q = self.query.len();
        let mut res: Vec<Option<M::Ans>> = (0..q).map(|_| None).collect();
        let ord = self.get_ord();
        let mut l = 0;
        let mut r = 0;
        for qi in ord {
            let (query, qd) = &self.query[qi];
            if l > query.l {
                for (offset, value) in self.data[query.l..l].iter().enumerate().rev() {
                    state.add_left(query.l + offset, value);
                }
                l = query.l;
            }
            if r < query.r {
                for (offset, value) in self.data[r..query.r].iter().enumerate() {
                    state.add_right(r + offset, value);
                }
                r = query.r;
            }
            if l < query.l {
                for (offset, value) in self.data[l..query.l].iter().enumerate() {
                    state.sub_left(l + offset, value);
                }
                l = query.l;
            }
            if r > query.r {
                for (offset, value) in self.data[query.r..r].iter().enumerate().rev() {
                    state.sub_right(query.r + offset, value);
                }
                r = query.r;
            }
            res[query.id] = Some(state.ans(query.id, qd));
        }
        res.into_iter().map(|x| x.unwrap()).collect()
    }
}
