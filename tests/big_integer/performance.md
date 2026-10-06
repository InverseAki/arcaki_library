# BigIntのNTT高速化（2026-10-04）

対象は [division_of_big_integers](https://judge.yosupo.jp/problem/division_of_big_integers)。添付された2220msの提出からBigInt部分を取り出し、[比較用の旧実装](../support/big_integer_ntt_baseline.rs) として保存した。現在の [実装](../../src/NumberTheory/big_integer.rs) には次の変更を入れている。

- **radix-4**：radix-2の2段をまとめる。配列の読み書きとtwiddle乗算を減らし、bit reversalの並べ替えは引き続き省く。
- **Montgomery乗算**：NTTの内部表現を切り替え、係数ごとの64bit剰余演算を乗算・シフト・加減算に置き換える。値域を `[0,2P)` に保ち、加減算の正規化も減らす。最後に通常の余りへ戻して従来どおりCRT・繰り上がりを行う。
- **SIMD**：ARMではNEONの4係数、x86_64ではAVX2の8係数をまとめて処理する。CPU対応を実行時に確認し、非対応CPUと小さい段はscalar版を使う。アラインメントは要求しない。
- 根・更新係数の表を2つの素数それぞれで一度だけ構築する。twiddleが1のブロックでは不要な乗算を省く。

除算ではNewton法で逆数を作る際に大きい乗算を繰り返すため、乗算の改善が除算にも効く。基数・二つの素数・CRTの係数上限・最大変換長・筆算への切替閾値は従来どおり。公開APIと `x=q*d+r, 0<=r<|d|` の規則も同じ。

## 計測方法と記録

macOS 26.5.2、Apple ARM64、rustc 1.93.0、`--edition=2021 -O`。実行時SIMD検出を使用するため `target-cpu=native` や全体へのAVX2指定は不要。

公式26ファイルでは、添付コードの**BigInt部分だけ**を差し替え、入力処理と出力処理をそろえて比較した。各ケース3回、旧→新と新→旧を交互に実行した中央値。全実行で公式の出力SHA256と一致を確認している。生成器・入力ハッシュ・出力ハッシュは公式リポジトリのcommit `1814c4e5205517e368bb57a8d1127eb961cfeaae` に基づく。

- [ARMでの公式26ケース比較](performance_official_arm.json)：入出力込み。
- [Rosettaでの公式26ケース比較](performance_official_rosetta.json)：x86_64のscalar経路、入出力込み。
- [ARMでの計算部分比較](performance_arithmetic_arm.json)：5回の中央値、parseとformatを除く。十進/十六進、商が短い・除数が短い・200万桁の入力を含む。
- [Rosettaでの計算部分比較](performance_arithmetic_rosetta.json)：同じ測り方で、小さめの12入力を比較。

公式 `length_ratio_integer_00` の入出力込み（3回の中央値）：

| 実行環境 | 変更前 | 変更後 | 速度比 |
| --- | --- | --- | --- |
| ARM64 / NEON | 1797.2ms | 326.3ms | 5.51倍 |
| x86_64 / Rosetta / scalar | 2182.4ms | 594.4ms | 3.67倍 |

小さい入力では従来の筆算を使うので、速度はほぼ同じ。改善幅はNTTを多用する大きな入力で大きくなる。

ARM環境での計算部分（5回の中央値）：

| 入力 | 変更前 | 変更後 | 速度比 |
| --- | --- | --- | --- |
| 十進32768桁 × 32768桁 | 2.85ms | 0.55ms | 5.16倍 |
| 十進20万桁 ÷ 10万桁 | 127.16ms | 24.01ms | 5.30倍 |
| 十進200万桁 ÷ 100万桁 | 1252.87ms | 207.61ms | 6.03倍 |
| 十進200万桁 ÷ 1000桁 | 316.72ms | 85.74ms | 3.69倍 |

Rosetta環境では `avx2=false` だった。したがってx86の計測値はAVX2版の速度を示さない。AVX2コードはmacOSとLinuxのx86_64向けコンパイルを確認したが、AVX2対応実機上での実行確認は今回できていない。NEON版とscalar版は実行検証済み。Library Checkerサーバーへのオンライン再提出も行っておらず、元の2220msに対するサーバー上の改善幅は未計測。

## 正しさの確認

[実行した検証とソースSHA256](performance_validation.json)。

- ライブラリ全体55テストをdebug/releaseで通過。
- 各NTT素数についてMontgomery乗算を独立した通常の剰余演算で照合。長さ1〜65536の順逆変換、奇数/偶数の段数、SIMD境界、ずらしたslice、scalarとSIMDの一致、独立した筆算畳み込みとの一致を確認。
- x86_64/RosettaのBigInt単体8テストもreleaseで通過（scalar経路）。
- Python intとの108,220件の比較（release）。正負の全組合せ、Euclid除算、筆算/NTT閾値、長さ比、剰余0/1/D-1、carry/borrow連鎖を含む。
- 公式Big Integer 6問、全141ファイルの入力・出力SHA256が一致：[記録](performance_official_all.json)。
- 十進200万桁・十六進160万桁の全桁最大、二乗、carry/borrow、冪の前後の10ケース：[記録](performance_hard_results.json)。

公式全体の検証記録は正しさの確認用。性能比較は別途実行し、他の検証処理との同時実行を避けた。

## 再実行・提出用コード

`arcaki_library` から、計算部分の比較は次で再実行できる。

```sh
python3 tests/big_integer/benchmark_ntt.py --repeats 5 --output /tmp/bigint-bench.json
python3 tests/big_integer/benchmark_ntt.py --quick --repeats 5 --target x86_64-apple-darwin --output /tmp/bigint-bench-x86.json
```

公式の入出力込み比較では、同じ入出力コードを使った変更前後の提出を `rustc --edition=2021 -O` でビルドし、実行ファイルを渡す。

```sh
python3 tests/big_integer/benchmark_official_division.py --repo /tmp/bigint-official --before /tmp/bigint-before --after /tmp/bigint-after --report /tmp/bigint-official-bench.json
```

[単独提出用ソース](submissions/division_of_big_integers.rs) は外部依存がなく、そのまま提出できる。入出力を簡潔にした提出用mainと現在のライブラリを結合したもの。性能比較には、こちらのmainではなく元の添付と同じmainを用いた。次で再生成できる。

```sh
python3 tests/big_integer/prepare_submission.py --output-dir tests/big_integer/submissions --problem division_of_big_integers
```

この記録の後に、[大きな基数とNTT以外の追加高速化](non_ntt.md) を実施した。最新版の提出用コードは同じファイルへ再生成している。
