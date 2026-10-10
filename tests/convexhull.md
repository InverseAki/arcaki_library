# 凸包と係数付きMinkowski和

実体は `src/Gemetory/convexhull.rs`。
`NumberTheory/ratio.rs`、`Gemetory/geometry.rs`、`Gemetory/convexhull.rs` を
同じスコープにコピーして使う。外部クレートへの追加依存なし。

## 使用例

```rust
include!("../src/NumberTheory/ratio.rs");
include!("../src/Gemetory/geometry.rs");
include!("../src/Gemetory/convexhull.rs");

fn main() {
    let points = [
        IntPoint::new(0, 0), IntPoint::new(2, 0),
        IntPoint::new(2, 2), IntPoint::new(0, 2),
        IntPoint::new(1, 1),
    ];
    // 一周丸ごとの頂点列。反時計回り、末尾に先頭を重複しない。
    let polygon = convex_hull_polygon(&points);
    assert_eq!(polygon.len(), 4);

    let a = ConvexHull::new(&points);
    let b = ConvexHull::new(&[IntPoint::new(0, 0), IntPoint::new(1, 0)]);
    let sum = a.minkowski_sum(&b); // A+B。結果はConvexHull<i128>
    assert_eq!(sum.vertices(), &[
        Point::new(0i128, 0), Point::new(3i128, 0),
        Point::new(3i128, 2), Point::new(0i128, 2),
    ]);
    let integer_sum = a.weighted_minkowski_sum(2i64, &b, -1i64); // 2A-B
    assert_eq!(integer_sum.len(), 4);
    let rational_sum = a.weighted_minkowski_sum(
        Ratio::from_fraction(1, 2), &b, Ratio::from_fraction(2, 3),
    ); // (1/2)A+(2/3)B。結果はConvexHull<Ratio128>
    assert_eq!(rational_sum.vertices(), &[
        Point::new(Ratio128::zero(), Ratio128::zero()),
        Point::new(Ratio128::from_fraction(5, 3), Ratio128::zero()),
        Point::new(Ratio128::from_fraction(5, 3), Ratio128::one()),
        Point::new(Ratio128::zero(), Ratio128::one()),
    ]);
}
```

## APIと計算量

|API|内容|計算量（数値演算回数）|
|---|---|---|
|`convex_hull_chains(&[Point<T>])`|上側・下側の2列。両方とも辞書順最小点から最大点へ|O(N log N)|
|`convex_hull_polygon(&[Point<T>])`|反時計回りの一周列|O(N log N)|
|`ConvexHull::new(&[Point<T>])`|一周列を内部に保持する凸包|O(N log N)|
|`vertices()` / `into_vertices()`|頂点列の参照 / 所有権を取得|O(1)|
|`len()` / `is_empty()`|頂点数 / 空判定|O(1)|
|`scaled(coefficient)`|凸包のスカラー倍|O(N)|
|`merge(&other)`|両方を包む凸包 conv(A ∪ B)|O(N+M)|
|`minkowski_sum(&other)`|A+B|O(N+M)|
|`weighted_minkowski_sum(alpha, &other, beta)`|alpha*A+beta*B|O(N+M)|

一周列は辞書順最小点から開始。重複点・辺の途中の点を除く。
空集合は空列、1点は1頂点、一直線は両端の2頂点。
入力順は自由。`T` は `i64` / `i128` / `Ratio` / `Ratio128`。
凸包の生成は入力座標型を保つ。

Minkowski和は `{ alpha*a + beta*b | a∈A, b∈B }`。
結果も同じ一周列の規約に従う。入力は同じ座標型の `ConvexHull` 2個。
2係数は同じ型で渡し、混在させたい場合は `Ratio128` 等へ揃える。
負の係数は原点対称な拡大縮小で、頂点順の反時計回りを維持する。
0倍した非空集合は原点1点。どちらかが空なら和も空。
点・線分・多角形のいずれの組合せでも同じAPIで合成できる。
係数は有限値のみ（空集合でも無限大係数を拒否）。

|入力座標|係数型|合成・scaledの出力座標|
|---|---|---|
|i64 / i128|i64 / i128|i128|
|Ratio / Ratio128|i64 / i128|Ratio128|
|いずれも|Ratio / Ratio128|Ratio128|

整数の比較には前回追加した厳密な積比較・角度比較を利用する。
Minkowski和では最下点から始まる辺を角度順でマージし、同方向の辺をまとめて進める。
頂点列全体を再ソートせず、最後の正規化も線形時間。
整数比較の内部では O(log M) 回の整数除算を使うため、上表は固定幅数値演算の回数である。

## 旧APIと数値範囲

旧tuple入力の `convex_hull` は残し、内部で新しい点の判定を使う。
引数を `&Vec<(i64,i64)>` から `&[(i64,i64)]` に広げたので通常の旧呼び出しはそのまま使える。
返り値は従来どおり `(上側, 下側)`。重複点は新たに除去する。
`cross_product` は同名を残し、返り値を `i64` から `i128` に拡張。
関数ポインタ型を旧シグネチャに固定している場合は変更が必要。

凸包の整数の向き判定では外積の値を生成せず、2積の大小のみ比較する。
したがって **i64座標の全域で凸包を生成できる**。
`i128` 座標の場合は、頂点間の差が `i128` に収まる必要がある。
Minkowski和の拡大縮小・頂点の和・辺の差にもi128範囲の制限があり、超過はreleaseでもpanic。
有理数の判定・演算は `Ratio128` の既約分子・分母に収まる必要がある。
旧 `cross_product` は外積そのものを返すため、こちらは値がi128に収まらなければpanicする。

## 検証

```sh
cargo test --offline --test convexhull --test geometry
cargo test --offline --release --test convexhull --test geometry
```

凸包6テスト・基礎幾何8テストがdebug/release両方で通過。

- 3×3格子の全512部分集合を独立なgift wrappingによる凸包と比較。
- 空・1点・平行/非平行な線分・三角形・長方形・菱形の各組合せを全頂点対の和と比較。
- 正・負・零の整数係数と有理数係数を検証。
- 固定seedの2,000組の点群で、整数座標と有理数座標の合成を全頂点対の和から作った凸包と比較。
- i64最小値・最大値を含む凸包、正規化、重複・一直線、旧API互換、範囲超過を確認。
- 20,000頂点同士の和が20,000頂点の2倍した凸包と一致することを確認。

外部ジャッジへの提出は未実施。

## 凸包の面積（2026-10-05）

`ConvexHull` に以下を追加した。

|API|返り値・内容|計算量|
|---|---|---|
|`area2()`|面積の2倍。整数座標なら `i128`、有理数座標なら `Ratio128`|O(N)|
|`area()`|厳密な面積 `Ratio128`|O(N)|
|`area_f64()`|近似面積 `f64`|O(N)|
|`to_polygon()`|同じ境界の `Polygon<T>` にコピー|O(N)|

空・点・線分は面積0。整数計算は範囲超過をreleaseでも検出する。
凸包の生成がi64全域に対応することと、その面積がi128に収まることは別。

```rust
let hull = ConvexHull::new(&[
    IntPoint::new(0, 0), IntPoint::new(1, 0), IntPoint::new(0, 1),
]);
assert_eq!(hull.area2(), 1i128);
assert_eq!(hull.area(), Ratio128::from_fraction(1, 2));
assert_eq!(hull.to_polygon().area(), hull.area());
```

凸包と元の凹多角形の面積の差、Polygonへの変換、負・零・有理数係数の
面積倍率、自己Minkowski和の面積、退化ケースを追加検証。
凸包7本・基礎幾何10本、計17テストがdebug/releaseで通過。

## 2つの凸包を包む凸包の合成（2026-10-05）

`ConvexHull::merge(&other)` は `conv(A ∪ B)`、つまり両方の凸包全体を包む最小の凸包を返す。
Minkowski和 `A+B` とは別の演算。入力と同じ座標型を保つ。

```rust
let a = ConvexHull::new(&[IntPoint::new(0, 0), IntPoint::new(2, 2)]);
let b = ConvexHull::new(&[IntPoint::new(0, 2), IntPoint::new(2, 0)]);
let merged = a.merge(&b);
assert_eq!(merged.vertices(), &[
    IntPoint::new(0, 0), IntPoint::new(2, 0),
    IntPoint::new(2, 2), IntPoint::new(0, 2),
]);
assert_eq!(merged.area(), Ratio128::int(4));
```

合成は **O(N+M) 時間・追加メモリ**（固定幅の数値演算回数）。
既存凸包の反時計回り列を最右点で上下の鎖に分けると、各鎖を辞書順に列挙できる。
鎖をマージして各凸包の頂点を辞書順にし、さらに2つの点列をマージして、
単調鎖法で最終凸包を作る。各段階は線形時間であり、再ソートは行わない。
頂点を連結して `ConvexHull::new` で作り直す方法の O((N+M) log(N+M)) から
ソートのlogを除去できる。最初に点集合から各凸包を作るコストは別途必要。

空との合成は元の凸包。包含、共有点、辺での接触、重複、一直線、
点・線分・多角形の組合せに対応する。座標型と数値範囲の制限は既存凸包の生成と同じ。
入力は変更せず、辞書順最小点から始まる反時計回りの凸包を返す。

追加3テストで、3×3格子の全512部分集合から得られる異なる凸包の全組合せを
独立なgift wrappingの凸包と比較し、可換性・包含した入力との再合成も確認。
包含・離れた形状・同一x座標・共有頂点・退化形状・両有理数型・整数座標全域を検証。
交互に分けた合計20,000頂点の凸包2個の合成を、全点から作り直した凸包と比較。
凸包10本・基礎幾何10本、計20テストがdebug/releaseとも通過。
