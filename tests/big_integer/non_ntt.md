# 大きな基数とNTT以外の高速化（2026-10-04）

比較対象は、直前にradix-4・Montgomery乗算・SIMDを採用した [旧版](../support/big_integer_before_non_ntt.rs)。前回の2220ms提出との比較とは異なる。

## 基数の実験と採用判断

保存形式を十進10^16 / 十六進2^48のu64桁へ変える版、および十進10^8 / 十六進2^32の版を試作した。加減算と短い除数・短い商の除算は速くなる。しかし、小さい乗算が遅くなり、大きな除算はほぼ同速、一部は遅くなった。

大きな積は、従来の二素数NTTに渡すため桁を10^4 / 2^16へ分割する必要がある。積を戻す際にも桁の結合が必要で、Newton反復中の各乗算でこの変換を繰り返す。10^16を直接NTTに渡すと係数が二つの素数の積を越えるため、そのまま同じCRTで復元することはできない。

全面変更は採用せず、**筆算除算だけ一時的に桁をまとめる方式**を採用した。

| 実行対象 | 十進の一時基数 | 十六進の一時基数 | 積・除算の中間型 |
| --- | --- | --- | --- |
| ARM64 | 10^16 | 2^48 | u128 / i128 |
| x86_64等 | 10^8 | 2^32 | u64 / i64 |

128bit版はx86の検証環境（Rosetta）で遅くなったため、こちらでは64bit版を使う。x86でも短い除数・短い商の除算の改善を確認した。Rosettaの結果は実機のLinux/x86_64の速度を保証するものではない。

条件は被除数64内部桁以上、かつ除数32内部桁以下または商の推定長32内部桁以下。小さい入力は従来の筆算、大きな積と一般の大きな除算は従来の基数でNTT・Newton法を使う。まとめた桁の最上位を改めて正規化し、剰余の正規化を戻してから通常の桁へ分解する。

**保存基数、公開型 `RadixBigInt<10000>` / `RadixBigInt<65536>`、`limb_len()`、符号・Euclid除算の規則は維持**している。

実験用の全保存形式変更版は [生成スクリプト](experiment_large_radix.py) と [10^16版の記録](non_ntt_experiment_radix16.json) / [10^8版の記録](non_ntt_experiment_radix8.json)。[x86で128bit筆算が遅くなった試作記録](non_ntt_trial_hybrid16_rosetta.json) も残した。

## その他の採用変更

- **CRT復元とcarryを融合**し、最初のNTT結果配列をそのまま結果の桁配列にする。中間のu64係数配列と、もう一つの結果配列の確保を省く。NTT本体・素数・係数上限は同じ。
- **Newton反復の配列再利用**。`S-a*x` や `2S-a*x` を積の配列へ直接書き込む。巨大な0配列とそのコピーを省く。
- 被除数と除数が等しい場合は直ちに商1・剰余0を返す。除数が一つの非零limbと低位の0だけで表せる場合は、桁の切り分けとscalar除算で処理する。
- 正規化倍率が1なら、被除数・除数のコピーと、剰余の再走査を省く。
- 加減算で長さの違う末尾をまとめてコピーし、carry / borrowが消えたら走査を終える。
- 十進出力は2桁の表、十六進出力はビット取り出しを使う。ASCIIのみをStringの末尾へまとめて追記し、4文字ごとのUTF-8検査を省く。既存のUTF-8文字列を保持することを単体テストで確認した。

## 計測記録

macOS 26.5.2 / arm64、rustc 1.93.0、Rust 2021、`-O`。変更前後を交互に実行し、計算部分は5回、公式の入出力込みは3回の中央値。公式比較のmainは添付されたコードのものを使い、BigInt部分だけ差し替えた。

- [ARMでの各操作の計測](non_ntt_arithmetic_arm.json)：parse / formatは個別に測定し、それ以外はparse / formatを除いた演算時間。
- [x86_64/Rosettaでの各操作の計測](non_ntt_arithmetic_rosetta.json)。AVX2非対応なのでscalar NTT経路。
- [公式十進除算26ファイルの比較](non_ntt_official_benchmark.json)：入力・出力込み、全実行で公式出力ハッシュと一致。
- [代表3ケースの最大RSS比較](non_ntt_memory.json)：別プロセスでPOSIX wait4により測定。メモリ記録の時間は1回の参考値であり、速度比較には上の中央値を使う。

ARMでの代表的な計測（5回の中央値）：

| 操作 | 変更前 | 変更後 | 速度比 |
| --- | --- | --- | --- |
| 十進30万文字の出力 | 0.647ms | 0.130ms | 4.96倍 |
| 十進20万桁＋10万桁 | 0.091ms | 0.053ms | 1.71倍 |
| 十進4096桁÷64桁 | 0.080ms | 0.035ms | 2.28倍 |
| 十進4096桁÷4000桁 | 0.111ms | 0.036ms | 3.06倍 |
| 十進200万桁÷100万桁 | 210.180ms | 206.313ms | 1.02倍 |

代表的な大きい除算 `a_max_b_random_02` の最大RSSは約51.5MiB→40.7MiB。

大きな除算はNTTが主なコストなので、今回の追加改善は小さめ。OSの負荷によるばらつきもある。Library Checkerサーバーへのオンライン再提出は未実施。

## 検証

[検証記録](non_ntt_validation.json)。

- 本体全64テストをdebug/releaseで通過。
- x86_64/RosettaでもBigInt・Ratioの単体テストを実行。
- Python intとの108,220件の比較を、ARMとx86_64/Rosettaの両方で通過。
- 公式Big Integer 6問、全141ファイルの入力・出力SHA256が一致：[公式検証記録](non_ntt_official_all.json)。
- 十進200万桁・十六進160万桁の最大桁パターン10ケース：[記録](non_ntt_hard_results.json)。
- 広い桁の筆算と従来のKnuth筆算を独立に比較。正規化倍率の境界、まとめる桁数の前後、32/33・63/64/65内部桁、剰余0/1/D-1を確認。
- 巨大な基数冪からの減算、carry / borrow連鎖、CRT配列の容量がちょうどNTT長になる場合の繰り上がりと再確保、Unicodeを含む文字列への追記を確認。

## 再現・提出

`arcaki_library` から実行する。

```sh
python3 tests/big_integer/experiment_large_radix.py --group 4 --output /tmp/bigint-wide16.rs
python3 tests/big_integer/benchmark_ntt.py --quick --repeats 5 --before-source tests/support/big_integer_before_non_ntt.rs --after-source /tmp/bigint-wide16.rs --output /tmp/wide16.json
python3 tests/big_integer/experiment_large_radix.py --group 2 --output /tmp/bigint-wide8.rs
python3 tests/big_integer/benchmark_ntt.py --repeats 5 --before-source tests/support/big_integer_before_non_ntt.rs --output /tmp/non-ntt.json
python3 tests/big_integer/benchmark_ntt.py --quick --repeats 5 --target x86_64-apple-darwin --before-source tests/support/big_integer_before_non_ntt.rs --output /tmp/non-ntt-x86.json
```

全保存形式変更版は性能実験用で、現在のライブラリに採用したコードではない。

[更新した単独提出用ソース](submissions/division_of_big_integers.rs) は次で再生成できる。

```sh
python3 tests/big_integer/prepare_submission.py --output-dir tests/big_integer/submissions --problem division_of_big_integers
```


ばらつきが大きかった入力は9回で追加比較した。小さい公式ケースは約162ms→160ms（[再計測](non_ntt_recheck_small.json)）、長さ比ケースは約267ms→256ms（[再計測](non_ntt_recheck_length_ratio.json)）、Rosettaの十六進20万桁÷10万桁は約41.4ms→40.2ms（[再計測](non_ntt_recheck_rosetta.json)）。最初の3/5回の生データも残している。
