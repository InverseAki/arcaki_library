# 矩形加算・点取得のライブラリ化（2026-10-06）

実装は `src/Basic/rectangle_add_point_get.rs`。独立してコピー可能で、点加算・矩形和のファイルと同じスコープに置いても public BIT などの名前が衝突しない。

## 使用例

```rust
let mut solver = RectangleAddPointGet::<i64>::build([
    (0, 0, 10, 10, 5), // (lx, ly, rx, ry, w)
]);
solver.push_query(0, 0);
solver.push_add(1, 1, 3, 3, -2);
solver.push_query(2, 2);
solver.push_query(10, 2);
assert_eq!(solver.solve(), vec![5, 3, 0]);
```

初期矩形なしなら `new()`。型は `RectangleAddPointGet<X = i32, Y = X>` と `RectangleAddPointGetQuery<X = i32, Y = X>`。`::<i64>` で両軸 i64、`::<i128, u64>` のように軸ごとに異なる型も使える。x は Ord、y は Ord + Clone、x に Clone / Copy は要求しない。重みと答案は i64。

build は Vec・配列・所有権を渡す iterator を受け取る関連関数。初期矩形を全操作より前の状態として扱う。境界は `[lx, rx) × [ly, ry)`。重複・負の重み・空矩形を許し、逆順の境界は assert で拒否する。重みの符号反転と内部の BIT の中間値も i64 に収まることが必要。

solve は質問順に返し、初期矩形も更新も質問も消去する。次の solve に初期矩形を持ち越さない。座標値の算術・INF の番兵・外部定数・外部クレートは不要。

## 提示コードからの整理

- y の圧縮を再帰ごとに行わず、solve 全体で一度だけ行う。圧縮対象は質問点の y のみで、更新境界は lower_bound で写す。
- 同じ x の2隅を y 区間の差分更新にまとめ、矩形1個を2個の x イベントにする。各イベントでは y の下端に w、上端に -w を入れる。
- 同じ x では矩形の開始・終了イベントを点取得より先に処理する。左端を含み、右端を含まない条件が保たれる。
- 提示コードは全4隅を処理すると BIT が自然にゼロへ戻る。この性質を動的操作の時間分割でも維持し、ノードごとの逆加算を行わない。質問の最大 x で打ち切ると右端を処理し損ねるため、動的な各走査は全イベントを処理する。
- 初期矩形は時間分割に含めず、全質問に一度だけ寄与させる。初期の開始・終了端は座標参照と添字だけをソートし、質問点を別にソートして走査する。初期の寄与計算後は BIT を一括ゼロクリアし、その後の動的操作を処理する。
- 走査バッファを再帰間で再利用し、質問がない・更新がない場合は早期終了する。空矩形や質問 y に寄与しない区間も走査イベントから省く。

初期 N 矩形、後続 M 操作として、比較・Clone・i64 の演算を定数時間とみなせば概ね `O((N+M) log(N+M+1) + M log²(M+1))` 時間、`O(N+M)` 空間。

## 検証

- 8種類の分布×3000 seed＝24,000操作列で、愚直解、提示方式、先頭加算方式、build の回答が一致。debug / 最適化ビルドの両方を実行。
- -1,0,1 の端点からできる36種類の矩形の全ペア＝1,296組について、境界外も含む25点を更新前後に質問し、半開境界・空矩形・重なり・時間順を愚直解と比較。
- 7テストで、i64/i128/u64 の端点、軸ごとに別型、Clone しない x 型、String の y、負の重み、空・clone・再利用・消去、逆順矩形の拒否、終了端が全質問より右にある走査のゼロ復帰を確認。
- 点加算・矩形和と同じスコープへ include する smoke test も通過。
- 11大規模分布×3方式×3回＝99実行で全回答のチェックサムと回答数が一致。

## Library Checker の公式ケース

[対応問題](https://judge.yosupo.jp/problem/rectangle_add_point_get)。公式登録19ケースについて、入力 SHA256 を公式 hash.json と照合し、公式 verifier、公式 C++ 正解、公式 checker で全ケースを検証した。全てローカル AC。オンライン提出は行っていない。

公式リポジトリの revision: `1814c4e5205517e368bb57a8d1127eb961cfeaae`。最終実装 SHA256: `81efdbfce807e343cb970ca220b342a229fef6ebb99d0cd3751ad501f4bc8549`。

単一ファイル化した入力アダプタもコンパイルして公式ケースに使用しており、include パスへの依存がないことも確認。例題、最大ランダム、点が多い・矩形が多い、小座標、2のべき乗の構造を含む。

入力・正解・出力・stderr・checker の結果は `/var/folders/4y/78lt_6517zl9klyctd9lxrgr0000gn/T/rectangle_add_official_avh5jv9y/cases` に保存した。ケース別の時間・ハッシュ・判定は `rectangle_add_point_get_official_results.json`。

## 性能比較

Apple arm64、rustc 1.93.0、`rustc --edition=2021 -O`。固定 seed 123456789、各3回の中央値。初期登録／build と後続操作の登録から solve 完了までを測定し、生成・コンパイル・プロセス起動・入出力は除外。RSS は macOS の wait4 で取得し、比較用の入力矩形・操作列も含む。

original は提示コードの4隅・再帰ごとの y 圧縮を再現した比較用実装。空の操作列に対応する基底条件だけ補い、BIT の長さは提示コードと同じ `2*N+M`。prepend は新ライブラリへ初期矩形を先頭 push_add する方式、build は初期矩形専用走査。

static は後続が質問のみ、mixed は更新・質問が半々、add_heavy は更新9割、query_heavy は質問9割、ties は各軸が8種類の座標。

| 初期N | 後続M | 分布 | 提示方式 | 新版の先頭加算 | build | 提示方式/build |
|---:|---:|---|---:|---:|---:|---:|
| 0 | 200,000 | mixed | 0.260247s | 0.116844s | 0.117078s | 2.22倍 |
| 100,000 | 100,000 | static | 0.069825s | 0.027482s | 0.025674s | 2.72倍 |
| 100,000 | 100,000 | mixed | 0.185305s | 0.072303s | 0.069521s | 2.67倍 |
| 100,000 | 100,000 | ties | 0.163291s | 0.036758s | 0.030711s | 5.32倍 |
| 100,000 | 100,000 | add_heavy | 0.241102s | 0.074360s | 0.071845s | 3.36倍 |
| 100,000 | 100,000 | query_heavy | 0.115702s | 0.052318s | 0.049941s | 2.32倍 |
| 400,000 | 100,000 | mixed | 0.398221s | 0.133487s | 0.118646s | 3.36倍 |
| 100,000 | 400,000 | mixed | 0.758714s | 0.318483s | 0.293380s | 2.59倍 |
| 500,000 | 500,000 | static | 0.395216s | 0.215207s | 0.169457s | 2.33倍 |
| 500,000 | 500,000 | mixed | 1.162838s | 0.498008s | 0.471896s | 2.46倍 |
| 1,000,000 | 10,000 | mixed | 0.650036s | 0.177801s | 0.191685s | 3.39倍 |

今回の分布では、圧縮・イベント数の削減が主な改善。build は時間分割を初期矩形と後続操作に分けるが、初期矩形が非常に多く質問が少ないケースでは、新版の先頭加算より遅い結果もあった。全分布で build が先頭加算を上回る保証はない。

## ピークRSS（MiB）

| 初期N | 後続M | 分布 | 提示方式 | 新版の先頭加算 | build |
|---:|---:|---|---:|---:|---:|
| 0 | 200,000 | mixed | 24.70 | 30.09 | 30.09 |
| 400,000 | 100,000 | mixed | 106.64 | 68.09 | 54.45 |
| 500,000 | 500,000 | static | 199.23 | 152.62 | 129.84 |
| 500,000 | 500,000 | mixed | 236.80 | 141.17 | 124.23 |
| 1,000,000 | 10,000 | mixed | 254.95 | 127.39 | 111.89 |

生データは `rectangle_add_point_get_benchmark_results.json`。他のCPU・Linux/x86、全入力分布での優位性は未確認。

## 再実行

competitive_programing から：

```sh
rustc --edition=2021 --test arcaki_library/tests/rectangle_add_point_get.rs -o /tmp/rectangle_add_tests
/tmp/rectangle_add_tests
rustc --edition=2021 --test -O arcaki_library/tests/rectangle_add_point_get.rs -o /tmp/rectangle_add_tests_release
/tmp/rectangle_add_tests_release
python3 arcaki_library/tests/run_rectangle_add_point_get_benchmark.py
python3 arcaki_library/tests/run_rectangle_add_point_get_official.py --lc-root /path/to/library-checker-problems
```

公式 runner はダウンロードやオンライン提出を行わず、公式ソースを別ディレクトリへコピーして実行する。入力アダプタは `examples/rectangle_add_point_get.rs`。
