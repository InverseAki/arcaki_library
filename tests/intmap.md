# 整数キーの IntMap：検証と性能比較

[使い方](../../algorithm_set/DataStructure/intmap.md) · [本体](../src/DataStructure/intmap.rs) · [使用例](../examples/associative_array.rs)

## 検証

2026-10-21、macOS ARM64。Rust 1.89.0 / 1.93.0で8テストをdebug・releaseの両方で実行して通過。

- u32 / u64 / usize各15万操作、計45万操作を標準HashMapと照合。挿入・置換・検索・削除・entry・更新・reserve・clear・clone・全内容を比較。
- 同じバケット位置に96キーを集中させ、削除・再挿入後の検索を検証。
- 0、最大値、空map、ゼロサイズ値、String、Clone/Default不要の値、IndexMut、所有・参照イテレータ、標準Entry、retain、drainを確認。
- 値のdrop回数、entryの遅延初期化とpanic、cloneのpanic時の後始末、clear中の値のdropがpanicした後の再利用を確認。drop自体のpanic時に残りの値まで破棄される保証は標準HashMapと同様に置かない。
- Associative Array形式の使用例をRust 1.89 releaseで実行。境界キーと任意のu64値を含む10万操作をPython dictと照合。外部ジャッジへの提出は行っていない。

```sh
cargo test --offline --test intmap
cargo test --offline --release --test intmap
cargo +1.89.0 test --offline --test intmap
cargo +1.89.0 test --offline --release --test intmap
cargo +1.89.0 build --offline --release --example associative_array
python3 tests/run_intmap_benchmark.py
```

## FxMapとの比較

比較対象は変更していない `src/Basic/fxhash.rs` のFxMap。Rust 1.89.0、`rustc --edition=2021 -O`。

キー3型 × サイズ3種 × 分布3種 × 操作10種 = 270条件。値はu64。分布は連番・全単射の疑似乱数・ビット偏り（u32は16bit回転、u64/usizeは32bit左シフト）。キー列と不存在検索列は重複せず、それぞれ決定的にシャッフルする。各条件で1回ウォームアップ後に6回計測し中央値を採る。実行順序は交互。準備・破棄・入出力を計測から除外し、結果と最終内容のチェックサム・要素数を照合する。

次表は連番・疑似乱数について、3キー型×2分布の倍率の中央値。倍率は `FxMapの時間 / IntMapの時間`。1を超えるとIntMapが速い。

| 要素数 | 予約済み挿入 | 予約なし構築 | 存在検索 | 不存在検索 | get_mut更新 | entry更新 | 削除 | 削除+挿入+検索 |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1,024 | 1.02× | 1.06× | 1.11× | 1.10× | 1.07× | 1.09× | 1.16× | 1.10× |
| 100,000 | 1.08× | 1.06× | 1.17× | 1.23× | 1.12× | 1.14× | 1.04× | 1.10× |
| 1,000,000 | 1.00× | 1.05× | 1.05× | 1.12× | 1.06× | 1.06× | 1.06× | 1.00× |

10万要素の存在検索は通常分布で1.10〜1.25倍、不存在検索は1.09〜1.38倍。ビット偏り分布ではFxMapの衝突傾向を避けて大幅に改善する条件もあるが、最大倍率を代表値にはしない。

全条件で速くなる保証はない。通常分布の100万要素で予約済み挿入は中央値でほぼ同速、混合操作もほぼ同速。一部条件では数%遅い。全走査はハッシュ計算をせず、配置の差・キャッシュ・計測揺らぎの影響を受ける。単一環境の測定であり、別CPU・別コンパイラ・別サイズ・値型・実問題の速度を保証しない。

[全270条件の数値](intmap_benchmark_results.csv) · [環境・コンパイラ・ソースSHA256・集計](intmap_benchmark_results.json) · [計測本体](benchmark_intmap.rs) · [再現スクリプト](run_intmap_benchmark.py)

## 採用方式

整数キーを固定定数とXORし、64×64→128bit積の上下64bitをXORする専用Hasherを標準HashMapに組み込む。u32 / u64 / usizeに限定するラッパーで、追加の格納領域・外部依存・独自unsafe実装はない。基本操作とメモリ確保は標準HashMapに任せる。

低充填率の独自線形探索表はランダムアクセス・走査・大規模構築に不利な条件が多かった。乗算1回とシフトだけの簡略ハッシュも不存在検索などで悪化したため、いずれも採用していない。
