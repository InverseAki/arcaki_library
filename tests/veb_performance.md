# VebTree の実装と性能比較

> この文書は初版の測定記録です。現行実装と今回の候補比較は [Gemini4-Argon 指摘後の高速化](veb_argon_performance.md) を参照してください。初版のソースは [baseline.rs](support/veb_argon/baseline.rs) に保存しています。

2026-10-25。Predecessor64 の置換候補として追加したビット集合併用の vEB 木。
**全面的な上位互換の速度は達成していない。** 所属判定や一部の検索・更新で速いが、疎な更新や入出力込みでは負ける条件があるため、既存実装を残して選択可能にした。

## 実装と契約

`src/DataStructure/veb_tree.rs` は単独でコピー可能。型名は `VebTree`。
公開メソッド・引数・戻り値・Clone/Debug は `Predecessor64` と同じ。
`use veb_tree::VebTree as Predecessor64;` で利用側の型名も合わせられる。

- `new(U)` の値域は `[0, max(U, 1))`。
- `insert/remove` は範囲外で状態変更前に panic。重複挿入・不在削除は無操作。
- `include` は範囲外で false。
- `prev/next` は狭義、`inprev/innext` は等号を含む。見つからない場合は `usize::MAX`。
- 範囲外の `prev/inprev` は最大値、`next/innext` は `usize::MAX`。
- `from_vec_u64/from_vec_usize` は各ワードの下位ビットを小さい位置とする。
- `from_01_string` はバイト位置で ASCII `1` のみを採用。値域は64の倍数へ切り上げ、パディング位置にも挿入できる。空の入力の値域は1。

下位は64ビットの葉と、その非空状態を表す64ビットの要約。4096位置ごとの非空状態をvEB木で管理する。vEBノードは最小値を子から除外し、最大値と非空クラスタの要約を保持する。再帰を要する処理と定数時間の処理を分けてインライン化する。
アルゴリズムの参考は [MIT 6.851 Lecture 11](https://ocw.mit.edu/courses/6-851-advanced-data-structures-spring-2012/5714d393dbd597b3f070ca4be17341ec_MIT6_851S12_L11.pdf)。資料のコードの転載ではない。

`include/min/max/is_empty` は O(1)。更新・隣接検索は O(1 + log log max(U, 2))。
領域全体を確保するため空間は O(ceil(U/64)) ワード。要素数比例の疎な木ではなく、usize全域を実用的に確保する用途には向かない。
`new` は O(ceil(U/64))。ワード列からの構築は葉の走査と非空4096ブロックの挿入を行うため、O(ceil(U/64) + ceil(U/4096) log log max(U,2)) が上界。
unsafe は検証済みの葉アクセスだけに限定。公開入力の実行時チェックを省略していない。

## 操作単体

Rust 1.89.0、`rustc -O`、aarch64-apple-darwin。固定seed。各条件100万操作、9回、実装順を交互に変更し中央値を比較。初期構築と破棄を含まない。91条件、全実装でチェックサム一致。
値域は64/4096/262144/16777216、初期要素は各stride区間から1個。更新は初期位置を対象に有無を切り替え、一部は重複・不在更新。mixedは約半数が更新。emptyは初期集合を空にして検索。

倍率は既存実装の時間 / VebTreeの時間。1より大きければVebTreeが速い。

|操作|最小倍率|条件間中央値|最大倍率|
|---|---:|---:|---:|
|strict|0.920|1.021|1.151|
|inclusive|0.910|1.004|1.180|
|update|0.845|1.031|1.197|
|mixed|0.875|1.000|1.141|
|contains|1.297|1.369|1.467|
|extrema|0.964|0.971|1.196|
|empty|0.916|1.045|1.130|

大きい値域 U=16777216 の主な結果（ns/操作）：

|stride|操作|Predecessor64|VebTree|倍率|
|---:|---|---:|---:|---:|
|2|strict|4.178|4.124|1.013|
|2|update|3.132|2.817|1.112|
|2|mixed|6.105|5.980|1.021|
|64|strict|9.556|9.230|1.035|
|64|update|7.061|6.605|1.069|
|64|mixed|11.394|11.026|1.033|
|4096|strict|8.875|9.645|0.920|
|4096|update|7.285|8.621|0.845|
|4096|mixed|9.787|11.184|0.875|
|16777216|strict|6.553|5.691|1.151|
|16777216|update|3.699|3.090|1.197|
|16777216|mixed|7.603|6.664|1.141|

## 入出力・構築込みとメモリ

値域1000万、100万操作、7回。共通の入力・出力コードで実装だけを切り替えた。プロセス起動、構築、出力ファイルへの書き込み、破棄を含む。全出力が一致。小ケースではPythonの独立した整列リストモデルとも10000操作を比較した。RSSは `wait4` が返す各子プロセスのピーク値で、集合以外に入力と出力バッファ等も含む。

|分布|Predecessor64 ms|VebTree ms|倍率|Predecessor64 RSS MiB|VebTree RSS MiB|
|---|---:|---:|---:|---:|---:|
|random|23.615|23.213|1.017|22.58|22.61|
|sparse|29.577|31.504|0.939|22.58|22.61|
|all0|22.560|22.179|1.017|22.58|22.61|
|all1|27.965|30.835|0.907|22.58|22.61|
|query012|19.642|19.669|0.999|22.58|22.61|

sparseは10個だけを初期配置して隣接検索。randomは半分存在し全種類の操作、all0は空集合で検索・所属判定、all1は全位置存在から削除・検索・所属判定、query012は半分存在し挿入・削除・所属判定のみ。
単体の倍率をそのままジャッジ時間の改善と解釈しない。CPU/コンパイラや分布で結果は変わる。

## 候補と再現

`tests/support/veb/` に棄却・中間候補を保存した。
- `recursive.rs`: 64位置の葉 + 通常の再帰vEB。疎な更新では約2倍遅く棄却。
- `inline.rs`: 葉のチェック済みunsafeアクセスとinline指定のみ。十分な改善にならず棄却。
- `hybrid.rs`: 下位4096位置を2段のビット演算に変更。
- `split.rs`: 検索の定数時間部分を再帰から分離。
- 本体: 更新も分離し、min/maxに端の葉の高速経路を追加。

各候補のCSV/JSONは `veb_candidate_*`。最後の候補は `veb_candidate_update_split`、最終の長時間再測定は `veb_final`。JSONにはコンパイラ・環境・ソースSHA256を記録した。

```sh
cargo +1.89.0 test --offline --test predecessor64 --test veb_tree
cargo +1.89.0 test --offline --release --test predecessor64 --test veb_tree
python3 tests/run_veb_benchmark.py --count 1000000 --repeats 9 --output veb_final
python3 tests/run_veb_e2e.py
python3 tests/run_veb_benchmark.py --veb-source tests/support/veb/recursive.rs --output veb_recursive_rerun
cargo +1.89.0 run --release --example veb_predecessor_problem < input.txt
```

## 正しさ

BTreeSetとの照合、サイズ10までの全状態、各64進境界、重複・不在更新、空集合への復帰、全コンストラクタ、Clone、UTF-8バイト位置、範囲外panicと状態保存を検証した。共通モデル・コンストラクタ検証は既存実装と共有する。追加で2^28+129の領域を使い、深い再帰と最小・最大削除を検証した。
Rust 1.89.0のdebug/releaseで既存14テスト、新規6テストが通過。外部ジャッジへの提出・公式ジェネレータによる検証は未実施。ここでの結果はローカル検証。
