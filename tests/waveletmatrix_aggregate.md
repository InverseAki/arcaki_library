# Wavelet Matrix の集約版

追加した本体は `src/DataStructure/waveletmatrix_prefix.rs` と `src/DataStructure/waveletmatrix_monoid.rs`。
各ファイルは隣の `waveletmatrix.rs` を内部モジュールへ読み込む。提出用に使う場合は、
使用するファイルと `waveletmatrix.rs` を同じディレクトリに置き、前者を `include!` する。
元の WaveletMatrix を同時に読み込んでも型名は衝突しない。

キーは固定の `usize`、データはキーと同じ長さの配列。キーは座標圧縮して渡してもよい。
全区間は半開、k と昇順化後の順位は 0-indexed。
同じキーは元の添字順に並べる。`matrix()` から既存の WaveletMatrix の検索 API も使える。
元の実装の `rank(x,r)` は最終段の位置であり出現数ではないので、
頻度には `matrix().rank_range(x,l,r)` を使う。

## 累積和・累積 XOR

```rust
include!("waveletmatrix_prefix.rs");

let keys = [3, 1, 3, 2];
let weights = [30i64, 10, 31, 20];
let sum = WaveletMatrixSum::new(&keys, &weights, WaveletSum::default());
assert_eq!(sum.prod(0, 4), 91);                  // 全データ
assert_eq!(sum.prod_less(0, 4, 3), 30);          // key < 3
assert_eq!(sum.prod_between(0, 4, 2, 4), 81);    // 2 <= key < 4
assert_eq!(sum.prefix_prod(0, 4, 3), 60);        // 昇順の先頭3個
assert_eq!(sum.prod_sorted(0, 4, 1, 3), 50);     // 昇順の順位 [1,3)

let data = [1u64, 2, 4, 8];
let xor = WaveletMatrixXor::new(&keys, &data, WaveletXor::default());
assert_eq!(xor.prefix_prod(0, 4, 3), 2 ^ 8 ^ 1);
```

`WaveletMatrixPrefix<G>` を直接使えば、可換な演算 `op` と累積値の差分
`difference(total,prefix)` を持つ型も扱える。`WaveletGroup` はそのインターフェース。
`WaveletSum<T>` は `Default` を加算の0とし、加算・減算を使用する。
`WaveletXor<T>` は `Default` をXORの0とする。
`i64/i128/u64/u128` 等を使える。整数和は構築中の各段の累積値、差分、
問い合わせ結果が型の表現範囲に収まる必要がある。浮動小数点の丸めによる非結合性は保証対象外。
unsigned の和も、実際の非負の累積和と差分として扱える。
データの更新はしない。

## 任意モノイド・データ更新

```rust
include!("waveletmatrix_monoid.rs");

struct Min;
impl WaveletMonoid for Min {
    type S = i64;
    const COMMUTATIVE: bool = true;
    fn identity() -> i64 { i64::MAX }
    fn op(&self, a: &i64, b: &i64) -> i64 { (*a).min(*b) }
}
let keys = [3, 1, 3, 2];
let mut wm = WaveletMatrixMonoid::new(&keys, &[30, 10, 31, 20], Min);
assert_eq!(wm.kth_index(0, 4, 2), 0); // キー3の同順位は元の添字順
assert_eq!(wm.kth_data(0, 4, 2), 30);
assert_eq!(wm.prod_sorted(0, 4, 1, 3), 20);
wm.set(0, -5);                       // 元の添字のデータ更新
assert_eq!(wm.get(0), -5);
assert_eq!(wm.prod_between(0, 4, 2, 4), -5);
```

`S: Clone` 以外のデータ型の制限はなく、タプルや構造体も使える。
データと集約値を別の形で保持したい場合は必要な情報を `S` に載せ、葉の値に変換して渡す。
演算は単位元と結合則を満たす必要がある。`op(&self,...)` はモノイド自身の設定も参照できる。
セグ木の trait とは別の trait なので、既存実装の使用時は上のように実装を追加する。

`COMMUTATIVE` の既定値は `false`。文字列連結、行列積、関数合成などでも、
集約順は **キー昇順、同キー内は元の添字順**。`prod(l,r)` もこの順で、
元配列の添字順の積とは異なる。
可換演算には `COMMUTATIVE = true` を明示して部分木単位の高速集約を有効にする。
非可換演算で `true` を指定すると結果の保証がなくなる。

- `get(i)` / `set(i,x)`: 元の添字で取得 / 更新。キーの変更は不可。
- `kth_index(l,r,k)` / `kth_data(l,r,k)`: 安定昇順の k 番目の元添字 / データ。
- `prod(l,r)`: 区間全体のデータを安定昇順で集約。
- `prod_less(l,r,upper)` / `prod_between(l,r,lower,upper)`: 値域を絞って集約。
- `prefix_prod(l,r,k)` / `prod_sorted(l,r,start,end)`: 昇順の先頭 k 個 / 順位区間を集約。
- 0個の集約は単位元。逆転した値域は空集合。
- 不正な添字区間、順位区間、k、キーとデータの長さ不一致はassertで検出。

## 計算量と使い分け

L は最大キーのビット長（空・全0なら1、最大 `usize::BITS`）、N は要素数。
演算とCloneが O(1) の場合の見積り。文字列や可変長配列の集約にはそのコストを加える。

| 操作 | 累積版 | モノイド版・可換 | モノイド版・非可換 |
|---|---|---|---|
| 構築・空間 | O(NL) | O(NL) | O(NL) |
| 値域・順位区間の集約 | O(L) | O(L log(N+1)) | O(L + t(L + log(N+1))) |
| `prod(l,r)` | O(1) | O(log(N+1)) | 上記と同じ |
| データ更新 | 非対応 | O(L log(N+1)) | O(L log(N+1)) |
| `kth_index` / `kth_data` | 非提供 | O(L) | O(L) |
| `get` | 非提供 | O(1) | O(1) |

空間には各段の累積列 / セグ木を含み、通常のビット列のみのWavelet Matrixより大きい。
構築時は各段の元添字列も一時的に保持する。空配列の場合は O(L) の定数領域。
非可換の t は、選択した要素の異なるキーの数（順位区間が空なら単位元を O(1) で返す）。
値の異なる葉ごとに辿り、同じ値のデータはセグ木でまとめる。
Wavelet Matrix の各段は部分木の内部まで昇順に並んでいるわけではないので、
非可換の集約で途中段を丸ごと掛け合わせると順序が変わる。このため一般の
非可換モノイドについて O(L log N) の集約は保証しない。

和・XORだけで静的なら累積版が軽く高速。min/max/gcdなど逆演算がない集約や
データ更新が必要ならモノイド版が有用。非可換かつ異なる値が大量にある場合は、
問い合わせごとの探索量を考慮して使う。

## 検証

```sh
cargo test --offline --test waveletmatrix_aggregate
cargo test --offline --release --test waveletmatrix_aggregate
```

愚直な `(key, original_index)` 安定順序との比較。空配列・空区間・全同値・重複値・
63/64/65要素・最大usize・固定seedランダム配列について、
値域、昇順の順位区間、符号付き / unsigned 和、XOR、可換モノイド、
非可換連結、k番目の添字・データ、更新後、範囲外assertを検証する。
これはローカル検証で、外部ジャッジへの提出は行っていない。

## 配列の値そのものを更新する場合

各添字の候補値を事前登録できる場合は、[WaveletMatrixOffline](waveletmatrix_offline.md) を使える。
この版は値の1点更新・区間 kth・総和を満たす最小個数に対応する。
