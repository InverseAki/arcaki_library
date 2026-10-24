// Source: https://github.com/InverseAki/arcaki_library/blob/main/src/OfflineQuery/point_add_rectangle_sum.rs
pub struct BIT<T>
where
    T: Copy + std::ops::Add<Output = T> + std::ops::Sub<Output = T> + PartialOrd,
{
    n: usize,
    vec: Vec<T>,
    zero: T,
}

impl<T> BIT<T>
where
    T: Copy + std::ops::Add<Output = T> + std::ops::Sub<Output = T> + PartialOrd,
{
    pub fn new(n: usize, zero: T) -> Self {
        let k = n.max(1);
        let base = vec![zero; k + 1];
        BIT {
            n: k,
            vec: base,
            zero,
        }
    }

    #[inline]
    pub fn add(&mut self, mut idx: usize, x: T) {
        idx += 1;
        while idx <= self.n {
            self.vec[idx] = self.vec[idx] + x;
            idx += idx & (!idx + 1);
        }
    }

    #[inline]
    pub fn g(&self, mut r: usize) -> T {
        let mut res = self.zero;
        while r > 0 {
            res = res + self.vec[r];
            r -= r & (!r + 1);
        }
        res
    }

    #[inline]
    pub fn prod(&self, l: usize, r: usize) -> T {
        self.g(r) - self.g(l)
    }
}

/// 内部専用のi64特化BIT（wrapping演算とリセットの最適化付き）
struct RawBitI64 {
    n: usize,
    vec: Vec<i64>,
}

impl RawBitI64 {
    #[inline]
    fn new(n: usize) -> Self {
        Self {
            n,
            vec: vec![0i64; n + 1],
        }
    }

    #[inline(always)]
    fn add(&mut self, mut idx: usize, x: i64) {
        idx += 1;
        let n = self.n;
        let ptr = self.vec.as_mut_ptr();
        while idx <= n {
            unsafe {
                let p = ptr.add(idx);
                *p = (*p).wrapping_add(x);
            }
            idx += idx & idx.wrapping_neg();
        }
    }

    #[inline(always)]
    fn prod(&self, mut l: usize, mut r: usize) -> i64 {
        let mut res = 0i64;
        let ptr = self.vec.as_ptr();
        if PREFIX {
            while r > 0 {
                res = res.wrapping_add(unsafe { *ptr.add(r) });
                r &= r - 1;
            }
            while l > 0 {
                res = res.wrapping_sub(unsafe { *ptr.add(l) });
                l &= l - 1;
            }
        } else {
            while r > l {
                res = res.wrapping_add(unsafe { *ptr.add(r) });
                r &= r - 1;
            }
            while l > r {
                res = res.wrapping_sub(unsafe { *ptr.add(l) });
                l &= l - 1;
            }
        }
        res
    }

    #[inline(always)]
    fn clear(&mut self) {
        self.vec.fill(0);
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PointAddRectangleSumQuery<X = i32, Y = X> {
    Add { x: X, y: Y, w: i64 },
    Query { lx: X, ly: Y, rx: X, ry: Y },
}

#[derive(Clone)]
pub struct PointAddRectangleSum<X = i32, Y = X> {
    initial: Vec<(X, Y, i64)>,
    data: Vec<PointAddRectangleSumQuery<X, Y>>,
    adds: usize,
    queries: usize,
}

const ADD_FLAG: u32 = 0x8000_0000;
const PRE_ADD_QUERY_BLOCK: u32 = 0x7FFF_FFFF;

/// 16バイトにパックされたCDQ用イベント構造体
/// - Query: block_and_flag = block (MSB=0), y_or_payload = q_idx or !q_idx, w_or_lohi = lo | (hi << 32)
/// - Add:   block_and_flag = block | ADD_FLAG (MSB=1), y_or_payload = y, w_or_lohi = w
#[derive(Clone, Copy)]
#[repr(C)]
struct CdqEvent {
    block_and_flag: u32,
    y_or_payload: i32,
    w_or_lohi: i64,
}

impl CdqEvent {
    #[inline(always)]
    fn is_add(&self) -> bool {
        (self.block_and_flag as i32) < 0
    }
}

impl<X: Ord, Y: Ord + Clone> PointAddRectangleSum<X, Y> {
    pub fn new() -> Self {
        Self {
            initial: Vec::new(),
            data: Vec::new(),
            adds: 0,
            queries: 0,
        }
    }

    pub fn build(points: impl IntoIterator<Item = (X, Y, i64)>) -> Self {
        let mut initial: Vec<_> = points.into_iter().filter(|p| p.2 != 0).collect();
        initial.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        Self {
            initial,
            data: Vec::new(),
            adds: 0,
            queries: 0,
        }
    }

    #[inline]
    pub fn push_add(&mut self, x: X, y: Y, w: i64) {
        if w != 0 {
            self.data.push(PointAddRectangleSumQuery::Add { x, y, w });
            self.adds += 1;
        }
    }

    #[inline]
    pub fn push_query(&mut self, lx: X, ly: Y, rx: X, ry: Y) {
        assert!(lx <= rx && ly <= ry, "invalid rectangle");
        self.data
            .push(PointAddRectangleSumQuery::Query { lx, ly, rx, ry });
        self.queries += 1;
    }

    pub fn solve(&mut self) -> Vec<i64> {
        let total_queries = self.queries;
        assert!(total_queries <= i32::MAX as usize, "too many queries");
        let mut ans = vec![0i64; total_queries];
        if total_queries == 0 || (self.initial.is_empty() && self.adds == 0) {
            self.initial.clear();
            self.data.clear();
            self.adds = 0;
            self.queries = 0;
            return ans;
        }

        let initial = std::mem::take(&mut self.initial);
        let data = std::mem::take(&mut self.data);
        self.adds = 0;
        self.queries = 0;

        // 最後の有効なQueryの位置を特定し、それ以降のAddを完全に無視する
        let valid_len = data
            .iter()
            .rposition(|op| {
                matches!(
                    op,
                    PointAddRectangleSumQuery::Query { lx, ly, rx, ry } if lx < rx && ly < ry
                )
            })
            .map_or(0, |pos| pos + 1);

        if valid_len == 0 {
            return ans;
        }

        // 有効範囲内のAddのY座標のみを収集して座標圧縮
        let mut dynamic_adds = 0usize;
        for op in &data[..valid_len] {
            if let PointAddRectangleSumQuery::Add { w, .. } = op {
                if *w != 0 {
                    dynamic_adds += 1;
                }
            }
        }

        if initial.is_empty() && dynamic_adds == 0 {
            return ans;
        }

        let mut ys = Vec::with_capacity(initial.len() + dynamic_adds);
        ys.extend(initial.iter().map(|p| p.1.clone()));
        for op in &data[..valid_len] {
            if let PointAddRectangleSumQuery::Add { y, w, .. } = op {
                if *w != 0 {
                    ys.push(y.clone());
                }
            }
        }
        ys.sort_unstable();
        ys.dedup();
        let yn = ys.len();
        assert!(yn <= i32::MAX as usize, "too many coordinates");
        if yn == 0 {
            return ans;
        }

        // 動的Addが一切ない場合（静的オフライン矩形和）の高速パス
        if dynamic_adds == 0 {
            let mut events = Vec::with_capacity(total_queries * 2);
            let mut q_idx = 0i32;
            for op in data.into_iter().take(valid_len) {
                if let PointAddRectangleSumQuery::Query { lx, ly, rx, ry } = op {
                    let cur_q = q_idx;
                    q_idx += 1;
                    if lx < rx && ly < ry {
                        let lo = ys.partition_point(|y| y < &ly);
                        if lo < yn {
                            let hi = lo + ys[lo..].partition_point(|y| y < &ry);
                            if lo < hi {
                                let lohi = ((lo as u64) | ((hi as u64) << 32)) as i64;
                                events.push((
                                    lx,
                                    CdqEvent {
                                        block_and_flag: 0,
                                        y_or_payload: cur_q,
                                        w_or_lohi: lohi,
                                    },
                                ));
                                events.push((
                                    rx,
                                    CdqEvent {
                                        block_and_flag: 0,
                                        y_or_payload: !cur_q,
                                        w_or_lohi: lohi,
                                    },
                                ));
                            }
                        }
                    }
                }
            }
            if events.is_empty() {
                return ans;
            }
            events.sort_unstable_by(|a, b| a.0.cmp(&b.0));
            let mut bit = RawBitI64::new(yn);
            let mut p = 0;
            for (x, ev) in &events {
                while p < initial.len() && &initial[p].0 < x {
                    let y_idx = ys.partition_point(|y| y < &initial[p].1);
                    bit.add(y_idx, initial[p].2);
                    p += 1;
                }
                Self::answer_event(ev, &bit, &mut ans);
            }
            return ans;
        }

        // 動的Addがある場合のイベント構築とブロック時刻割り当て
        // Block k = [1個以上のAdd] + [1個以上のQuery]
        let mut events = Vec::with_capacity(dynamic_adds + total_queries * 2);
        let mut q_idx = 0i32;
        let mut seen_add = false;
        let mut last_was_query = false;
        let mut cur_block = 0u32;

        for op in data.into_iter().take(valid_len) {
            match op {
                PointAddRectangleSumQuery::Add { x, y, w } => {
                    if w == 0 {
                        continue;
                    }
                    if !seen_add {
                        seen_add = true;
                    } else if last_was_query {
                        cur_block += 1;
                        assert!(cur_block < PRE_ADD_QUERY_BLOCK, "too many blocks");
                        last_was_query = false;
                    }
                    let y_idx = ys.partition_point(|v| v < &y) as i32;
                    events.push((
                        x,
                        CdqEvent {
                            block_and_flag: cur_block | ADD_FLAG,
                            y_or_payload: y_idx,
                            w_or_lohi: w,
                        },
                    ));
                }
                PointAddRectangleSumQuery::Query { lx, ly, rx, ry } => {
                    let cur_q = q_idx;
                    q_idx += 1;
                    if !seen_add && initial.is_empty() {
                        continue;
                    }
                    if lx < rx && ly < ry {
                        let lo = ys.partition_point(|y| y < &ly);
                        if lo < yn {
                            let hi = lo + ys[lo..].partition_point(|y| y < &ry);
                            if lo < hi {
                                let block = if !seen_add {
                                    PRE_ADD_QUERY_BLOCK
                                } else {
                                    last_was_query = true;
                                    cur_block
                                };
                                let lohi = ((lo as u64) | ((hi as u64) << 32)) as i64;
                                events.push((
                                    lx,
                                    CdqEvent {
                                        block_and_flag: block,
                                        y_or_payload: cur_q,
                                        w_or_lohi: lohi,
                                    },
                                ));
                                events.push((
                                    rx,
                                    CdqEvent {
                                        block_and_flag: block,
                                        y_or_payload: !cur_q,
                                        w_or_lohi: lohi,
                                    },
                                ));
                            }
                        }
                    }
                }
            }
        }

        if events.is_empty() {
            return ans;
        }

        // 全体で1回だけ (x, is_add) の順でソート（QueryはMSB=0, AddはMSB=1なのでそのまま比較可能）
        events.sort_unstable_by(|a, b| {
            a.0.cmp(&b.0)
                .then_with(|| (a.1.block_and_flag & ADD_FLAG).cmp(&(b.1.block_and_flag & ADD_FLAG)))
        });

        let mut bit = RawBitI64::new(yn);

        // 初期点群の平面走査（すでにX順になっているeventsをそのまま利用）
        if !initial.is_empty() {
            let mut p = 0;
            for (x, ev) in &events {
                if !ev.is_add() {
                    while p < initial.len() && &initial[p].0 < x {
                        let y_idx = ys.partition_point(|y| y < &initial[p].1);
                        bit.add(y_idx, initial[p].2);
                        p += 1;
                    }
                    Self::answer_event(ev, &bit, &mut ans);
                }
            }
            bit.clear();
        }
        drop(initial);
        drop(ys);

        // 最後のブロックに有効なQueryが1つもなかった場合はそのブロックのAddを除外
        let num_blocks = if seen_add {
            if last_was_query {
                (cur_block + 1) as usize
            } else {
                cur_block as usize
            }
        } else {
            0
        };

        if num_blocks == 0 {
            return ans;
        }

        // Xを破棄しつつ、CDQ対象のイベントのみを抽出し、各ブロックの要素数を集計
        let mut block_counts = vec![0usize; num_blocks];
        let mut cdq_events = Vec::with_capacity(events.len());
        for (_, ev) in events {
            let b = (ev.block_and_flag & !ADD_FLAG) as usize;
            if b < num_blocks {
                unsafe {
                    *block_counts.get_unchecked_mut(b) += 1;
                }
                cdq_events.push(ev);
            }
        }

        if cdq_events.is_empty() {
            return ans;
        }

        let mut block_offsets = Vec::with_capacity(num_blocks + 1);
        block_offsets.push(0);
        let mut acc = 0;
        for &c in &block_counts {
            acc += c;
            block_offsets.push(acc);
        }

        let mut buf = vec![
            CdqEvent {
                block_and_flag: 0,
                y_or_payload: 0,
                w_or_lohi: 0,
            };
            cdq_events.len()
        ];

        Self::cdq_dfs(
            0,
            num_blocks,
            &mut cdq_events,
            &mut buf,
            &block_offsets,
            &mut bit,
            &mut ans,
        );

        ans
    }

    /// 完全ブランチレスなクエリ加算・減算処理
    #[inline(always)]
    fn answer_event(e: &CdqEvent, bit: &RawBitI64, ans: &mut [i64]) {
        let lohi = e.w_or_lohi as u64;
        let lo = (lohi as u32) as usize;
        let hi = (lohi >> 32) as usize;
        let sum = bit.prod(lo, hi);

        let payload = e.y_or_payload as i64;
        let sign = payload >> 63; // payload >= 0 なら 0, payload < 0 なら -1
        let idx = (payload ^ sign) as usize;
        let mask = !sign; // payload >= 0 なら -1, payload < 0 なら 0
        let delta = (sum ^ mask).wrapping_sub(mask); // payload >= 0 なら -sum, payload < 0 なら +sum

        unsafe {
            let p = ans.get_unchecked_mut(idx);
            *p = (*p).wrapping_add(delta);
        }
    }

    /// ソート不要・書き戻し不要の Ping-Pong CDQ分割統治
    fn cdq_dfs(
        l: usize,
        r: usize,
        src: &mut [CdqEvent],
        dst: &mut [CdqEvent],
        block_offsets: &[usize],
        bit: &mut RawBitI64,
        ans: &mut [i64],
    ) {
        if src.is_empty() {
            return;
        }

        if src.len() <= CUTOFF {
            for (i, e) in src.iter().enumerate() {
                if e.is_add() { continue; }
                let lo = e.w_or_lohi as u32;
                let hi = (e.w_or_lohi as u64 >> 32) as u32;
                let mut sum = 0i64;
                for a in &src[..i] {
                    if a.is_add() && (a.block_and_flag & !ADD_FLAG) <= e.block_and_flag
                        && lo <= a.y_or_payload as u32 && (a.y_or_payload as u32) < hi {
                        sum = sum.wrapping_add(a.w_or_lohi);
                    }
                }
                let sign = (e.y_or_payload >> 31) as i64;
                let idx = (e.y_or_payload as i64 ^ sign) as usize;
                let mask = !sign;
                ans[idx] = ans[idx].wrapping_add((sum ^ mask).wrapping_sub(mask));
            }
            return;
        }
        // 葉ノード（単一ブロック）: [Add群 -> Query群] なのでバッファコピーなしでインプレース処理
        if r - l == 1 {
            let mut added_count = 0usize;
            for e in src.iter() {
                if e.is_add() {
                    bit.add(e.y_or_payload as usize, e.w_or_lohi);
                    added_count += 1;
                } else {
                    Self::answer_event(e, bit, ans);
                }
            }
            if added_count * CLEAR_FACTOR > bit.n {
                bit.clear();
            } else if added_count > 0 {
                for e in src.iter() {
                    if e.is_add() {
                        bit.add(e.y_or_payload as usize, e.w_or_lohi.wrapping_neg());
                    }
                }
            }
            return;
        }

        let m = (l + r) >> 1;
        let left_len = unsafe { *block_offsets.get_unchecked(m) - *block_offsets.get_unchecked(l) };
        if left_len == 0 {
            Self::cdq_dfs(m, r, src, dst, block_offsets, bit, ans);
            return;
        }
        if left_len == src.len() {
            Self::cdq_dfs(l, m, src, dst, block_offsets, bit, ans);
            return;
        }

        let (dst_left, dst_right) = dst.split_at_mut(left_len);
        let m_u32 = m as u32;
        let m_add_u32 = m_u32 | ADD_FLAG;

        let mut li = 0usize;
        let mut ri = 0usize;
        let mut added_count = 0usize;

        let left_ptr = dst_left.as_mut_ptr();
        let right_ptr = dst_right.as_mut_ptr();

        for &e in src.iter() {
            if e.is_add() {
                if e.block_and_flag < m_add_u32 {
                    bit.add(e.y_or_payload as usize, e.w_or_lohi);
                    added_count += 1;
                    unsafe {
                        *left_ptr.add(li) = e;
                    }
                    li += 1;
                } else {
                    unsafe {
                        *right_ptr.add(ri) = e;
                    }
                    ri += 1;
                }
            } else if e.block_and_flag < m_u32 {
                unsafe {
                    *left_ptr.add(li) = e;
                }
                li += 1;
            } else {
                Self::answer_event(&e, bit, ans);
                unsafe {
                    *right_ptr.add(ri) = e;
                }
                ri += 1;
            }
        }

        // BITのロールバック（サイズに応じてmemsetと個別減算を切り替え）
        if added_count * CLEAR_FACTOR > bit.n {
            bit.clear();
        } else if added_count > 0 {
            for e in dst_left.iter() {
                if e.is_add() {
                    bit.add(e.y_or_payload as usize, e.w_or_lohi.wrapping_neg());
                }
            }
        }

        let (src_left, src_right) = src.split_at_mut(left_len);
        Self::cdq_dfs(l, m, dst_left, src_left, block_offsets, bit, ans);
        Self::cdq_dfs(m, r, dst_right, src_right, block_offsets, bit, ans);
    }
}

impl<X: Ord, Y: Ord + Clone> Default for PointAddRectangleSum<X, Y> {
    fn default() -> Self {
        Self::new()
    }
}
