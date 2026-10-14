// Source: https://github.com/InverseAki/arcaki_library/blob/main/src/Graph/topologicalsort.rs
use std::cmp::Reverse;
use std::collections::BinaryHeap;

pub fn topological_sort(n: usize, e: &Vec<(usize, usize)>) -> Option<Vec<usize>>{
    let mut edge = vec![Vec::new(); n];
    let mut cnt = vec![0; n];
    for &(u, v) in e{
        edge[u].push(v);
        cnt[v] += 1;
    }
    let mut res = Vec::with_capacity(n);
    let mut stack = Vec::new();
    for i in 0..n{
        if cnt[i]==0{
            stack.push(i);
        }
    }
    while let Some(p) = stack.pop(){
        res.push(p);
        for &nex in &edge[p]{
            cnt[nex] -= 1;
            if cnt[nex] == 0{
                stack.push(nex);
            }
        }
    }
    if res.len()==n{
        Some(res)
    } else {
        None
    }
}

pub fn lexical_topological_sort(n: usize, e: &Vec<(usize, usize)>) -> Option<Vec<usize>>{
    let mut edge = vec![Vec::new(); n];
    let mut cnt = vec![0; n];
    for &(u, v) in e{
        edge[u].push(v);
        cnt[v] += 1;
    }
    let mut res = Vec::with_capacity(n);
    let mut heap = BinaryHeap::new();
    for i in 0..n{
        if cnt[i]==0{
            heap.push(Reverse(i));
        }
    }
    while let Some(Reverse(p)) = heap.pop(){
        res.push(p);
        for &nex in &edge[p]{
            cnt[nex] -= 1;
            if cnt[nex] == 0{
                heap.push(Reverse(nex));
            }
        }
    }
    if res.len()==n{
        Some(res)
    } else {
        None
    }
}