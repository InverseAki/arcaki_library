# Rollback可能なデータ構造

すべて `DataStructure/rollbackvector.rs` に依存し、DS内部で更新前の履歴長を積む。
元のUF・重み付きUFの公開APIは維持。セグ木は別ファイル・別の型として追加した。

| 本体 | 型 | 更新1回のrollback |
| --- | --- | --- |
| `Graph/rollbackunionfind.rs` | `RollbackUnionFind` | O(1) |
| `Graph/weighted_rollback_unionfind.rs` | `RollbackWeightedUnionFind<M>` | O(1) |
| `SegmentTree/rollbacksegtree.rs` | `RollbackSegtree<M>` | O(log n) |
| `SegmentTree/rollbacklazysegtree.rs` | `RollbackLazySegtree<M>` | O(log n) |

値・作用・群の演算がO(1)の場合の計算量。UFのfind/併合はO(log n)、セグ木の構築はO(n)、更新・区間積・境界探索はO(log n)。
UFの領域はO(n + 保存中の操作数)、セグ木はO(n + 保存中の更新数 * log n)。

## 履歴

- `rollback()`：直前の公開更新操作を1回取り消す。履歴がなければ何もしない。
- `snapshot()`：現在の状態を維持して履歴を捨てる。保存状態への復帰ではない。履歴数に比例する時間。
- `all_back()`：最後のsnapshot（なければ初期状態）まで戻す。保存中の変更数に比例する時間。
- 失敗したmerge、重み付きUFの整合済み/矛盾したunion、遅延セグ木の空区間更新も1操作として数える。
- get/prod/探索等の読み取りは状態・履歴を変更しない。

```rust
let mut uf = RollbackUnionFind::new(3);
uf.merge(0, 1);
uf.merge(0, 1); // 失敗も1操作
uf.rollback(); // 失敗したmergeを取り消す
assert!(uf.same(0, 1));
uf.rollback();
assert!(!uf.same(0, 1));
```

重み付きUFの `union(u, v, w)` は `diff(u, v) = w` を追加する。
矛盾時はfalseを返して状態を変えない。矛盾数は保持しない。
UFMonoidの名前・署名は従来どおりだが、必要な性質は群（逆元を持つ）で、可換性は不要。

## セグ木

点更新版の `RollbackSegtreeMonoid` は従来の `SegtreeMonoid` と同じ署名。
`new(n)`, `from(Vec)`, `len`, `is_empty`, `set`, `push`, `get`, `prod`, `all_prod`, `max_right`, `min_left` を提供。
`push(i, x)` は `A[i] = op(A[i], x)`。
区間は0-indexedの半開区間 `[l, r)`。空配列・非2冪長に対応し、範囲外はpanicする。

遅延版のtraitは `RollbackLazySegtreeMonoid`。従来版と異なり、値モノイドを別の関連型Mに分けず、Sと値の演算を直接定義する。

```rust
struct RangeAffineSum;
impl RollbackLazySegtreeMonoid for RangeAffineSum {
    type S = (i64, i64); // 合計、要素数
    type F = (i64, i64); // x -> a*x+b
    fn id_e() -> Self::S { (0, 0) }
    fn op(a: &Self::S, b: &Self::S) -> Self::S { (a.0+b.0, a.1+b.1) }
    fn identity() -> Self::F { (1, 0) }
    fn map(f: &Self::F, x: &Self::S) -> Self::S { (f.0*x.0+f.1*x.1, x.1) }
    fn composition(f: &Self::F, g: &Self::F) -> Self::F {
        (f.0*g.0, f.0*g.1+f.1) // gの後にf
    }
}
let mut seg = RollbackLazySegtree::<RangeAffineSum>::from(vec![(1,1),(2,1),(3,1)]);
seg.apply_range(0, 3, (1, 5));
assert_eq!(seg.prod(0, 3), (21, 3));
seg.set(1, (10, 1));
seg.rollback(); // setだけを取り消す
assert_eq!(seg.get(1), (7, 1));
seg.rollback(); // apply_rangeを取り消す
assert_eq!(seg.all_prod(), (6, 3));
```

遅延版は `new(n)`, `from(Vec)`, `build(&[S])`, `set`, `apply(p, f)`, `apply_range(l, r, f)`, `get`, `prod`, `all_prod`, `get_slice`, `max_right`, `min_left`, `len`, `is_empty` を提供。
`new(n)` は値の単位元で初期化するため、区間和のように長さを持つ値では上例のようにfrom/buildで各要素の長さを渡す。
遅延作用は更新時にだけpushし、読み取りは祖先の作用を引き継いで計算する。
`get_slice` はO((r-l) log n)。両セグ木の探索述語は値の参照を受け、単位元でtrueかつ区間を伸ばす方向にtrueからfalseへの単調性が必要。

## 提出用にコピーする場合

リポジトリ内で使う際は各DSが冒頭のpath指定からRollbackVectorを読み込む。
1ファイルのmain.rsへ貼る場合は、RollbackVectorを1回貼り、各DSの冒頭のpath属性と `mod rollback_*_vector;` を取り除き、`rollback_*_vector::RollbackVector` を `RollbackVector` に置き換える。
trait名は点更新版・遅延版で異なり、両方を同じスコープへ貼れる。

## 検証

```sh
cargo test --offline --test rollback_vector --test structures --test rollback_structures
cargo test --offline --release --test rollback_vector --test structures --test rollback_structures
```

保存した配列/分割/制約グラフとの比較、空配列、非2冪長、重複更新、失敗・矛盾した併合、snapshot/all_back、範囲外、非可換な積・作用・群、区間代入/加算と点代入の混在、読み取り後のrollback、境界探索を確認。
外部ジャッジへの提出は行っていない。
