pub struct MinMaxHeap<T> where T: Copy+Ord{
    v: Vec<T>
}

impl<T> MinMaxHeap<T> where T: Copy+Ord{
    pub fn new()->Self{
        MinMaxHeap { v: Vec::new() }
    }

    pub fn with_capacity(n: usize)->Self{
        MinMaxHeap { v: Vec::with_capacity(n) }
    }

    #[inline]
    pub fn len(&self)->usize{
        self.v.len()
    }

    #[inline]
    pub fn is_empty(&self)->bool{
        self.v.is_empty()
    }

    #[inline]
    pub fn clear(&mut self){
        self.v.clear();
    }

    #[inline]
    pub fn shrink_to_fit(&mut self){
        self.v.shrink_to_fit();
    }

    #[inline]
    fn left_child(p: usize)->usize{
        (p<<1)+2-(p&1)
    }

    #[inline]
    fn right_child(p: usize)->usize{
        (p<<1)+4-(p&1)
    }

    #[inline]
    fn parent(&self, p: usize)->usize{
        if p < 2{self.v.len()}
        else {
            ((p>>1)-1)&!1
        }
    }

    #[inline]
    pub fn push(&mut self, x: T){
        let mut p = self.v.len();
        self.v.push(x);
        if (p|1) < self.v.len() && self.v[p&!1] > self.v[p|1]{
            self.v.swap(p&!1,p|1);
            p ^= 1;
        }
        loop {
            let pre =self.parent(p);
            if !(pre<self.v.len()&&self.v[pre] > self.v[p]){break;}
            self.v.swap(p, pre);
            p = pre;
        }
        loop {
            let pre=(self.parent(p))|1;
            if !(pre<self.v.len()&&self.v[pre] < self.v[p]){break;}
            self.v.swap(p, pre);
            p = pre;
        }
    }

    #[inline]
    pub fn pop_min(&mut self)->Option<T>{
        if self.is_empty(){return None;}
        else if self.len()==1{return Some(self.v.pop().unwrap());}
        let res = self.v.swap_remove(0);
        let mut p = 0;
        self.mi_down(&mut p);
        if (p^1)<self.v.len()&&self.v[p]>self.v[p^1]{
            self.v.swap(p,p^1);
            p ^= 1;
            self.mx_up(&mut p);
        }
        Some(res)
    }

    #[inline]
    pub fn pop_max(&mut self)->Option<T>{
        if self.is_empty(){return None;}
        else if self.len()<=2{return Some(self.v.pop().unwrap());}
        let res = self.v.swap_remove(1);
        let mut p = 1;
        self.mx_down(&mut p);
        if (p^1)<self.v.len()&&self.v[p]<self.v[p^1]{
            self.v.swap(p,p^1);
            p ^= 1;
            self.mi_up(&mut p);
        }
        Some(res)
    }

    #[inline]
    fn mi_down(&mut self, p: &mut usize){
        loop {
            let mut c = Self::left_child(*p);
            if c >= self.v.len(){break;}
            let rc = Self::right_child(*p);
            if rc < self.v.len() && self.v[rc] < self.v[c]{
                c = rc;
            }
            if self.v[c] < self.v[*p]{
                self.v.swap(c, *p);
                *p = c;
            } else {
                break;
            }
        }
    }

    #[inline]
    fn mx_down(&mut self, p: &mut usize){
        loop {
            let mut c = Self::left_child(*p);
            if c >= self.v.len(){break;}
            let rc = Self::right_child(*p);
            if rc < self.v.len() && self.v[rc] > self.v[c]{
                c = rc;
            }
            if self.v[c] > self.v[*p]{
                self.v.swap(c, *p);
                *p = c;
            } else {
                break;
            }
        }
    }

    #[inline]
    fn mi_up(&mut self, p: &mut usize){
        loop {
            let pre = self.parent(*p);
            if pre < self.v.len() && self.v[pre] > self.v[*p]{
                self.v.swap(pre, *p);
                *p = pre;
            } else {
                break;
            }
        }
    }

    #[inline]
    fn mx_up(&mut self, p: &mut usize){
        loop {
            let pre = self.parent(*p)|1;
            if pre < self.v.len() && self.v[pre] < self.v[*p]{
                self.v.swap(pre, *p);
                *p = pre;
            } else {
                break;
            }
        }
    }
}
