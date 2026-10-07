# 静的な矩形加算・矩形和：4本BIT（2026-10-07）

実装は `src/Basic/static_rectangle_add_rectangle_sum.rs`。単独コピー可能で、既存の点加算・矩形和、矩形加算・点取得と同じスコープに include しても衝突しない。

## API

```rust
let mut solver = StaticRectangleAddRectangleSum::<i64>::build([(0,0,3,4,5)]);
solver.push_query(1,1,5,5);
solver.push_add(1,1,2,2,-2);
assert_eq!(solver.solve(),vec![28i128]);
```

型は `StaticRectangleAddRectangleSum<X = i32, Y = X, W = i128>`。各軸の座標と重み・回答型を指定可能。加算・質問は `[lx,rx) × [ly,ry)` で、空矩形・負の重み・重複を許す。逆順の境界は拒否する。加算のない状態は new、初期矩形をまとめて渡す場合は build。

静的な構造なので、登録順にかかわらず全ての加算を全ての質問に適用する。solve 後は全登録を消去し、答案は質問順に返す。

W は Copy と Add/Sub/Mul を持つ可換環。整数の場合は、係数・中間値・回答が W に収まることが必要。型推論で W が入力の重み型になることもあるため、十分な計算型を使用例のように明示する。公式制約の正確な整数和は i128 に収まる。

solve は Default を加法単位元とし、Clone + From で座標を W に変換する。solve_with(zero,convert_x,convert_y) は単位元と実座標の変換を明示し、modint や Clone できない座標にも使える。core には Ord だけを要求する。

```rust
// 既存の NumberTheory/mint.rs を使用。
 type M = Mint<998244353>;
 let mut solver = StaticRectangleAddRectangleSum::<i64,i64,M>::build([(0,0,3,4,M::new(5))]);
 solver.push_query(1,1,5,5);
 let cv = |v: &i64| M::new(v.rem_euclid(998244353) as usize);
 let answers = solver.solve_with(M::new(0),cv,cv);
 assert_eq!(answers[0].val(),30);
```

## 4本BITの式

矩形の4隅を符号付き差分として扱う。隅 (a,b) に重み z があるとき、その右上の prefix への寄与は z*(x-a)*(y-b)。

独立した4個の Vec が、それぞれ z, z*a, z*b, z*a*b の BIT を構成する。添字の走査は共有し、prefix 和は

`F(x,y) = x*y*S0 - y*Sx - x*Sy + Sxy`

となる。質問の答案は `F(rx,ry)-F(lx,ry)-F(rx,ly)+F(lx,ly)`。

更新・質問とも x の左右端だけをイベント化してソートする。更新イベントは y の両端へ符号付きの4係数を入れ、質問イベントは y の両端の prefix を読む。同じ x では質問を先に処理する。

y は加算矩形の端点だけを一度圧縮する。圧縮は BIT 添字のためだけに使い、長さ・面積は実座標を W に変換して計算する。時間分割は不要。

N 加算、Q 質問で O((N+Q) log(N+Q+1)) 時間、O(N+Q) 空間。比較・変換・W の演算を定数時間とした評価。

## 検証

- 2分布×6000 seed＝12,000ケースで、矩形の交差幅×交差高さを直接足す愚直解と比較。i128 の正確な値と既存 Mint<998244353> の結果を、それぞれ debug/release で確認。
- -1,0,1 の端点からなる36種類の矩形の全ペア＝1,296組に対し、-2,-1,0,1,2 の端点からなる225種類の質問を全て比較。整数・modint とも一致（各方式291,600質問）。
- 7テストで、空、負の座標・重み、重複・重なり、疎な座標の実際の長さ、大きな重み、u64 を含む軸ごとに異なる型、Clone しない独自座標型、整数の別重み型、static の登録順、Clone、消去・再利用、逆順境界の拒否を確認。
- 既存の2種類の矩形ライブラリと同一スコープに include する smoke test が通過。
- 4大規模分布×2方式×3回＝24実行で、i128 の正確な答案を法998244353で正規化したチェックサムと modint のチェックサム・回答数が全て一致。

## 公式ケース

[Library Checker](https://judge.yosupo.jp/problem/static_rectangle_add_rectangle_sum) は法998244353の答案を要求する。入力アダプタ `examples/static_rectangle_add_rectangle_sum.rs` は既存の Mint を使う。

公式登録13ケース全てについて、入力SHA256をhash.jsonと照合し、公式 verifier、C++正解、checker で全件ローカルAC。例題、小規模、N/Qが小さいケース、ランダム、最大サイズを含む。オンライン提出は行っていない。

公式 revision: `1814c4e5205517e368bb57a8d1127eb961cfeaae`。最終本体 SHA256: `7ba14591a84fe321d5eb5c94083a718f781a947a322de2140d5008b3b5154e57`。

本体と既存 Mint を展開した単一ファイルをコンパイルして公式ケースへ使用し、include パスを必要としないことも確認した。

入力・正解・出力・stderr・checker 判定は `/var/folders/4y/78lt_6517zl9klyctd9lxrgr0000gn/T/static_rectangle_official_co4b04j0/cases` に保存。ケース別の時間・ハッシュは `static_rectangle_add_rectangle_sum_official_results.json`。

## 時間・ピークRSS

Apple arm64、rustc 1.93.0、rustc --edition=2021 -O。seed 123456789、各3回の中央値。登録/build＋solve を測定し、生成・起動・コンパイル・入出力は除外。RSS は macOS の wait4 で取得し、両方式共通の入力矩形・質問のデータも含む。

| N | Q | 分布 | i128 | modint | i128 RSS | modint RSS |
|---:|---:|---|---:|---:|---:|---:|
| 200,000 | 200,000 | mixed | 0.319713s | 0.256012s | 95.55 MiB | 72.64 MiB |
| 200,000 | 200,000 | ties | 0.027649s | 0.025566s | 58.45 MiB | 51.17 MiB |
| 200,000 | 10,000 | mixed | 0.155787s | 0.138553s | 65.03 MiB | 48.03 MiB |
| 10,000 | 200,000 | mixed | 0.059231s | 0.054535s | 35.53 MiB | 29.95 MiB |

mixed はランダム座標、ties は各軸8種類の座標。測定値はこの環境の入力分布に対する結果で、Linux/x86や他の重み型での性能は未確認。生データは `static_rectangle_add_rectangle_sum_benchmark_results.json`。

## 再実行

competitive_programing から：

```sh
rustc --edition=2021 --test arcaki_library/tests/static_rectangle_add_rectangle_sum.rs -o /tmp/static_rectangle_tests
/tmp/static_rectangle_tests
rustc --edition=2021 --test -O arcaki_library/tests/static_rectangle_add_rectangle_sum.rs -o /tmp/static_rectangle_tests_release
/tmp/static_rectangle_tests_release
python3 arcaki_library/tests/run_static_rectangle_add_rectangle_sum_benchmark.py
python3 arcaki_library/tests/run_static_rectangle_add_rectangle_sum_official.py --lc-root /path/to/library-checker-problems
```

公式 runner はダウンロード・オンライン提出を行わず、別ディレクトリへ公式ソースをコピーして実行する。
