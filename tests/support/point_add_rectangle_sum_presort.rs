// Performance experiment. The production source is unchanged.
use super::baseline::{BIT, PointAddRectangleSumQuery as Op};

#[derive(Clone, Copy, Default)]
struct Event {
    x: i32,
    lo: u32,
    hi: u32,
    time: u32,
    payload: i64, // add weight, or signed answer index
}

const ADD: u32 = u32::MAX;

pub fn solve<const ADD_ONLY: bool>(ops: &[Op]) -> Vec<i64> {
    solve_impl::<ADD_ONLY, false>(ops)
}

pub fn solve_merge(ops: &[Op]) -> Vec<i64> {
    solve_impl::<true, true>(ops)
}

fn solve_impl<const ADD_ONLY: bool, const MERGE: bool>(ops: &[Op]) -> Vec<i64> {
    let mut ys = Vec::new();
    let mut count = 0;
    for op in ops {
        match *op {
            Op::Add { y, .. } => ys.push(y),
            Op::Query { ly, ry, .. } => {
                count += 1;
                if !ADD_ONLY { ys.extend([ly, ry]); }
            }
        }
    }
    let mut answers = vec![0; count];
    if count == 0 || ys.is_empty() { return answers; }
    ys.sort_unstable();
    ys.dedup();
    let rank = |y| ys.partition_point(|&v| v < y) as u32;
    let mut events = Vec::with_capacity(ops.len() + count);
    let mut answer = 0;
    for (time, op) in ops.iter().enumerate() {
        match *op {
            Op::Add { x, y, w } => events.push(Event {
                x, lo: rank(y), hi: ADD, time: time as u32, payload: w,
                ..Event::default()
            }),
            Op::Query { lx, ly, rx, ry } => {
                let lo = rank(ly);
                let hi = rank(ry);
                if lx < rx && lo < hi {
                    events.push(Event { x: lx, lo, hi, time: time as u32,
                        payload: answer, ..Event::default() });
                    events.push(Event { x: rx, lo, hi, time: time as u32,
                        payload: -answer - 1, ..Event::default() });
                }
                answer += 1;
            }
        }
    }
    // Boundaries before adds at the same x implements x < boundary.
    let mut scratch = vec![Event::default(); events.len()];
    let mut bit = BIT::new(ys.len(), 0i64);
    if MERGE {
        dfs_merge(&mut events, &mut scratch, &mut bit, &mut answers);
    } else {
        events.sort_unstable_by_key(|e| (e.x, e.hi == ADD));
        dfs(&mut events, &mut scratch, 0, ops.len(), &mut bit, &mut answers);
    }
    answers
}

fn dfs_merge(events: &mut [Event], scratch: &mut [Event], bit: &mut BIT<i64>, answers: &mut [i64]) {
    if events.len() <= 1 { return; }
    // A homogeneous time interval has no internal add -> query contribution.
    let adds = events.iter().filter(|e| e.hi == ADD).count();
    if adds == 0 || adds == events.len() {
        events.sort_unstable_by_key(|e| e.x);
        return;
    }
    let m = events.len() / 2;
    let (left, right) = events.split_at_mut(m);
    let (ls, rs) = scratch.split_at_mut(m);
    dfs_merge(left, ls, bit, answers);
    dfs_merge(right, rs, bit, answers);
    let mut i = 0;
    for e in right.iter().filter(|e| e.hi != ADD) {
        while i < left.len() && left[i].x < e.x {
            if left[i].hi == ADD { bit.add(left[i].lo as usize, left[i].payload); }
            i += 1;
        }
        let sum = bit.prod(e.lo as usize, e.hi as usize);
        if e.payload >= 0 { answers[e.payload as usize] -= sum; }
        else { answers[(-e.payload - 1) as usize] += sum; }
    }
    for e in &left[..i] {
        if e.hi == ADD { bit.add(e.lo as usize, -e.payload); }
    }
    let (mut i, mut j, mut k) = (0, 0, 0);
    while i < left.len() && j < right.len() {
        if left[i].x <= right[j].x { scratch[k] = left[i]; i += 1; }
        else { scratch[k] = right[j]; j += 1; }
        k += 1;
    }
    scratch[k..k + left.len() - i].copy_from_slice(&left[i..]);
    k += left.len() - i;
    scratch[k..].copy_from_slice(&right[j..]);
    events.copy_from_slice(scratch);
}

fn dfs(events: &mut [Event], scratch: &mut [Event], l: usize, r: usize,
       bit: &mut BIT<i64>, answers: &mut [i64]) {
    if r - l <= 1 || events.is_empty() { return; }
    let m = (l + r) / 2;
    let mut left_len = 0;
    let mut left_add = false;
    let mut left_query = false;
    let mut right_add = false;
    let mut right_query = false;
    for e in events.iter() {
        if (e.time as usize) < m {
            left_len += 1;
            if e.hi == ADD { left_add = true; } else { left_query = true; }
        } else if e.hi == ADD { right_add = true; } else { right_query = true; }
    }
    if left_add && right_query {
        for e in events.iter() {
            if (e.time as usize) < m {
                if e.hi == ADD { bit.add(e.lo as usize, e.payload); }
            } else if e.hi != ADD {
                let sum = bit.prod(e.lo as usize, e.hi as usize);
                if e.payload >= 0 { answers[e.payload as usize] -= sum; }
                else { answers[(-e.payload - 1) as usize] += sum; }
            }
        }
        for e in events.iter() {
            if (e.time as usize) < m && e.hi == ADD {
                bit.add(e.lo as usize, -e.payload);
            }
        }
    }
    if !(left_add && left_query || right_add && right_query) { return; }
    let (mut a, mut b) = (0, left_len);
    for &e in events.iter() {
        if (e.time as usize) < m { scratch[a] = e; a += 1; }
        else { scratch[b] = e; b += 1; }
    }
    events.copy_from_slice(&scratch[..events.len()]);
    let (left, right) = events.split_at_mut(left_len);
    let (ls, rs) = scratch.split_at_mut(left_len);
    if left_add && left_query { dfs(left, ls, l, m, bit, answers); }
    if right_add && right_query { dfs(right, rs, m, r, bit, answers); }
}
