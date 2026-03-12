pub fn merge<T>(a: &[T], b: &[T])->Vec<T> where T: Ord+Copy{
    let mut i = 0;
    let mut j = 0;
    let mut res = Vec::with_capacity(a.len()+b.len());
    while i < a.len() || j < b.len() {
        if i==a.len() || (j < b.len() && a[i] > b[j]){
            res.push(b[j]);
            j += 1;
        } else {
            res.push(a[i]);
            i += 1;
        }
    }
    res
}
