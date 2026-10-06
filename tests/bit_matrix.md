# BitMatrix の整数行列積

正本は `src/Basic/bitset.rs`。既存のBitSetと同じファイルに追加した。
従来も `and_count_ones` / `or_count_ones` / `xor_count_ones` と右行列の転置を組み合わせれば実現できたが、行列積APIはなかった。

```rust
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign};
// bitset.rs をこのスコープにコピーする。
let mut a = BitMatrix::new(2, 3);
a.set(0, 0, true);
a.set(0, 2, true);
a.set(1, 1, true);
let mut b = BitMatrix::new(3, 2);
b.set(0, 0, true);
b.set(1, 1, true);
b.set(2, 0, true);
assert_eq!(a.mul_and(&b), vec![vec![2, 0], vec![0, 1]]);
assert_eq!(a.mul_or(&b), vec![vec![2, 3], vec![3, 1]]);
assert_eq!(a.mul_xor(&b), vec![vec![0, 3], vec![3, 0]]);
```

- `mul_and`: `C[i][j] = Σ_k (A[i][k] & B[k][j])`
- `mul_or`: `C[i][j] = Σ_k (A[i][k] | B[k][j])`
- `mul_xor`: `C[i][j] = Σ_k (A[i][k] ^ B[k][j])`

入力は0/1、出力は `Vec<Vec<usize>>`。集約は整数の和で、値の範囲は0..=K。
OR–ANDの論理行列積が必要なら `mul_and` の結果を `x > 0` に、
GF(2)の積なら `x & 1 != 0` に変換する。
`mul_xor` はGF(2)の行列積ではない。

`A` が H×K、`B` が K×W のとき、積の計算は `O(HW ceil(K/64))`。
これに出力初期化 `O(HW)` と右辺転置（最悪 `O(KW+K+W)`）が加わる。
N×Nでは `O(N² + N³/64)`。一時領域は転置した右辺の `O(W ceil(K/64))` ワードと
出力の `O(HW)` 個のusize、各行のメタデータ。

`from_rows(width, Vec<BitSet>)` で既存の行を移動できる。
`height()`、`width()`、`rows()`、`row(i)`、`get(i,j)`、`set(i,j,value)`、`transpose()` を提供。
行の長さ不一致・積の次元不一致・範囲外のセルアクセスはreleaseでもpanic。
0×W、H×0、内側次元0にも対応し、内側次元0の積は全セル0。
0行の結果の列数は `Vec<Vec<usize>>` には保存されないため、必要なら `B.width()` を保持する。

## 論理行列積と累乗

整数の個数を返す `mul_*` の出力は0/1に閉じないため、その積をそのまま繰り返す
bitset累乗は定義しない。論理行列積 `prod_*` と、その累乗 `pow_*` を追加した。
この系列の接尾辞はk方向の**集約演算**を表す。

| 積 / 累乗 | 各セルの定義 | 0乗の行列 |
|---|---|---|
| `prod_and` / `pow_and` | `AND_k (A[i][k] OR B[k][j])` | 対角0・非対角1 |
| `prod_or` / `pow_or` | `OR_k (A[i][k] AND B[k][j])` | 対角1・非対角0 |
| `prod_xor` / `pow_xor` | `XOR_k (A[i][k] AND B[k][j])` | 対角1・非対角0 |

すべて `BitMatrix` を返す。`prod_*` は長方形も可、`pow_*(exponent: u64)` は正方行列だけ。
AND-ORはOR-ANDの双対で、全ビットを反転するとOR-ANDに対応する。
OR-ANDはちょうどe辺の歩道が存在するか、XOR-ANDはその個数の偶奇を求める。

```rust
let reachable = a.pow_or(100);   // OR-AND
let parity = a.pow_xor(100);     // GF(2)
let dual = a.pow_and(100);       // AND-OR
let product = a.prod_or(&a);
assert_eq!(a.pow_or(2), product);
```

内側次元0の `prod_and` は全true、`prod_or` / `prod_xor` は全false。
非正方行列の累乗は指数0でもpanic。0×0の累乗は0×0。
積は最悪 `O(HW ceil(K/64))` に転置・出力初期化が加わり、
累乗は `O((N² + N³/64) log(e+1))`、0乗の構築は `O(N²/64+N)`。
AND/OR/XORの論理行列積は結合的で、それぞれ上記の単位行列を持つため、繰り返し二乗法を使う。

全2×2行列の組合せの積と0..=12乗を独立したbool演算で検証。
N=0/1/63/64/65の0..=3乗、長方形・空次元、左右の単位元、
3周期の行列で `u64::MAX` 乗（AND-ORは反転した行列）も検証した。
追加後のdebug/releaseテストは各5本通過、計測用1本は通常実行ではignore。

## ローカル検証（2026-10-04）

```sh
cargo test --offline --test bit_matrix
cargo test --offline --release --test bit_matrix
cargo test --offline --release --test bit_matrix benchmark -- --ignored --nocapture
```

3本のテストで全2×2行列の組合せ、長方形、空次元、K=63/64/65/127/128/129、
全0・全1・単位行列・交互ビット・ワード境界・固定seedの乱数を愚直解と比較。
転置を2回適用した復元、未使用ビットのマスク、無効なサイズ・添字も確認。

releaseで512×512の乱数行列を測定した1回の結果（転置・出力確保込み）：

| 演算 | BitMatrix | bool配列を転置して1セルずつ集計する比較実装 |
|---|---:|---:|
| AND | 1.11 ms | 11.68 ms |
| OR | 1.15 ms | 11.81 ms |
| XOR | 0.95 ms | 23.92 ms |

性能は環境・入力依存。外部ジャッジへの提出は行っていない。
