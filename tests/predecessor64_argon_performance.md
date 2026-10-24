# Predecessor64: Gemini4-Argon の提案検証と追加最適化

2026-10-21〜22。添付の7提案を、変更前のソースを固定して比較した。
公開API・`!0` の返り値・文字列のバイト単位の解釈・パディング領域を維持した。
最終ソースは `src/DataStructure/predecessor64.rs`。

## 採用した変更

- 検索・更新の最下層を切り出し、`offset[0]` の読み取りとループ処理を省略。
- 公開メソッドで入力位置を確認した後、内部のワードアクセスを2つの補助関数に集約して境界チェックを省略。
- `from_vec_u64` は64ワードの完全な塊と端数を分けて構築。
- `from_01_string(String)` は8バイトずつASCIIの `1` と比較し、乗算でビットへ圧縮。
- `new(0)` を `new(1)` と同じ空集合に修正。ワード列の値域計算はオーバーフローを検査。

初期比較13実装、追加比較10実装、最後の上層伝播・イテレータ案を含め、
変更の組み合わせも比較した。単に全提案を重ねるより、上記の組み合わせを選んだ。

## 7提案の判定

| 提案 | 判定 | 検証結果・理由 |
|---|---|---|
| ① `get_unchecked` | 条件付き採用 | 公開メソッドの実行時検査を含めて比較。`debug_assert!` だけの提示コードは、範囲外の入力から未定義動作になるため採用不可。内部アクセスに限定した版で小幅な改善を確認。offset配列の通常のアクセス検査は残す。 |
| ② 最下層の切り出し | 採用 | `offset[0]` の加算、最初の反復、inclusive判定を分離。マスクとワードシフトの両方を試し、マスク方式を維持。数倍の改善という結果ではない。 |
| ③ 重複時の書き込み省略 | 不採用 | 初期44条件では更新が変更前比約10.3%遅く、混合も改善しなかった。重複挿入・不在削除だけの専用測定なら最終版より約3〜8%有利になるため、用途限定の利点はある。 |
| ④ min/maxの端ワード確認を削除 | 不採用 | 初期の追加測定でmin/maxは幾何平均約12.3%遅く、密な集合では約43%遅い。空集合・中央の1点では少し有利。既存の端ワード先行を維持。 |
| ⑤ 根を先頭へ移動 | 不採用 | 階層順を逆転した候補は主要操作で安定した改善なし。構築は約58%遅い。逆転だけでは部分階層のoffsetを削除できず、完全木での固定配置はサイズが64の冪を少し超えた場合のメモリ増加が大きい。葉を先頭に置く構成を維持。 |
| ⑥ offset縮小・深さ固定 | 不採用 | `[usize; 6]` は64bit環境で構造体88バイト。元は128バイトで、6個にしても64バイトには収まらない。6階層は値域 `2^36` までで、それを超える既存の対応範囲を失う。縮小候補に明瞭な速度優位なし。深さ1〜5のconst特殊化を実行時に選ぶ候補も試したが、密度による逆転があり全体ではほぼ同速。11階層対応を維持。 |
| ⑦ 構築・usize変換 | 一部採用 | 条件式への置換だけはほぼ同速。`chunks_exact(64)` による構築と文字列パックには大きな改善。`from_vec_usize` のAPI変更・ポインタ変換は不要。 |

補足:

- `reserve_exact` はcapacityが不足すれば再割当し得る。「ゼロアロケーション」は保証されない。
- `Vec<usize>::into_iter().map(|x| x as u64).collect()` の単体コードは、ローカルの64bit Rust 1.89.0ではVecのメタデータを移すだけの機械語になった。これは32bit環境や将来のコンパイラに対する保証ではない。`from_vec_u64` 側の上層用の再割当は別問題。
- `63 ^ leading_zeros()` と減算は、入力が非零なら等価。明示的な非零分岐を置いた単体関数では、ARM64 Rust 1.89.0とx86_64 Rust 1.93.0で同じ関数へ統合された。0のときは等価ではない。木全体のXOR候補にも明瞭な改善がなかったため減算を維持。
- 専用CPU命令が使えるかはCPUとコンパイル設定次第。実測はARM64のみ。x86_64は機械語の生成とライブラリのクロスコンパイルを確認し、実行性能は測っていない。x86_64のBMI向けの変更は導入していない。
- 挿入・削除は疎密・操作分布によって伝播回数が変わり、最悪計算量は引き続き `O(log₆₄ U)`。

## 最終版の操作性能

Rust 1.89.0、aarch64-apple-darwin、`-O`。
変更前との条件別中央値の比を幾何平均した**処理時間の短縮率**。
各条件100万操作、11反復、実装の実行順を交互に変更。

| 操作 | 通常設定 | `target-cpu=native` |
|---|---:|---:|
| 狭義の前後検索 | 4.8% | 4.2% |
| 等号込みの前後検索 | 4.7% | 5.0% |
| 挿入・削除 | 3.5% | 3.8% |
| 検索・更新の混合 | 2.4% | 2.5% |

値域64 / 4096 / 262144 / 16777216、stride 2 / 64 / 4096のうち成立する11組合せ。
更新は初期点のプール内で行い、約25%は状態を変えない操作。
初期化・入力生成・入出力は時間から除外し、最終状態は `black_box` で保持。
検索結果のチェックサムを全候補・全反復で照合。
通常設定での個別の時間比は、狭義検索0.897〜0.980、等号込み0.920〜0.986、
更新0.912〜1.042、混合0.944〜1.002。全条件での高速化は保証しない。

別の空集合・中央の1点・ワード末尾・密な集合の測定では、
min/max平均5.8%、所属確認3.1%、狭義検索5.5%、等号込み3.1%短縮。

## 構築性能

入力のコピー準備と構築した集合の破棄を時間から除外。各33反復、中央値。
追加測定はワード数1 / 64 / 65 / 4096 / 4097 / 156250 / 262144 / 262145、
全零・全非零・疎・ランダムの4パターン。
文字列は156250ワード相当以下を対象とし、ASCII以外の意味は変更していない。

| 入力 | 構築 | 変更前 | 最終版 | 短縮率 |
|---|---|---:|---:|---:|
| 262144ワード・全非零 | u64ワード列 | 100.875µs | 62.500µs | 38.0% |
| 262145ワード・全非零 | u64ワード列 | 70.709µs | 32.083µs | 54.6% |
| 262145ワード・全非零 | usizeワード列 | 150.792µs | 112.792µs | 25.2% |
| 1000万文字・全0 | 文字列 | 3.041ms | 2.053ms | 32.5% |
| 1000万文字・全1 | 文字列 | 19.666ms | 2.040ms | 89.6% |
| 1000万文字・ランダム | 文字列 | 18.352ms | 2.054ms | 88.8% |

追加測定全条件の時間比の幾何平均は、u64構築0.703、usize構築0.824、文字列0.395。
極小の入力ではほぼ同速で、文字列では条件によって最大約27%の遅れもある。
大きい入力の全1で得た9.6倍を全入力に一般化しない。
同じ内容でもサイズ・再割当・実行中のCPU状態により絶対時間は変動する。

## 入出力込みの測定

`examples/predecessor_problem.rs` の集合実装だけを差し替えた。
Rust 1.89.0 `-O`、値域1000万、100万クエリ、9反復の中央値。
通常ファイル入力、標準出力を回収し、全バイト一致を確認。
プロセス起動・入力・構築・整数整形・出力・破棄を含む。
初期文字列は共通の `Input::binary_u64_len` で処理するため、
`from_01_string` の改善はこの測定には含まれない。

| 分布 | 変更前 | 最終版 | 時間の変化 |
|---|---:|---:|---:|
| random | 24.645ms | 24.460ms | -0.8% |
| sparse | 33.447ms | 32.454ms | -3.0% |
| all0 | 24.565ms | 24.152ms | -1.7% |
| all1 | 33.715ms | 33.604ms | -0.3% |
| query012 | 37.425ms | 37.812ms | +1.0% |

公式生成器に似た分布の**ローカル生成入力**。公式テストや外部ジャッジへの提出結果ではない。
操作単体の数%の改善は、全体では小さく、逆転もある。
この表だけから1%程度の差を安定した勝敗とは判断しない。

## 安全性と境界の契約

`word` / `word_mut` の `unsafe` は、次の不変条件を根拠とする。

1. 全コンストラクタは `mx > 0`、1〜11階層、根が1ワードの正確なlayoutを作る。
2. `insert/remove` は `assert!(p < mx)` をリリースでも実行し、書き込み前に検査する。
3. `include` は値域外ならfalse。`prev/inprev` は値域外なら最大要素、`next/innext` は `!0`。よって葉のアクセスは値域内。
4. 上層ビットは直下のワードが非零のときだけ1。更新は空/非空の遷移だけを伝播する。探索で選んだ子は存在し、非零であり、下降先の位置は有効。
5. フィールドは非公開で、cloneは全状態を複製する。文字列のパディングと空入力も正しいlayoutに統一。
6. `from_vec_u64` の値域の乗算は検査し、オーバーフローした値域を持つ構造体を作らない。

値域外更新はpanicし、状態を変更しない。`new(0)` は値域1として0を挿入可能。
`from_01_string` の値域は64の倍数で、パディング領域への更新は引き続き可能。

`tests/predecessor64.rs` の14テストはRust 1.89.0のdebug/releaseで全通過。
小値域の全部分集合、階層境界、ランダム更新をBTreeSetと照合し、
構築・clone・余分なcapacity・Unicodeを含む文字列を確認。
追加で8ビット全256パターンと0〜72バイトの端数、ASCII各値、
範囲外位置 `mx` / `mx+1` / `usize::MAX`、panic後の状態を検証。
全階層の要約ビットを実データと照合するテスト、および割当なしで
`usize::MAX` までのlayoutを検証するテストも追加した。
入出力込みの小ケースはPythonの独立モデルと10000操作を照合。

## 再現用ファイル

- `support/predecessor64_argon/baseline.rs`: 今回の変更前のソース。
- `run_predecessor64_benchmark.py`: 候補を一時ディレクトリに生成し、モデル・性能を比較。外部クレート不要。
- `support/predecessor64_argon/{bench,edges,constructors,noops}.rs`: 測定テンプレート。
- `predecessor64_argon_candidates.{csv,json}`: 初期13実装。
- `predecessor64_argon_refined.{csv,json}`: 切り出し・unchecked・シフト・構築の組合せ10実装。
- `predecessor64_argon_final_candidates.{csv,json}`: 最後の伝播・イテレータ案との比較。
- `predecessor64_argon_{final,native}.{csv,json}`: 最終版の操作・初回の3サイズの構築・端のケース・不在/重複更新。
- `predecessor64_argon_constructors_extended.{csv,json}`: 最終版の8サイズへの構築測定拡張。
- `predecessor64_argon_noops.{csv,json}`: 書き込み省略案との不在/重複更新比較。
- `run_predecessor64_e2e.py` / `predecessor64_argon_e2e.{csv,json}`: 全体測定と環境情報。
- `support/predecessor64_argon/codegen.rs` / `codegen_{arm,x86}.s`: usize変換とXOR比較の機械語。ARMはRust 1.89.0、x86はRust 1.93.0。

`arcaki_library` から実行。CSV/JSONは同名の結果を上書きする。
現在の構築テンプレートは追加測定の8サイズを含む。

```sh
cargo +1.89.0 test --offline --test predecessor64
cargo +1.89.0 test --offline --release --test predecessor64
python3 tests/run_predecessor64_benchmark.py --variants baseline,current --count 1000000 --repeats 11 --output predecessor64_argon_final
python3 tests/run_predecessor64_benchmark.py --variants baseline,current --count 1000000 --repeats 11 --native --output predecessor64_argon_native
python3 tests/run_predecessor64_benchmark.py --variants baseline,current,nostore --only noops --count 1000000 --repeats 11 --output predecessor64_argon_noops
python3 tests/run_predecessor64_benchmark.py --variants baseline,current --only constructors --repeats 11 --output predecessor64_argon_constructors_extended
python3 tests/run_predecessor64_benchmark.py --model --variants baseline,guarded,peeled,unchecked,nostore,root,branchless,chunks,packed,xor,unroll,topdown,compact --count 500000 --repeats 7 --output predecessor64_argon_candidates
python3 tests/run_predecessor64_e2e.py
rustc +1.89.0 --edition=2021 -O --emit=asm tests/support/predecessor64_argon/codegen.rs -o /tmp/predecessor64-codegen-arm.s
rustc --edition=2021 -O --target x86_64-unknown-linux-gnu --emit=asm tests/support/predecessor64_argon/codegen.rs -o /tmp/predecessor64-codegen-x86.s
```

初期候補の一部（固定6階層など）は比較専用で、本体の対応範囲や境界の契約を満たさない。
`--model` の候補モデルは有効範囲での更新・検索を検証するもので、
本体についての全コンストラクタ・範囲外・内部要約検証は上記のCargoテストで行う。
