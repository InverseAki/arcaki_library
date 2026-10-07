# 統合後の検証

実装の正本は `src/`。各ファイルを従来どおり main.rs にコピーして使う。
`src/lib.rs` は従来どおり空で、テストから本体ファイルを直接読み込む。

```sh
cargo test --offline
cargo test --offline --release
python3 tests/run_math_migration.py
python3 tests/run_convolution_998244353.py
python3 tests/run_convolution_1000000007.py
```

後半3本は既存の `../librarychecker/src/main.rs` から同梱MIを抽出して結合する。
ACLでも確認する場合は各スクリプトに `--acl-rlib /path/to/libac_library.rlib` を渡す。
998244353の最大長試験は `--full` で明示的に実行する。

## インターフェース

- 既存の `Basic/couter.rs` の綴り、型名・メソッド名・引数順を維持。
- `Basic/math.rs` の旧 `mod_inverse`、`fast_mod_pow`、`factorial_i64`、`factorial`、`comb` を保持。階乗・組合せの旧APIは同一スコープの `MOD: i64` に依存する。
- `MintCombination` の `MI` は従来の同梱 `Mint` も使える。新たな `modulus()` は要求せず、素数modで前計算範囲が法未満であることは利用者の前提。
- `NumberTheory/mint_combination.rs` 内の `convolution_merge` を維持し、修正版に更新。`Fps/convolution.rs` にも同じ補助関数があるので、同一スコープには片方だけをコピーする。
- `BarrettCombination` と3種類の畳み込みバックエンドは新規ファイルとして追加。

## 修正に伴う挙動

- BITの空配列の長さは0。セグ木の探索結果は論理長まで。範囲外アクセスはassertで検出する。
- Rollback UnionFindは失敗したmergeを含めて巻き戻せる。snapshotは従来どおり現在状態で履歴を破棄する。
- Counterは個数0のキーを保持せず、subは実際に減らした分だけtotalを減らす。総個数のオーバーフローと_exの前提違反はreleaseでもpanic。
- gcd/extended_gcdは非負入力。floor/moduloは余りが非負のユークリッド除算。負の除数の場合、floorは数学的な床とは異なる。
- 畳み込みの0本の積は[1]、空多項式を含む積は[]。

ローカルの愚直比較・境界検証とLibrary Checkerへの提出は別。今回の統合では新たな外部提出はしない。
既存の多倍長整数の検証については `big_integer/README.md` を参照。

## 2026-10-04 統合時の結果

- 本体Cargoテスト18本がdebug/release両方で通過。既存LCAテスト4本も含む。
- 数学・組合せ・畳み込み補助5本、998244353畳み込み5本、1000000007畳み込み3本が、同梱MIとローカルACL 0.1.1のそれぞれでdebug/release両方通過。
- 従来のMint型（modulus()なし）とMintCombination、従来配置のconvolution_merge、数学の旧関数とi64の関数型を追加検証。
- 旧ReprovedのCargoテスト12本もdebug/release両方で通過。
- 998244353の最大長試験2本は今回再実行していない。性能・メモリの過去記録はReproved/history/を参照。

## Splay木の改善（2026-10-04）

性能比較・APIの注意点・検証記録: [splay_performance.md](splay_performance.md)。
本体テストはSplayの4本を追加して計22本。

## 集約・遅延作用のないSplay木（2026-10-04）

`NilMonoid<T>` と `ProdMonoid<M>` を追加。列操作用のget/set/push/remove/pop/to_vecも追加。
使い方・仕様・性能比較: [splay_modes.md](splay_modes.md)。本体テストは計27本。

## Vec風のSplayVector（2026-10-04）

[SplayVectorの使い方・対応API・計算量](splay_vector.md)。値にClone/Debug/Defaultは不要。
本体テストはSplayVectorの4本を加えて計31本。


## 正方行列の統合（2026-10-04）

`Basic/matrix.rs` の `SquareMatrix<M>` に統合し、従来の `DoublingMatrix<M>` と MI用 `Matrix` の入口を維持。
[使い方・積の高速化・計測と検証](matrix.md)。旧2ファイルは `matrix.rs` を読み込むため、コピー時は共通本体も必要。
本体テストは行列7本を加えて計38本。追加の同梱MI/ACL結合検証は `python3 tests/run_matrix.py`。


## 有理数の修正・拡張（2026-10-04）

`Ratio`（i64）、`Ratio128`（i128）、`BigRatio`（既存BigInt）を用意。
従来の `new(分母, 分子)` は維持し、通常順の `from_fraction(分子, 分母)` も追加。
[仕様・使い方・検証結果](ratio.md)。BigRatioは `ratio.rs`・`big_integer.rs`・`big_ratio.rs` を同じスコープに用意する。
本体テストは有理数/GCDの7本を加えて計45本。`python3 tests/check_ratio.py` でPythonとの41,365ケースをdebug/release両方検証。


## 多倍長整数の除算をEuclid除算へ統一（2026-10-04）

BigInt/HexBigIntの `/`・`%`・`/=`・`%=`・`div_rem` は `x=q*d+r`、`0<=r<|d|`。
`div_rem_euclid` も同じ規則。BigRatioの丸め・検証器・使用例・解説も更新した。
多倍長整数の単体テスト6本をCargoから実行するようにし、本体テストは計51本。
[検証と符号付き除算の仕様](big_integer/README.md)。

## Rollback可能なデータ構造（2026-10-04）

UFと重み付きUFをRollbackVectorに基づく実装へ変更し、RollbackSegtree・RollbackLazySegtreeを追加。
直前の更新1回のrollback、snapshot（履歴破棄）、all_backを提供。
[使い方・依存ファイル・計算量・検証](rollback_structures.md)。


## ドロネー三角形分割と Euclidean MST（2026-10-04）

整数座標の決定的 O(N log N) 実装を追加。重複点・一直線・共円に対応。
[API・数値範囲・公式全29ケースの検証結果](euclidean_mst/README.md)。


## 2次元の基本幾何（2026-10-04）

`Gemetory/geometry.rs` に整数・有理数座標の点、正規化した直線、線分と基本演算を追加。
[使用例・返り値の型・数値範囲・検証](geometry.md)。
`cargo test --offline --test geometry` とreleaseで6本通過。格子上の線分交差390,625ケースを独立計算と比較。


## Rollback Mo の追加型・削除型（2026-10-04）

`Basic/rollbackmo.rs` に `RollbackMoState / RollbackMoSolver` と
`RollbackMoDeleteState / RollbackMoDeleteSolver` を追加。
`mo.rs` と同じ Data/Query/Ans、左右操作の添え字・データ、質問登録方式を使う。
左右対称の State も提供。トークン付き snapshot/rollback を使い、solve は開始状態へ復元する。
旧 `RollbackMoMonoid / solve_rollback_mo` は互換用に維持。
[仕様・使用例・計算量・検証](../../algorithm_set/Basic/rollbackmo.md)。
`cargo test --offline --test rollback_mo` とreleaseで7本通過。


## 候補登録型の更新可能 Wavelet Matrix（2026-10-05）

各添字の可能な値を先に登録する `WaveletMatrixOffline` を追加。
値の1点更新、区間 kth、値域・順位の総和、総和を満たす最小個数に対応。
[使い方・計算量・検証・最大規模の計測](waveletmatrix_offline.md)。
[ABC467 G の実行例](../examples/abc467_g.rs)も同梱。

2026-10-05: 直線の `cmp_angle` / `cmp_slope`、点・線分の `cmp_angle` を追加。整数比較は積overflowを避け、geometryの計8テストがdebug/releaseで通過。詳細は [geometry.md](geometry.md)。


## 凸包・係数付きMinkowski和（2026-10-05）

`Gemetory/convexhull.rs` を基本幾何に統合。点型の上下鎖・一周列、`ConvexHull` と線形時間の係数付きMinkowski和を追加。旧tuple入力は互換入口として保持。
[API・返り値・数値範囲・検証](convexhull.md)。凸包6本・基礎幾何8本をdebug/releaseで確認。

2026-10-05: 凹多角形も保持できる反時計回りの `Polygon` を基本幾何に追加。
`Polygon` / `ConvexHull` に `area2`・`area`・`area_f64`、凸包に `to_polygon` を追加。
仕様は [geometry.md](geometry.md)・[convexhull.md](convexhull.md)。関連17テストがdebug/releaseで通過。

2026-10-05: `ConvexHull::merge` を追加。既存凸包の上下鎖をマージして conv(A ∪ B) を O(N+M) で合成。再ソート不要。詳細は [convexhull.md](convexhull.md)。関連20テストがdebug/releaseで通過。

## 点加算・矩形和のジェネリクスと初期点 build（2026-10-06）

[API・検証・性能比較](point_add_rectangle_sum_build.md)。座標型を各軸で指定でき、初期点は一度の走査で全質問へ寄与させる。24,000ケースの愚直比較と型・境界テストを debug/release で実行。

## 矩形加算・点取得（2026-10-06）

[API・検証・性能比較](rectangle_add_point_get.md)。座標型の指定、初期矩形 build、負の重み・半開境界に対応する独立コピー可能な実装。24,000ケースの愚直比較、1,296組の矩形・境界比較を debug/release で実行し、公式登録19ケースもローカル verifier/checker で確認。

## 静的な矩形加算・矩形和（2026-10-07）

[API・4本BIT・検証と実測](static_rectangle_add_rectangle_sum.md)。座標・重み型を指定可能。i128 と既存 Mint の両方で12,000ケースの愚直比較、1,296組×225質問の全比較を debug/release で実行。公式13ケースもローカル verifier/checker で全件AC。


## ABC478 Gでの幾何の速度改善（2026-10-07）

整数比較のchecked積、有理数の共通分母による向き判定、Ratio拡張の再約分省略、整数Ratio演算の高速経路を追加。既存の型・返り値・巨大値fallbackを維持。
[測定・再現手順・提出例](geometry_performance/README.md)。関連32テスト・Python有理数照合41,365ケースがdebug/releaseで通過。サンプル3件と小ケース200件、10万点の構造付き5形状も比較。

## 線分追加 Li Chao Tree（2026-10-07）

`SegmentTree/lichaotree_segment.rs` に単独コピーできる `LiChaoSegmentTree` を追加。
直線版と同じ min/max 指定・i64座標/傾き・i128切片/評価値・閉区間を採用。
`add_segment(a,b,l,r)`、`add_line(a,b)`、`query(x)` を提供する。
[仕様・使い方・検証範囲](../../algorithm_set/SegmentTree/lichaotree_segment.md)。
`cargo test --offline --test lichaotree_segment` と同 release の8テストが通過。

## テスト配置と整数補助の統合

- 多倍長整数の内部テストは `big_integer/ntt.rs` と `big_integer/non_ntt.rs`。本体からテスト時のみ読み込む。
- AVL 木・SortableSequence の検証メソッドは `support/keyed_avl_invariants.rs` と `support/sortable_sequence_invariants.rs`。
- `Basic/ratio.rs` に整数用の `floor` は含めない。整数補助は `Basic/math.rs` を使う。
- `NumberTheory/crt.rs` は `Basic/math.rs` と同じスコープにコピーする。旧名 `ext_gcd` も math が提供する。
- `NumberTheory/modcombination.rs` は固定 `MOD = 1_000_000_007` と math の互換入口。math と同じスコープに重ねてコピーせず、どちらかを使う。
- `floor_sum.rs` は総和計算の別アルゴリズムとして維持する。
- `math_integration.rs` で math・ratio・CRT の同時コピー、負の除数・整数境界、旧組合せ入口を検証する。
