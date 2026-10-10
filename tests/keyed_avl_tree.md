# キー付きモノイドAVL木と区間ソート

本体:

- `../src/DataStructure/keyed_avl_tree.rs`: `KeyedAvlTree<K, M>`
- `../src/DataStructure/sortable_sequence.rs`: `SortableSequence<K, M>`
- `../examples/point_set_range_sort_range_composite.rs`: 問題形式の入出力
- `../../librarychecker/submissions/point_set_range_sort_range_composite.rs`: 依存ファイル不要の生成済み提出用コード

対象は [Point Set Range Sort Range Composite](https://judge.yosupo.jp/problem/point_set_range_sort_range_composite)。
キーと値を分離して持ち、キーは比較だけに使う。モノイドの積は値だけから計算する。
キーは `Ord`、値は `Clone` が必要。キーのClone、値のDebug/Default、モノイド型のCloneは不要。
AVL条件で最悪の木の高さを保証し、unsafe・乱数・外部クレートは使用しない。

## 木だけを使う

提出用には `keyed_avl_tree.rs` をコピーする。

```rust
struct Sum;
impl KeyedAvlMonoid for Sum {
    type S = i64;
    fn identity() -> i64 { 0 }
    fn op(a: &i64, b: &i64) -> i64 { a + b }
}
let mut t = KeyedAvlTree::<i32, Sum>::from_sorted(vec![(10, 7), (30, 2)]);
t.insert(20, 4);
assert_eq!(t.get_index(1), (&20, &4));
assert_eq!(t.prod(0, 2), 11); // key=10,20の値を集約
let (left, right) = t.split(2); // 昇順の先頭2要素と残り
let t = left.merge(right);
assert_eq!(t.all_prod(), 13);
let (lt, ge) = t.split_key(&20); // key<20とkey>=20
assert_eq!(lt.all_prod(), 7);
assert_eq!(ge.all_prod(), 6);
```

| 操作 | 意味 | 計算量 |
|---|---|---|
| `new`, `singleton`, `len`, `is_empty` | 空木・1要素木・長さ | O(1) |
| `from_sorted(Vec<(K,S)>)` | 狭義キー昇順の配列から構築 | O(n) |
| `get(&key)`, `get_index(i)` | キー検索・昇順i番目の参照 | O(log n) |
| `insert(key,value)` | 挿入。既存キーは置換し旧値を返す | O(log n) |
| `remove(&key)` | 削除して値を返す。未存在ならNone | O(log n) |
| `split(k)`, `split_off(k)` | 昇順の先頭k個で分割 | O(log n) |
| `split_key(&key)` | key未満とkey以上へ分割 | O(log n) |
| `merge(other)`, `append(&mut other)` | 2つのキー集合を統合 | O(m log(n/m+1)) |
| `concat(other)` | キー範囲が順に並ぶ2木を連結 | O(log(n+m)) |
| `prod(l,r)` | 昇順の位置[l,r)の値の積 | O(log n) |
| `prod_reverse(l,r)` | 同じ位置範囲をr-1からlへ集約 | O(log n) |
| `all_prod`, `all_prod_reverse` | 全体の昇順・降順の積 | O(1) |
| `to_vec` | キー昇順に列挙。これだけK:Cloneが必要 | O(n) |

mergeの計算量では小さい側をm、大きい側をnとする。空木との統合はO(1)。
全ての計算量はモノイド演算、Clone、キー比較がO(1)の場合。

`merge` はキーの範囲が重なる場合も対応する。たとえばキー `{1,3,5}` と `{2,4,6}` を統合できる。
両方に同一キーがある場合はpanic。`concat` は左の全キーが右の全キーより小さい必要があり、違反はpanic。
`split`/`split_key`/`merge`/`concat` は木を消費する非永続操作。
`split_off` は左側をselfに残し右側を返す。`append` はotherの要素を移してotherを空にする。
範囲・位置の前提違反はreleaseでもpanic。消費するmergeで重複キーが検出された場合、元の木は保持されない。

## 区間ソートする列

`sortable_sequence.rs` は同じディレクトリの `keyed_avl_tree.rs` を読み込む。
モジュール利用時は `sortable_sequence::{KeyedAvlMonoid, SortableSequence}` をimportする。
列の位置とキーの順序は別で、初期列はキー順でなくてもよい。

```rust
let mut a = SortableSequence::<i32, Sum>::from_vec(vec![(30, 2), (10, 7), (20, 4)]);
a.sort_asc(0, 3);
assert_eq!(a.get(0), (&10, &7));
a.sort_desc(0, 3);
assert_eq!(a.get(0), (&30, &2));
a.set(1, 40, 8); // 列の位置1のkeyとvalueを置換
assert_eq!(a.prod(0, 2), 10);
```

現在のキーは列全体で一意であること。set後にもこの条件が必要。
一意性の全体検査は行わない。対象の公式問題では、初期値・更新で使うキーが全て異なる。

- `set(i,key,value)`・`get(i)`・`prod(l,r)`: 最悪O(log n)
- `sort_asc(l,r)`・`sort_desc(l,r)`・`sort(l,r,descending)`: 区間ソート
- `all_prod`・`len`・`is_empty`: O(1)
- 構築: O(n)、領域: O(n)

全区間は0-indexedの[l,r)。空列・空区間も扱える。列長は固定。
ソート済みブロックごとにキー付きAVL木を保持し、各ブロックの積は別のセグ木で管理する。
区間境界だけをsplitし、対象ブロックのキー集合をmergeする。降順では逆方向の積を使う。
prodはブロックを分割せず、端の部分積と中央のブロック積を組み合わせる。

区間ソート1回の時間は一律O(log n)ではない。対象がs要素・kブロックの場合、
境界分割とブロック管理はO((k+1)log(n+1))、さらに各AVL mergeの時間がかかる。
保守的な1回の上界はO((s+1)log(n+1))。
以下の実測は対象問題のテストでの性能確認であり、全操作列への償却上界の証明とは区別する。

## 非可換モノイド

例の関数合成では `op(f,g)` を `g∘f` とする。
値を(a,b)、f(x)=ax+bとすれば、mod 998244353で

```rust
fn op(f: &(u64,u64), g: &(u64,u64)) -> (u64,u64) {
    (f.0*g.0 % MOD, (f.1*g.0+g.1) % MOD)
}
```

この順序でprod(l,r)は左から関数を適用する。降順のソートでは要素順が逆になるため、
各ノードは昇順・降順両方の積を保存する。反転用の追加traitは要求しない。

## 再検証

```sh
cargo test --offline --test keyed_avl_tree
cargo test --offline --release --test keyed_avl_tree
cargo run --offline --release --example point_set_range_sort_range_composite < input.txt
python3 tests/check_keyed_avl_tree.py
python3 tests/check_keyed_avl_tree.py --official-root /path/to/library-checker-problems
```

検証スクリプトは本体・問題用例から単独提出コードを再生成し、rustc -Oでコンパイルする。
公式リポジトリを指定した場合はそこにparams.hを生成し、公式のverifierとcorrect.cpp、
info.tomlに指定された全ジェネレータを使う。ネット接続はしない。

単体テスト6本は、AVLの高さ差・サイズ・キー順・両方向の集約を内部まで検査する。
位置/キーの全境界分割、空木、非2冪長、交互キー集合の統合、順次挿入/削除、
BTreeMap/愚直列との固定seed比較、逆順ブロックの部分積、不正な範囲、
キーClone・値Debug・Defaultなしの型を含む。

問題形式でもPython愚直解との100seed比較を実施。
公式23ケースと独自の最大サイズ4ケースは、入力を公式verifierで検査し、公式解との出力一致を確認する。
最大サイズの独自ケースは、全体の昇降順反復、移動する長区間、一点更新と全体ソートの反復、
大量の一点更新後に細分化したブロックを再統合する操作列。
測定結果は `keyed_avl_results.json` に保存する。オンラインジャッジへの提出は行っていない。
