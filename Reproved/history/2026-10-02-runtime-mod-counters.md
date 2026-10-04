# 実行時mod・汎用整数・Counter改善

既存src/と3環境のmain.rsには変更なし。前回追加したReproved/Basic/math.rsとbarrett.rsを更新し、新規ファイルを追加した。

## API

- Barrett32::mod_pow(a: u32, exp: u64) -> u32。既存powも同じ処理に転送して維持。
- NumberTheory/barrett_combination.rs: BarrettCombination::new(n, modulus)、with_barrett(n, barrett)、失敗をNoneで返すtry_new/try_with_barrett。Basic/barrett.rsと同じスコープへコピーする。
- f/fi/inv/p/cは既存MintCombinationと同じ呼出形式でu32を返す。h(n,k)は重複組合せ。max_n/modulusも取得可能。
- 法2..=u32::MAXかつn!が可逆であることが必要。素数ならn<mod、合成数なら最小素因数より小さいnに限定される。任意のn/合成数modの二項係数やLucas法は対象外。k>nでは0、それ以外の表の範囲外参照はpanic。前計算O(n+log mod)、問い合わせO(1)、2本のu32配列で約8(n+1)bytes。
- gcd/floor/moduloは全標準整数型で利用可能。extended_gcdは全符号付き整数型。外部クレート不要。gcd/extended_gcdは非負入力。floor/moduloの非負剰余仕様を維持。i128でも中間を狭い型へ変換しない。

## Counter / HashCounter

新規Basic/counter.rs、Basic/hashcounter.rs。元のcouter.rs（元の綴りのまま）は維持。

- subの総個数を、実際に削除した個数だけ減らす。
- add(x,0)は何もせず、個数0のキーを残さない。
- entryで減算・削除の検索重複を排除。delもremoveの戻り値を使う。
- lenは種類数のまま。totalで総個数を公開。
- one_updateは元APIの意味を維持: xを最大1個減らしてyを1個増やす。xが存在しなくてもyを増やす。
- add_exはキー存在、sub_exはキー存在と十分な個数を要求。前提違反/総個数オーバーフローはreleaseでもpanicし、その操作の変更前に検出する。
- 小さいmapから大きいmapへmergeし、rhsを空にする。HashCounterはdrainでrhsの確保済み容量を残す。
- HashCounterはOrd不要、Eq+Hash。Copyも必須ではない。デフォルトのハッシャーを維持しつつwith_hasher/型引数で変更可能。例: `HashCounter::<u64, BuildHasherDefault<FxHasher>>::default()`。

## 検証

```sh
cargo test --manifest-path arcaki_library/Reproved/Cargo.toml --offline --locked
cargo test --manifest-path arcaki_library/Reproved/Cargo.toml --offline --locked --release
python3 arcaki_library/Reproved/tests/run_math_migration.py
python3 arcaki_library/Reproved/tests/benchmark_counters.py
```

cargoの既存5+新規4テストがdebug/releaseで通過。前回の移植検証4テストも両モードで通過。
新規テストは、全標準整数型の境界、i8非負入力の全組合せ、i128ランダム拡張GCD、二項係数のPascal三角形比較、複数modの同時保持、Counterのランダム2万操作、異なるハッシャー、過剰減算とオーバーフローなど。

簡易性能比較: 4096種類のキーに対するランダムone_sub/one_addを50万組、rustc -O、5回中央値。この環境の初回計測でCounterは82.26→41.29ms、HashCounterは15.78→8.95ms。個数0での削除やmergeを含まない限定的な操作列であり、全用途の高速化率を表すものではない。HashCounterは両方デフォルトハッシャー。
