# SplayVector<T>（2026-10-04）

本体は `../src/DataStructure/splay.rs`。値だけを持つSplayTree<NilMonoid<T>>を、Vecに近いAPIで扱うラッパー。
TにClone、Debug、Defaultを要求しない。元のSplayTreeのAPIとProdMonoid/遅延更新の用途は保持する。

```rust
let mut v = SplayVector::from(vec![10, 20, 30]);
v.push(40);
v[1] = 7;
assert_eq!(v.get(1), Some(&7));
assert_eq!(v.remove(0), 10);
v.reverse();
assert_eq!(v.to_vec(), vec![40, 30, 7]);
for x in &mut v { *x += 1; }
assert_eq!(v.pop(), Some(8));
let a: Vec<_> = v.into();
assert_eq!(a, vec![41, 31]);
```

## 対応API

- new / with_capacity / from_vec / Default / From<Vec<T>> / From<[T;N]> / collect
- len / is_empty / capacity / reserve / reserve_exact / shrink_to / shrink_to_fit
- get / get_mut / first / first_mut / last / last_mut / v[i] / v[i] = x
- push / pop / insert / remove / swap / swap_remove / set
- clear / truncate / resize / resize_with / extend / extend_from_slice / append / split_off
- retain / retain_mut / drain
- iter / iter_mut / 所有するinto_iter / &v と &mut v でのfor
- reverse（全体反転） / reverse_range（範囲反転）
- to_vec / into_vec / From<SplayVector<T>> for Vec<T>
- Clone（T:Clone） / Debug（T:Debug） / PartialEq / Eq

getはVecと同様にOption<&T>を返す。get_mutはOption<&mut T>。範囲外のgetはNone、添字アクセス・remove・insert等はpanic。
remove/pop/所有イテレータ/into_vecは値をcloneせず取り出す。
Clone、to_vec、resize、extend_from_slice、参照からのextendだけがT:Cloneを要求する。
イテレータは両端から消費でき、ExactSizeIteratorとFusedIteratorを実装する。

## Vecとの違いと計算量

木のノードは連続したメモリには並ばない。as_slice/as_mut_slice/as_ptr、スライスへのDeref、範囲添字v[l..r]は提供しない。
範囲操作はreverse_range/drainで、連続配列が必要ならto_vec（複製）/into_vec（移動）で扱う。

| 操作 | 計算量 |
| --- | --- |
| len/is_empty、末尾push、全体reverse | O(1)（確保・値の破棄等の費用は別） |
| get、読み取りv[i]、first/last | O(木の高さ)、最悪O(n) |
| get_splayed、get_mut、書き込みv[i]、insert/remove/pop/swap、reverse_range | 償却O(log n) |
| from_vec、collect、clear、to_vec/into_vec、全イテレータの走査 | O(n) |
| append/extendでk個追加 | O(k) |
| drain/split_off/truncateでk個削除 | 償却O(k log n) |
| retain/retain_mut | 償却O(n log n) |

共有参照がある間は、読み取りget/index/iterが木の回転・反転伝播を行わない。
ノードに残った反転フラグを祖先から合成し、論理的な要素順で読む。
読み取りもsplayしたい場合はget_splayed(&mut self,k)を使う。戻り値はOption<&T>。
大量の順次取得はiterを使うと、偏った木でも全体O(n)で走査できる。

iter/iter_mut/所有イテレータはO(木の高さ)の作業スタックを使い、反転フラグを読むだけで前後から走査する。
所有イテレータは未消費の値を含む木を保持し、破棄時に残りを解放する。
容量系APIは所有ノードのポインタ配列に対する操作。容量内のpushでもノード自体のBoxの確保は発生する。
append/clearで元の所有ポインタ配列の容量を保持する。
drainは指定範囲を先に除去し、その値を返す。Drainを途中で破棄すると未消費の値を破棄する。

## 検証

```sh
cargo test --offline
cargo test --offline --release
```

本体全31テストをdebug/releaseで通過。SplayVectorは4本。
6種類の初期長×3seed×2,000操作（36,000操作）をVecと比較。
未伝播の反転後の参照取得、前後を混ぜて取り出す共有/可変/所有イテレータ、同時に保持した複数の可変参照も検証。
長さ0〜10で全方向パターンを走査し、各要素をちょうど一度返すことを確認。
Clone/Debugを持たない所有型の移動・部分消費・drain・clearで、全値が一度ずつ破棄されることを確認。
容量保持、Vecへの変換、不正な添字・範囲も確認した。
Miriと外部提出はこの作業では未実施。
