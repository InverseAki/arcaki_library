# chmax / chmin

実装: `src/Basic/chminmax.rs`。main.rs にそのままコピーして使える。
モジュールから使う場合は `#[macro_use] mod chminmax;` とし、メソッド用に
`use chminmax::{Chmax, Chmin};` を追加する。

```rust
chmax!(dp[i], dp[j] + cost);
chmin!(dp[i][j], dp[x][y] + cost);
if chmax!(answer, candidate) {
    // 更新時のみ実行
}
```

- マクロの左辺には更新先の式を渡す。`&mut dp[i]` ではなく `dp[i]`。
- 右辺を先に評価し、その後に左辺を一度だけ評価して可変借用する。
  そのため、同じ Vec / 多次元 Vec を右辺で読める。
- 左辺・右辺の評価は各一回。更新されない場合も両方を評価する。
- 更新したら `true`、等値または比較不能なら `false`。末尾カンマも許可。
- `PartialOrd` のみを要求する。`String` など非 Copy 型にも対応する。
  同じ配列の非 Copy 要素を候補にする場合は、必要に応じて `.clone()` する。
- 右辺の結果が更新先のコンテナへの借用を保持するケースは、借用エラーになり得る。
  このマクロは所有した候補値を先に作ることで DP の通常の借用衝突を回避する。
- `.chmax(rhs)` / `.chmin(rhs)` も利用可能。ただし、同じ配列を参照する更新には
  マクロを使う。新規ファイルと problem_solve のメソッドは `bool` を返す。
- 浮動小数点の NaN は比較不能なのでマクロ・汎用トレイトでは更新しない。

反映先: problem_solve の main/generate/naive/validator、yukicoder の main/memo、
random_test の main、librarychecker の main。
既存メソッドの戻り値は維持し、yukicoder / random_test は `()` のまま。
librarychecker の型別メソッドも従来のまま保持する（f64::max/min の NaN 処理を維持）。
マクロは全反映先で同じ定義。

## 検証（2026-10-07）

```sh
rustc --edition=2021 --test tests/chminmax.rs -o /tmp/chminmax-debug
/tmp/chminmax-debug
rustc --edition=2021 -O --test tests/chminmax.rs -o /tmp/chminmax-release
/tmp/chminmax-release
```

6 テストが debug / release ともに通過。同じ配列・二次元配列、評価回数と順序、
非 Copy 型、マクロ内変数名と呼び出し元の衝突、構造体フィールド・参照・タプル、
等値・NaN・符号付きゼロ、整数の全組合せを確認。
各テンプレートから実際の定義を抽出して、既存メソッドの戻り値に合わせた
同じ6テストを debug / release で実行し通過。

`cargo check --offline --bins` は problem_solve / yukicoder / librarychecker で通過。
problem_solve の generate には既存の未使用変数などの警告が残る。
random_test の全体チェックは、quote v1.0.44 がローカルにないため未完了。
追加定義の単独コンパイル・テストは通過している。
