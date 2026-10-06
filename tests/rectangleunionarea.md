# Rectangle union area 検証

2026-10-05。ソースを直接 include したテストを以下で実行し、debug / release とも 8 テスト成功。

```sh
rustc --edition=2021 --test tests/rectangleunionarea.rs -o /tmp/rectangle_union_test_debug
/tmp/rectangle_union_test_debug
rustc --edition=2021 -O --test tests/rectangleunionarea.rs -o /tmp/rectangle_union_test_release
/tmp/rectangle_union_test_release
```

- 3 × 3 の座標点で作る非退化矩形 9 個の全 512 部分集合と圧縮セル総当たりを比較。
- 座標 -1..=2 で作る面積 0 を含む 100 矩形の全 10,000 ペアを比較。
- 固定 seed のランダム 10,000 件を圧縮セル総当たりと比較。
- 重複 10,000 個、入れ子、接する細帯・矩形、中央に穴、空入力、極値の退化矩形。
- 全標準整数型、i128 の大面積、u64 の最大値近傍、Hash を持たない独自型。
- 逆向きの x / y は panic。

## 性能

ローカル macOS、`rustc -O --edition=2021`、座標・面積は i64。各ケース 200,000 矩形、3 回の中央値（ms）。入力生成を除き、座標圧縮・ソート・走査・メモリ確保と解放を含む。旧版は変更前のソースと同じ FxMap 実装を直接 include、新版は最終ソースを include し、各回で面積の一致も確認。

| 入力 | 旧版 | 新版 | 旧版 / 新版 |
|---|---:|---:|---:|
| ランダムな大矩形 | 173.46 | 93.47 | 1.86 |
| 入れ子 | 118.89 | 63.24 | 1.88 |
| x 範囲が共通の隣接細帯 | 45.31 | 16.06 | 2.82 |
| ランダムな小矩形 | 113.12 | 39.27 | 2.88 |

小矩形は幅・高さ 10、ランダム座標は 0..1,000,000、乱数初期値は 735 の xorshift。単一環境での測定で、実行時間には揺らぎがある。競技ジャッジの性能を保証するものではない。
