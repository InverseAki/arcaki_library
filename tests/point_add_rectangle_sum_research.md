# PointAddRectangleSum 性能検討（2026-10-06）

この資料はジェネリクス・build 追加前の性能検討記録。後の実装変更は [build の検証記録](point_add_rectangle_sum_build.md) を参照。この検討時点では本体を変更していない。推奨は、現状の時間方向の分割統治を維持し、y の座標圧縮対象を加算点に限定すること。同一 x のイベント順を表す第二キーの簡略化も小さな追加候補。

## 推奨する変更

1. 圧縮用の `ys` に `Add.y` だけを登録する。質問境界は `ys.partition_point(|&y| y < boundary)` で求める。質問境界自体を BIT の座標に含める必要はない。圧縮・検索・BIT のサイズが減る。
2. ソートキーを `(x, ly)` から `(x, is_add)` にする。同じ x では質問を先、加算を後に置けば、半開区間の条件を保てる。質問同士を ly 順に並べる必要はない。
3. 加算のみ・質問のみの早期終了は別の小改善候補。今回は圧縮案に組み込まず比較した。

推奨候補は `support/point_add_rectangle_sum_compressed.rs`。`SHARED = false, SIMPLE_TIES = true` が圧縮変更＋第二キー変更、両方 false が圧縮変更のみ。API と解答順、solve 後のデータ消去は現状と同じ。

## 計測結果

Apple arm64、rustc 1.93.0、`rustc --edition=2021 -O`。固定 seed 123456789、各3回の中央値。登録／候補内部形式への変換から solve 完了までを測定し、生成・コンパイル・起動・入出力は除外する。各方式を別プロセスで実行し、全回答のチェックサムと回答数の一致を確認。RSS は wait4 で測り、全方式共通の入力操作列も含む。

操作数は加算と質問の合計。mixed は交互、initial は全加算の後に全質問、add_heavy は加算90%、query_heavy は質問90%。ties は座標が8種類だけのケース。

| 操作数・ケース | 現状 | y圧縮のみ | 圧縮＋第二キー変更 | 後者の時間短縮 |
|---|---:|---:|---:|---:|
| 200,000 / ties | 0.058557s | 0.059752s | 0.055775s | 4.8% |
| 1,000,000 / mixed | 0.841985s | 0.751566s | 0.752266s | 10.7% |
| 1,000,000 / initial | 0.167934s | 0.138362s | 0.137106s | 18.4% |
| 1,000,000 / add_heavy | 0.697865s | 0.657367s | 0.660911s | 5.3% |
| 1,000,000 / query_heavy | 0.913329s | 0.744708s | 0.755016s | 17.3% |

100万操作 mixed のピークRSSは、現状95.44 MiB → 推奨候補84.03 MiB。query_heavy は107.62 MiB → 87.16 MiB。y が元から少ない ties では圧縮だけの利点はほぼなく、微小な回帰もあった。第二キー変更で現状より約5%短縮した。

## 大きな変更を採用しない理由

- `presort` / `add_only`: 全イベントを x 順に一度ソートして、時間方向へ安定振り分けする方式。各再帰の走査・コピーが重い。mixed 100万操作では約0.987 / 0.926sで、現状0.842sより遅く、メモリも増えた。
- `merge`: 時系列に再帰し、子の x 順イベントをマージする方式。add_heavy 100万操作では0.556sで現状より約20%速いが、initial は0.187sで現状より遅い。mixed でも圧縮変更のみより遅く、メモリが増えるため標準実装としては推奨しない。
- `shared`: y圧縮変更に加えて、BIT.prod の二つの prefix が共有する祖先を省略する方式。通常の広い区間では分岐の増加が不利で、今回の主要ケースで圧縮変更のみを上回らなかった。

計算量はどの案も概ね O(Q log² Q)。今回の判断は定数・実メモリに基づく。各ノードのソート除去は理論上改善があるが、全体の BIT 操作が引き続き log² の項を持つ。

## 正しさの確認と現状の注意点

- 9種類の分布×2000 seed＝18,000操作列を愚直解と比較。debug・最適化ビルドの両方で7方式すべて一致。空、1操作、負の重み、重複点、同一座標、空矩形、i32 の端点、質問より後の更新、solve 後の再利用を確認。
- 13種類の大規模ケース×7方式×3回＝273実行でチェックサムと回答数を比較。
- 本体は `I` が未定義。今回の現状比較は、外側から `const I: i32 = i32::MAX` を補っている。単独コピー可能にするにはこの依存の解消が必要。第二キーを簡略化してもイベント種別判定の番兵を使う限り、定数の定義は必要。
- 空の操作列は、現状の実装でも空配列を返す。dfs の基底条件に明示されていないが、両方の emp フラグが true のため再帰しない。既存の algorithm_set の説明にある「そのまま solve しない」は現状の動作と合わない。実行でも確認した。
- 候補のイベント時刻と y の圧縮順位は u32 に格納する。これは実験用表現であり、production へ採用する場合は必要な上限を明示する。圧縮変更のみの候補は元実装と同じ型を維持する。
- 外部ジャッジ提出、Linux/x86での計測、全ての入力分布での優位性は未確認。

## 再実行

リポジトリの親 `competitive_programing` から：

```sh
rustc --edition=2021 --test arcaki_library/tests/point_add_rectangle_sum_research.rs -o /tmp/point_rectangle_tests
/tmp/point_rectangle_tests
rustc --edition=2021 --test -O arcaki_library/tests/point_add_rectangle_sum_research.rs -o /tmp/point_rectangle_tests_release
/tmp/point_rectangle_tests_release
python3 arcaki_library/tests/run_point_add_rectangle_sum_research.py
```

全実測値は `point_add_rectangle_sum_research_results.json`。benchmark の旧実装は `support/point_add_rectangle_sum_baseline.rs` に保存した、ジェネリクス・build 追加前のスナップショットを include する。
