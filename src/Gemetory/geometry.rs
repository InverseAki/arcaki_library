// 2次元の厳密幾何。Basic/ratio.rs を同じスコープにコピーする。
// 中間値はi128/Ratio128。範囲超過はreleaseでもpanic。無限座標は不可。

/// 座標の計算型。整数座標はi128へ、有理数座標はRatio128へ拡張する。
pub trait GeometryCoordinate: Copy + Ord {
    type Wide: GeometryNumber;
    fn wide(self) -> Self::Wide;
}
/// 幾何演算用の閉じた数値型。独自実装時もオーバーフローを検出すること。
pub trait GeometryNumber: GeometryCoordinate<Wide = Self> {
    fn zero() -> Self;
    fn add(self, rhs: Self) -> Self;
    fn sub(self, rhs: Self) -> Self;
    fn mul(self, rhs: Self) -> Self;
    fn neg(self) -> Self;
    fn rational(self) -> Ratio128;
    /// a*b と c*d を比較する。整数実装は積を作らず全域に対応。
    fn cmp_products(a: Self, b: Self, c: Self, d: Self) -> std::cmp::Ordering {
        a.mul(b).cmp(&c.mul(d))
    }

    fn abs(self) -> Self {
        if self < Self::zero() {
            self.neg()
        } else {
            self
        }
    }
    fn sign(self) -> i8 {
        match self.cmp(&Self::zero()) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        }
    }
}
impl GeometryCoordinate for i64 {
    type Wide = i128;
    fn wide(self) -> i128 {
        self as i128
    }
}
impl GeometryCoordinate for i128 {
    type Wide = i128;
    fn wide(self) -> i128 {
        self
    }
}
impl GeometryNumber for i128 {
    fn cmp_products(a: Self, b: Self, c: Self, d: Self) -> std::cmp::Ordering {
        geometry_cmp_integer_products(a, b, c, d)
    }

    fn zero() -> Self {
        0
    }
    fn add(self, rhs: Self) -> Self {
        self.checked_add(rhs).expect("geometry overflow")
    }
    fn sub(self, rhs: Self) -> Self {
        self.checked_sub(rhs).expect("geometry overflow")
    }
    fn mul(self, rhs: Self) -> Self {
        self.checked_mul(rhs).expect("geometry overflow")
    }
    fn neg(self) -> Self {
        self.checked_neg().expect("geometry overflow")
    }
    fn rational(self) -> Ratio128 {
        Ratio128::int(self)
    }
}
impl GeometryCoordinate for Ratio {
    type Wide = Ratio128;
    fn wide(self) -> Ratio128 {
        assert!(self.is_finite(), "infinite coordinate");
        Ratio128::from_fraction(*self.numerator() as i128, *self.denominator() as i128)
    }
}
impl GeometryCoordinate for Ratio128 {
    type Wide = Ratio128;
    fn wide(self) -> Self {
        assert!(self.is_finite(), "infinite coordinate");
        self
    }
}
impl GeometryNumber for Ratio128 {
    fn zero() -> Self {
        Ratio128::zero()
    }
    fn add(self, rhs: Self) -> Self {
        self + rhs
    }
    fn sub(self, rhs: Self) -> Self {
        self - rhs
    }
    fn mul(self, rhs: Self) -> Self {
        self * rhs
    }
    fn neg(self) -> Self {
        -self
    }
    fn rational(self) -> Self {
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Point<T = i64> {
    pub x: T,
    pub y: T,
}
pub type IntPoint = Point<i64>;
pub type RationalPoint = Point<Ratio128>;
impl<T: GeometryCoordinate> Point<T> {
    pub fn new(x: T, y: T) -> Self {
        x.wide();
        y.wide();
        Self { x, y }
    }
    pub fn wide(self) -> Point<T::Wide> {
        Point {
            x: self.x.wide(),
            y: self.y.wide(),
        }
    }
    pub fn to_rational(self) -> RationalPoint {
        let p = self.wide();
        Point::new(p.x.rational(), p.y.rational())
    }
    /// 加減算・スカラー倍は拡張型で返す。i64の引き算も先にi128へ拡張。
    pub fn translated(self, v: Self) -> Point<T::Wide> {
        let (p, v) = (self.wide(), v.wide());
        Point::new(p.x.add(v.x), p.y.add(v.y))
    }
    pub fn difference(self, other: Self) -> Point<T::Wide> {
        let (p, q) = (self.wide(), other.wide());
        Point::new(p.x.sub(q.x), p.y.sub(q.y))
    }
    pub fn scaled(self, k: T::Wide) -> Point<T::Wide> {
        let p = self.wide();
        let k = k.wide();
        Point::new(p.x.mul(k), p.y.mul(k))
    }
    pub fn rotated90(self) -> Point<T::Wide> {
        let p = self.wide();
        Point::new(p.y.neg(), p.x)
    }
    pub fn dot(self, other: Self) -> T::Wide {
        let (p, q) = (self.wide(), other.wide());
        p.x.mul(q.x).add(p.y.mul(q.y))
    }
    pub fn cross(self, other: Self) -> T::Wide {
        let (p, q) = (self.wide(), other.wide());
        p.x.mul(q.y).sub(p.y.mul(q.x))
    }
    pub fn norm_squared(self) -> T::Wide {
        self.dot(self)
    }
    pub fn norm(self) -> f64 {
        self.norm_squared().rational().to_f64().sqrt()
    }
    pub fn distance_squared(self, other: Self) -> T::Wide {
        self.difference(other).norm_squared()
    }
    pub fn distance(self, other: Self) -> f64 {
        self.distance_squared(other).rational().to_f64().sqrt()
    }
    pub fn manhattan_distance(self, other: Self) -> T::Wide {
        let p = self.difference(other);
        p.x.abs().add(p.y.abs())
    }
    pub fn midpoint(self, other: Self) -> RationalPoint {
        let (p, q) = (self.to_rational(), other.to_rational());
        Point::new((p.x + q.x) / 2, (p.y + q.y) / 2)
    }
    /// 角度は[-pi, pi]。零ベクトルはNone。
    pub fn angle(self) -> Option<f64> {
        let p = self.to_rational();
        if p.x.is_zero() && p.y.is_zero() {
            None
        } else {
            Some(p.y.to_f64().atan2(p.x.to_f64()))
        }
    }
    /// 2ベクトル間の符号付き角度[-pi, pi]。零ベクトルならNone。
    pub fn angle_to(self, other: Self) -> Option<f64> {
        let delta = other.angle()? - self.angle()?;
        Some(delta.sin().atan2(delta.cos()))
    }

    /// +x軸から反時計回りの偏角[0, 2pi)を厳密比較。零ベクトルはNone。
    pub fn cmp_angle(self, other: Self) -> Option<std::cmp::Ordering> {
        let (p, q) = (self.wide(), other.wide());
        let z = T::Wide::zero();
        if (p.x == z && p.y == z) || (q.x == z && q.y == z) {
            return None;
        }
        let half = |p: Point<T::Wide>| p.y < z || (p.y == z && p.x < z);
        let order = half(p).cmp(&half(q));
        Some(if order.is_eq() {
            T::Wide::cmp_products(p.x, q.y, p.y, q.x).reverse()
        } else {
            order
        })
    }
    pub fn is_parallel(self, other: Self) -> bool {
        self.cross(other) == T::Wide::zero()
    }
    pub fn is_perpendicular(self, other: Self) -> bool {
        self.dot(other) == T::Wide::zero()
    }
    /// 整数の除算も切り捨てず有理数で返す。
    pub fn divided(self, divisor: T) -> RationalPoint {
        let divisor = divisor.wide().rational();
        assert!(!divisor.is_zero(), "zero divisor");
        let p = self.to_rational();
        Point::new(p.x / divisor, p.y / divisor)
    }
    /// 任意角回転だけはf64座標を返す（厳密値でない）。
    pub fn rotated(self, radians: f64) -> Point<f64> {
        let p = self.to_rational();
        let (s, c) = radians.sin_cos();
        Point {
            x: p.x.to_f64() * c - p.y.to_f64() * s,
            y: p.x.to_f64() * s + p.y.to_f64() * c,
        }
    }
}
impl<T: GeometryCoordinate> std::ops::Add for Point<T> {
    type Output = Point<T::Wide>;
    fn add(self, rhs: Self) -> Self::Output {
        self.translated(rhs)
    }
}
impl<T: GeometryCoordinate> std::ops::Sub for Point<T> {
    type Output = Point<T::Wide>;
    fn sub(self, rhs: Self) -> Self::Output {
        self.difference(rhs)
    }
}
impl<T: GeometryCoordinate> std::ops::Neg for Point<T> {
    type Output = Point<T::Wide>;
    fn neg(self) -> Self::Output {
        let p = self.wide();
        Point::new(p.x.neg(), p.y.neg())
    }
}
impl<T: GeometryCoordinate> std::ops::Mul<T> for Point<T> {
    type Output = Point<T::Wide>;
    fn mul(self, rhs: T) -> Self::Output {
        self.scaled(rhs.wide())
    }
}
impl<T: GeometryCoordinate> std::ops::Div<T> for Point<T> {
    type Output = RationalPoint;
    fn div(self, rhs: T) -> Self::Output {
        self.divided(rhs)
    }
}

pub fn orientation<T: GeometryCoordinate>(a: Point<T>, b: Point<T>, c: Point<T>) -> i8 {
    signed_triangle_area2(a, b, c).sign()
}
pub fn signed_triangle_area2<T: GeometryCoordinate>(
    a: Point<T>,
    b: Point<T>,
    c: Point<T>,
) -> T::Wide {
    b.difference(a).cross(c.difference(a))
}
pub fn triangle_area2<T: GeometryCoordinate>(a: Point<T>, b: Point<T>, c: Point<T>) -> T::Wide {
    signed_triangle_area2(a, b, c).abs()
}
pub fn triangle_area<T: GeometryCoordinate>(a: Point<T>, b: Point<T>, c: Point<T>) -> Ratio128 {
    triangle_area2(a, b, c).rational() / 2
}
pub fn triangle_centroid<T: GeometryCoordinate>(
    a: Point<T>,
    b: Point<T>,
    c: Point<T>,
) -> RationalPoint {
    (a.to_rational() + b.to_rational() + c.to_rational()) / Ratio128::int(3)
}
/// 三角形の外心。一直線・重複点の場合はNone。
pub fn triangle_circumcenter<T: GeometryCoordinate>(
    a: Point<T>,
    b: Point<T>,
    c: Point<T>,
) -> Option<RationalPoint> {
    let (a, b, c) = (a.to_rational(), b.to_rational(), c.to_rational());
    let (u, v) = (b - a, c - a);
    let det = u.cross(v) * 2;
    if det.is_zero() {
        return None;
    }
    let (s, t) = (u.norm_squared(), v.norm_squared());
    Some(a + Point::new((s * v.y - t * u.y) / det, (u.x * t - v.x * s) / det))
}

pub fn signed_polygon_area2<T: GeometryCoordinate>(points: &[Point<T>]) -> T::Wide {
    let mut sum = T::Wide::zero();
    if points.len() < 3 {
        return sum;
    }
    for i in 0..points.len() {
        sum = sum.add(points[i].cross(points[(i + 1) % points.len()]));
    }
    sum
}
pub fn polygon_area<T: GeometryCoordinate>(points: &[Point<T>]) -> Ratio128 {
    signed_polygon_area2(points).abs().rational() / 2
}

/// 周回順の頂点列で表す単純多角形（凹を含む、穴なし）。
/// 入力が時計回りなら反転して反時計回りに揃える。自己交差の検出はしない。
/// 空・点・線分・面積0の退化多角形も許し、面積は0。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Polygon<T: GeometryCoordinate = i64> {
    vertices: Vec<Point<T>>,
}
impl<T: GeometryCoordinate> Polygon<T> {
    /// 頂点は境界を周回する順序で渡す。点集合から凸包を作る処理ではない。
    /// 連続重複・末尾の先頭点を除去。最初の頂点は維持する。O(N)。
    pub fn new(vertices: &[Point<T>]) -> Self {
        let mut vertices = vertices.to_vec();
        for p in &vertices {
            p.wide();
        }
        vertices.dedup();
        if vertices.len() > 1 && vertices.first() == vertices.last() {
            vertices.pop();
        }
        if signed_polygon_area2(&vertices) < T::Wide::zero() {
            vertices[1..].reverse();
        }
        Self { vertices }
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
    /// 面積の2倍。整数座標はi128、有理数座標はRatio128。O(N)。
    pub fn area2(&self) -> T::Wide {
        signed_polygon_area2(&self.vertices).abs()
    }
    /// 半整数も切り捨てない厳密な面積。O(N)。
    pub fn area(&self) -> Ratio128 {
        self.area2().rational() / 2
    }
    pub fn area_f64(&self) -> f64 {
        self.area().to_f64()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Ccw {
    CounterClockwise,
    Clockwise,
    Behind,
    Beyond,
    OnSegment,
}
/// a=bの場合、c=aはOnSegment、それ以外はBeyond。
pub fn ccw<T: GeometryCoordinate>(a: Point<T>, b: Point<T>, c: Point<T>) -> Ccw {
    let (u, v) = (b.difference(a), c.difference(a));
    match u.cross(v).sign() {
        1 => Ccw::CounterClockwise,
        -1 => Ccw::Clockwise,
        _ => {
            if u.dot(v) < T::Wide::zero() {
                Ccw::Behind
            } else if u.norm_squared() < v.norm_squared() {
                Ccw::Beyond
            } else {
                Ccw::OnSegment
            }
        }
    }
}

/// ax+by+c=0。gcd(|a|,|b|,|c|)=1、最初の非零係数は正。
/// フィールドを非公開にして正規形を維持する。
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Line {
    a: i128,
    b: i128,
    c: i128,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LineIntersection {
    Parallel,
    Coincident,
    Point(RationalPoint),
}
impl Line {
    /// a=b=0は直線でないためNone。
    pub fn new(a: i128, b: i128, c: i128) -> Option<Self> {
        if a == 0 && b == 0 {
            return None;
        }
        let g = geometry_gcd(
            geometry_gcd(a.unsigned_abs(), b.unsigned_abs()),
            c.unsigned_abs(),
        );
        fn reduce(v: i128, g: u128) -> i128 {
            if g > i128::MAX as u128 {
                if v == 0 {
                    0
                } else {
                    -1
                }
            } else {
                v / (g as i128)
            }
        }
        let (mut a, mut b, mut c) = (reduce(a, g), reduce(b, g), reduce(c, g));
        if a < 0 || (a == 0 && b < 0) {
            a = GeometryNumber::neg(a);
            b = GeometryNumber::neg(b);
            c = GeometryNumber::neg(c);
        }
        Some(Self { a, b, c })
    }
    /// 有理数係数の分母を払って同じ正規形にする。
    pub fn from_rational_coefficients(a: Ratio128, b: Ratio128, c: Ratio128) -> Option<Self> {
        assert!(a.is_finite() && b.is_finite() && c.is_finite());
        if a.is_zero() && b.is_zero() {
            return None;
        }
        let mut lcm = 1i128;
        for r in [a, b, c] {
            let d = *r.denominator();
            lcm = GeometryNumber::mul(lcm / (geometry_gcd(lcm as u128, d as u128) as i128), d);
        }
        Self::new(
            GeometryNumber::mul(*a.numerator(), lcm / *a.denominator()),
            GeometryNumber::mul(*b.numerator(), lcm / *b.denominator()),
            GeometryNumber::mul(*c.numerator(), lcm / *c.denominator()),
        )
    }
    /// 同一点ならNone。整数座標は整数演算で構築する。
    pub fn through<T: GeometryCoordinate>(p: Point<T>, q: Point<T>) -> Option<Self> {
        let (p, q) = (p.wide(), q.wide());
        let a = p.y.sub(q.y);
        let b = q.x.sub(p.x);
        let c = p.x.mul(q.y).sub(p.y.mul(q.x));
        Self::from_rational_coefficients(a.rational(), b.rational(), c.rational())
    }
    pub fn coefficients(self) -> (i128, i128, i128) {
        (self.a, self.b, self.c)
    }
    pub fn evaluate<T: GeometryCoordinate>(self, p: Point<T>) -> Ratio128 {
        let p = p.to_rational();
        Ratio128::int(self.a) * p.x + Ratio128::int(self.b) * p.y + Ratio128::int(self.c)
    }
    pub fn contains<T: GeometryCoordinate>(self, p: Point<T>) -> bool {
        self.evaluate(p).is_zero()
    }
    /// 正規化された法線に対する符号（throughの点順には依存しない）。
    pub fn side<T: GeometryCoordinate>(self, p: Point<T>) -> i8 {
        self.evaluate(p).signum()
    }

    /// 向きを持たない直線の角度[0, pi)を比較。平行ならEqual（cは無視）。
    /// 水平、正の傾き、垂直、負の傾きの順。i128係数の全域で積overflowなし。
    pub fn cmp_angle(self, other: Self) -> std::cmp::Ordering {
        match (self.a == 0, other.a == 0) {
            (true, true) => std::cmp::Ordering::Equal,
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            (false, false) => geometry_cmp_integer_products(self.b, other.a, other.b, self.a),
        }
    }
    /// 数値としての傾き -a/b を比較。垂直は+∞として最後に置く。
    pub fn cmp_slope(self, other: Self) -> std::cmp::Ordering {
        match (self.b == 0, other.b == 0) {
            (true, true) => std::cmp::Ordering::Equal,
            (true, false) => std::cmp::Ordering::Greater,
            (false, true) => std::cmp::Ordering::Less,
            (false, false) => {
                let c = geometry_cmp_integer_products(self.a, other.b, other.a, self.b).reverse();
                if (self.b < 0) != (other.b < 0) {
                    c.reverse()
                } else {
                    c
                }
            }
        }
    }
    pub fn is_parallel(self, other: Self) -> bool {
        self.normal().cross(other.normal()).is_zero()
    }
    pub fn is_perpendicular(self, other: Self) -> bool {
        self.normal().dot(other.normal()).is_zero()
    }
    pub fn normal(self) -> RationalPoint {
        Point::new(Ratio128::int(self.a), Ratio128::int(self.b))
    }
    pub fn direction(self) -> RationalPoint {
        self.normal().rotated90()
    }
    pub fn intersection(self, other: Self) -> LineIntersection {
        let (a, b, c) = (
            Ratio128::int(self.a),
            Ratio128::int(self.b),
            Ratio128::int(self.c),
        );
        let (d, e, f) = (
            Ratio128::int(other.a),
            Ratio128::int(other.b),
            Ratio128::int(other.c),
        );
        let det = a * e - b * d;
        if det.is_zero() {
            if self == other {
                LineIntersection::Coincident
            } else {
                LineIntersection::Parallel
            }
        } else {
            LineIntersection::Point(Point::new((b * f - c * e) / det, (c * d - a * f) / det))
        }
    }
    pub fn projection<T: GeometryCoordinate>(self, p: Point<T>) -> RationalPoint {
        let p = p.to_rational();
        let n = self.normal();
        p - n * (self.evaluate(p) / n.norm_squared())
    }
    pub fn reflection<T: GeometryCoordinate>(self, p: Point<T>) -> RationalPoint {
        let p = p.to_rational();
        self.projection(p) * Ratio128::int(2) - p
    }
    pub fn distance_squared<T: GeometryCoordinate>(self, p: Point<T>) -> Ratio128 {
        let e = self.evaluate(p);
        e * e / self.normal().norm_squared()
    }
    pub fn distance<T: GeometryCoordinate>(self, p: Point<T>) -> f64 {
        self.distance_squared(p).to_f64().sqrt()
    }
    pub fn parallel_through<T: GeometryCoordinate>(self, p: Point<T>) -> Self {
        let p = p.to_rational();
        let (a, b) = (Ratio128::int(self.a), Ratio128::int(self.b));
        Self::from_rational_coefficients(a, b, -(a * p.x + b * p.y)).unwrap()
    }
    pub fn perpendicular_through<T: GeometryCoordinate>(self, p: Point<T>) -> Self {
        let p = p.to_rational();
        let (a, b) = (Ratio128::int(self.b), -Ratio128::int(self.a));
        Self::from_rational_coefficients(a, b, -(a * p.x + b * p.y)).unwrap()
    }
}
// 符号を分け、|a|*|b| vs |c|*|d| を |a|/|c| vs |d|/|b| へ変換。
// 連分数比較なのでi128::MINの絶対値もu128で扱え、積を生成しない。
fn geometry_cmp_integer_products(a: i128, b: i128, c: i128, d: i128) -> std::cmp::Ordering {
    let sign = |a: i128, b: i128| {
        if a == 0 || b == 0 {
            0i8
        } else if (a < 0) != (b < 0) {
            -1
        } else {
            1
        }
    };
    let (s, t) = (sign(a, b), sign(c, d));
    let order = s.cmp(&t);
    if !order.is_eq() || s == 0 {
        return order;
    }
    let order = geometry_cmp_unsigned_fractions(
        a.unsigned_abs(),
        c.unsigned_abs(),
        d.unsigned_abs(),
        b.unsigned_abs(),
    );
    if s < 0 {
        order.reverse()
    } else {
        order
    }
}
fn geometry_cmp_unsigned_fractions(
    mut a: u128,
    mut b: u128,
    mut c: u128,
    mut d: u128,
) -> std::cmp::Ordering {
    let mut reverse = false;
    loop {
        let order = (a / b).cmp(&(c / d));
        if !order.is_eq() {
            return if reverse { order.reverse() } else { order };
        }
        let (r, s) = (a % b, c % d);
        if r == 0 || s == 0 {
            let order = (r != 0).cmp(&(s != 0));
            return if reverse { order.reverse() } else { order };
        }
        (a, b, c, d) = (b, r, d, s);
        reverse = !reverse;
    }
}

fn geometry_gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Segment<T = i64> {
    pub a: Point<T>,
    pub b: Point<T>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SegmentIntersection {
    None,
    Point(RationalPoint),
    Overlap(Segment<Ratio128>),
}
impl<T: GeometryCoordinate> Segment<T> {
    pub fn new(a: Point<T>, b: Point<T>) -> Self {
        a.wide();
        b.wide();
        Self { a, b }
    }

    /// 始点aから終点bへの向きの角度[0, 2pi)。零長ならNone。
    /// i64の端点差はi128へ拡張するため、i64座標全域で比較可能。
    pub fn cmp_angle(self, other: Self) -> Option<std::cmp::Ordering> {
        self.b
            .difference(self.a)
            .cmp_angle(other.b.difference(other.a))
    }
    pub fn line(self) -> Option<Line> {
        Line::through(self.a, self.b)
    }
    pub fn length_squared(self) -> T::Wide {
        self.a.distance_squared(self.b)
    }
    pub fn length(self) -> f64 {
        self.a.distance(self.b)
    }
    pub fn contains(self, p: Point<T>) -> bool {
        orientation(self.a, self.b, p) == 0
            && p.x >= self.a.x.min(self.b.x)
            && p.x <= self.a.x.max(self.b.x)
            && p.y >= self.a.y.min(self.b.y)
            && p.y <= self.a.y.max(self.b.y)
    }
    pub fn intersects(self, other: Self) -> bool {
        let (a, b, c, d) = (self.a, self.b, other.a, other.b);
        let (s, t, u, v) = (
            orientation(a, b, c),
            orientation(a, b, d),
            orientation(c, d, a),
            orientation(c, d, b),
        );
        (s * t < 0 && u * v < 0)
            || self.contains(c)
            || self.contains(d)
            || other.contains(a)
            || other.contains(b)
    }
    /// 重なりの端点は辞書順で返す。零長線分にも対応。
    pub fn intersection(self, other: Self) -> SegmentIntersection {
        if !self.intersects(other) {
            return SegmentIntersection::None;
        }
        let mut common = Vec::new();
        for p in [self.a, self.b, other.a, other.b] {
            if self.contains(p) && other.contains(p) {
                common.push(p);
            }
        }
        common.sort();
        common.dedup();
        if let Some(&p) = common.first() {
            let q = *common.last().unwrap();
            return if p == q {
                SegmentIntersection::Point(p.to_rational())
            } else {
                SegmentIntersection::Overlap(Segment::new(p.to_rational(), q.to_rational()))
            };
        }
        match self.line().unwrap().intersection(other.line().unwrap()) {
            LineIntersection::Point(p) => SegmentIntersection::Point(p),
            _ => unreachable!(),
        }
    }
    pub fn closest_point(self, p: Point<T>) -> RationalPoint {
        let (a, b, p) = (self.a.to_rational(), self.b.to_rational(), p.to_rational());
        let v = b - a;
        let n = v.norm_squared();
        if n.is_zero() {
            return a;
        }
        let t = ((p - a).dot(v) / n)
            .max(Ratio128::zero())
            .min(Ratio128::one());
        a + v * t
    }
    pub fn distance_squared(self, p: Point<T>) -> Ratio128 {
        self.closest_point(p).distance_squared(p.to_rational())
    }
    pub fn distance(self, p: Point<T>) -> f64 {
        self.distance_squared(p).to_f64().sqrt()
    }
    pub fn segment_distance_squared(self, other: Self) -> Ratio128 {
        if self.intersects(other) {
            Ratio128::zero()
        } else {
            self.distance_squared(other.a)
                .min(self.distance_squared(other.b))
                .min(other.distance_squared(self.a))
                .min(other.distance_squared(self.b))
        }
    }
    pub fn segment_distance(self, other: Self) -> f64 {
        self.segment_distance_squared(other).to_f64().sqrt()
    }
}
