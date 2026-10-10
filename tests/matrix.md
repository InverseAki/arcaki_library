# 正方行列の統合

正本は `src/LinearAlgebra/matrix.rs` の `SquareMatrix<M>`。行優先の連続した `Vec<M::S>` を持つ。
加算・乗算の定義を `MatrixMonoid` に渡すので、通常の積、mod積、min-plus、boolの到達可能性を同じ型で扱える。
行列積はO(n³)、累乗はO(n³ log k)、逆行列はO(n³)。セルの演算をO(1)とした計算量。

## 基本的な使い方

`matrix.rs` だけをコピーすれば、MIやグローバルMODなしで次を使える。

```rust
type Mat = SquareMatrix<ModMatrixMonoid<998244353>>;
let a = Mat::new(2, vec![1, 1, 1, 0]);
let b = a.pow(10);
assert_eq!(b[(0, 0)], 89);
let c = a.mul(&b);           // prodも同じ
let d = &a * &b;
let inverse = a.try_inv();   // 特異行列ならNone。invはpanic
let mut out = Mat::zeros(2);
a.mul_into(&b, &mut out);    // outのセル領域を再利用
```

`new` は `Vec` を移動し、`&Vec` / `&[T]` は複製する。配列とその参照も受け取る。
`zeros`、`identity` / `e`、`get` / `set`、`size`、`row` / `row_mut`、
`as_slice` / `as_mut_slice`、`into_vec` もある。サイズ不一致・範囲外アクセスはpanic。

- `ModMatrixMonoid<P>`: セルはu32、1 <= P <= u32::MAX、値は必ず `[0,P)` に正規化して渡す。法1や合成数の積も使える。
- `MinPlusMonoid`: セルはi64、加算はmin、乗算は和、無限大は `MinPlusMonoid::INF = 1 << 60`。有限値と有限値どうしの演算結果は絶対値がINF未満。
- `BoolMatrixMonoid`: セルはbool、加算はOR、乗算はAND。

独自型は従来と同じ `MatrixMonoid` の `S`、`zero`、`one`、`sum`、`mul` を実装すればよい。
既存の実装に追加の必須メソッドはない。`S: Clone` だけでよく、Copyやマーカー型自身のCloneは不要。
乗法の順序は `a[i,k] * b[k,j]`、各セルの積算順序はkの昇順を維持する。
`zero` は加法の単位元かつ乗法の吸収元であることが前提。
追加の `is_additive_zero`、`mul_add`、`multiply_kernel` は必要な型だけで上書きできる。

逆行列が必要な型には `MatrixField` も実装する。体の演算が前提。
`ModMatrixMonoid<P>` の逆行列はPが素数か検査し、合成数ならpanic。
MI互換型は従来同様、MIの逆元演算が正しい体で使う前提（素数modなど）。

## 従来の入口

`doubling_matrix.rs` と `mint_matrix.rs` は共通の `matrix.rs` をincludeする薄い入口になった。

- `DoublingMatrix<M>` は `SquareMatrix<M>` の型エイリアス。従来の `new(n, &v)`、`e`、`prod`、`pow`、`get/set` を維持。
- `AddMulMonoid` は従来同様、同じスコープの定数 `MOD: i64` に依存する。セルは `[0,MOD)`、MODは正。
- MI用 `Matrix` は `SquareMatrix<MintMatrixMonoid>` の型エイリアス。従来の `new(n, v)`、`identity`、`mul`、`inv`、`[(i,j)]` を維持し、`pow` も使える。
- MIは同じスコープに用意する。新たな `modulus()` / `raw()` は要求しない。従来の `Mint`、同梱MI、ACLの固定mod型で結合検証済み。

リポジトリ内から `include!` すれば共通ファイルを自動的に読み込む。
main.rsに手でコピーする場合は、共通の `matrix.rs` を1回コピーしたうえで、互換入口の `include!("matrix.rs");` を除いた部分をコピーする。
両方の入口を同じスコープで使う場合も共通本体は1回だけにする。
または入口ファイルと `matrix.rs` を同じフォルダに置く。

旧mintの逆行列には借用チェックでコンパイルできない式があったため、共通実装で解消した。
旧min-plusの `INF + 負の値` が有限扱いになる問題も修正した。INFはいずれの側でも吸収元になる。

## 積の高速化

1. 汎用積は連続した行を走査するi-k-j順を使い、大きい行列は32×32×64の単位で処理する。内側はsliceをzipして、繰り返しの添字計算を減らす。
2. u32までの法では積をu64にまとめて足し、最後に剰余を取る。1セルごとに積算する個数は
   `min(64, (u64::MAX-(P-1))/(P-1)²)`。直前の剰余も含めてオーバーフローしない上限で、998244353なら18項ずつ。
3. mod専用積は最大8KiBの積算領域を使う。小さい行列は16行、大きい行列は8行×128列の処理単位を採用。4×256、8×128、16×64を実測して選んだ。
4. 左辺の0が25%を超えたら0の積を省略。全体が0なら早期終了する。min-plusもINFの積を省略し、右辺が全て有限値なら内側のINF判定を省ける。
5. `AddMulMonoid` とMI入口でもn>=16なら同じmod専用積を使う。変換はO(n²)で、変換費用も含めて旧実装と比較した。小さい行列は従来型の直接演算を使う。
6. `pow` は最初の選ばれたべき乗をコピーし、単位行列との積を1回省く。出力用の行列領域も使い回す。mod用の変換・積算領域は各積で確保するため、全体が無確保になるわけではない。

法がu32に収まらない `AddMulMonoid` はi128の積へフォールバックする。
法がu32上限に近い場合は積算個数が1になり、剰余回数の削減効果は小さい。
unsafe・外部依存・アーキテクチャ固有の命令は使っていない。

## 検証

```sh
cargo test --offline
cargo test --offline --release
python3 tests/run_matrix.py
python3 tests/run_matrix.py --acl-rlib ../problem_solve/target/debug/deps/libac_library-577cc9b5b2f2bce4.rlib
python3 tests/benchmark_matrix.py --sizes 2,4,8,16,32,64,128,256,512 --repeats 5 --output tests/matrix_benchmark_results.json
```

本体の行列テスト7本。u128の愚直積と比較し、空・端数サイズ・バッチ境界・タイル境界、
全てP-1・0・疎行列、法1からu32::MAX、i64::MAXのフォールバック、累乗、逆行列、特異行列、
INFと負の値、bool、非Copyで非可換なセル乗法、サイズ不一致・不正な剰余を検証する。
同梱MIとローカルACLでそれぞれ結合テスト3本をdebug/release両方実行する。
性能測定も各計測前に旧実装との全セル一致を検査する。

比較用の旧ソースは `tests/support/*matrix_baseline.rs` に保存した。
旧mintの逆行列の借用エラーだけを計測スクリプト上で修正し、積の実装は変更していない。
Library Checkerへの外部提出は今回行っていない。

## ローカル計測結果

macOS-26.5.2-arm64-arm-64bit-Mach-O、rustc 1.93.0 (254b59607 2026-01-19)、`rustc -O`、5回の中央値。
法998244353、同梱MIを使用。各積はA×A、疎行列はセルの約90%が0。
行列の構築を除き、積の結果の確保と旧入口のu32変換を含む。

| 入力 | 旧Doubling / 新（ms） | 倍率 | 旧MI Matrix / 新（ms） | 倍率 | 直接u32（ms） |
|---|---:|---:|---:|---:|---:|
| 密 64×64 | 0.174 / 0.043 | 4.02× | 0.234 / 0.042 | 5.62× | 0.039 |
| 密 128×128 | 1.426 / 0.296 | 4.82× | 1.753 / 0.293 | 5.99× | 0.280 |
| 密 256×256 | 10.938 / 2.279 | 4.80× | 13.430 / 2.260 | 5.94× | 2.176 |
| 密 512×512 | 87.238 / 17.562 | 4.97× | 106.569 / 17.430 | 6.11× | 17.196 |
| 疎 256×256 | 11.199 / 0.720 | 15.54× | 13.755 / 0.714 | 19.28× | 0.632 |
| 疎 512×512 | 85.912 / 5.086 | 16.89× | 107.205 / 4.991 | 21.48× | 4.846 |

min-plusの密な512×512では旧19.51ms、新19.53msでほぼ同程度。疎な256×256は旧2.502ms → 新0.481ms（5.20倍）。

極小サイズで一律に速くなるわけではない。例えば旧Doublingの密な8×8は約0.45µs、新は約0.50µs。
型・法・サイズ・密度・CPUで結果は変わる。特にu32上限近くの法ではバッチが小さくなる。
サンプル値、旧ソースと新ソースのハッシュは `matrix_benchmark_results.json` に保存。
