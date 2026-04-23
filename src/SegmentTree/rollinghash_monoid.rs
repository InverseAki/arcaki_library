const RM: u64 = (1u64<<61)-1;
const RB: u64 = 1001399;
#[inline]
pub fn mod_add(a: u64, b: u64)->u64{
    let mut x = a+b;
    if x >= RM{x -= RM;}
    x
}
#[inline]
pub fn mod_sub(a:u64,b:u64)->u64{
    if a>=b{a-b}else{RM+a-b}
}
#[inline]
fn mod_mul(a:u64,b:u64)->u64{
    let t = a as u128*b as u128;
    let mut x = (t>>61)as u64+(t as u64&RM);
    x=(x>>61)+(x&RM);
    if x >= RM{x -=RM;}
    x
}
struct M;
impl Monoid for M{
    type S=(u64,u64,u64);

    fn identity() -> Self::S {
        (0,0,1)
    }

    fn binary_operation(a: &Self::S, b: &Self::S) -> Self::S {
        (mod_add(mod_mul(a.0, b.2), b.0),mod_add(a.1, mod_mul(a.2, b.1)),mod_mul(a.2, b.2))
    }
}
