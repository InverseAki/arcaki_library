pub trait Chmax: PartialOrd + Sized {
    #[inline]
    fn chmax(&mut self, rhs: Self) -> bool {
        if *self < rhs {
            *self = rhs;
            true
        } else {
            false
        }
    }
}
impl<T: PartialOrd> Chmax for T {}

pub trait Chmin: PartialOrd + Sized {
    #[inline]
    fn chmin(&mut self, rhs: Self) -> bool {
        if *self > rhs {
            *self = rhs;
            true
        } else {
            false
        }
    }
}
impl<T: PartialOrd> Chmin for T {}

#[allow(unused_macros)]
macro_rules! chmax {
    ($lhs:expr, $rhs:expr $(,)?) => {{
        let candidate = $rhs;
        let target = &mut $lhs;
        if *target < candidate {
            *target = candidate;
            true
        } else {
            false
        }
    }};
}

#[allow(unused_macros)]
macro_rules! chmin {
    ($lhs:expr, $rhs:expr $(,)?) => {{
        let candidate = $rhs;
        let target = &mut $lhs;
        if *target > candidate {
            *target = candidate;
            true
        } else {
            false
        }
    }};
}
