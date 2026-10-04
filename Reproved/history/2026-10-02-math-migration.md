# 数学・畳み込み補助の分離

既存の src/、3環境の main.rs、既存の Reproved ファイルは変更せず、新規ファイルだけ追加。
常設テンプレートからの削除・共通テンプレート生成はまだ行っていない。

## 配置

- Basic/math.rs: gcd(i64)、floor、modulo、extended_gcd。
- Basic/barrett.rs: Barrett32 の既存 new/mul を維持。modulus/pow/try_inv/inv を追加。
- NumberTheory/mint_combination.rs: problem_solve の MintCombination を移植。固定の素数mod MI を前提にする。n < modulus、inv(0)禁止を明示。
- Fps/convolution.rs: convolution_merge / convolution_merge_with_mx を移植。MI と convolution を同じスコープに用意する。NTT本体は含まない。

GCD・拡張GCDは非負の i64 を対象とし、ゼロも対応する。
floor(n,a), modulo(n,a) は n=a*q+r, 0<=r<|a| を満たす q,r。負の除数では floor は数学的な床と異なる。除数0は禁止、i64::MIN/-1の商は表現不可なのでpanic、余りは0。

Barrett32 は元実装と同じく 2<=mod<=u32::MAX。mul は任意のu32入力、powの指数はu64。try_invは合成数modでも対応し、非互いに素ならNone、invはpanic。64bitの法、負数の直接入力、法1は今回の対象外。
固定MIは固定のまま共有し、実行時modは別のBarrett32インスタンスで扱う。旧mod_inverse/fast_mod_powのラッパーや定数MODに依存する整数版階乗は追加していない（元のsrcには残っている）。

畳み込み補助は入力vsの内容を消費する。0本の積は[1]、空多項式を含む積は[]、mx=0は[]。mxは最大次数ではなく係数数。1本の場合にも切り詰める。

## 検証

競プロ用フォルダをcwdとして実行:

```sh
python3 arcaki_library/Reproved/tests/run_math_migration.py
```

librarychecker/src/main.rs の同梱MI実装をその場で抽出して結合する。テンプレートの構成変更で抽出不能になれば失敗し、古いコピーで黙って検証しない。外部依存なし。畳み込みバックエンドは愚直実装なので、同梱NTT本体の検証ではない。

ACLとの結合も確認する場合、既存ビルドのrlibを明示:

```sh
python3 arcaki_library/Reproved/tests/run_math_migration.py --acl-rlib problem_solve/target/debug/deps/libac_library-577cc9b5b2f2bce4.rlib
```

rlibの名前・rustcの互換性はビルド環境に依存する。今回ローカルのac-library-rs 0.1.1で確認した。
4テストが同梱MI/ACLそれぞれdebug・releaseで通過。整数除算の符号・境界、Bézout等式、Barrettのランダム乗算/累乗/逆元、Pascal三角形との組合せ比較、複数多項式の逐次積と切り詰めの比較を実施。既存のcargo testも5テスト通過。
既存lib.rsは変更せず、新規検証は上記スクリプトから実行する（cargo testだけでは新規4テストを実行しない）。

## 未実施

固定modと実行時modの性能比較、同梱NTT本体の検証、3環境のmain.rs全体との結合検証、Library Checkerへの提出。
