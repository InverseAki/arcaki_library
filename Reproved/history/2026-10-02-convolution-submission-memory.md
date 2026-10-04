# 提出コードのメモリ超過対策（1024MiB制限）

対象はユーザー添付の提出コード。元の添付ファイル・librarychecker/src/main.rs等の既存解答は変更していない。
修正版全文は `librarychecker/submissions/convolution_998244353_memory_fixed.rs`。

## メモリが増える箇所

1. `res.into_iter().map(|v|v.val().to_string()).collect::<Vec<_>>().join(" ")`。
   最大結果長2^25-1でStringが約3355万個できる。64bit環境のStringは1要素24bytesなのでVecの要素領域だけで約768MiB。これとは別に各数字のヒープ領域・アロケータ管理領域・join後の巨大文字列を確保する。1024MiBを超える原因となる。
2. Input::newのread_to_endが入力全量を保持し、畳み込み・出力まで残る。
   各入力2^24、9桁の係数なら入力テキストは約320MiB。Vecのcapacityは実データより大きくなり得る。
3. 既存借用版の畳み込み自体も最大時に入力込み約544MiBの配列を使う。

使わない関数の定義を削除することより、大きな実行時バッファと文字列集合をなくすことが優先。

## 修正

- 入力を64KiBバッファの逐次読み込みに変更。バッファ境界、CRLF、末尾改行なし、EOF/異常入力の扱いを確認。
- 出力は1MiBのBufWriterに逐次書き込み。整数の変換はスタック上の10bytesのみで、Stringを生成しない。
- 読み込み後にInputをdrop。
- `convolution_mod998244353_owned(a,b)`を追加。元の借用APIも維持。
  大きい入力は片側ずつNTTに変換して元Vecをdropしてから結果を確保する。
  生存配列量の見積りは最大416MiB。アロケータが解放領域を保持するため、RSSが416MiBまで下がるとは限らない。
- 修正版のsolveはowned APIを使用する。

最小限の対処だけなら元Inputのままでも「読み込み後drop(input)」と「collect/joinをやめてBufWriterへ逐次出力」が重要。ただし修正版全文では入力時のピークも抑えている。

## 検証

```sh
python3 arcaki_library/Reproved/tests/check_convolution_submission.py --full
python3 arcaki_library/Reproved/tests/run_convolution_998244353.py --full --acl-rlib problem_solve/target/debug/deps/libac_library-577cc9b5b2f2bce4.rlib
```

提出コード全体の検証では、Rust生成器→修正版提出バイナリ→Rust検証器をパイプ接続し、大きなファイルやPython上の巨大配列を作らない。
各入力長2^24、全係数998244352を生成し、出力全33554431係数を既知解と照合して通過。
macOSで提出バイナリだけのwait4による最大RSSは546.69MiB。パイプ全体の経過時間3.02秒（ローカル単発、生成/読み込み/畳み込み/出力/検証を含む）。ジャッジ環境での時間・メモリを保証する数値ではないが、1024MiBより十分小さい。

小ケース8件をPython愚直解と比較し、10万個の9桁整数で入力バッファ境界も確認。
ライブラリは同梱MI/ACL両方でdebug/releaseの通常5テストが通過、借用版/所有権版それぞれの最大長既知解も通過。最大長テストはメモリ計測を混同しないよう直列に実行する。
