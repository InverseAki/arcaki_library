// max→false, min→true
#[derive(Clone, Copy, Debug)]
struct LiChaoLine {
    a: i64, b: i128,
}

impl LiChaoLine {
    #[inline]
    fn eval(self, x: i64)->i128{
        self.a as i128*x as i128+self.b
    }
}

#[derive(Clone, Debug)]
pub struct LiChaoNode {
    line: LiChaoLine, child: [usize;2],
}

#[derive(Clone, Debug)]
pub struct LiChaoTree<const LICAOISMIN: bool> {
    x_min: i64, x_max: i64, 
    nodes: Vec<LiChaoNode>,
}

impl<const LICAOISMIN: bool> LiChaoTree<LICAOISMIN> {
    const NONE: usize = usize::MAX;
    pub fn new(x_min: i64, x_max: i64)->Self{
        assert!(x_min<=x_max);
        Self{x_min,x_max,nodes:Vec::new(),}
    }
    fn push_node(&mut self, line: LiChaoLine)->usize {
        let idx = self.nodes.len();
        self.nodes.push(LiChaoNode { line, child: [Self::NONE;2] });
        idx
    }
    #[inline]
    fn midpoint(l:i64,r:i64)->i64{(l as i128+(r as i128-l as i128)/2)as i64}
    #[inline]
    fn better(x:i128,y:i128)->bool{
        if LICAOISMIN{x<y}else{x>y}
    }
    pub fn add_line(&mut self, a: i64, b: i128){
        let mut line = LiChaoLine{a,b};
        if self.nodes.is_empty(){self.push_node(line);return;}
        let (mut l, mut r) = (self.x_min, self.x_max);
        let mut idx = 0;
        loop{
            let m = Self::midpoint(l, r);
            if Self::better(line.eval(m),self.nodes[idx].line.eval(m)){
                swap(&mut line, &mut self.nodes[idx].line);
            }
            if l==r{return;}
            let x = self.nodes[idx].line;
            let dir = if Self::better(line.eval(l),x.eval(l)) {
                r = m;0
            } else if Self::better(line.eval(r),x.eval(r)){
                l = m+1;1
            } else {return;};
            let next = self.nodes[idx].child[dir];
            if next==Self::NONE{let nex = self.push_node(line);self.nodes[idx].child[dir]=nex;return;}
            idx = next;
        }
    }
    pub fn query(&self,x: i64)->Option<i128>{
        assert!(self.x_min <= x && x <= self.x_max);
        if self.nodes.is_empty(){return None;}
        let (mut l, mut r) = (self.x_min, self.x_max);
        let mut idx = 0;let mut ans = self.nodes[0].line.eval(x);
        loop {
            ans = if LICAOISMIN{ans.min(self.nodes[idx].line.eval(x))}
            else{ans.max(self.nodes[idx].line.eval(x))};
            if l==r{break;}
            let m = Self::midpoint(l, r);
            let dir = if x <= m{r=m;0}else{l=m+1;1};
            let nex = self.nodes[idx].child[dir];
            if nex==Self::NONE{break;}
            idx = nex;
        }
        Some(ans)
    }
}
