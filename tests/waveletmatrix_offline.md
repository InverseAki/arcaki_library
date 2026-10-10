# 候補登録型の更新可能 Wavelet Matrix

本体: [waveletmatrix_offline.rs](../src/DataStructure/waveletmatrix_offline.rs)

[ABC467 G — Many Sweets Problem](https://atcoder.jp/contests/abc467/tasks/abc467_g)
のような「配列の1点更新＋区間内で総和を満たす最小個数」を扱う。
区間 kth、値域の個数・総和、昇順の順位区間の総和にも対応する。

## 使い方

```rust
include!("waveletmatrix_offline.rs");

let initial = [8, 2, 4, 1, 7, 3, 6];
// 添字は0-indexed。初期値は自動登録される。
// 更新の実行順は不要。可能性のある (添字, 値) をすべて登録する。
let updates = [(0, 1), (4, 2), (5, 4), (5, 5), (6, 9)];
let mut wm = WaveletMatrixOffline::new(&initial, &updates);
assert_eq!(wm.kth_smallest(0, 7, 3), 4);
assert_eq!(wm.range_sum(0, 7), 31);
wm.set(0, 1);
assert_eq!(wm.get(0), 1);
assert_eq!(wm.min_count_for_sum(3, 7, 9), Some(2));
wm.set(4, 2);
assert_eq!(wm.min_count_for_sum(0, 3, 8), None);
assert_eq!(wm.sum_largest(0, 3, 2), 6);
assert_eq!(wm.sum_between(0, 7, 2, 5), 11);
```

提出用には `waveletmatrix_offline.rs` と既存の `waveletmatrix.rs` を隣に置き、
前者を `include!` する。本体は後者のビット列を内部モジュールで利用する。
累積版・モノイド版・元の WaveletMatrix と同じスコープに読み込んでも型名は衝突しない。

ABC467 G の入力処理と回答まで含む実行例: [examples/abc467_g.rs](../examples/abc467_g.rs)。
全クエリを先読みし、`(c,x)` を候補登録してから順に実行する。
問題の1-indexed・閉区間の入力は、実行例で0-indexed・半開区間へ変換している。

## API と制約

値は非負の `usize`、総和・総和の目標は `u128`。各元添字には常に値が1つ存在する。
値0や `usize::MAX` も使用でき、合計が `u64::MAX` を超えても扱える。

| API | 内容 |
|---|---|
| `new(initial, updates)` | 初期配列と可能な更新候補を登録・構築 |
| `len()` / `is_empty()` | 元の配列の長さ / 空か |
| `candidate_len()` | 初期値を含む、重複除去後の候補点数 |
| `get(i)` / `set(i,x)` | 元の添字で値を取得 / 更新 |
| `kth_smallest(l,r,k)` / `kth_largest(l,r,k)` | 現在の区間の小さい / 大きい方から k 番目 |
| `rank_range(x,l,r)` | 値 x の現在の出現数 |
| `range_freq(l,r,upper)` | 値が upper 未満の個数 |
| `range_freq_between(l,r,lower,upper)` | 値域 `[lower,upper)` の個数 |
| `range_sum(l,r)` | 区間の総和 |
| `sum_less(l,r,upper)` / `sum_between(l,r,lower,upper)` | 値域の総和 |
| `sum_smallest(l,r,k)` / `sum_largest(l,r,k)` | 小さい / 大きい k 個の総和 |
| `sum_sorted(l,r,start,end)` | 昇順の順位区間 `[start,end)` の総和 |
| `min_count_for_sum(l,r,target)` | 総和が target 以上になる最小個数、達成不能なら None |
| `prev_value(l,r,upper)` / `next_value(l,r,lower)` | upper 未満の最大値 / lower 以上の最小値 |

添字・順位は0-indexed、区間は半開。`sum_smallest/sum_largest` は k=0 と区間長も許す。
`kth_*` は k が区間長未満である必要がある。
空集合の個数・総和は0、逆転した値域は空集合。
`min_count_for_sum` は target=0 なら空区間でも Some(0)、空区間かつ target>0 なら None。
不正な添字・順位・未登録更新はassert/panicで検出する。
未登録更新は状態を変更する前に検出するので、panicを捕捉した後も状態を保つ。

**登録は `(添字, 値)` ごとに必要**。別の添字に登録済みの値でも、この添字に未登録なら更新不可。
同じ候補の重複登録と同じ値への再更新は許す。
グローバルに出現する値だけを知っていても、全添字への全値登録は候補数が膨らむ。
各添字が取りうる値を絞ることが、この版を使う前提になる。

## 内部構造

1. 初期値と更新候補を `(元添字, 値)` 順に並べ、重複を除く。
2. 値を圧縮し、この候補列に静的 Wavelet Matrix を構築する。
3. 各元添字の現在値の候補だけを有効にする。有効なら個数1・総和は値、無効なら両方0。
4. 各段の並びに対する個数・総和の BIT を構築する。

元の区間 `[l,r)` は、最上段の候補列で
`[offsets[l], offsets[r])` に対応する。
各段のビット列で区間や候補点の位置を写す操作は固定のまま。
更新時には旧候補の個数・総和を引き、新候補を加える。

kth では **候補の個数ではなく、BITの有効個数** を見て進む。
閾値に対する最小個数は、大きい値側の有効総和を見て進む。
その側だけで足りればそちらへ降り、足りなければその側の全個数を答えに加え、
目標からその総和を引いて小さい側へ降りる。
同値の葉 v に着いたら `ceil(残り目標 / v)` 個を加える。
正の目標で総和が足りると確認してから降りるので、到達する葉は v>0 である。
値0を含む場合も、不要な0を選ばずに最小個数を求める。
個数や値で二分探索する必要はない。

これは任意のキー変更を行う動的ビット列の Wavelet Matrix ではなく、
**候補点を先に固定し、有効状態をオンラインに更新する構造**。
登録後のクエリ順は自由で、結果に応じて次の登録済み更新を選ぶこともできる。
前の `WaveletMatrixMonoid` の「固定キーの別データを更新」とは異なり、
この版は配列の値そのものを変更できる。
任意データ・非可換モノイドはこの数値用の版では扱わない。

## 計算量・メモリ

R = 初期値と重複を含む登録候補の数、P = 重複除去後の候補点数、
σ = 候補に現れる異なる値の数、L = max(1, ceil(log2 σ))。

| 操作 | 計算量 |
|---|---|
| 構築 | O(R log(R+1) + PL) |
| 永続領域 | O(PL + N) |
| `set` | O(L log(P+1)) |
| `kth_*`・値域集約・順位の総和・最小個数 | O(L log(P+1)) |
| `rank_range` | O(L + log(P+1)) |
| `range_sum` | O(log(P+1)) |
| `get`・長さ関連 | O(1) |

構築時にはさらに O(R+P) の一時領域を使う。
64-bit環境でBITの個数を `usize`、総和を `u128` で保持するため、
BITだけでおよそ `24(P+1)(L+1)` bytes。
このほか、ビット列・候補・元添字のオフセット等を保持する。
入力の更新先が Q 個なら P <= N+Q。

## 検証

```sh
cargo test --offline --test waveletmatrix_offline
cargo test --offline --release --test waveletmatrix_offline
python3 tests/check_waveletmatrix_offline.py < /dev/null
python3 tests/check_waveletmatrix_offline.py --benchmark --output tests/waveletmatrix_offline_results.json < /dev/null
```

Pythonの乱数seedは標準入力から指定する。空入力なら467。
Rustの4テストはdebug/release両方通過。
長さ4・値集合 `{0,1,3}` の全81状態と全区間を確認し、
最小個数は全ての部分集合から選ぶ独立な愚直解とも比較した。
固定seedの更新列、全同値、昇順・降順、候補点数63/64/65/127/128/129、
1添字への候補集中、最大usize、u64を超える和、未登録更新の状態保持も検証した。

実行例は公式サンプル2個と、100ケース・計10,000更新クエリを
debug/releaseそれぞれで愚直解と比較して通過。
公式ジャッジへの提出は行っていない。

最大規模の4ケースはすべて N=Q=100,000。
同値・候補集中・昇順の3ケースは全回答を独立に検証。
ランダム候補のケースは全回答の総和による達成可否・個数の範囲を確認し、
別途128クエリを区間のソートによる厳密解と比較した。
時間・最大RSSは [waveletmatrix_offline_results.json](waveletmatrix_offline_results.json) に記録。
入力生成・コンパイル時間を除き、入力読込・構築・全クエリ・出力と計測用Pythonの起動を含む。
最大RSSは、新規の計測親プロセスで対象バイナリだけの `RUSAGE_CHILDREN` を取得する。
ローカル環境での測定であり、AtCoder上の実行時間ではない。

2026-10-05 のローカル測定（macOS arm64、rustc 1.93.0、`rustc -O`）:

| ケース | 入力読込から出力まで | 最大RSS |
|---|---:|---:|
| 全同値 | 0.090秒 | 25.9 MiB |
| 1添字に100,000候補が集中 | 0.287秒 | 107.3 MiB |
| 昇順 | 0.165秒 | 62.4 MiB |
| ランダム候補・値変更・任意区間 | 0.488秒 | 113.1 MiB |
