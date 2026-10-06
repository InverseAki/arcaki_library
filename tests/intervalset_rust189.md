# Rust 1.89.0 対応 IntervalSet / IntervalSetV

2026-10-06 に追加。現在の extract_if 版は別ファイルとして維持する。
コピーするときは用途に応じて次の一つを選ぶ。

- [IntervalSet](../src/Basic/intervalset_rust189.rs)
- [IntervalSetV](../src/Basic/intervalsetv_rust189.rs)

型名・引数順は通常版と同じ。with_data は Vec を確保せずイテレータを返す。
固定長の記録バッファを使い、途中破棄でも更新を完了する。
記録を保存する場合は collect::<Vec<_>>() を使う。
同名の型を持つため、同じ名前空間に通常版と互換版を両方貼らない。

## 性能上の変更

- 左端が変わらない短縮は、range_mut で得た右端を直接更新する。
- 左側の既存区間を統合するときは左端キーを残し、最後に右端を上書きする。
- 次の検索下限を、直前に処理した区間の右端へ進める。
- next をインライン化し、通常APIや途中破棄では残りの更新を専用ループで完了する。

木の内部カーソルを持つ実装ではなく、区間ごとに根から検索する。
計算量は従来どおり O((k+1) log s)、記録用追加領域は O(1)。
既存版の変更記録順序・区間表現を維持する。
IntervalSetVで旧版が同値の隣接区間を残すケースも今回の範囲では維持する。

## 正しさ

実際の rustc 1.89.0 (29483883e 2025-08-04) で以下を実行し、合計7テスト成功。

```sh
cargo +1.89.0 test --test intervalset_rust189 --test intervalsetv_rust189
cargo +1.89.0 test --release --test intervalset_rust189 --test intervalsetv_rust189
```

IntervalSetは61,952操作、IntervalSetVは177,147操作を全列挙。
旧版との記録内容・順序・最終集合、点ごとの単純モデル、通常API、
全ての記録位置での途中破棄を比較する。
それぞれ10万操作の連続更新と i64::MIN / MAX 境界も検証。
外部ジャッジへの提出は行っていない。

## 性能比較

macOS arm64、rustc 1.89.0、rustc -O、各条件5回の中央値。
構造ごとの両実装を同じバイナリに含め、実行順は交互。
結果のチェックサムも比較する。

比較元は、IntervalSetが [extract_if導入直前のイテレータ版](support/intervalset_research_baseline.rs)、
IntervalSetVが [変更記録をVecで返す旧版](support/intervalsetv_before_extract.rs)。
IntervalSetの比較元には区間短縮時の直接更新が既に含まれる。

初期区間数65,536、noop/shorten/randomは100万操作。
noopは既存区間内への挿入、shortenは右側2点の削除・挿入、
randomは長さ1..64の挿入・削除。
IntervalSetVの初期値は0/1/2、randomの挿入値は0..3。
bulk_kは初期集合を複製してk区間ずつ統合・削除する処理を16回。

更新時間には初期構築・複製・結果確認・集合破棄を含めない。
全体時間には初期構築・複製・結果確認とbulk各回の集合破棄を含む
（最終集合の破棄は計時後）。
記録ありは変更記録を逐次集約し、通常APIは insert/remove を使う。
倍率は比較元 / 互換版、時間はms。

| 型 | API | 条件 | 更新: 比較元 | 更新: 互換版 | 倍率 | 全体: 比較元 | 全体: 互換版 |
|---|---|---|---:|---:|---:|---:|---:|
| intervalset | 記録あり | noop | 79.489 | 78.785 | 1.01 | 95.982 | 95.402 |
| intervalset | 記録あり | shorten | 158.157 | 133.608 | 1.18 | 174.609 | 149.994 |
| intervalset | 記録あり | random | 211.579 | 201.381 | 1.05 | 225.152 | 214.543 |
| intervalset | 記録あり | bulk_1 | 229.889 | 169.691 | 1.35 | 245.165 | 184.886 |
| intervalset | 記録あり | bulk_8 | 109.114 | 98.144 | 1.11 | 122.006 | 111.110 |
| intervalset | 記録あり | bulk_256 | 52.034 | 51.637 | 1.01 | 64.502 | 64.196 |
| intervalset | 記録あり | bulk_65536 | 34.918 | 34.727 | 1.01 | 47.211 | 47.189 |
| intervalset | 通常API | noop | 73.788 | 75.350 | 0.98 | 90.060 | 91.946 |
| intervalset | 通常API | shorten | 146.600 | 125.849 | 1.16 | 163.207 | 142.256 |
| intervalset | 通常API | random | 201.131 | 190.638 | 1.06 | 214.228 | 203.795 |
| intervalset | 通常API | bulk_1 | 201.734 | 152.483 | 1.32 | 216.832 | 167.864 |
| intervalset | 通常API | bulk_8 | 103.903 | 94.712 | 1.10 | 116.771 | 107.682 |
| intervalset | 通常API | bulk_256 | 49.968 | 49.583 | 1.01 | 62.413 | 62.080 |
| intervalset | 通常API | bulk_65536 | 32.800 | 32.886 | 1.00 | 45.168 | 45.404 |
| intervalsetv | 記録あり | noop | 320.179 | 205.127 | 1.56 | 336.863 | 221.496 |
| intervalsetv | 記録あり | shorten | 269.905 | 170.003 | 1.59 | 286.544 | 187.766 |
| intervalsetv | 記録あり | random | 286.034 | 220.896 | 1.29 | 299.432 | 234.192 |
| intervalsetv | 記録あり | bulk_1 | 352.228 | 234.467 | 1.50 | 368.389 | 251.059 |
| intervalsetv | 記録あり | bulk_8 | 150.176 | 115.066 | 1.31 | 164.873 | 129.564 |
| intervalsetv | 記録あり | bulk_256 | 56.851 | 54.676 | 1.04 | 70.232 | 68.515 |
| intervalsetv | 記録あり | bulk_65536 | 36.340 | 37.258 | 0.98 | 49.686 | 50.696 |
| intervalsetv | 通常API | noop | 299.353 | 206.626 | 1.45 | 316.275 | 223.557 |
| intervalsetv | 通常API | shorten | 258.675 | 169.504 | 1.53 | 275.986 | 186.353 |
| intervalsetv | 通常API | random | 278.856 | 213.294 | 1.31 | 293.505 | 226.678 |
| intervalsetv | 通常API | bulk_1 | 306.849 | 225.092 | 1.36 | 324.199 | 243.217 |
| intervalsetv | 通常API | bulk_8 | 125.981 | 113.448 | 1.11 | 141.078 | 132.885 |
| intervalsetv | 通常API | bulk_256 | 53.358 | 53.606 | 1.00 | 67.245 | 67.924 |
| intervalsetv | 通常API | bulk_65536 | 34.665 | 34.976 | 0.99 | 48.177 | 48.895 |

今回の条件では小さい更新が改善し、256区間以上の大量更新はほぼ同等。
一律に高速とはせず、全測定値を掲載している。
extract_if版の大量更新での高速化を、そのまま1.89版で実現したものではない。
他のCPU・ワークロードで同じ倍率になる保証はない。

再測定は `python3 tests/run_intervalset_rust189_benchmark.py`。
[全測定値](intervalset_rust189_benchmark_results.json)。
