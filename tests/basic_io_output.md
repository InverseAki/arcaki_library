# 01入力と整数出力

実装: `src/Basic/basic_io.rs` / `src/Basic/basic_output.rs`。
Rust 1.89互換、依存クレートなし。両方と Predecessor64 を main に直接コピーしても使える。
既存の Input のメソッドと返り値は維持した。

## 01文字列を読む

```rust
let mut input = Input::new();
let n = input.usize();
let q = input.usize();
let base: Vec<u64> = input.binary_u64_len(n);
let mut set = Predecessor64::from_vec_u64(base);
```

- `binary_u64()`: 次の01トークンを読み、Vec<u64>へ変換。
- `binary_u64_len(n)`: 次の01トークンの長さが既知の場合。トークン終端までの走査を省く。
- 先頭文字は最初のワードのbit 0。例えば `100000001` は `[257]`。
- ワード数は `ceil(n/64)`、末尾の未使用ビットは0。
- 元の文字列や中間のVec<u8>を作らず、入力バッファから直接パックする。
- 8文字をu64で読み、下位ビットのマスクと乗算で8ビットへ圧縮。
  endianに依存しない `from_le_bytes` を使い、unsafeやSIMDには依存しない。
- 01以外のトークン内バイトはpanic。
- 長さ指定版は入力不足・長すぎるトークンもpanic。ゼロ長では何も消費せず空のVecを返す。
- 長さ指定版は入力のn文字直後にASCII空白かEOFを要求する。nに改行・区切りは含めない。
- Input全体のEOF・不正入力の契約を変更するものではない。

## 整数を書く

```rust
let mut out = Output::new();
out.usize(12).space().i64_line(-34); // 12 -34\n
out.u128_line(u128::MAX);
out.string("Yes").newline();
out.flush().unwrap();
```

型ごとの通常出力と改行付き出力を用意した:

| 型 | 通常 | 改行付き |
|---|---|---|
| u8 / u16 / u32 / u64 / u128 / usize | u8 / u16 / u32 / u64 / u128 / usize | u8_line / u16_line / u32_line / u64_line / u128_line / usize_line |
| i8 / i16 / i32 / i64 / i128 / isize | i8 / i16 / i32 / i64 / i128 / isize | i8_line / i16_line / i32_line / i64_line / i128_line / isize_line |

各出力メソッドは `&mut Self` を返すのでチェーンできる。
整数の10進表現をスタック上の配列に作り、改行もまとめてバッファへコピーする。
数字ごとの一時Stringやヒープ割当は発生しない。
符号付き最小値は `unsigned_abs()` で処理し、debugでもオーバーフローしない。
整数のu8出力と、バイトをそのまま書く `byte` は区別する。

バッファは標準の `BufWriter<W>`（初期容量64KiB）。満杯になれば順に出力するので、
全回答を保持する必要はない。I/O用のメモリは出力量に比例して増えない。
Output自体の追加メモリは O(バッファ容量)、整数変換は O(桁数)。

```rust
let mut out = Output::from_writer(std::io::stdout().lock());
out.i32_line(set.innext(k) as i32); // Library Checkerの値域では !0 -> -1 にできる
out.flush().unwrap();
```

- `new()` / `with_capacity(capacity)`: stdoutへ出力。
- `from_writer(writer)` / `with_writer_capacity(capacity, writer)`: 任意のWriteへ出力。
- `bytes(&[u8])` / `byte(u8)` / `string(&str)` / `space()` / `newline()` も利用できる。
- `flush()` はI/OのResultを返す。通常の整数・バイト出力のI/Oエラーはpanic。
- 対話問題では問い合わせ後に明示的にflushする。Input::new()は入力を最後まで読むため、
  対話入力には別の入力方法を使う。
- Drop時はBufWriterが残りのバイトを書き出すが、エラーを確認するため最後のflushを推奨する。
- `into_inner()` でバッファを出力してwriterを取り出せる（Result）。テストならVec<u8>を使える。

## 検証（2026-10-07）

```sh
cargo +1.89.0 test --offline --test basic_io_output
cargo +1.89.0 test --offline --release --test basic_io_output
cargo +1.89.0 build --offline --release --example predecessor_problem
```

6テストがdebug/releaseで通過。

- 8ビットの全256通り、長さ0–140と4095/4096/4097/10000、ワード末尾のパディング。
- 長さ指定/不指定、ASCII空白、CRLF、EOF、ゼロ長、後続の整数読み取り。
- 不正バイト、長さ不足・過剰、長さのオーバーフロー。
- 全12整数型、最小値/最大値、10の累乗の前後、型ごとの乱数1000個。
- バッファ容量0/1/2/7/64/65536、短い書き込み、Interrupted、flush、Drop、テキストと生バイト。
- Input/Output/Predecessor64を同じmainへコピーしたコードもRust 1.89でコンパイルし、公式サンプルを確認。

## 提出全体の性能

`examples/predecessor_problem.rs` が新APIを使う例。
旧貼付コード・以前の提出専用修正版・新APIを使う例を、同じローカル入力で比較した。
集合本体は同じ実装。Rust 1.89 `-O`、aarch64、N=1000万、Q=100万、7回の中央値。
各実装の実行順を巡回し、出力を全バイト比較した。
時間はプロセス起動・終了・出力回収まで含む。

| ケース | 貼付コード | 以前の専用修正版 | 新しいInput/Output |
|---|---:|---:|---:|
| random | 63.02ms | 32.74ms | 25.74ms |
| sparse | 88.19ms | 42.46ms | 34.31ms |
| all0 | 62.25ms | 25.53ms | 22.57ms |
| all1 | 60.55ms | 29.57ms | 24.08ms |
| query012 | 43.10ms | 24.95ms | 18.71ms |

公式生成器と同種の分布で作った自作入力であり、公式入力ファイルそのものではない。
小さいケースは独立したPythonモデルと照合。大きい5ケースは3実装・全7反復で出力一致。
外部ジャッジへは提出していない。

再実行:

```sh
python3 tests/predecessor64_performance/librarychecker/run.py --library-io
```

生データは `tests/predecessor64_performance/librarychecker/results_library_io.csv`、
実行条件は `metadata_library_io.json`。
前回の測定ファイルを残すため、今回の出力ファイル名には `_library_io` を付けた。
