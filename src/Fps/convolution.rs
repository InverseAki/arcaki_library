// mod 998244353（各入力長<=2^24）: convolution_mod998244353.rsを併用し、
// use convolution_mod998244353 as convolution; で選択。
// mod 1_000_000_007: convolution_mod1000000007.rsを同時にコピーし、
// use convolution_mod1000000007 as convolution; でバックエンドを選択できる。
// 前提: MI と convolution(&[MI], &[MI]) -> Vec<MI> が同じスコープにある。
// ACL版/同梱版のいずれも可。NTT本体はこのファイルには含めない。
// vs の内容は消費する。空の積は [1]、空の多項式を含む積は []。
pub fn convolution_merge(vs: &mut Vec<Vec<MI>>) -> Vec<MI> {
    convolution_merge_with_mx(vs, usize::MAX)
}

/// 低次側 mx 個の係数のみ返す。mx は最大次数ではなく係数数。
/// 1本だけの場合も切り詰める。各段階で切り詰めてよいのは通常の多項式積のため。
pub fn convolution_merge_with_mx(vs: &mut Vec<Vec<MI>>, mx: usize) -> Vec<MI> {
    if mx == 0 || vs.iter().any(Vec::is_empty) {
        for v in vs.iter_mut() {
            v.clear();
        }
        return vec![];
    }
    if vs.is_empty() {
        return vec![MI::new(1)];
    }
    let mut heap = std::collections::BinaryHeap::new();
    for (i, v) in vs.iter_mut().enumerate() {
        v.truncate(mx);
        heap.push((std::cmp::Reverse(v.len()), i));
    }
    while heap.len() > 1 {
        let (_, i) = heap.pop().unwrap();
        let (_, j) = heap.pop().unwrap();
        let a = std::mem::take(&mut vs[i]);
        let b = std::mem::take(&mut vs[j]);
        let mut c = convolution(&a, &b);
        c.truncate(mx);
        vs[i] = c;
        heap.push((std::cmp::Reverse(vs[i].len()), i));
    }
    std::mem::take(&mut vs[heap.pop().unwrap().1])
}
