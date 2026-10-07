#[path = "baseline.rs"] mod baseline;
#[path = "update_only.rs"] mod update_only;
#[path = "candidates.rs"] mod candidates;
#[path = "flat.rs"] mod flat;
#[path = "../../src/Basic/predecessor64.rs"] mod current;

trait Set: Sized {
    fn new(n: usize) -> Self;
    fn insert(&mut self, p: usize);
    fn remove(&mut self, p: usize);
    fn include(&self, p: usize) -> bool;
    fn prev(&self, p: usize) -> usize;
    fn next(&self, p: usize) -> usize;
    fn inprev(&self, p: usize) -> usize;
    fn innext(&self, p: usize) -> usize;
    fn min(&self) -> usize;
    fn max(&self) -> usize;
    fn is_empty(&self) -> bool;
}
macro_rules! impl_set {
    ($t:ty) => { impl Set for $t {
        fn new(n: usize) -> Self { <$t>::new(n) }
        #[inline(always)] fn insert(&mut self, p: usize) { self.insert(p) }
        #[inline(always)] fn remove(&mut self, p: usize) { self.remove(p) }
        #[inline(always)] fn include(&self, p: usize) -> bool { self.include(p) }
        #[inline(always)] fn prev(&self, p: usize) -> usize { self.prev(p) }
        #[inline(always)] fn next(&self, p: usize) -> usize { self.next(p) }
        #[inline(always)] fn inprev(&self, p: usize) -> usize { self.inprev(p) }
        #[inline(always)] fn innext(&self, p: usize) -> usize { self.innext(p) }
        #[inline(always)] fn min(&self) -> usize { self.min() }
        #[inline(always)] fn max(&self) -> usize { self.max() }
        #[inline(always)] fn is_empty(&self) -> bool { self.is_empty() }
    } };
}
impl_set!(baseline::Predecessor64);
impl_set!(update_only::Predecessor64);
impl_set!(candidates::Incremental);
impl_set!(candidates::LeafFirst);
impl_set!(flat::Flat);
impl_set!(candidates::Shifted);
impl_set!(flat::FlatShifted);
impl_set!(current::Predecessor64);

fn rand(seed: &mut u64) -> usize {
    *seed = seed.wrapping_add(0x9e3779b97f4a7c15);
    let mut z = *seed;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    (z ^ (z >> 31)) as usize
}
