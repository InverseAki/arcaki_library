# 統合後の検証

実装の正本は `src/`。各ファイルを従来どおり main.rs にコピーして使う。
`src/lib.rs` は従来どおり空で、テストから本体ファイルを直接読み込む。

```sh
cargo test --offline
cargo test --offline --release
python3 tests/run_math_migration.py
python3 tests/run_convolution_998244353.py
python3 tests/run_convolution_1000000007.py
```

後半3本は既存の `../librarychecker/src/main.rs` から同梱MIを抽出して結合する。
ACLでも確認する場合は各スクリプトに `--acl-rlib /path/to/libac_library.rlib` を渡す。
998244353の最大長試験は `--full` で明示的に実行する。

## インターフェース

- 既存の `Basic/couter.rs` の綴り、型名・メソッド名・引数順を維持。
- `Basic/math.rs` の旧 `mod_inverse`、`fast_mod_pow`、`factorial_i64`、`factorial`、`comb` を保持。階乗・組合せの旧APIは同一スコープの `MOD: i64` に依存する。
- `MintCombination` の `MI` は従来の同梱 `Mint` も使える。新たな `modulus()` は要求せず、素数modで前計算範囲が法未満であることは利用者の前提。
- `NumberTheory/mint_combination.rs` 内の `convolution_merge` を維持し、修正版に更新。`Fps/convolution.rs` にも同じ補助関数があるので、同一スコープには片方だけをコピーする。
- `BarrettCombination` と3種類の畳み込みバックエンドは新規ファイルとして追加。

## 修正に伴う挙動

- BITの空配列の長さは0。セグ木の探索結果は論理長まで。範囲外アクセスはassertで検出する。
- Rollback UnionFindは失敗したmergeを含めて巻き戻せる。snapshotは従来どおり現在状態で履歴を破棄する。
- Counterは個数0のキーを保持せず、subは実際に減らした分だけtotalを減らす。総個数のオーバーフローと_exの前提違反はreleaseでもpanic。
- gcd/extended_gcdは非負入力。floor/moduloは余りが非負のユークリッド除算。負の除数の場合、floorは数学的な床とは異なる。
- 畳み込みの0本の積は[1]、空多項式を含む積は[]。

ローカルの愚直比較・境界検証とLibrary Checkerへの提出は別。今回の統合では新たな外部提出はしない。
既存の多倍長整数の検証については `big_integer/README.md` を参照。

## 2026-10-04 統合時の結果

- 本体Cargoテスト18本がdebug/release両方で通過。既存LCAテスト4本も含む。
- 数学・組合せ・畳み込み補助5本、998244353畳み込み5本、1000000007畳み込み3本が、同梱MIとローカルACL 0.1.1のそれぞれでdebug/release両方通過。
- 従来のMint型（modulus()なし）とMintCombination、従来配置のconvolution_merge、数学の旧関数とi64の関数型を追加検証。
- 旧ReprovedのCargoテスト12本もdebug/release両方で通過。
- 998244353の最大長試験2本は今回再実行していない。性能・メモリの過去記録はReproved/history/を参照。

## Splay木の改善（2026-10-04）

性能比較・APIの注意点・検証記録: [splay_performance.md](splay_performance.md)。
本体テストはSplayの4本を追加して計22本。

## 集約・遅延作用のないSplay木（2026-10-04）

`NilMonoid<T>` と `ProdMonoid<M>` を追加。列操作用のget/set/push/remove/pop/to_vecも追加。
使い方・仕様・性能比較: [splay_modes.md](splay_modes.md)。本体テストは計27本。
