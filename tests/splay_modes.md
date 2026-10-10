# NilMonoid / ProdMonoid（2026-10-04）

Vec風のAPIは [SplayVector<T>](splay_vector.md) を参照。

本体: `../src/DataStructure/splay.rs`。

| 型 | 要素 | 区間積 | 遅延更新 |
| --- | --- | --- | --- |
| `SplayTree<NilMonoid<T>>` | `T` | 保存・計算しない | なし |
| `SplayTree<ProdMonoid<M>>` | `M::S` | `M::S` | なし |
| 従来の `SplayTree<F>` (`F: SplayLazyMonoid`) | `F::M::S` | `F::M::S` | `F::F` |

木の回転・位置探索・反転・所有権管理は共通。型に応じた保存型と定数フラグを使い、不要な処理を静的に省く。
通常のSplayMonoid / SplayLazyMonoidの定義は変更していない。
内部のSplaySpecへの自動対応により、既存の `F: SplayLazyMonoid` の汎用コードもそのまま動く。

## 列だけの場合

```rust
let mut v = SplayTree::<NilMonoid<String>>::new();
v.push("first".into());
v.push("second".into());
v.insert(1, "middle".into());
v.reverse(0, 3);
assert_eq!(v.get(0), "second");
v.set(1, "changed".into());
assert_eq!(v.remove(1), "changed");
assert_eq!(v.pop(), Some("first".into()));
assert_eq!(v.to_vec(), vec!["second"]);
```

TにClone/Debug/Defaultや単位元は要求しない。値を複製して返すget/to_vecだけがT:Cloneを要求する。番兵はOption<T>のNoneで表し、通常の要素はSome(T)。
`get`と`to_vec`は値をcloneして返す。`insert`・`push`・`set`は値を移動し、`remove`・`pop`はcloneせず所有する値を返す。
区間積用のTの複製を持たないので、単なる列操作のために値をcloneしない。

## 区間積が欲しいがapplyは不要な場合

```rust
struct Sum;
impl SplayMonoid for Sum {
    type S = i64;
    fn identity() -> i64 { 0 }
    fn op(a: &i64, b: &i64) -> i64 { a + b }
    fn reverse_prod(_: &mut i64) {}
}
let mut v = SplayTree::<ProdMonoid<Sum>>::from_vec(vec![10, 20, 30]);
assert_eq!(v.prod(0, 3), 60);
v.set(1, 7);
assert_eq!(v.prod(0, 3), 47);
v.reverse(0, 3);
assert_eq!(v.get(0), 30);
```

既存のSplayMonoidをProdMonoid<M>で包む。空の作用のidentity/map/compositionを手書きする必要はない。
非可換な積も扱える。その場合reverse_prodは反転後の区間積を正しく計算できる形にする。

## 操作と計算量

全モードで `new`、`from_vec`、`len`、`is_empty`、`insert`、`erase`、`reverse`、`get`、`set`、`push`、`remove`、`pop`、`to_vec` が使える。
位置操作は償却O(log n)。from_vec/to_vecはO(n)。値のcloneやモノイド演算の費用は別。
通常のVecの添字アクセスと同じO(1)になるという意味ではない。

NilMonoidのprod(l,r)は()を返す。NilMonoid/ProdMonoidのapply(l,r,())は何もしない。
いずれも範囲は検査する。空区間は通常どおり受け付ける。

## 検証

```sh
cargo test --offline
cargo test --offline --release
python3 tests/benchmark_splay_modes.py --output tests/splay_modes_benchmark_results.json
```

本体の全27テストがdebug/releaseで通過。Splay関連は9本。
NilMonoidは7種類の初期長×3seed×2,500操作（52,500操作）、ProdMonoidは6種類×3,000操作（18,000操作）を配列と比較。
取得・置換・挿入・削除・反転・push/pop・空のapply・区間積を混在させて検証。
所有型の削除・全体破棄時のDropとclone回数、Defaultのない要素型、既存の遅延作用が残る状態でのget/set/remove/to_vecも確認。
従来の型射影を使った汎用関数とNodeのコンストラクタもコンパイル・実行して互換性を確認した。

Miriと外部提出は未実施。最初の高速化の記録は `splay_performance.md` を参照。

## 性能比較

ローカルmacOS/arm64、rustc -O。65,536要素、同じ列操作500,000回、7回の中央値。
通常版はi64の和とi64の乗算作用を持つ型。3モードでget/set/insert/remove/reverseの同じ操作列を実行し、取得値と最終的な全要素のchecksum一致を確認。
作用や区間積の問い合わせは、この共通列操作の比較では使用しない。

| 操作列 | 通常版 | ProdMonoid | NilMonoid |
| --- | ---: | ---: | ---: |
| ランダム位置取得 | 147.77 ms | 137.40 ms | 132.36 ms |
| 近い位置の反復取得 | 6.64 ms | 6.62 ms | 5.84 ms |
| 取得・置換・反転 | 238.28 ms | 213.69 ms | 209.64 ms |
| 削除・挿入 | 257.86 ms | 235.19 ms | 216.74 ms |

この軽い整数演算では、ProdMonoidはほぼ同速〜約1.12倍、NilMonoidは約1.12〜1.19倍。
各試行にはOS負荷等によるばらつきがあり、型・操作分布ごとに速度差は変わる。
従来の遅延作用を使うベンチマークでも出力checksumが一致した（splay_modes_legacy_results.json）。

比較用設定の1ノードのサイズ（Boxの割当管理領域と所有用Vecのポインタは別）:

| 値の型 | 通常版（作用はi64） | ProdMonoid | NilMonoid |
| --- | ---: | ---: | ---: |
| i64 | 72 bytes | 64 bytes | 64 bytes |
| String | 104 bytes | 96 bytes | 72 bytes |

NilMonoidのOption<T>のタグ領域と、構造体のalignmentによる詰め物があるため、区間積の型を()にしても常にTのサイズ分だけ小さくなるわけではない。
Stringの数字はノード本体のみ。文字列のヒープ上の文字領域は含まない。
NilMonoidでは区間積用のStringの複製・結合も不要になる。

各試行・コンパイラ・source SHA256は `splay_modes_benchmark_results.json` に保存。
今回の長めの計測を再現するコマンド:

```sh
python3 tests/benchmark_splay_modes.py --operations 500000 --repeats 7 --output tests/splay_modes_benchmark_results.json
```
