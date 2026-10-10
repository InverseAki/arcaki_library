# Library Checker の89msと最速16msの差を調べる

2026-10-07。ユーザーが貼付した提出コードを比較した。
本体の `src/DataStructure/predecessor64.rs` はこの調査で変更していない。
前回の操作ベンチマークには、入出力と初期文字列のパックが含まれていなかった。

## 最速提出で確認できたこと

[提出403023](https://judge.yosupo.jp/submission/403023) はC++23、16ms。
ケース別にも max_random が15ms、max_sparse が14/16ms、max_all1 が15/16msだった。
ソースをブラウザで読み、以下を確認した。

- ベースは同じ64分木と連続したワード配列。
- 更新の空/非空の遷移だけを上層に伝播する。
- 下層のワードのシフトによる検索、上昇・下降のテンプレート展開。
- 検索方向をまとめた処理で、局所検索を分岐の少ない形にする。
- 16要素以下ではソート済みのキャッシュを作り、AVX2の比較で順位を求める。
  更新時にキャッシュを無効化し、次の検索時に作り直す。
- ASCIIの0/1をAVX2で32文字ずつパックする。
- Linuxの通常ファイルを mmap する入力と、桁数上限のある整数読み取り。
- 整数出力は直接バッファに書き、4桁単位の変換表などを使う。

16msは64分木だけの性能ではなく、この全体を合わせた提出の実行時間。
「Rustだから5倍遅い」という説明にはならない。

公式の[疎な入力の生成器](https://github.com/yosupo06/library-checker-problems/blob/master/data_structure/predecessor_problem/gen/max_sparse.cpp)は
初期状態の各位置を確率1/1,000,000で存在させ、更新を行わず検索だけを出す。
[上限](https://github.com/yosupo06/library-checker-problems/blob/master/data_structure/predecessor_problem/info.toml)が
N=10,000,000なので、初期要素数の期待値は10。
16要素以下のキャッシュがこのケースに適していると推測できる。

## 貼付コードで見つかった負荷

初期化:

```rust
for (i, &x) in ip.bytes().iter().enumerate() {
    s[i >> 6] |= ((x - b'0') as usize) << (i & 63);
}
```

同じワードに64回、読み取り・変更・書き込みする。
空の集合でも1000万文字に対してこの処理を行う。

整数出力:

```rust
res.push_str(&(set.innext(k) as i32).to_string());
```

整数ごとに一時的な String を割り当て、別の String へコピーして破棄する。
全体を最後にまとめて出力しても、この割当のコストは残る。

## ローカル実測

Rust 1.89.0、aarch64-apple-darwin、`-O`。
同じ Predecessor64 のコードを全候補で使い、solve の初期化と出力だけを変えた。
N=10,000,000、Q=1,000,000。7回測定し、実装の順序を巡回。
標準入力は通常ファイル、標準出力はPythonが回収して全バイト照合した。

**公式入力ファイルそのものではなく、公式生成器と同種の分布で作ったローカル入力。**
sparse は要素数10の更新なし、all0 は所属/隣接検索のみ、all1 は削除もあり。
query012 は挿入・削除・所属確認のみ。
Wall time はプロセス起動・終了・出力回収まで含む。

| ケース | 貼付コード | 出力だけ改善 | パックだけ改善 | 両方改善 |
|---|---:|---:|---:|---:|
| random | 60.00ms | 49.02ms | 40.90ms | 30.98ms |
| sparse | 83.14ms | 57.28ms | 64.94ms | 39.56ms |
| all0 | 61.91ms | 44.12ms | 43.73ms | 25.61ms |
| all1 | 65.87ms | 49.85ms | 44.60ms | 31.57ms |
| query012 | 57.03ms | 60.25ms | 36.12ms | 36.97ms |

表は全体の中央値。query012 には to_string の出力がないため、
出力方式の変更による効果はなく、測定の逆転もある。
ここからLibrary Checkerでの時間を直接換算することはできない。
89msとなった提出のケース別内訳は、この調査では取得していない。

sparse の内部計測（各フェーズの中央値）:

| フェーズ | 貼付コード | 両方改善 |
|---|---:|---:|
| 入力全体の読み込み | 2.70ms | 2.70ms |
| 初期文字列の解析・パック・構築 | 23.31ms | 5.00ms |
| クエリの整数解析・集合操作・出力整形 | 48.57ms | 22.96ms |
| バッファの書き出し | 3.17ms | 3.25ms |

これらはフェーズを個別に集計した中央値なので、足しても全体の中央値には一致しない。
特に全体の時間にはプロセス起動と終了なども入る。

`writeln!(&mut String, ...)` で一時文字列をなくすだけの候補も測定した。
sparse は66.08ms、random は51.80ms。
直接整数を出力バッファへ書く候補のほうが速かった。

## 用意した修正版

`submission.rs` は貼付コードの solve を次のように変更した、計測出力のない提出用ファイル。
この調査では外部へ提出していない。

- 8文字を u64 として読み、下位ビットのマスクと乗算で8ビットへ圧縮する。
  64文字をまとめたワードを1回書き込む。AVX2も unsafe も追加していない。
- 0/1の文字列という問題の入力条件を前提とする。
- 結果は Vec<u8> に直接書き、一時 String を作らない。
- Predecessor64、整数入力器、クエリの分岐は変更していない。

全5候補は値域129・5000操作のPythonの独立モデルと照合した。
大きい5ケースの出力は、全候補・全反復で一致した。
追加のテストは8ビットの全256通り、部分ワードと複数ワード、整数出力の桁の境界。
submission.rs は公式サンプルも確認した。

まずこの修正版で初期化・出力の負荷を下げるのがよい。
さらに最速へ近づけるなら、Nが既知の初期文字列読み取り、
16要素以下のキャッシュ、上昇・下降の展開、実行環境に合ったSIMDを
それぞれ end-to-end で測定する。これらは本体への変更としては未採用。

## ファイル

- `pasted.rs`: 貼付コードのコピー。
- `original.rs` / `fmt.rs` / `digits.rs` / `pack.rs` / `pack_digits.rs`: フェーズ計測付きの比較用コード。
- `submission.rs`: 計測を除いた修正版。
- `results.csv`: 175回の全体・フェーズ別の生データ。
- `metadata.json`: 実行環境と条件。
- `source.json`: 貼付コードの出所とハッシュ。
- `prepare.py` / `run.py`: 生成とモデル照合・測定の再現用。

arcaki_library のディレクトリで:

```sh
python3 tests/predecessor64_performance/librarychecker/prepare.py
python3 tests/predecessor64_performance/librarychecker/run.py
rustc +1.89.0 --edition=2021 --test -Awarnings tests/predecessor64_performance/librarychecker/submission.rs -o /tmp/predecessor64-submission-test
/tmp/predecessor64-submission-test
```

run.py はCSV/JSONを上書きする。大きな生成入力は一時ディレクトリに作り、
正常終了時に削除する。乱数 seed は12345で固定。
