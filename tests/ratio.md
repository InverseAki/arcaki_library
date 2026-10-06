# 有理数 Ratio・Ratio128・BigRatio

| 型 | 分子・分母の型 | 必要なソース |
|---|---|---|
| `Ratio` | i64 | `src/Basic/ratio.rs` |
| `Ratio128` | i128 | 同上 |
| `BigRatio` | 既存のBigInt（十進多倍長） | 上記と `src/NumberTheory/big_integer.rs`、`src/Basic/big_ratio.rs` |

共通本体は `Rational<T>`。RatioとRatio128は従来どおりCopy、BigRatioはClone。
ソースの外部クレート・gcd関数・演算traitのimportは不要。
BigRatioは3ファイルを同じスコープへコピーするか、同じモジュール内でincludeする。
BigIntを別モジュールで定義した場合は、その `BigInt` と `RadixBigInt` を有理数のモジュールへuseすればよい。
`Rational<HexBigInt>` も使える。この場合は文字列の分子・分母も十六進になる。

## 作り方と演算

```rust
let a = Ratio::new(3, 2);            // 旧API: 分母3・分子2、つまり2/3
let b = Ratio::from_fraction(2, 3);  // 新API: 分子2・分母3
assert_eq!(a, b);
let c: Ratio128 = "-7/3".parse().unwrap();
assert_eq!((c.floor(), c.ceil(), c.trunc()), (-3, -2, -2));
println!("{}", c);                  // -7/3

let mut big = BigRatio::from_fraction(1, 3);
big += 2;                           // 通常の整数もそのまま使える
let product = &big * &big;           // 参照どうしの演算で元の値を保持
assert_eq!(product, BigRatio::from_fraction(49, 9));
let huge: BigRatio = "1234567890123456789012345678901234567890/7".parse().unwrap();
```

- `new(分母, 分子)`、`int`、`inv`、`is_inf`、旧演算子・代入演算・整数との比較を維持。newの引数順は変更しない。
- `from_fraction(分子, 分母)`、`zero` / `one`、`Default`、`numerator` / `denominator`（参照）、`into_parts`（分子・分母順）を追加。
- 値と参照の四則演算、参照を右辺にした代入演算、`Neg`、整数を左辺にした四則演算も使える。
- BigRatioはBigIntとの演算・比較に加え、標準の符号付き/符号なし整数との演算・比較・Fromに対応。
- `abs`、`recip`（invの別名）、`pow(u64)`、`is_zero`、`is_finite`、`signum` を追加。指数0は0や無限大でも1。
- `floor` / `ceil` / `trunc` はそれぞれ負方向・正方向・0方向への丸め。同じ整数型を返す。
- `Display` / `FromStr`、`sum` / `product` に対応。文字列は整数、`分子/分母`、`inf` / `-inf`。整数値の表示は分母を省略する。
- Ratio → Ratio128 / BigRatio、Ratio128 → BigRatioは `From`。小さい型に戻す場合は `TryFrom`。
- 固定長の `to_f64` は近似変換。BigRatioの全桁を維持するなら文字列または分子・分母を使う。

整数との演算では、Ratioの整数はi64、Ratio128はi128。数値リテラルはそのまま推論される。
固定長版の文字列も、各成分がその整数型に収まることが必要。

## 正規化とエラー

有限値は必ず分母>0・約分済み、0は0/1。
分母0は分子の符号だけで±∞に正規化し、`infinity` / `negative_infinity` でも作れる。
0/0は値として保持しない。こうすることでEq・Ord・Hashが一致する。
全順序は `-inf < 有限値 < inf`。

| 計算 | 結果 |
|---|---|
| 有限値 / 0（有限値が0以外） | 分子の符号の∞ |
| 有限値 / ∞ | 0 |
| 0の逆数 | +∞ |
| ∞の逆数 | 0 |
| ∞ × 有限の非零値、∞ / 有限値 | 符号を反映した∞（∞ / 0は元の符号の∞） |
| 同符号の∞の加算 | 同符号の∞ |
| `0/0`、`inf-inf`、`0*inf`、`inf/inf` | `Indeterminate` |
| 無限大の丸め | `NotFinite` |

`try_new` / `try_from_fraction`、`try_add` / `try_sub` / `try_mul` / `try_div`、
`try_neg` / `try_abs` / `try_inv` / `try_pow`、`try_floor` / `try_ceil` / `try_trunc` は `Result<_, RatioError>` を返す。
通常のコンストラクタ・演算子・メソッドは、失敗時にdebug/releaseともpanicする。
演算子のオーバーフローを黙ってwrapさせない。

```rust
let max = Ratio::int(i64::MAX);
assert_eq!(max.try_add(&Ratio::one()), Err(RatioError::Overflow));
assert_eq!(Ratio::try_new(0, 0), Err(RatioError::Indeterminate));
```

固定長版は入力をunsignedの絶対値で約分するのでMINも正しく扱う。
例えば `Ratio::new(i64::MIN, i64::MIN)` は1で、`Ratio::new(i64::MIN, 1)` は正の分母2^63が格納できないためOverflow。
減算や除算を「右辺の符号反転・逆数を先に格納する」方法にしないので、MIN-MIN=0やMIN/MIN=1も計算できる。

乗除算は掛ける前に交差約分する。加減算は分母のGCDを使い、残りの共通因子も落としてから格納する。
i128の中間交差積・和は最大256bitの内部作業領域で扱う。約分後の分子・分母が型に収まるなら、中間値だけが大きくても計算できる。
比較も大きな交差積を正確に比較し、大小関係がオーバーフローで逆転しない。
BigRatioは整数演算・約分の全てに既存BigIntを使う。

旧ratio.rsにあったi64用の補助 `floor(a,b)` も維持し、負の除数も数学的なfloorに修正した。
同名の補助関数を持つmath.rsもコピーする場合は片方だけにする（従来から同名の関数がある）。

## 追加したGCD

```rust
let a: BigInt = "-12345678901234567890".parse().unwrap();
let g = a.gcd(&BigInt::from(15));
assert_eq!(g, BigInt::from(15));
```

`BigInt::gcd(&self, rhs)` と `HexBigInt::gcd` は非負の最大公約数を返す。gcd(0,0)=0。
既存の剰余によるEuclid互除法を使い、u128に収まった段階で整数の互除法へ切り替える。
多倍長での実行時間は桁数と互除法の回数に依存する。超大規模なGCD専用アルゴリズム（Lehmer・half-GCD）は使っていない。

## 検証結果（2026-10-04）

- 本体テスト計51本をdebug/release両方で通過。有理数/GCDは7本。
- Python `fractions.Fraction` と `math.gcd` による41,365ケースをdebug/release両方で通過。
- i64/i128のMIN/MAX、符号、分母0、不定形、Eq/Ord/Hash、約分前だけ巨大になる加算、乗除算の交差約分、丸め、文字列、参照/整数/代入演算、型変換、iteratorを確認。
- 内部256bitの積・u128での商剰余もBigIntと独立に比較。u128上限近くの除数を含む。
- 3,000桁の分子・分母、巨大な共通因子、桁数と乗除算の切替付近、隣り合うフィボナッチ数（6,000回生成）などの構造的ケースを含む。
- big_integerの単体テスト6本（Euclid除算の回帰を含む）もdebug/release両方で通過。過去の公式全ケースやオンライン提出は今回実行していない。

```sh
cargo test --offline
cargo test --offline --release
python3 tests/check_ratio.py --output tests/ratio_check_results.json
```

seed=20261004。比較対象・件数・ソースのSHA256は `ratio_check_results.json` に記録。
差分があればスクリプトは `ratio_failure.txt` に入力と期待値を保存する。


big_integerの通常の除算も2026-10-04にEuclid除算へ統一。
BigRatioの丸めは非負の余りを使って計算するので、例えば-7/3はfloor=-3、ceil=-2、trunc=-2になる。
