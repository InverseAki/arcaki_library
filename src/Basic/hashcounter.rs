#[derive(Debug, Clone)]
pub struct HashCounter<T: Ord+Hash>{
    c: usize,
    map: HashMap<T, usize>,
}

impl<T: Copy+Ord+Hash> HashCounter<T>{
    pub fn new()->Self{
        HashCounter{
            c: 0,
            map: HashMap::new(),
        }
    }

    #[inline(always)]
    pub fn one_add(&mut self, x: T){
        *self.map.entry(x).or_insert(0) += 1;
        self.c += 1;
    }

    #[inline(always)]
    pub fn one_sub(&mut self, x: T){
        if !self.map.contains_key(&x){return}
        let e = self.map.entry(x).or_insert(0);
        *e = e.saturating_sub(1);
        if self.map[&x] <= 0{
            self.map.remove(&x);
        }
        self.c = self.c.saturating_sub(1);
    }

    #[inline(always)]
    pub fn one_update(&mut self, x: T, y: T){
        self.one_sub(x);
        self.one_add(y);
    }

    #[inline(always)]
    pub fn del(&mut self, x: T){
        self.c = self.c.saturating_sub(*self.map.get(&x).unwrap_or(&0));
        self.map.remove(&x);
    }

    #[inline(always)]
    pub fn add(&mut self, x: T, c: usize){
        *self.map.entry(x).or_insert(0) += c;
        self.c += c;
    }

    #[inline(always)]
    pub fn add_ex(&mut self, x: T, c: usize){
        let e = self.map.get_mut(&x).unwrap();
        *e += c;
        self.c += c;
    }

    #[inline(always)]
    pub fn sub(&mut self, x: T, c: usize){
        let e = self.map.entry(x).or_insert(0);
        *e = e.saturating_sub(c);
        if self.map[&x] == 0{
            self.map.remove(&x);
        }
        self.c = self.c.saturating_sub(c);
    }

    #[inline(always)]
    pub fn sub_ex(&mut self, x: T, c: usize){
        let e = self.map.get_mut(&x).unwrap();
        *e -= c;
        if *e==0{
            self.map.remove(&x);
        }
        self.c -= c;
    }

    #[inline(always)]
    pub fn include(&self, x: T)->bool{
        self.map.contains_key(&x)
    }

    #[inline(always)]
    pub fn cnt(&self, x: T)->usize{
        *self.map.get(&x).unwrap_or(&0)
    }

    #[inline(always)]
    pub fn is_empty(&self)->bool{
        self.map.is_empty()
    }

    #[inline(always)]
    pub fn len(&self)->usize{
        self.map.len()
    }

    #[inline(always)]
    pub fn clear(&mut self){
        self.map.clear();
        self.c = 0;
    }

    #[inline(always)]
    pub fn merge(&mut self, rhs: &mut HashCounter<T>){
        if self.len() < rhs.len(){
            swap(self, rhs);
        }
        for (&k, &v) in rhs.map.iter(){
            self.add(k, v);
        }
        rhs.clear();
    }
}
