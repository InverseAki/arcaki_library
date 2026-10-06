# 整数座標のドロネー三角形分割 / Euclidean MST

本体: [`../../src/Gemetory/delaunay.rs`](../../src/Gemetory/delaunay.rs)。外部crate不要、1ファイルをコピーして利用できる。
提出用の入出力例: [`../../examples/euclidean_mst.rs`](../../examples/euclidean_mst.rs)。

```rust
let points: Vec<(i64, i64)> = vec![(0, 0), (2, 0), (0, 2), (0, 0)];
let dt = DelaunayTriangulation::new(&points);
let mst: Vec<(usize, usize)> = euclidean_mst(&points);
```

## API・仕様

`DelaunayTriangulation::new(&[(i64, i64)])` の返り値:

| フィールド | 意味 |
| --- | --- |
| `edges` | 三角形分割の無向辺 `(u, v)`。`u < v`、辞書順、重複なし。凸包の辺も含む |
| `triangles` | 反時計回りの `[a, b, c]`。最小の番号を先頭へ回転し、辞書順に並べる |
| `representatives` | 各入力点に対応する、同じ座標を持つ最小の入力番号 |
| `duplicate_edges` | 重複点と代表点を結ぶ長さ0の辺 `(代表番号, 重複点番号)` |

すべての頂点番号は元の入力の **0-indexed**。入力順を変更しない。
同じ座標の点は1点へまとめてから三角形分割するため、`edges` と `triangles` には代表点だけが現れる。
元の全点を結ぶグラフとして利用する場合は `duplicate_edges` も追加する。
代表点が一直線上なら、座標の辞書順で隣り合う点を結ぶ鎖を返し、三角形は空になる。
空入力・1点・全点同一も扱う。同一円周上の点は決定的に対角線を選ぶ。浮動小数点や乱数は使わない。

`euclidean_mst(&points)` は元の全入力点を結ぶ MST の辺を返す。長さ0の辺も含めて、空入力なら0本、それ以外なら `N-1` 本。
重複点の辺を追加し、三角形分割の辺に対して距離の二乗で Kruskal を行う。
平方根は辺の大小関係を保存するため、これで通常のユークリッド距離の MST が得られる。
返却順は決定的だが、全辺を番号順や距離順に並べるという仕様ではない。

## 数値・計算量の前提

- 座標型は `i64`。**各軸について `max - min <= 1_000_000_000` が必要**。絶対座標は `i64` 全域でよい。
- 範囲外は debug/release ともに assert で検出する。浮動小数点座標や無制限の `i64` 座標範囲には非対応。
- 向き判定と外接円判定は `i128`。減算前にキャストするので、`i64::MIN/MAX` 付近への平行移動も扱える。
- 外接円判定では対象点からの差分を使う。差分の絶対値が `B` 以下なら、3項の絶対値合計は高々 `12 B^4`。`B=10^9` なら `i128::MAX` 未満で、中間値もオーバーフローしない。
- 距離の二乗は高々 `2 * 10^18` で、辺のソートには `u64` を使う。
- ソートと分割統治で決定的 **時間 `O(N log N)`、空間 `O(N)`**。削除した quad-edge の領域を再利用する。三角形の抽出は線形、公開結果の辞書順ソートは `O(N log N)`。
- 再帰の深さは `O(log N)`。MST も辺が `O(N)` 本なので同じ計算量。

[Euclidean MST の公式制約](https://github.com/yosupo06/library-checker-problems/blob/master/geo/euclidean_mst/info.toml) は `N <= 200000`、`|x|, |y| <= 10000` で、上の数値範囲に収まる。
[出力仕様](https://github.com/yosupo06/library-checker-problems/blob/master/geo/euclidean_mst/task.md) は辺の端点を `N-1` 行出力する形式。利用例はその形式に対応する。

## 検証

リポジトリ `arcaki_library/` で実行:

```sh
cargo test --offline --test delaunay
cargo test --offline --release --test delaunay
cargo test --offline --release --test delaunay -- --ignored --nocapture
rustc --edition=2021 -O examples/euclidean_mst.rs -o /tmp/euclidean_mst
python3 tests/euclidean_mst/run_official.py --lc-root /path/to/library-checker-problems
```

通常テスト4本は debug/release 両方で通過。3×3格子の全512部分集合とその並べ替え、固定seedの1,500ケース、重複・一直線・共円・境界座標を含む。
三角形の反時計回り、外接円の内部に点がないこと、辺が頂点を飛び越えないこと、辺の非交差、Eulerの関係、代表番号を検証する。
MSTは連結性と辺数を確認し、全点対の独立な Prim の結果と距離の二乗の多重集合を比較する。

大規模テスト1本は重複のない20万点を4種類（一直線・500×400格子・2段の細長い配置・ほぼ円周上）用意する。
Eulerの関係、隣接面の向きと局所的な空円条件、MSTの連結性を検証し、最初の3種類では各MST辺の既知の最短距離も確認した。

2026-10-04: 公式で登録された **全29ケースをローカルの公式 checker が受理**。
各入力のSHA-256が公式 `hash.json` と一致すること、公式 verifier の通過も確認。
`all_same`、`max_colinear`、`max_slender`、`near_circle`、`near_grid`、`small_chunks` を含む。
本体と入出力例を結合した単一ファイル提出コードもコンパイルし、サンプルと20万点のランダムケースでcheckerを通過した。
オンラインジャッジへの提出は行っていない。計測結果は [`results.json`](results.json) を参照。
macOS arm64 / `rustc 1.93.0`、`rustc -O` で20万点の公式ケースは最大約0.35秒。
`max_random_02` の子プロセスの最大RSSは85,737,472 bytes（約81.8 MiB）。計測は入出力を含むローカル環境の値であり、ジャッジ環境の実行時間とは異なる。

公式ランナーは元の Library Checker checkout を変更せず、一時ディレクトリへ対象問題と `common/` をコピーして生成する。
`--work-dir` で作業先、`--results` で記録先を指定できる。作業先は成功・失敗ともに残すため、入力・出力・単一ファイルの `submission.rs` を確認できる。
Python 3.11以上、Rust、C++17コンパイラが必要。ダウンロードは行わない。
