#[derive(Clone)]
pub struct IntervalSetV<T, V> where T: Ord + Copy, V: Eq + Copy{
    s: BTreeMap<T, (T, V)>,
}

impl<T, V> IntervalSetV<T, V> where T: Ord + Copy, V: Eq + Copy{
    pub fn new() -> Self {
        Self { s: BTreeMap::new() }
    }

    pub fn insert(&mut self, mut l: T, mut r: T, v: V){
        if l >= r {return;}
        if let Some((&ll, &(lr, lv))) = self.s.range(..=l).next_back() {
            if lv == v {
                if l <= lr {
                    self.s.remove(&ll);
                    l = ll;
                    if r < lr {
                        r = lr;
                    }
                }
            } else {
                if l < lr {
                    self.s.remove(&ll);
                    if ll < l {
                        self.s.insert(ll, (l, lv));
                    }
                    if r < lr {
                        self.s.insert(r, (lr, lv));
                    }
                }
            }
        }
        while let Some((&nl, &(nr, nv))) = self.s.range(l..).next() {
            if nv == v {
                if nl <= r {
                    self.s.remove(&nl);
                    if r < nr {
                        r = nr;
                    }
                    continue;
                } else {
                    break;
                }
            } else {
                if nr <= r {
                    self.s.remove(&nl);
                    continue;
                } else if nl < r {
                    self.s.remove(&nl);
                    self.s.insert(r, (nr, nv));
                    break;
                } else {
                    break;
                }
            }
        }
        self.s.insert(l, (r, v));
    }

    pub fn insert_with_data(&mut self, mut l: T, mut r: T, v: V) -> Vec<(T, T, V, bool)> {
        let mut res = Vec::new();
        if l >= r {
            return res;
        }
        if let Some((&ll, &(lr, lv))) = self.s.range(..=l).next_back() {
            if lv == v {
                if l <= lr {
                    self.s.remove(&ll);
                    res.push((ll, lr, lv, false));
                    l = ll;
                    if r < lr {
                        r = lr;
                    }
                }
            } else {
                if l < lr {
                    self.s.remove(&ll);
                    res.push((ll, lr, lv, false));
                    if ll < l {
                        self.s.insert(ll, (l, lv));
                        res.push((ll, l, lv, true));
                    }
                    if r < lr {
                        self.s.insert(r, (lr, lv));
                        res.push((r, lr, lv, true));
                    }
                }
            }
        }
        while let Some((&nl, &(nr, nv))) = self.s.range(l..).next() {
            if nv == v {
                if nl <= r {
                    self.s.remove(&nl);
                    res.push((nl, nr, nv, false));
                    if r < nr {
                        r = nr;
                    }
                    continue;
                } else {
                    break;
                }
            } else {
                if nr <= r {
                    self.s.remove(&nl);
                    res.push((nl, nr, nv, false));
                    continue;
                } else if nl < r {
                    self.s.remove(&nl);
                    res.push((nl, nr, nv, false));
                    self.s.insert(r, (nr, nv));
                    res.push((r, nr, nv, true));
                    break;
                } else {
                    break;
                }
            }
        }
        self.s.insert(l, (r, v));
        res.push((l, r, v, true));
        res
    }

    pub fn remove(&mut self, l: T, r: T){
        if l >= r {
            return;
        }
        if let Some((&ll, &(lr, lv))) = self.s.range(..=l).next_back() {
            if r <= lr {
                self.s.remove(&ll);
                if ll < l {
                    self.s.insert(ll, (l, lv));
                }
                if r < lr {
                    self.s.insert(r, (lr, lv));
                }
                return;
            } else if l < lr {
                self.s.remove(&ll);
                if ll < l {
                    self.s.insert(ll, (l, lv));
                }
            }
        }

        while let Some((&nl, &(nr, nv))) = self.s.range(l..).next() {
            if nr <= r {
                self.s.remove(&nl);
            } else if nl < r {
                self.s.remove(&nl);
                self.s.insert(r, (nr, nv));
                break;
            } else {
                break;
            }
        }
    }

    pub fn remove_with_data(&mut self, l: T, r: T) -> Vec<(T, T, V, bool)> {
        let mut res = Vec::new();
        if l >= r {
            return res;
        }
        if let Some((&ll, &(lr, lv))) = self.s.range(..=l).next_back() {
            if r <= lr {
                self.s.remove(&ll);
                res.push((ll, lr, lv, false));
                if ll < l {
                    self.s.insert(ll, (l, lv));
                    res.push((ll, l, lv, true));
                }
                if r < lr {
                    self.s.insert(r, (lr, lv));
                    res.push((r, lr, lv, true));
                }
                return res;
            } else if l < lr {
                self.s.remove(&ll);
                res.push((ll, lr, lv, false));
                if ll < l {
                    self.s.insert(ll, (l, lv));
                    res.push((ll, l, lv, true));
                }
            }
        }

        while let Some((&nl, &(nr, nv))) = self.s.range(l..).next() {
            if nr <= r {
                self.s.remove(&nl);
                res.push((nl, nr, nv, false));
            } else if nl < r {
                self.s.remove(&nl);
                res.push((nl, nr, nv, false));
                self.s.insert(r, (nr, nv));
                res.push((r, nr, nv, true));
                break;
            } else {
                break;
            }
        }
        res
    }

    pub fn contains(&self, p: T) -> bool {
        if let Some((&_l, &(r, _v))) = self.s.range(..=p).next_back() {
            p < r
        } else {
            false
        }
    }

    pub fn get(&self, p: T) -> Option<V> {
        if let Some((&_l, &(r, v))) = self.s.range(..=p).next_back() {
            if p < r {
                Some(v)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn section(&self, p: T) -> Option<(T, T, V)> {
        if let Some((&l, &(r, v))) = self.s.range(..=p).next_back() {
            if p < r {
                Some((l, r, v))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (T, T, V)> + '_ {
        self.s.iter().map(|(&l, &(r, v))| (l, r, v))
    }

    pub fn is_empty(&self) -> bool {
        self.s.is_empty()
    }

    pub fn clear(&mut self) {
        self.s.clear();
    }

    pub fn range(&self, l: T, r: T) -> Vec<(T, T, V)> {
        let mut res = Vec::new();
        if l >= r {return res;}
        if let Some((_, &(lr, lv))) = self.s.range(..=l).next_back() {
            if l < lr {
                let a = l;
                let b = if lr < r { lr } else { r };
                if a < b {
                    res.push((a, b, lv));
                }
            }
        }
        for (&nl, &(nr, nv)) in self.s.range(l..) {
            if nl >= r {
                break;
            }
            let a = nl;
            let b = if nr < r { nr } else { r };
            if a < b {
                res.push((a, b, nv));
            }
        }
        res
    }
}
