# 最近点対

`src/Gemetory/closestpair.rs` は `ratio.rs`、`geometry.rs` と同じスコープへコピーする。

```rust
include!("../src/NumberTheory/ratio.rs");
include!("../src/Gemetory/geometry.rs");
include!("../src/Gemetory/closestpair.rs");

let points = [IntPoint::new(0, 0), IntPoint::new(3, 4)];
assert_eq!(closest_pair_squared(&points), Some(25i128));
assert_eq!(closest_pair(&points), Some(5.0));
```

- `closest_pair_squared<T: GeometryCoordinate>(&[Point<T>]) -> Option<T::Wide>`: 最小二乗距離。`i64` / `i128` 座標では `i128`、`Ratio` / `Ratio128` 座標では `Ratio128`。
- `closest_pair<T: GeometryCoordinate>(&[Point<T>]) -> Option<f64>`: 厳密な二乗距離を最後に浮動小数点へ変換し、平方根を返す。
- 2点未満は `None`、重複点があれば `Some(0)`。入力の順序は変更しない。
- 分割統治で O(N log N) 回の比較・数値演算、追加領域 O(N)。再帰内で作業配列を再利用する。
- 数値範囲は `geometry.rs` に従う。計算途中の座標差・その二乗・二乗和が各数値型の表現範囲に収まる必要がある。最小距離が表現可能でも、別の候補の計算でoverflowする場合はpanicする。無限座標もpanicする。

旧APIの `&mut Vec<(f64, f64)> -> f64` から、`Point<T>` のスライスと `Option` を使うAPIへ変更。
浮動小数点座標は `GeometryCoordinate` の対象外。

`tests/closestpair.rs` で退化入力、入力保持、小格子の全512部分集合、
2,000個の決定的な点群と全点対比較、整数・両有理数型、巨大座標、
100,000点の同一直線・重複点、格子、overflow・無限座標の拒否を検証する。
