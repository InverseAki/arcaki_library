pub fn build_mi_cartesian_tree<T>(a: &[T])->(Vec<(usize, usize)>, Vec<usize>, usize) where T: Ord{
    if a.is_empty(){return (Vec::new(), Vec::new(), !0)}
    let n = a.len();
    let mut stack: Vec<usize> = Vec::with_capacity(n);
    let mut root = vec![!0; n];
    let mut left = vec![!0; n];
    let mut right = vec![!0; n];
    let mut r = 0;
    for i in 0..n{
        let mut pp = !0;
        while let Some(&last) = stack.last(){
            if a[last] >= a[i]{
                pp = last;
                stack.pop();
            } else {
                break;
            }
        }
        if pp != !0{
            left[i] = pp;
            root[pp] = i;
        }
        if let Some(&last) = stack.last(){
            root[i] = last;
            right[last] = i;
        } else {
            r = i;
        }
        stack.push(i);
    }
    (left.into_iter().zip(right).collect::<Vec<_>>(), root, r)
}

pub fn build_mx_cartesian_tree<T>(a: &[T])->(Vec<(usize, usize)>, Vec<usize>, usize) where T: Ord{
    if a.is_empty(){return (Vec::new(), Vec::new(), !0)}
    let n = a.len();
    let mut stack: Vec<usize> = Vec::with_capacity(n);
    let mut root = vec![!0; n];
    let mut left = vec![!0; n];
    let mut right = vec![!0; n];
    let mut r = 0;
    for i in 0..n{
        let mut pp = !0;
        while let Some(&last) = stack.last(){
            if a[last] <= a[i]{
                pp = last;
                stack.pop();
            } else {
                break;
            }
        }
        if pp != !0{
            left[i] = pp;
            root[pp] = i;
        }
        if let Some(&last) = stack.last(){
            root[i] = last;
            right[last] = i;
        } else {
            r = i;
        }
        stack.push(i);
    }
    (left.into_iter().zip(right).collect::<Vec<_>>(), root, r)
}
