pub fn convolution_merge(vs: &mut Vec<Vec<MI>>) -> Vec<MI> {
    convolution_merge_with_mx(vs, usize::MAX)
}

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
