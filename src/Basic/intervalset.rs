// 隣接をマージするタイプ
#[derive(Clone)]
pub struct IntervalSet<T: Ord+Copy>{
    s: BTreeMap<T, T>,
}

impl<T> IntervalSet<T> where T: Ord+Copy {
    pub fn new()->Self {
        IntervalSet { s:BTreeMap::new(), }
    }

    pub fn insert(&mut self, mut l: T, mut r: T){
        if l >= r{return;}
        if let Some((&ll, &lr)) = self.s.range(..=l).next_back(){
            if r <= lr{
                return;
            } else if l <= lr{
                self.s.remove(&ll);
                l = ll;
            }
        }
        while let Some((&nl, &nr)) = self.s.range(l..).next(){
            if nr <= r{
                self.s.remove(&nl);
            } else if nl <= r{
                self.s.remove(&nl);
                r = nr;
                continue;
            } else {
                break;
            }
        }
        if l < r{
            self.s.insert(l, r);
        }
    }

    pub fn remove(&mut self, l: T, r: T){
        if l >= r{return;}
        if let Some((&ll, &lr)) = self.s.range(..=l).next_back(){
            if r <= lr{
                self.s.remove(&ll);
                if ll < l{
                    self.s.insert(ll, l);
                }
                if r < lr{
                    self.s.insert(r, lr);
                }
                return;
            } else if l < lr{
                if ll < l{
                    self.s.insert(ll, l);
                } else {
                    self.s.remove(&ll);
                }
            }
        }
        while let Some((&nl, &nr)) = self.s.range(l..).next(){
            if nr <= r{
                self.s.remove(&nl);
            } else if nl < r{
                self.s.remove(&nl);
                self.s.insert(r, nr);
                continue;
            } else {
                break;
            }
        } 
    }

    pub fn insert_with_data(&mut self, mut l: T, mut r: T)->Vec<(T, T, bool)>{
        let mut res = Vec::new();
        if l >= r{return res;}
        if let Some((&ll, &lr)) = self.s.range(..=l).next_back(){
            if r <= lr{
                return res;
            } else if l <= lr{
                res.push((ll, lr, false));
                self.s.remove(&ll);
                l = ll;
            }
        }
        while let Some((&nl, &nr)) = self.s.range(l..).next(){
            if nr <= r{
                self.s.remove(&nl);
                res.push((nl, nr, false));
            } else if nl <= r{
                self.s.remove(&nl);
                res.push((nl, nr, false));
                r = nr;
                continue;
            } else {
                break;
            }
        }
        if l < r{
            res.push((l, r, true));
            self.s.insert(l, r);
        }
        res
    }

    pub fn remove_with_data(&mut self, l: T, r: T)->Vec<(T, T, bool)>{
        let mut res = Vec::new();
        if l >= r{return res;}
        if let Some((&ll, &lr)) = self.s.range(..=l).next_back(){
            if r <= lr{
                self.s.remove(&ll);
                res.push((ll, lr, false));
                if ll < l{
                    self.s.insert(ll, l);
                    res.push((ll, l, true));
                }
                if r < lr{
                    self.s.insert(r, lr);
                    res.push((r, lr, true));
                }
                return res;
            } else if l < lr{
                if ll < l{
                    self.s.insert(ll, l);
                    res.push((ll, lr, false));
                    res.push((ll, l, true));
                } else {
                    self.s.remove(&ll);
                    res.push((ll, lr, false));
                }
            }
        }
        while let Some((&nl, &nr)) = self.s.range(l..).next(){
            if nr <= r{
                self.s.remove(&nl);
                res.push((nl, nr, false));
            } else if nl < r{
                self.s.remove(&nl);
                self.s.insert(r, nr);
                res.push((nl, nr, false));
                res.push((r, nr, true));
                continue;
            } else {
                break;
            }
        } 
        res
    }

    pub fn contains(&self, p: T)->bool{
        if let Some((&_, &r)) = self.s.range(..=p).next_back(){
            p < r
        } else {
            false
        }
    }

    pub fn section(&self, p: T)->Option<(T, T)>{
        if let Some((&l, &r)) = self.s.range(..=p).next_back(){
            Some((l, r))
        } else {
            None
        }
    }
}
