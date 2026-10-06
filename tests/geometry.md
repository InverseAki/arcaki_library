# 2次元の基本幾何

本体: `src/Gemetory/geometry.rs`（既存フォルダの綴りに合わせた）。
提出時は `src/Basic/ratio.rs` と同じスコープへコピーする。既存の
`closestpair.rs` / `delaunay.rs` は変更しない。
凸包は新方式へ更新済み。詳細は [convexhull.md](convexhull.md) を参照。
`src/lib.rs` は既存方針どおり空で、Cargoテストから直接includeする。

```rust
include!("../src/Basic/ratio.rs");
include!("../src/Gemetory/geometry.rs");

fn main() {
    let a = IntPoint::new(0, 0);
    let b = IntPoint::new(3, 0);
    let c = IntPoint::new(0, 1);
    assert_eq!(a.distance_squared(b), 9i128);
    assert_eq!(triangle_area(a, b, c), Ratio128::from_fraction(3, 2));
    let line = Line::through(b, c).unwrap();
    assert_eq!(line.coefficients(), (1, 3, -3));
    let axis = Line::new(1, 0, 0).unwrap();
    assert_eq!(line.intersection(axis), LineIntersection::Point(c.to_rational()));
    let half = Point::new(Ratio::from_fraction(1, 2), Ratio::zero());
    assert_eq!(half.norm_squared(), Ratio128::from_fraction(1, 4));
}
```

## 型と返り値

|入力座標|基本計算型 `T::Wide`|
|---|---|
|`i64`（`Point` のデフォルト、`IntPoint`）|`i128`|
|`i128`|`i128`|
|`Ratio` = `Rational<i64>`|`Ratio128`|
|`Ratio128`（`RationalPoint`）|`Ratio128`|

- `+`, `-`, 単項 `-`, スカラー `*` と、`translated` / `difference` / `scaled` / `rotated90` は基本計算型の点を返す。`i64` の差も拡張してから計算する。
- `/` と `divided` は整数座標でも `RationalPoint` を返す。0除算はpanic。
- `dot`, `cross`, `norm_squared`, `distance_squared`, `manhattan_distance`, 面積の2倍は基本計算型。
- 中点・重心・外心・直線交点・射影・鏡映・線分上の最近点は `RationalPoint`。
- 面積と点対直線・点対線分・線分対線分の距離の2乗は `Ratio128`。
- 距離・長さ・角度は `f64`。任意角回転 `rotated` は `Point<f64>`（出力専用で、厳密幾何の入力には使わない）。角度単位はラジアン。
- 異なる座標型同士の点演算は、まず両方を `wide()` または `to_rational()` で揃える。
- 点の辞書順はx優先。`Eq` / `Ord` / `Hash` を利用できる。点と線分の座標は公開。

## 基本API

|対象|API|
|---|---|
|点・ベクトル|`new`, `wide`, `to_rational`, `translated`, `difference`, `scaled`, `divided`, 四則演算、`rotated90`, `rotated`|
|内積・外積・距離|`dot`, `cross`, `norm_squared`, `norm`, `distance_squared`, `distance`, `manhattan_distance`|
|方向|`angle`, `angle_to`, `is_parallel`, `is_perpendicular`|
|三角形|`orientation`, `ccw`, `signed_triangle_area2`, `triangle_area2`, `triangle_area`, `triangle_centroid`, `triangle_circumcenter`|
|多角形|`signed_polygon_area2`, `polygon_area`|
|直線|`new`, `from_rational_coefficients`, `through`, `coefficients`, `evaluate`, `contains`, `side`, `normal`, `direction`, `is_parallel`, `is_perpendicular`, `intersection`, `projection`, `reflection`, `distance_squared`, `distance`, `parallel_through`, `perpendicular_through`|
|線分|`new`, `line`, `contains`, `length_squared`, `length`, `intersects`, `intersection`, `closest_point`, `distance_squared`, `distance`, `segment_distance_squared`, `segment_distance`|

`orientation` は反時計回り=1、時計回り=-1、一直線=0。
`ccw` はさらに始点より後方・終点より先・線分上を区別する。
`a=b` の場合は `c=a` のとき線分上、それ以外は終点より先とする。
角度は零ベクトルで `None`。ベクトルの平行・垂直判定は零ベクトルでも内積・外積の式どおりtrue。
三角形の外心は一直線・重複点で `None`。

## 直線の正規形

`ax + by + c = 0` の `(a,b,c): (i128,i128,i128)` を保持する。
3係数全体のgcdで割り、最初の非零係数を正にする。
係数を非公開にし、同じ直線は `Eq` / `Ord` / `Hash` で同じ値として扱う。
有理数座標から作るときも、分母を払って同じ正規形にする。

`a=b=0` と同一点を結ぶ直線は `None`。
交点は `LineIntersection::{Parallel, Coincident, Point}` で区別する。
`side` の符号は正規化した法線に対する符号であり、`through` の点順には依存しない。
点順に対する左右の判定には `orientation` を使う。

## 線分・面積の仕様

線分は両端を含む。零長線分・端点接触・一直線上の重なりを扱う。
交差結果は `SegmentIntersection::{None, Point, Overlap}`。
重なりの両端は辞書順。距離は交差・接触・重なりで0。

符号付き面積は反時計回りを正とする。多角形の頂点は周回順で渡す。
0～2頂点では面積0。自己交差多角形は靴紐公式の符号付き面積を計算し、
領域の和集合面積を計算するものではない。多角形面積は O(N)、他は固定個数の数値演算。

## 数値範囲

無限大座標は不可。`new` と計算時の型拡張で検出する。
すべての厳密計算は有限幅の `i128` / `Ratio128` を使うため、任意のi64座標全域に対して
すべての演算が成功する保証はない。積・和・差・正規形の符号変更・
有理数の既約分子分母が範囲を超えるとdebug/releaseともpanicし、wrapしない。
`i128::MIN` の係数は約分後に符号変更できる場合のみ対応する。

例えば絶対値10^9以下の整数点なら通常の内積・外積・点間距離の2乗は十分収まる。
ただし直線係数の積、距離の有理数演算、多角形の面積総和などにも別途範囲の確認が必要。
有理数直線の分母払いはchecked LCMと積を使い、最終的な正規形が小さくても
中間値が収まらなければpanicする。多倍長有理数は今回の対象外。
`f64` 出力は丸めを伴い、極端な値では∞になり得る。

## 検証

```sh
cargo test --offline --test geometry
cargo test --offline --release --test geometry
```

6本のテストで整数・両有理数型の基本演算、直線正規化、交点、射影・鏡映、
外心、面積、距離、零長・平行・重なり・overflow・無限座標を検証。
格子25点の全4点組 390,625 ケースで、線分交差を独立なパラメータ方程式と比較し、
返った交点が両線分上にあること・直線が両端を通ること・端点逆順での同一性も確認。
外部ジャッジへの提出は未実施。

## 整数による角度・傾きの厳密比較（2026-10-05）

`f64`・三角関数・epsilonを使わず、`std::cmp::Ordering` を返す。

|API|順序・仕様|
|---|---|
|`Line::cmp_angle(other)`|向きを持たない直線の角度 `[0, π)`。水平→正の傾き→垂直→負の傾き。平行なら `Equal`、切片 `c` は無視|
|`Line::cmp_slope(other)`|数値としての傾き `-a/b` の昇順。負の傾き→水平→正の傾き→垂直。垂直線は `+∞` として扱う|
|`Point::cmp_angle(other)`|ベクトルの偏角 `[0, 2π)`。+x軸から反時計回り。零ベクトルが含まれると `None`、同じ向きなら `Some(Equal)`|
|`Segment::cmp_angle(other)`|始点 `a` →終点 `b` の向きで比較。点の比較と同じ順序。零長線分が含まれると `None`|

```rust
let mut lines = vec![
    Line::new(1, 1, 0).unwrap(),  // 傾き -1
    Line::new(0, 1, 0).unwrap(),  // 水平
    Line::new(1, 0, 0).unwrap(),  // 垂直
    Line::new(1, -1, 0).unwrap(), // 傾き +1
];
lines.sort_by(|a, b| a.cmp_angle(*b)); // 水平, +1, 垂直, -1
lines.sort_by(|a, b| a.cmp_slope(*b)); // -1, 水平, +1, 垂直

let a = Segment::new(IntPoint::new(0, 0), IntPoint::new(1, 0));
let b = Segment::new(IntPoint::new(0, 0), IntPoint::new(0, 1));
assert_eq!(a.cmp_angle(b), Some(std::cmp::Ordering::Less));
```

直線の `Ord` は従来どおり係数の辞書順。角度ソートには上の比較関数を明示する。
線分の両端を反転すると向きはπ変わる。向きを無視して比較するなら
`segment.line()` で直線にして `Line::cmp_angle` を使う。
これらは方向・傾きの順序を比較するAPIで、2直線が作る角度の大きさを返すものではない。

整数では積 `a*b` と `c*d` の比較を、符号判定と絶対値の分数比較に変換する。
分数比較は連分数（商・余り）の反復で行い、積を作らない。
直線は構築済みの全 `i128` 係数、整数ベクトルは全 `i128` 成分、
`i64` 線分は端点座標の全域で角度比較できる。`i128` 線分は端点差が `i128` に収まる必要がある。
直線を構築する際の中間演算に関する既存の範囲制限は引き続き適用される。
比較は値のビット長に対して O(log M) 回の整数除算。
有理数ベクトルの比較も厳密だが、積を作るため `Ratio128` の既存の範囲制限に従う。

追加2テストを含め計8テストがdebug/releaseで通過。
整数積比較は最小値・最大値・零・正負を含む11値の全4項組14,641ケースを、
既存有理数バックエンドの独立な256bit積と比較。
小格子の288方向の全組82,944ケースで角度・傾き順を確認し、
全域の係数・端点差、平行、反対方向、水平・垂直、零長も検証。

## 反時計回りの多角形 `Polygon`（2026-10-05）

`geometry.rs` 内の `Polygon<T=i64>` は、凹みを含む単純多角形の周回順の頂点を保持する。
穴・自己交差は対象外で、自己交差の検出は行わない。

```rust
let boundary = [
    IntPoint::new(0, 0), IntPoint::new(3, 0), IntPoint::new(3, 1),
    IntPoint::new(1, 1), IntPoint::new(1, 3), IntPoint::new(0, 3),
];
let polygon = Polygon::new(&boundary); // 凹みを残したL字
assert_eq!(polygon.area2(), 10i128);
assert_eq!(polygon.area(), Ratio128::int(5));
assert_eq!(polygon.area_f64(), 5.0);
```

- `new(&[Point<T>])`: 境界を周回する順序の頂点を渡す。時計回りなら先頭点を維持して反転する。入力の点をソート・凸包化しない。
- 連続重複と末尾の先頭点を除去する。辺の途中の点は残す。面積が0の退化列は方向を変更しない。
- `vertices()` / `into_vertices()` / `len()` / `is_empty()` を提供。頂点列を非公開にして方向の規約を維持する。
- `area2()`: 非負の面積の2倍。整数座標は `i128`、`Ratio` / `Ratio128` 座標は `Ratio128`。
- `area()`: 面積の厳密値を `Ratio128` で返す。半整数を切り捨てない。
- `area_f64()`: 面積を `f64` で返す。

構築と各面積メソッドは O(N)。空・点・線分は面積0。
計算は既存の靴紐公式を使い、数値範囲の制限は既存の面積関数と同じ。
入力が自己交差する場合は反時計回り・領域面積の保証がない。

追加2テストで凹多角形、時計回り入力、周回の開始位置、閉じた入力、重複点、
平行移動、両有理数型、半整数面積、退化列・無限座標を確認。
基礎幾何10本と凸包7本、計17テストがdebug/releaseとも通過。
