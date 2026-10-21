# GF(2) 行列積

実装: `src/DataStructure/bitset.rs` の `BitMatrix::prod_xor`。
`pow_xor` も同じ高速な積を使う。既存 API・格納形式・他の論理積は維持した。
外部クレート・unsafe・CPU 固有命令の指定は不要。
コピーするときは既存と同じく次の import を付ける。

```rust
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign};
```

```rust
let a = BitMatrix::from_rows(3, vec![
    BitSet::from_binary(b"101"),
    BitSet::from_binary(b"011"),
]);
let b = BitMatrix::from_rows(2, vec![
    BitSet::from_binary(b"10"),
    BitSet::from_binary(b"11"),
    BitSet::from_binary(b"01"),
]);
let c = a.prod_xor(&b);
let mut line = vec![0; c.width()];
c.row(0).write_binary(&mut line);
assert_eq!(line, b"11");
c.row(1).write_binary(&mut line);
assert_eq!(line, b"10");
```

`mul_xor` は従来どおり XOR の個数を返す別の演算。
GF(2) の積には `prod_xor` を使う。
`from_binary` は先頭の文字をビット0に格納し、不正な文字は panic。
`write_binary` は同じ順序で書き出し、長さが異なるバッファは panic。
両方とも8文字単位で変換し、空列・64ビット境界の端数も扱う。

## 高速化

- 通常の密行列は Four Russians 法。右辺の4/6/8行の XOR 全組合せを作り、左辺の4/6/8ビットを添字にして読む。
- 4表または8表を同時に処理し、結果の読み書きをまとめる。連続した u64 の XOR はコンパイラの自動ベクトル化を利用する。
- 表の列方向の幅を1/2/4/8/16/32/64ワードから選び、幅4096以上ではパネルに分割する。作業表は最大1 MiB。
- 左辺64行以下または内側次元32以下なら4ビット表、左辺65..512行は6ビット表、それ以外は8ビット表。
- 左辺の1の密度が1/32以下なら1の位置だけを列挙して右辺の行を XOR。
- 密度が31/32以上なら左辺を反転し、右辺全行の XOR を補正項として加える。ほぼ全て1のケースも少ない位置列挙で処理できる。
- 左辺4行以下も直接行 XOR。右辺64列以下かつ内側次元64以上では、上記の疎/密判定の後に転置・内積を使い、ワードごとの AND を XOR 集約して最後に一度だけ popcount する。
- 空次元は直ちに0行列を返す。端数ビットは0を維持し、入力は変更しない。長方形と `pow_xor` に対応。

H×K と K×W の積で L=ceil(W/64)、表のビット幅をbとすると、
表構築・XORの主要部分は O((H+2^b) ceil(K/b) L)。
表は一度に4/8個だけ保持するため、全ての表を保存する必要はない。
出力は O(HL) ワード。作業表のほか、疎な積は1行分、内積方式は転置右辺の領域を使う。

## 検証と再計測

[対象問題](https://judge.yosupo.jp/problem/matrix_product_mod_2) の
[公式制約](https://github.com/yosupo06/library-checker-problems/blob/master/linear_algebra/matrix_product_mod_2/info.toml) は各次元1..4096、制限10秒。
提出用の入出力例は `examples/matrix_product_mod_2.rs`。

```sh
cargo test --offline --test bit_matrix
cargo test --offline --release --test bit_matrix
python3 tests/run_gf2_benchmark.py
python3 tests/check_matrix_product_mod_2.py /path/to/library-checker-problems/linear_algebra/matrix_product_mod_2
```

公式リポジトリでは先に `generate.py -p matrix_product_mod_2` 等で `params.h` を生成しておく。
検証スクリプトは公式の生成器をコンパイルし、全26ケースの入力と Rust の出力の SHA-256 を
公式 `hash.json` と照合する。ネット上のジャッジへの提出は行っていない。

ベンチマークは変更前の `prod_xor`、単純な行 XOR、今回の自動選択と、
表幅2/4/6/8/10/12ビット・同時表数・パネル幅を変えた候補を比較する。
各候補の結果の一致を計測前に確認する。1回のウォームアップ後、3回の中央値を記録。
積の出力確保・前処理を含み、入出力は含まない。
疎・零・ほぼ全1・全1行列も別途測る。
`gf2_benchmark_results.json` と `matrix_product_mod_2_official_results.json` に測定結果を保存する。
CPU 固有の最速実装や他環境での同じ速度は保証しない。

## ローカル測定結果（2026-10-20）

ARM64 macOS / rustc 1.93.0 / Cargo release（CPU 固有オプションなし）。
単位 ms、乱数密行列、3回の中央値。出力確保・前処理込み、入出力は除く。

| H×K×W | 従来の prod_xor | 単純な行 XOR | 今回 |
|---|---:|---:|---:|
| 512×512×512 | 3.663 | 0.597 | 0.157 |
| 1024×1024×1024 | 12.508 | 2.542 | 0.501 |
| 4096×4096×4096 | 321.723 | 107.746 | 21.024 |
| 16×4096×4096 | 12.293 | 0.421 | 0.237 |
| 4096×4096×16 | 1.358 | 17.075 | 0.768 |
| 4096×16×4096 | 118.818 | 0.463 | 0.255 |
| 256×4096×4096 | 30.785 | 6.732 | 2.005 |

4096正方の乱数行列は従来比約15.3倍、単純な行 XOR 比約5.1倍。
全1の4096正方行列は約0.42 ms、1の密度が約63/64の行列は約5.2 ms。
密度ベンチマークの mode は0=約1/64、1=全0、2=約63/64、3=全1。
作業表の幅を広げ過ぎる12ビット法や、同時16表の候補は採用しなかった。

全体の debug テスト181件、および最終変更後の行列テスト debug/release 各7件が通過。
ベンチマーク内でも9形状すべてで各候補を比較し、密度別ケースは直接行 XOR と照合。
公式全26ケースの入出力ハッシュ一致は別ファイルに記録する。
測定値はこの環境でのローカル実測であり、外部ジャッジでの AC 記録ではない。
