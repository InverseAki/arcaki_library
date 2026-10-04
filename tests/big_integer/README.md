# 多倍長整数の検証

[実装](../../src/NumberTheory/big_integer.rs) · [日本語解説](../../../algorithm_set/NumberTheory/big_integer.md)

2026-10-02 に実施。外部クレートを使わず、rustc 1.93.0 / Rust 2021 でビルドした。実行環境は macOS 26.5.2 / arm64。記録した時間はこのローカル環境の値で、Library Checker のサーバー上の計測ではない。

## 結果

- 単体テスト5本を debug / `-O` の両方で通過。
- Python int との108,220件の比較を、debug / `-O` の両方で通過。
- 公式 Big Integer 6問の全141テストファイルについて、入力・出力の SHA256 が公式 `hash.json` と完全一致。
- 最大桁級の全桁最大・carry・borrow・冪の前後の10ケースを追加確認。
- 6問それぞれの単独提出用ソースを生成・コンパイルし、公式サンプルの出力ハッシュと照合。

オンライン提出は行っていない。公式テストをローカルで再現した結果であり、AC提出番号はない。

### 公式テストごとの最大実行時間

| 問題 | ファイル数 | ローカル最大 | 公式制限 |
| --- | --- | --- | --- |
| 十進加算 | 22 | 0.076秒 | 5秒 |
| 十進乗算 | 20 | 0.352秒 | 5秒 |
| 十進除算 | 26 | 2.404秒 | 10秒 |
| 十六進加算 | 24 | 0.078秒 | 5秒 |
| 十六進乗算 | 22 | 0.458秒 | 5秒 |
| 十六進除算 | 27 | 1.828秒 | 10秒 |

数値は入出力を含む一回のプロセス実行時間。生成器のコンパイル・入力生成・ハッシュ照合の時間は含まない。別のチェックと同時に実行したケースもあり、厳密なマイクロベンチマークではない。

詳細：[official_results.json](official_results.json)。公式リポジトリの commit は `1814c4e5205517e368bb57a8d1127eb961cfeaae`。記録には実装の SHA256 も保存している。

代表的な最大級ケースでは最大 RSS 約33〜52 MiBを計測。内訳と追加計測は [memory_results.json](memory_results.json)。追加の構造的難例は [hard_results.json](hard_results.json)。

## 各ファイル

| ファイル | 内容 |
| --- | --- |
| [unit.rs](unit.rs) | parse、負の0、整数変換境界、順序、符号、Euclid除算、代入演算、pow、NTT二乗の単体確認 |
| [driver.rs](driver.rs) | 加算・減算・乗算・商剰余の比較用入出力。引数 add/sub/mul/div と dec/hex |
| [check.py](check.py) | Python int との再現可能な差分比較。seed=20261002 |
| [hard_cases.py](hard_cases.py) | 十進200万桁・十六進160万桁の最大桁パターン等。期待出力は閉じた式から作る |
| [run_official.py](run_official.py) | 取得済みの公式生成器を全ケース実行し、公式ハッシュと照合 |
| [measure_memory.py](measure_memory.py) | POSIX `wait4` で代表ケースの最大RSSを計測 |
| [prepare_submission.py](prepare_submission.py) | 6問用の外部依存のない単独ソースを生成 |

## 再実行

リポジトリ親の `competitive_programing` から実行する。

```sh
rustc --edition=2021 --test arcaki_library/tests/big_integer/unit.rs -o /tmp/bigint-unit
/tmp/bigint-unit
rustc --edition=2021 --test -O arcaki_library/tests/big_integer/unit.rs -o /tmp/bigint-unit-release
/tmp/bigint-unit-release
python3 arcaki_library/tests/big_integer/check.py --large
```

debug で大規模比較も行う場合は、driver を最適化なしでビルドして `--binary` で渡す。失敗した差分ケースは failure.txt へ保存する。

公式のテストを再現するには、公開リポジトリを用意する。検証スクリプト自身はネットワーク取得・オンライン提出をしない。

```sh
git clone --depth 1 --filter=blob:none --sparse https://github.com/yosupo06/library-checker-problems.git /tmp/bigint-official
git -C /tmp/bigint-official sparse-checkout set big_integer common
python3 arcaki_library/tests/big_integer/run_official.py --repo /tmp/bigint-official
```

公式生成器のコンパイルに C++17 コンパイラ、設定の読取りに Python 3.11 以上が必要。生成器用の params.h をその一時チェックアウトへ生成する。入力・実行ファイルは一時ディレクトリに置き、結果の記録だけ残す。公式に含まれる全ケースの入力ハッシュも照合するため、生成器を都合のよいサイズへ縮めてはいない。

提出用ソースを生成するには次を使う。

```sh
python3 arcaki_library/tests/big_integer/prepare_submission.py --output-dir /tmp/bigint-submissions
rustc --edition=2021 -O /tmp/bigint-submissions/division_of_big_integers.rs -o /tmp/bigint-division
```

## 確認対象の難例

NTTへ切り替わる48/49内部桁、筆算除算の32/33内部桁、2冪前後の長さ、片方一桁、短い商、除数が短く多数のブロックに分かれる被除数、符号の全組合せ、先頭が小さい除数、正規化倍率の違い、剰余0/1/D-1、全桁最大、carry/borrow連鎖を含む。

公式の FFT killer、length_ratio_integer、burnikel_ziegler_bound、r_nearly_zero も全seedで照合済み。追加難例の冪そのものは、公式の桁上限を一桁越える境界も含む。

## 範囲

Library Checker の6問に必要な演算に加え、負数の商剰余、整数変換、pow、比較を実装した。十進と十六進の相互基数変換、任意基数、GCD・平方根・ビット演算は未提供。NTT最大長2^25の限界そのものまでメモリを使うテストは行っていない。対応する公式最大規模は全て確認済み。
