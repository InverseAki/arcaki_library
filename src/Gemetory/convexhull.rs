// Basic/ratio.rs と Gemetory/geometry.rs を同じスコープに用意する。
// 凸包は重複点・辺途中の点を除く。整数の向き判定は積を作らない厳密比較。

fn hull_turn<T: GeometryCoordinate>(a: Point<T>, b: Point<T>, c: Point<T>) -> std::cmp::Ordering {
    let (u, v) = (b.difference(a), c.difference(a));
    T::Wide::cmp_products(u.x, v.y, u.y, v.x)
}

/// 上側・下側の凸包。両方とも辞書順最小点から最大点へ向かい、両端を含む。
/// 空は([],[])、1点は両側にその点、一直線では両端だけ。O(N log N)。
pub fn convex_hull_chains<T: GeometryCoordinate>(
    points: &[Point<T>],
) -> (Vec<Point<T>>, Vec<Point<T>>) {
    let mut points = points.to_vec();
    for p in &points {
        p.wide();
    }
    points.sort_unstable();
    points.dedup();
    hull_chains_from_sorted(&points)
}

// 辞書順に整列・重複除去済みの点列から、上下の鎖を線形時間で構築。
fn hull_chains_from_sorted<T: GeometryCoordinate>(
    points: &[Point<T>],
) -> (Vec<Point<T>>, Vec<Point<T>>) {
    let (mut upper, mut lower) = (Vec::new(), Vec::new());
    for &p in points {
        while upper.len() >= 2
            && !hull_turn(upper[upper.len() - 2], upper[upper.len() - 1], p).is_lt()
        {
            upper.pop();
        }
        while lower.len() >= 2
            && !hull_turn(lower[lower.len() - 2], lower[lower.len() - 1], p).is_gt()
        {
            lower.pop();
        }
        upper.push(p);
        lower.push(p);
    }
    (upper, lower)
}

/// 反時計回りに一周する凸包。先頭は辞書順最小点で、末尾に先頭を重複しない。
/// 空/1点/線分はそれぞれ0/1/2頂点。O(N log N)。
pub fn convex_hull_polygon<T: GeometryCoordinate>(points: &[Point<T>]) -> Vec<Point<T>> {
    let (upper, lower) = convex_hull_chains(points);
    hull_join_chains(upper, lower)
}
fn hull_join_chains<T: GeometryCoordinate>(
    upper: Vec<Point<T>>,
    mut lower: Vec<Point<T>>,
) -> Vec<Point<T>> {
    if upper.len() > 2 {
        lower.extend(upper[1..upper.len() - 1].iter().rev().copied());
    }
    lower
}

/// 旧tuple入力APIの互換入口。判定は新しいPoint版と共通。
pub fn convex_hull(points: &[(i64, i64)]) -> (Vec<(i64, i64)>, Vec<(i64, i64)>) {
    let points: Vec<_> = points.iter().map(|&(x, y)| IntPoint::new(x, y)).collect();
    let (upper, lower) = convex_hull_chains(&points);
    let tuples = |points: Vec<IntPoint>| points.into_iter().map(|p| (p.x, p.y)).collect();
    (tuples(upper), tuples(lower))
}
/// 旧関数名。整数の差・積はi128へ拡張し、面積の2倍を返す。
pub fn cross_product(a: (i64, i64), b: (i64, i64), c: (i64, i64)) -> i128 {
    signed_triangle_area2(
        IntPoint::new(a.0, a.1),
        IntPoint::new(b.0, b.1),
        IntPoint::new(c.0, c.1),
    )
}

/// 正規化した凸包。構築後は厳密凸・反時計回り・先頭は辞書順最小点。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConvexHull<T: GeometryCoordinate = i64> {
    vertices: Vec<Point<T>>,
}
impl<T: GeometryCoordinate> ConvexHull<T> {
    pub fn new(points: &[Point<T>]) -> Self {
        Self {
            vertices: convex_hull_polygon(points),
        }
    }
    pub fn vertices(&self) -> &[Point<T>] {
        &self.vertices
    }
    pub fn into_vertices(self) -> Vec<Point<T>> {
        self.vertices
    }
    pub fn len(&self) -> usize {
        self.vertices.len()
    }
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }

    /// 2つの凸包を包む最小の凸包 conv(A ∪ B)。座標型を保持。O(N+M)。
    /// 頂点を再ソートせず上下の単調鎖をマージする。空との合成は元の凸包。
    pub fn merge(&self, other: &Self) -> Self {
        let a = hull_sorted_vertices(&self.vertices);
        let b = hull_sorted_vertices(&other.vertices);
        let points = hull_merge_sorted(a.into_iter(), b.into_iter());
        let (upper, lower) = hull_chains_from_sorted(&points);
        Self {
            vertices: hull_join_chains(upper, lower),
        }
    }
    /// 凸包の面積の2倍。整数座標はi128、有理数座標はRatio128。O(N)。
    pub fn area2(&self) -> T::Wide {
        signed_polygon_area2(&self.vertices).abs()
    }
    /// 凸包の厳密な面積。空・点・線分は0。O(N)。
    pub fn area(&self) -> Ratio128 {
        self.area2().rational() / 2
    }
    pub fn area_f64(&self) -> f64 {
        self.area().to_f64()
    }
    /// 同じ境界を持つ反時計回りの多角形へコピーする。O(N)。
    pub fn to_polygon(&self) -> Polygon<T> {
        Polygon::new(&self.vertices)
    }
    /// 凸包全体のスカラー倍。負の係数は原点対称、零は原点1点（空は空）。O(N)。
    pub fn scaled<C: HullCoefficient<T>>(&self, coefficient: C) -> ConvexHull<C::Output> {
        coefficient.validate();
        let vertices = self
            .vertices
            .iter()
            .map(|&p| coefficient.scale(p))
            .collect();
        ConvexHull {
            vertices: hull_normalize_cycle(vertices),
        }
    }
    /// alpha*A + beta*B = {alpha*a + beta*b} の凸包。O(N+M)。
    pub fn weighted_minkowski_sum<C: HullCoefficient<T>>(
        &self,
        alpha: C,
        other: &Self,
        beta: C,
    ) -> ConvexHull<C::Output> {
        let (a, b) = (self.scaled(alpha), other.scaled(beta));
        hull_merge_minkowski(&a, &b)
    }
    /// A+Bの凸包。i64座標はi128、有理数座標はRatio128に拡張。O(N+M)。
    pub fn minkowski_sum(&self, other: &Self) -> ConvexHull<T::Wide>
    where
        i64: HullCoefficient<T, Output = T::Wide>,
    {
        self.weighted_minkowski_sum(1i64, other, 1i64)
    }
}

// 一周列の上下鎖は、それぞれ辞書順に単調。最右点で分割してマージする。
fn hull_sorted_vertices<T: GeometryCoordinate>(points: &[Point<T>]) -> Vec<Point<T>> {
    let Some((right, _)) = points.iter().enumerate().max_by_key(|(_, p)| **p) else {
        return Vec::new();
    };
    let lower = points[..=right].iter().copied();
    let upper = std::iter::once(points[0]).chain(points[right..].iter().rev().copied());
    hull_merge_sorted(lower, upper)
}
fn hull_merge_sorted<T: GeometryCoordinate>(
    left: impl Iterator<Item = Point<T>>,
    right: impl Iterator<Item = Point<T>>,
) -> Vec<Point<T>> {
    let (mut left, mut right) = (left.peekable(), right.peekable());
    let mut points = Vec::new();
    loop {
        let point = match (left.peek(), right.peek()) {
            (Some(a), Some(b)) => {
                if a <= b {
                    left.next().unwrap()
                } else {
                    right.next().unwrap()
                }
            }
            (Some(_), None) => left.next().unwrap(),
            (None, Some(_)) => right.next().unwrap(),
            (None, None) => break,
        };
        if points.last() != Some(&point) {
            points.push(point);
        }
    }
    points
}

/// 整数係数を整数座標には整数で、有理数座標には有理数で掛けるための変換。
pub trait HullScalar: GeometryNumber {
    fn from_integer(value: i128) -> Self;
}
impl HullScalar for i128 {
    fn from_integer(value: i128) -> Self {
        value
    }
}
impl HullScalar for Ratio128 {
    fn from_integer(value: i128) -> Self {
        Ratio128::int(value)
    }
}
/// 係数から返り値の型を決める。整数係数なら座標のWide、有理数係数ならRatio128。
pub trait HullCoefficient<T: GeometryCoordinate>: Copy {
    type Output: GeometryNumber;
    fn validate(self) {}
    fn scale(self, point: Point<T>) -> Point<Self::Output>;
}
impl<T: GeometryCoordinate> HullCoefficient<T> for i64
where
    T::Wide: HullScalar,
{
    type Output = T::Wide;
    fn scale(self, point: Point<T>) -> Point<Self::Output> {
        point.scaled(T::Wide::from_integer(self as i128))
    }
}
impl<T: GeometryCoordinate> HullCoefficient<T> for i128
where
    T::Wide: HullScalar,
{
    type Output = T::Wide;
    fn scale(self, point: Point<T>) -> Point<Self::Output> {
        point.scaled(T::Wide::from_integer(self))
    }
}
impl<T: GeometryCoordinate> HullCoefficient<T> for Ratio {
    type Output = Ratio128;
    fn validate(self) {
        self.wide();
    }
    fn scale(self, point: Point<T>) -> RationalPoint {
        point.to_rational().scaled(self.wide())
    }
}
impl<T: GeometryCoordinate> HullCoefficient<T> for Ratio128 {
    type Output = Ratio128;
    fn validate(self) {
        self.wide();
    }
    fn scale(self, point: Point<T>) -> RationalPoint {
        point.to_rational().scaled(self.wide())
    }
}

// 既に凸な一周列のみ。縮退・同方向辺を線形時間で除去し、辞書順最小点へ回転。
fn hull_normalize_cycle<T: GeometryNumber>(mut points: Vec<Point<T>>) -> Vec<Point<T>> {
    points.dedup();
    if points.len() > 1 && points.first() == points.last() {
        points.pop();
    }
    let mut stack = Vec::with_capacity(points.len());
    for p in points {
        while stack.len() >= 2
            && hull_turn(stack[stack.len() - 2], stack[stack.len() - 1], p).is_eq()
        {
            stack.pop();
        }
        stack.push(p);
    }
    while stack.len() > 2
        && hull_turn(stack[stack.len() - 2], stack[stack.len() - 1], stack[0]).is_eq()
    {
        stack.pop();
    }
    let mut start = 0;
    while stack.len() - start > 2
        && hull_turn(stack[stack.len() - 1], stack[start], stack[start + 1]).is_eq()
    {
        start += 1;
    }
    if start > 0 {
        stack.drain(..start);
    }
    if let Some((index, _)) = stack.iter().enumerate().min_by_key(|(_, p)| **p) {
        stack.rotate_left(index);
    }
    stack
}

// 辺の角度[0,2pi)が昇順になるよう、y最小・同値ならx最小の点から始める。
fn hull_edge_start<T: GeometryCoordinate>(points: &[Point<T>]) -> usize {
    points
        .iter()
        .enumerate()
        .min_by_key(|(_, p)| (p.y, p.x))
        .unwrap()
        .0
}
fn hull_merge_minkowski<T: GeometryNumber>(a: &ConvexHull<T>, b: &ConvexHull<T>) -> ConvexHull<T> {
    if a.is_empty() || b.is_empty() {
        return ConvexHull {
            vertices: Vec::new(),
        };
    }
    if a.len() == 1 {
        return ConvexHull {
            vertices: b.vertices.iter().map(|&p| p + a.vertices[0]).collect(),
        };
    }
    if b.len() == 1 {
        return ConvexHull {
            vertices: a.vertices.iter().map(|&p| p + b.vertices[0]).collect(),
        };
    }
    let (sa, sb) = (hull_edge_start(&a.vertices), hull_edge_start(&b.vertices));
    let (mut i, mut j) = (0, 0);
    let mut vertices = Vec::with_capacity(a.len() + b.len());
    while i < a.len() || j < b.len() {
        // 累積した辺の和ではなく、現在の頂点対の和を取る（誤差・中間値の肥大を防ぐ）。
        vertices.push(a.vertices[(sa + i) % a.len()] + b.vertices[(sb + j) % b.len()]);
        let order = if i == a.len() {
            std::cmp::Ordering::Greater
        } else if j == b.len() {
            std::cmp::Ordering::Less
        } else {
            let u = a.vertices[(sa + i + 1) % a.len()] - a.vertices[(sa + i) % a.len()];
            let v = b.vertices[(sb + j + 1) % b.len()] - b.vertices[(sb + j) % b.len()];
            u.cmp_angle(v).expect("zero edge in convex hull")
        };
        if !order.is_gt() {
            i += 1;
        }
        if !order.is_lt() {
            j += 1;
        }
    }
    ConvexHull {
        vertices: hull_normalize_cycle(vertices),
    }
}
