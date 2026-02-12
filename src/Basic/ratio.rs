#[derive(Copy, Clone, Debug)]
pub struct Ratio{
    x: i64,
    y: i64,
}

pub fn floor(a:i64, b:i64)->i64{let res=(a%b+b)%b;(a-res)/b}

impl Ratio {
    #[inline]
    pub fn new(mut x: i64, mut y: i64) -> Self {
        if x < 0{
            (x, y) = (-x, -y);
        }
        let g = gcd(x.abs(), y.abs()).abs();
        if g==0{
            return Ratio{x,y};
        }
        Ratio {x: x/g, y:floor(y,g)}
    }

    #[inline]
    pub fn int(x: i64)->Self{
        Ratio{x:1, y: x}
    }

    #[inline]
    pub fn inv(self)->Self{
        Ratio::new(self.y,self.x)
    }

    #[inline]
    pub fn is_inf(self)->bool{
        self.x==0
    }
}

impl Neg for Ratio{
    type Output = Ratio;

    fn neg(self) -> Self::Output {
        Ratio::new(self.x, -self.y)
    }
}

impl From<i64> for Ratio{
    fn from(value: i64) -> Self {
        Ratio::int(value)
    }
}

impl PartialEq<Self> for Ratio {
    fn eq(&self, other: &Self) -> bool {
        self.y*other.x == self.x*other.y
    }
}

impl PartialOrd<Self> for Ratio{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let x1 = self.y*other.x;
        let y1 = self.x*other.y;
        x1.partial_cmp(&y1)
    }
}

impl Eq for Ratio {}

impl Ord for Ratio {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Add for Ratio{
    type Output = Ratio;

    fn add(self, rhs: Self) -> Self::Output {
        Ratio::new(self.x*rhs.x, self.y*rhs.x+self.x*rhs.y)
    }
}

impl Add<i64> for Ratio{
    type Output = Ratio;

    fn add(self, rhs: i64) -> Self::Output {
        Ratio::new(self.x, self.y+self.x*rhs)
    }
}

impl AddAssign for Ratio{
    fn add_assign(&mut self, rhs: Self) {
        *self = *self+rhs;
    }
}

impl AddAssign<i64> for Ratio{
    fn add_assign(&mut self, rhs: i64) {
        *self = *self+rhs;
    }
}

impl Sub for Ratio{
    type Output = Ratio;
    fn sub(self, rhs: Self) -> Self::Output {
        Ratio::new(self.x*rhs.x, self.y*rhs.x-self.x*rhs.y)
    }
}

impl Sub<i64> for Ratio{
    type Output = Ratio;
    fn sub(self, rhs: i64) -> Self::Output {
        Ratio::new(self.x, self.y-self.x*rhs)
    }
}

impl SubAssign for Ratio{
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self-rhs;
    }
}

impl SubAssign<i64> for Ratio{
    fn sub_assign(&mut self, rhs: i64) {
        *self = *self-rhs;
    }
}

impl Mul for Ratio{
    type Output = Ratio;

    fn mul(self, rhs: Self) -> Self::Output {
        Ratio::new(self.x*rhs.x, self.y*rhs.y)
    }
}

impl Mul<i64> for Ratio{
    type Output = Ratio;

    fn mul(self, rhs: i64) -> Self::Output {
        Ratio::new(self.x, self.y*rhs)
    }
}

impl MulAssign for Ratio{
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self*rhs;
    }
}

impl MulAssign<i64> for Ratio{
    fn mul_assign(&mut self, rhs: i64) {
        *self = *self*rhs;
    }
}

impl Div for Ratio{
    type Output = Ratio;

    fn div(self, rhs: Self) -> Self::Output {
        self.clone()*rhs.inv()
    }
}

impl Div<i64> for Ratio{
    type Output = Ratio;

    fn div(self, rhs: i64) -> Self::Output {
        Ratio::new(self.x*rhs, self.y)
    }
}

impl DivAssign for Ratio{
    fn div_assign(&mut self, rhs: Self) {
        *self = *self/rhs;
    }
}

impl DivAssign<i64> for Ratio{
    fn div_assign(&mut self, rhs: i64) {
        *self = *self/rhs;
    }
}

impl PartialEq<i64> for Ratio{
    fn eq(&self, other: &i64) -> bool {
        *self == Ratio::int(*other)
    }
}

impl PartialOrd<i64> for Ratio{
    fn partial_cmp(&self, other: &i64) -> Option<Ordering> {
        self.partial_cmp(&Ratio::int(*other))
    }
}

impl PartialEq<Ratio> for i64{
    fn eq(&self, other: &Ratio) -> bool {
        Ratio::int(*self)==*other
    }
}

impl PartialOrd<Ratio> for i64{
    fn partial_cmp(&self, other: &Ratio) -> Option<Ordering> {
        Ratio::int(*self).partial_cmp(other)
    }
}

impl Hash for Ratio{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.y.hash(state);
        self.x.hash(state);
    }
}