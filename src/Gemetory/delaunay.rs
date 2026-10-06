/// Integer Delaunay triangulation (deterministic O(n log n) time, O(n) space).
///
/// All indices refer to the original input. Coincident points are represented by
/// their smallest input index. `edges` has no zero-length edges; `duplicate_edges`
/// connects each other coincident point to its representative. Collinear points
/// produce a sorted chain and no triangles. Cocircular faces are triangulated
/// deterministically, with an arbitrary choice of diagonal.
///
/// Each coordinate's range (max - min) must be at most 1_000_000_000. Absolute
/// coordinates may be any i64. This bound makes every i128 predicate exact;
/// invalid input panics in both debug and release. No floating point is used.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DelaunayTriangulation {
    pub edges: Vec<(usize, usize)>,
    /// Counterclockwise triangles, rotated to start at the smallest input index.
    pub triangles: Vec<[usize; 3]>,
    pub duplicate_edges: Vec<(usize, usize)>,
    /// representatives[i] is the smallest index having the same coordinates as i.
    pub representatives: Vec<usize>,
}

impl DelaunayTriangulation {
    pub fn new(points: &[(i64, i64)]) -> Self {
        let n = points.len();
        if let Some(&(x, y)) = points.first() {
            let (mut xmin, mut xmax, mut ymin, mut ymax) = (x, x, y, y);
            for &(x, y) in points {
                xmin = xmin.min(x);
                xmax = xmax.max(x);
                ymin = ymin.min(y);
                ymax = ymax.max(y);
            }
            assert!(
                xmax as i128 - xmin as i128 <= 1_000_000_000
                    && ymax as i128 - ymin as i128 <= 1_000_000_000,
                "Delaunay coordinate ranges must be at most 1_000_000_000"
            );
        }
        let mut order: Vec<_> = (0..n).collect();
        order.sort_unstable_by_key(|&i| (points[i], i));
        let mut unique = Vec::new();
        let mut ids = Vec::new();
        let mut representatives = vec![0; n];
        let mut duplicate_edges = Vec::new();
        for i in order {
            if unique.last() == Some(&points[i]) {
                let r = *ids.last().unwrap();
                representatives[i] = r;
                duplicate_edges.push((r, i));
            } else {
                unique.push(points[i]);
                ids.push(i);
                representatives[i] = i;
            }
        }
        let mut result = Self {
            edges: Vec::new(),
            triangles: Vec::new(),
            duplicate_edges,
            representatives,
        };
        if unique.len() < 2 {
            return result;
        }
        let mut mesh = DelaunayMesh {
            points: unique,
            quads: Vec::new(),
            free: Vec::new(),
        };
        mesh.build(0, mesh.points.len());
        for (q, quad) in mesh.quads.iter().enumerate() {
            if !quad.alive {
                continue;
            }
            let a = ids[mesh.org(4 * q)];
            let b = ids[mesh.dest(4 * q)];
            result.edges.push((a.min(b), a.max(b)));
        }
        // Each directed primal edge belongs to exactly one left face. The outer
        // face has clockwise orientation, including when it has three edges.
        let mut visited = vec![false; mesh.quads.len() * 2];
        for q in 0..mesh.quads.len() {
            if !mesh.quads[q].alive {
                continue;
            }
            for direction in [0, 2] {
                let start = 4 * q + direction;
                if visited[start / 2] {
                    continue;
                }
                let mut e = start;
                let mut vertices = [0; 3];
                let mut size = 0;
                loop {
                    visited[e / 2] = true;
                    if size < 3 {
                        vertices[size] = mesh.org(e);
                    }
                    size += 1;
                    e = mesh.lnext(e);
                    if e == start {
                        break;
                    }
                }
                if size == 3 && mesh.orient(vertices[0], vertices[1], vertices[2]) > 0 {
                    let mut tri = vertices.map(|v| ids[v]);
                    let first = (0..3).min_by_key(|&i| tri[i]).unwrap();
                    tri.rotate_left(first);
                    result.triangles.push(tri);
                }
            }
        }
        result.edges.sort_unstable();
        result.triangles.sort_unstable();
        result
    }
}

/// Returns the n-1 edges of a Euclidean MST, in original input indices.
/// Empty input returns []. Same preconditions and complexity as triangulation.
/// Squared distances suffice for Kruskal: taking sqrt preserves edge order.
pub fn euclidean_mst(points: &[(i64, i64)]) -> Vec<(usize, usize)> {
    let triangulation = DelaunayTriangulation::new(points);
    let mut candidates: Vec<_> = triangulation
        .edges
        .iter()
        .map(|&(a, b)| {
            let dx = points[a].0 as i128 - points[b].0 as i128;
            let dy = points[a].1 as i128 - points[b].1 as i128;
            ((dx * dx + dy * dy) as u64, a, b)
        })
        .collect();
    candidates.sort_unstable();
    let mut parent: Vec<_> = (0..points.len()).collect();
    let mut size = vec![1usize; points.len()];
    fn root(parent: &mut [usize], mut v: usize) -> usize {
        while parent[v] != v {
            parent[v] = parent[parent[v]];
            v = parent[v];
        }
        v
    }
    let mut mst = triangulation.duplicate_edges;
    for &(a, b) in &mst {
        parent[b] = a;
        size[a] += 1;
    }
    for (_, a, b) in candidates {
        let (mut u, mut v) = (root(&mut parent, a), root(&mut parent, b));
        if u == v {
            continue;
        }
        if size[u] < size[v] {
            std::mem::swap(&mut u, &mut v);
        }
        parent[v] = u;
        size[u] += size[v];
        mst.push((a, b));
        if mst.len() == points.len() - 1 {
            break;
        }
    }
    debug_assert_eq!(mst.len(), points.len().saturating_sub(1));
    mst
}

// Quad-edge topology: e and e^2 are primal edges; the other two are dual.
// Recycle deleted quads so merging does not retain O(n log n) allocations.
struct DelaunayQuad {
    next: [usize; 4],
    vertices: [usize; 2],
    alive: bool,
}
struct DelaunayMesh {
    points: Vec<(i64, i64)>,
    quads: Vec<DelaunayQuad>,
    free: Vec<usize>,
}
impl DelaunayMesh {
    fn rot(e: usize) -> usize {
        (e & !3) | ((e + 1) & 3)
    }
    fn invrot(e: usize) -> usize {
        (e & !3) | ((e + 3) & 3)
    }
    fn sym(e: usize) -> usize {
        e ^ 2
    }
    fn org(&self, e: usize) -> usize {
        self.quads[e / 4].vertices[(e & 3) / 2]
    }
    fn dest(&self, e: usize) -> usize {
        self.org(Self::sym(e))
    }
    fn onext(&self, e: usize) -> usize {
        self.quads[e / 4].next[e & 3]
    }
    fn oprev(&self, e: usize) -> usize {
        Self::rot(self.onext(Self::rot(e)))
    }
    fn lnext(&self, e: usize) -> usize {
        Self::rot(self.onext(Self::invrot(e)))
    }
    fn rprev(&self, e: usize) -> usize {
        self.onext(Self::sym(e))
    }
    fn set_next(&mut self, e: usize, next: usize) {
        self.quads[e / 4].next[e & 3] = next;
    }
    fn make_edge(&mut self, a: usize, b: usize) -> usize {
        let q = self.free.pop().unwrap_or(self.quads.len());
        let e = q * 4;
        let quad = DelaunayQuad {
            next: [e, e + 3, e + 2, e + 1],
            vertices: [a, b],
            alive: true,
        };
        if q == self.quads.len() {
            self.quads.push(quad);
        } else {
            self.quads[q] = quad;
        }
        e
    }
    fn splice(&mut self, a: usize, b: usize) {
        let alpha = Self::rot(self.onext(a));
        let beta = Self::rot(self.onext(b));
        let (an, bn, alphan, betan) = (
            self.onext(a),
            self.onext(b),
            self.onext(alpha),
            self.onext(beta),
        );
        self.set_next(a, bn);
        self.set_next(b, an);
        self.set_next(alpha, betan);
        self.set_next(beta, alphan);
    }
    fn connect(&mut self, a: usize, b: usize) -> usize {
        let e = self.make_edge(self.dest(a), self.org(b));
        self.splice(e, self.lnext(a));
        self.splice(Self::sym(e), b);
        e
    }
    fn delete(&mut self, e: usize) {
        self.splice(e, self.oprev(e));
        let rev = Self::sym(e);
        self.splice(rev, self.oprev(rev));
        self.quads[e / 4].alive = false;
        self.free.push(e / 4);
    }
    fn orient(&self, a: usize, b: usize, c: usize) -> i128 {
        let (ax, ay) = self.points[a];
        let (bx, by) = self.points[b];
        let (cx, cy) = self.points[c];
        (bx as i128 - ax as i128) * (cy as i128 - ay as i128)
            - (by as i128 - ay as i128) * (cx as i128 - ax as i128)
    }
    fn left_of(&self, p: usize, e: usize) -> bool {
        self.orient(self.org(e), self.dest(e), p) > 0
    }
    fn right_of(&self, p: usize, e: usize) -> bool {
        self.orient(self.org(e), self.dest(e), p) < 0
    }
    // Positive precisely when d is inside the oriented CCW circle abc.
    // Each difference <= B; each of the three terms <= 4 B^4.
    // With B=10^9, 12 B^4 < i128::MAX, including intermediate sums.
    fn in_circle(&self, a: usize, b: usize, c: usize, d: usize) -> bool {
        let (dx, dy) = self.points[d];
        let relative = |p: usize| {
            let (x, y) = self.points[p];
            (x as i128 - dx as i128, y as i128 - dy as i128)
        };
        let (ax, ay) = relative(a);
        let (bx, by) = relative(b);
        let (cx, cy) = relative(c);
        (ax * ax + ay * ay) * (bx * cy - by * cx)
            + (bx * bx + by * by) * (cx * ay - cy * ax)
            + (cx * cx + cy * cy) * (ax * by - ay * bx)
            > 0
    }
    // Returns the left/right outer hull edges, pointing respectively up/down.
    fn build(&mut self, lo: usize, hi: usize) -> (usize, usize) {
        if hi - lo == 2 {
            let e = self.make_edge(lo, lo + 1);
            return (e, Self::sym(e));
        }
        if hi - lo == 3 {
            let a = self.make_edge(lo, lo + 1);
            let b = self.make_edge(lo + 1, lo + 2);
            self.splice(Self::sym(a), b);
            let orientation = self.orient(lo, lo + 1, lo + 2);
            if orientation == 0 {
                return (a, Self::sym(b));
            }
            let c = self.connect(b, a);
            return if orientation > 0 {
                (a, Self::sym(b))
            } else {
                (Self::sym(c), c)
            };
        }
        let mid = (lo + hi) / 2;
        let (mut ldo, mut ldi) = self.build(lo, mid);
        let (mut rdi, mut rdo) = self.build(mid, hi);
        // Find the lower common tangent of the two convex hulls.
        loop {
            if self.left_of(self.org(rdi), ldi) {
                ldi = self.lnext(ldi);
            } else if self.right_of(self.org(ldi), rdi) {
                rdi = self.rprev(rdi);
            } else {
                break;
            }
        }
        let mut base = self.connect(Self::sym(rdi), ldi);
        if self.org(ldi) == self.org(ldo) {
            ldo = Self::sym(base);
        }
        if self.org(rdi) == self.org(rdo) {
            rdo = base;
        }
        // Grow the seam upwards, removing edges that violate the empty-circle
        // condition. Strict predicates leave ties stable for cocircular points.
        loop {
            let mut left = self.onext(Self::sym(base));
            if self.right_of(self.dest(left), base) {
                loop {
                    let next = self.onext(left);
                    if !self.in_circle(
                        self.dest(base),
                        self.org(base),
                        self.dest(left),
                        self.dest(next),
                    ) {
                        break;
                    }
                    self.delete(left);
                    left = next;
                }
            }
            let mut right = self.oprev(base);
            if self.right_of(self.dest(right), base) {
                loop {
                    let next = self.oprev(right);
                    if !self.in_circle(
                        self.dest(base),
                        self.org(base),
                        self.dest(right),
                        self.dest(next),
                    ) {
                        break;
                    }
                    self.delete(right);
                    right = next;
                }
            }
            let lv = self.right_of(self.dest(left), base);
            let rv = self.right_of(self.dest(right), base);
            if !lv && !rv {
                break;
            }
            if !lv
                || (rv
                    && self.in_circle(
                        self.dest(left),
                        self.org(left),
                        self.org(right),
                        self.dest(right),
                    ))
            {
                base = self.connect(right, Self::sym(base));
            } else {
                base = self.connect(Self::sym(base), Self::sym(left));
            }
        }
        (ldo, rdo)
    }
}
