# convolution_u64

Fps/convolution_u64.rs を追加。stdのみでコピペ可能。
`convolution_u64(a: &[u64], b: &[u64]) -> Vec<u64>` はmod 2^64の畳み込み。
整数としての正確な積ではなくwrapping_add/wrapping_mulと同じ意味。
空入力は空。結果長は2^24以下。元のsrc/、各main.rs、既存の畳み込み実装は変更していない。

短い側<=60は愚直、それ以外は5素数でのNTTと混合基数CRT。
素数/原始根: (754974721,11), (1224736769,3), (2013265921,31), (1811939329,13), (2113929217,5)。
結果長制限よりmin(n,m)<=2^23、非負整数係数は2^23*(2^64-1)^2未満。5素数の積はこの上限を超える。
CRTの係数を段階的に求め、最終加算だけmod 2^64にするため、真の係数がu128を超えても正しい。
単に3素数の結果やconvolution_i64をu64へcastする方法とは異なり、入力値に追加の上限は設けていない。
時間O(N log N)、領域O(N)。NTTは5回の畳み込み（計15変換）。最大長付近では保持する5本の剰余列等に大きなメモリが必要。

検証:

```sh
cargo test --manifest-path arcaki_library/Reproved/Cargo.toml --offline --locked --test convolution_u64
cargo test --manifest-path arcaki_library/Reproved/Cargo.toml --offline --locked --release --test convolution_u64
```

3テストがdebug/releaseで通過。空・1要素・60/61境界・非2冪・64bit全幅ランダム・全MAX・上位bitのみ・零多項式・単項式を確認。NTT素数の素数性、2^24乗根の位数、CRT逆元、素数積の十分性も検証。

## 性能面の次候補（未実装・速度比較未実施）

現在はbit reversalを伴うradix-2 NTT。
- radix-4 butterfly化で2段をまとめ、独立したbit reversalを避ける構成を検討。
- 変換用の根/更新率の事前計算・再利用、繰り返し呼出し時の作業バッファ再利用。
- 入力長比に応じた愚直切替閾値の計測。

ACL本家のconvolution.hppにはradix-4 butterflyとfft_info/rateの再利用がある:
https://github.com/atcoder/ac-library/blob/master/atcoder/convolution.hpp

改善率は入力長・長さ比・CPU・繰り返し回数に依存するため未測定の倍率は示さない。
