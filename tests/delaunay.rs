#[path = "../src/Gemetory/delaunay.rs"]
mod delaunay;
use delaunay::{euclidean_mst, DelaunayTriangulation};
use std::collections::{BTreeMap, BTreeSet};
type P = (i64, i64);
fn orient(a: P, b: P, c: P) -> i128 {
    (b.0 as i128 - a.0 as i128) * (c.1 as i128 - a.1 as i128)
        - (b.1 as i128 - a.1 as i128) * (c.0 as i128 - a.0 as i128)
}
fn circle(a: P, b: P, c: P, d: P) -> i128 {
    let (ax, ay) = (a.0 as i128 - d.0 as i128, a.1 as i128 - d.1 as i128);
    let (bx, by) = (b.0 as i128 - d.0 as i128, b.1 as i128 - d.1 as i128);
    let (cx, cy) = (c.0 as i128 - d.0 as i128, c.1 as i128 - d.1 as i128);
    (ax * ax + ay * ay) * (bx * cy - by * cx) - (bx * bx + by * by) * (ax * cy - ay * cx)
        + (cx * cx + cy * cy) * (ax * by - ay * bx)
}
fn distance(a: P, b: P) -> i128 {
    let dx = a.0 as i128 - b.0 as i128;
    let dy = a.1 as i128 - b.1 as i128;
    dx * dx + dy * dy
}
fn prim(p: &[P]) -> Vec<i128> {
    if p.is_empty() {
        return vec![];
    }
    let mut best = vec![i128::MAX; p.len()];
    let mut used = vec![false; p.len()];
    best[0] = 0;
    let mut weights = Vec::new();
    for i in 0..p.len() {
        let v = (0..p.len())
            .filter(|&v| !used[v])
            .min_by_key(|&v| best[v])
            .unwrap();
        if i > 0 {
            weights.push(best[v]);
        }
        used[v] = true;
        for u in 0..p.len() {
            best[u] = best[u].min(distance(p[u], p[v]));
        }
    }
    weights.sort_unstable();
    weights
}
fn check(p: &[P]) {
    let d = DelaunayTriangulation::new(p);
    assert_eq!(d, DelaunayTriangulation::new(p), "determinism");
    let unique: BTreeSet<_> = d.representatives.iter().copied().collect();
    let edges: BTreeSet<_> = d.edges.iter().copied().collect();
    assert_eq!(edges.len(), d.edges.len());
    assert_eq!(d.duplicate_edges.len(), p.len() - unique.len());
    for (i, &r) in d.representatives.iter().enumerate() {
        assert_eq!(p[i], p[r]);
        assert!(r <= i);
        assert_eq!(r, p.iter().position(|x| x == &p[i]).unwrap());
    }
    let mut incidence = BTreeMap::new();
    for &[a, b, c] in &d.triangles {
        assert!(a < b && a < c);
        assert!(orient(p[a], p[b], p[c]) > 0);
        for &v in &unique {
            assert!(
                circle(p[a], p[b], p[c], p[v]) <= 0,
                "nonempty circle: {p:?}"
            );
        }
        for (u, v) in [(a, b), (b, c), (c, a)] {
            let e = (u.min(v), u.max(v));
            assert!(edges.contains(&e));
            *incidence.entry(e).or_insert(0) += 1;
        }
    }
    assert!(incidence.values().all(|&count| count <= 2));
    assert_eq!(
        edges.len(),
        unique.len().saturating_sub(1) + d.triangles.len(),
        "Euler: {p:?}"
    );
    for &(a, b) in &d.edges {
        assert!(a < b && unique.contains(&a) && unique.contains(&b));
        for &v in &unique {
            if v == a || v == b {
                continue;
            }
            let on_segment = orient(p[a], p[b], p[v]) == 0
                && p[a].0.min(p[b].0) <= p[v].0
                && p[v].0 <= p[a].0.max(p[b].0)
                && p[a].1.min(p[b].1) <= p[v].1
                && p[v].1 <= p[a].1.max(p[b].1);
            assert!(!on_segment, "edge skips a vertex: {p:?}");
        }
        for &(c, e) in &d.edges {
            if a == c || a == e || b == c || b == e {
                continue;
            }
            let ab_c = orient(p[a], p[b], p[c]).signum();
            let ab_e = orient(p[a], p[b], p[e]).signum();
            let ce_a = orient(p[c], p[e], p[a]).signum();
            let ce_b = orient(p[c], p[e], p[b]).signum();
            assert!(
                !(ab_c * ab_e < 0 && ce_a * ce_b < 0),
                "crossing edges: {p:?}"
            );
        }
    }
    let mst = euclidean_mst(p);
    assert_eq!(mst.len(), p.len().saturating_sub(1));
    let mut adjacency = vec![vec![]; p.len()];
    for &(a, b) in &mst {
        adjacency[a].push(b);
        adjacency[b].push(a);
    }
    if !p.is_empty() {
        let mut seen = vec![false; p.len()];
        let mut stack = vec![0];
        seen[0] = true;
        while let Some(v) = stack.pop() {
            for &u in &adjacency[v] {
                if !seen[u] {
                    seen[u] = true;
                    stack.push(u);
                }
            }
        }
        assert!(seen.iter().all(|&s| s), "disconnected MST: {p:?}");
    }
    let mut weights: Vec<_> = mst.iter().map(|&(a, b)| distance(p[a], p[b])).collect();
    weights.sort_unstable();
    assert_eq!(weights, prim(p), "MST: {p:?}");
}
fn random(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}
#[test]
fn boundaries_and_degeneracies() {
    for p in [
        vec![],
        vec![(1, 2)],
        vec![(2, 2); 20],
        vec![(0, 0), (1, 0)],
        vec![(0, 0), (1, 0), (0, 1)],
        vec![(0, 1), (1, 0), (0, 0)],
        vec![(0, 0), (1, 0), (1, 1), (0, 1)],
        vec![
            (0, 5),
            (3, 4),
            (4, 3),
            (5, 0),
            (4, -3),
            (3, -4),
            (0, -5),
            (-3, -4),
            (-4, -3),
            (-5, 0),
            (-4, 3),
            (-3, 4),
        ],
        (0..70).map(|i| (i * 3, i * -7)).collect(),
        (0..70).map(|i| (5, i)).collect(),
        (0..70).map(|i| (i, 5)).collect(),
        (0..70).map(|i| (i, i % 2)).collect(),
        vec![
            (0, 0),
            (1_000_000_000, 0),
            (1_000_000_000, 1_000_000_000),
            (0, 1_000_000_000),
            (500_000_000, 500_000_001),
        ],
    ] {
        check(&p);
    }
    for offset in [i64::MIN, i64::MAX - 1_000_000_000] {
        check(&[
            (offset, offset),
            (offset + 1_000_000_000, offset),
            (offset, offset + 1_000_000_000),
            (offset + 12345, offset + 54321),
        ]);
    }
}
#[test]
fn exhaustive_small_grid_and_permutations() {
    let grid: Vec<_> = (0..9).map(|i| (i / 3, i % 3)).collect();
    let mut state = 777;
    for mask in 0..512 {
        let mut p: Vec<_> = (0..9)
            .filter(|&i| mask >> i & 1 != 0)
            .map(|i| grid[i])
            .collect();
        check(&p);
        for i in (1..p.len()).rev() {
            let j = random(&mut state) as usize % (i + 1);
            p.swap(i, j);
        }
        check(&p);
    }
}
#[test]
fn differential_random_and_structured() {
    let mut state = 0x9187_abcd_7821;
    for case in 0..1500 {
        let n = (random(&mut state) % 55) as usize;
        let p: Vec<_> = (0..n)
            .map(|_| {
                let x = (random(&mut state) % 20001) as i64 - 10000;
                let y = (random(&mut state) % 20001) as i64 - 10000;
                match case % 7 {
                    0 => (x, y),
                    1 => (x % 6, y % 6),
                    2 => (x, y % 2),
                    3 => (x % 2, y),
                    4 => (x, x * 2),
                    5 => (x * 40000, y * 40000),
                    _ => (x, x * x),
                }
            })
            .collect();
        check(&p);
    }
}
#[test]
fn coordinate_range_rejected() {
    for p in [
        vec![(0, 0), (1_000_000_001, 0)],
        vec![(i64::MIN, 0), (i64::MAX, 0)],
        vec![(0, 0), (0, 1_000_000_001)],
    ] {
        assert!(std::panic::catch_unwind(|| DelaunayTriangulation::new(&p)).is_err());
    }
}

#[test]
#[ignore = "20万点の構造的ケース。--release -- --ignored --nocapture で実行"]
fn maximum_scale_structured() {
    let n = 200_000;
    for shape in 0..4 {
        let p: Vec<P> = (0..n)
            .map(|i| match shape {
                0 => (i as i64, 3 * i as i64),
                1 => ((i / 400) as i64, (i % 400) as i64),
                2 => (i as i64, (i % 2) as i64),
                _ => {
                    let angle = std::f64::consts::TAU * i as f64 / n as f64;
                    (
                        (100_000_000.0 * angle.cos()).round() as i64,
                        (100_000_000.0 * angle.sin()).round() as i64,
                    )
                }
            })
            .collect();
        let start = std::time::Instant::now();
        let d = DelaunayTriangulation::new(&p);
        assert!(d.duplicate_edges.is_empty());
        assert_eq!(d.edges.len(), n - 1 + d.triangles.len());
        let mut faces: BTreeMap<_, Vec<_>> = BTreeMap::new();
        for &[a, b, c] in &d.triangles {
            assert!(orient(p[a], p[b], p[c]) > 0);
            for (u, v, opposite) in [(a, b, c), (b, c, a), (c, a, b)] {
                faces
                    .entry((u.min(v), u.max(v)))
                    .or_default()
                    .push((u, v, opposite));
            }
        }
        for face in faces.values() {
            assert!(face.len() <= 2);
            if face.len() == 2 {
                let (a, b, c) = face[0];
                let (u, v, d) = face[1];
                assert_eq!((a, b), (v, u));
                assert!(circle(p[a], p[b], p[c], p[d]) <= 0);
            }
        }
        let mst = euclidean_mst(&p);
        assert_eq!(mst.len(), n - 1);
        let mut adjacency = vec![vec![]; n];
        for &(a, b) in &mst {
            adjacency[a].push(b);
            adjacency[b].push(a);
            if shape < 3 {
                assert_eq!(distance(p[a], p[b]), [10, 1, 2][shape]);
            }
        }
        let mut seen = vec![false; n];
        let mut stack = vec![0];
        seen[0] = true;
        while let Some(v) = stack.pop() {
            for &u in &adjacency[v] {
                if !seen[u] {
                    seen[u] = true;
                    stack.push(u);
                }
            }
        }
        assert!(seen.iter().all(|&s| s));
        println!(
            "shape {shape}: {n} unique points, {} edges, {} triangles, checked in {:?}",
            d.edges.len(),
            d.triangles.len(),
            start.elapsed()
        );
    }
}
