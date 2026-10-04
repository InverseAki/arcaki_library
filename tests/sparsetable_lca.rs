#![allow(dead_code)]

use std::collections::VecDeque;
use std::ops::{Index, IndexMut};

include!("../src/Basic/sparsetable.rs");
include!("../src/Tree/tree.rs");
include!("../src/Tree/sparsetablelca.rs");

#[test]
fn sparse_table_all_intervals() {
    for n in 0..70 {
        let values: Vec<_> = (0..n).map(|i| (i * 37 + 11) % 19).collect();
        let min = SparseTableMI::build(&values);
        let max = SparseTableMX::build(&values);
        let vertices: Vec<_> = (0..n).rev().collect();
        let keyed = SparseTableMI::build_by_key(&vertices, |v| values[v]);
        for l in 0..n {
            for r in l + 1..=n {
                assert_eq!(min.query(l, r), *values[l..r].iter().min().unwrap());
                assert_eq!(max.query(l, r), *values[l..r].iter().max().unwrap());
                let expected = *vertices[l..r].iter().min_by_key(|&&v| values[v]).unwrap();
                assert_eq!(keyed.query_by_key(l, r, |v| values[v]), expected);
            }
        }
    }
}

fn check_tree(n: usize, edges: &[(usize, usize)], root: usize) {
    let graph = UnweightedGraph::new(n, edges);
    let depth = graph.bfs(root);
    let mut parent = vec![root; n];
    for v in 0..n {
        if v != root {
            parent[v] = *graph[v].iter().find(|&&p| depth[p] + 1 == depth[v]).unwrap();
        }
    }
    let original = STLCA::new(root, &graph);
    let lca = original.clone();
    drop(original);
    for u in 0..n {
        let distances = graph.bfs(u);
        for v in 0..n {
            let (mut a, mut b) = (u, v);
            while depth[a] > depth[b] { a = parent[a]; }
            while depth[b] > depth[a] { b = parent[b]; }
            while a != b { a = parent[a]; b = parent[b]; }
            assert_eq!(lca.lca(u, v), a, "n={n}, root={root}, u={u}, v={v}");
            assert_eq!(lca.distance(u, v), distances[v]);
        }
    }
}

#[test]
fn structured_trees_and_roots() {
    for n in [1, 2, 3, 7, 16, 31, 64, 127] {
        for shape in 0..3 {
            let edges: Vec<_> = (1..n).map(|v| {
                let parent = match shape { 0 => v - 1, 1 => 0, _ => (v - 1) / 2 };
                (v, parent)
            }).collect();
            for root in [0, n / 2, n - 1] {
                check_tree(n, &edges, root);
            }
        }
    }
}

#[test]
fn random_trees_against_parent_walk() {
    let mut state = 20261004u64;
    let mut next = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state as usize
    };
    for _ in 0..100 {
        let n = 1 + next() % 70;
        let mut labels: Vec<_> = (0..n).collect();
        for i in (1..n).rev() { labels.swap(i, next() % (i + 1)); }
        let edges: Vec<_> = (1..n).map(|v| (labels[v], labels[next() % v])).collect();
        check_tree(n, &edges, next() % n);
    }
}

#[test]
fn large_star_table_storage() {
    let n = 100_000;
    let edges: Vec<_> = (1..n).map(|v| (0, v)).collect();
    let graph = UnweightedGraph::new(n, &edges);
    let lca = STLCA::new(0, &graph);
    assert_eq!(lca.lca(1, n - 1), 0);
    assert_eq!(lca.distance(1, n - 1), 2);
    let tour_len = 2 * n - 1;
    let cells: usize = (0..usize::BITS).map(|k| 1usize << k)
        .take_while(|&w| w <= tour_len).map(|w| tour_len - w + 1).sum();
    let stored: usize = lca.data.table.iter().map(Vec::len).sum();
    assert_eq!(stored, cells);
    let bytes: usize = lca.data.table.iter()
        .map(|row| row.capacity() * std::mem::size_of::<usize>()).sum();
    assert_eq!(bytes, cells * std::mem::size_of::<usize>());
    assert_eq!(lca.dist.len(), n);
    eprintln!("SparseTable payload: {bytes} bytes; old pair payload: {} bytes", cells * std::mem::size_of::<(usize, usize)>());
}
