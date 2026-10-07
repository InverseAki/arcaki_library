#![allow(clippy::print_literal, clippy::needless_doctest_main)]
#[allow(unused)]
mod __mini_proconio {
    use std::any::type_name;
    use std::fmt::Debug;
    use std::io::{self, BufRead, BufReader, Read, Stdin};
    use std::iter::Peekable;
    use std::ptr::NonNull;
    use std::str::{FromStr, SplitWhitespace};
    use std::sync::{Mutex, OnceLock};

    pub(super) struct Tokens {
        tokens: Peekable<SplitWhitespace<'static>>,
        _ctx: CurrentContext,
    }
    impl Tokens {
        pub(super) fn next_token(&mut self) -> Option<&str> { self.tokens.next() }
        pub(super) fn is_empty(&mut self) -> bool { self.tokens.peek().is_none() }
    }
    impl From<String> for Tokens {
        fn from(s: String) -> Self {
            let ctx = CurrentContext::from(s);
            let it = unsafe { ctx.0.as_ref().split_whitespace().peekable() };
            Self { tokens: it, _ctx: ctx }
        }
    }
    unsafe impl Send for Tokens {}
    unsafe impl Sync for Tokens {}
    struct CurrentContext(NonNull<str>);
    impl From<String> for CurrentContext {
        fn from(s: String) -> Self {
            let b = s.into_boxed_str();
            Self(NonNull::new(Box::leak(b)).unwrap())
        }
    }
    impl Drop for CurrentContext {
        fn drop(&mut self) {
            unsafe { drop(Box::from_raw(self.0.as_mut())) };
        }
    }

    pub mod source {
        use super::*;
        pub mod line {
            use super::*;
            pub struct LineSource<R: BufRead> {
                tokens: Tokens,
                reader: R,
            }
            impl<R: BufRead> LineSource<R> {
                pub fn new(reader: R) -> Self {
                    Self { tokens: "".to_owned().into(), reader }
                }
                fn prepare(&mut self) {
                    while self.tokens.is_empty() {
                        let mut line = String::new();
                        let n = self.reader.read_line(&mut line)
                            .expect("failed to read line");
                        if n == 0 { return; }
                        self.tokens = line.into();
                    }
                }
            }
            impl<R: BufRead> Source<R> for LineSource<R> {
                fn next_token(&mut self) -> Option<&str> {
                    self.prepare();
                    self.tokens.next_token()
                }
                fn is_empty(&mut self) -> bool {
                    self.prepare();
                    self.tokens.is_empty()
                }
            }
            impl<'a> From<&'a str> for LineSource<BufReader<&'a [u8]>> {
                fn from(s: &'a str) -> Self {
                    Self::new(BufReader::new(s.as_bytes()))
                }
            }
        }

        pub mod once {
            use super::*;
            use std::marker::PhantomData;
            pub struct OnceSource<R: BufRead> {
                tokens: Tokens,
                _read: PhantomData<R>,
            }
            impl<R: BufRead> OnceSource<R> {
                pub fn new(mut r: R) -> Self {
                    let mut s = String::new();
                    r.read_to_string(&mut s).expect("failed to read");
                    Self { tokens: s.into(), _read: PhantomData }
                }
            }
            impl<R: BufRead> Source<R> for OnceSource<R> {
                fn next_token(&mut self) -> Option<&str> { self.tokens.next_token() }
                fn is_empty(&mut self) -> bool { self.tokens.is_empty() }
            }
            impl<'a> From<&'a str> for OnceSource<BufReader<&'a [u8]>> {
                fn from(s: &'a str) -> Self { Self::new(BufReader::new(s.as_bytes())) }
            }
        }

        pub mod auto {
            #[cfg(debug_assertions)]
            pub use super::line::LineSource as AutoSource;
            #[cfg(not(debug_assertions))]
            pub use super::once::OnceSource as AutoSource;
        }

        pub trait Source<R: BufRead> {
            fn next_token(&mut self) -> Option<&str>;
            #[allow(clippy::wrong_self_convention)]
            fn is_empty(&mut self) -> bool;
            fn next_token_unwrap(&mut self) -> &str {
                self.next_token().expect(
                    "no more tokens (input exhausted or format mismatch)")
            }
        }

        pub trait Readable {
            type Output;
            fn read<R: BufRead, S: Source<R>>(source: &mut S) -> Self::Output;
        }
        impl<T: FromStr> Readable for T where T::Err: Debug {
            type Output = T;
            fn read<R: BufRead, S: Source<R>>(source: &mut S) -> T {
                let tk = source.next_token_unwrap();
                match tk.parse() {
                    Ok(v) => v,
                    Err(e) => panic!(
                        "failed to parse `{}` as {}: {:?}",
                        tk, type_name::<T>(), e
                    ),
                }
            }
        }

        pub trait RuntimeReadable {
            type Output;
            fn read<R: BufRead, S: Source<R>>(self, source: &mut S) -> Self::Output;
        }
    }

    pub mod marker {
        use super::source::{Readable, Source};
        use std::io::BufRead;

        pub enum Chars {}
        impl Readable for Chars {
            type Output = Vec<char>;
            fn read<R: BufRead, S: Source<R>>(s: &mut S) -> Vec<char> {
                s.next_token_unwrap().chars().collect()
            }
        }

        pub enum Bytes {}
        impl Readable for Bytes {
            type Output = Vec<u8>;
            fn read<R: BufRead, S: Source<R>>(s: &mut S) -> Vec<u8> {
                s.next_token_unwrap().bytes().collect()
            }
        }

        pub enum Usize1 {}
        impl Readable for Usize1 {
            type Output = usize;
            fn read<R: BufRead, S: Source<R>>(s: &mut S) -> usize {
                usize::read(s).checked_sub(1)
                    .expect("attempted to read 0 as Usize1")
            }
        }
    }

    pub use self::source::{Readable as __Readable, RuntimeReadable};
    pub mod source_pub { pub use super::source::{self, auto::AutoSource, line::LineSource}; }

    pub enum StdinSource<R: BufRead> {
            Normal(source::auto::AutoSource<R>),
            Interactive(source::line::LineSource<R>),
            Unknown(source::line::LineSource<R>),
        }
    impl<R: BufRead> source::Source<R> for StdinSource<R> {
        fn next_token(&mut self) -> Option<&str> {
            match self {
                Self::Normal(s) => s.next_token(),
                Self::Interactive(s) | Self::Unknown(s) => s.next_token(),
            }
        }
        fn is_empty(&mut self) -> bool {
            match self {
                Self::Normal(s) => s.is_empty(),
                Self::Interactive(s) | Self::Unknown(s) => s.is_empty(),
            }
        }
    }

    pub static STDIN_SOURCE: OnceLock<Mutex<StdinSource<BufReader<Stdin>>>> = OnceLock::new();

    #[doc(hidden)]
    pub fn __acquire_global_stdin_lock() -> std::sync::MutexGuard<'static, StdinSource<BufReader<Stdin>>> {
        STDIN_SOURCE.get_or_init(|| {
            Mutex::new(StdinSource::Unknown(source::line::LineSource::new(
                BufReader::new(io::stdin()),
            )))
        }).lock().expect("failed to lock stdin")
    }

    #[macro_export]
    macro_rules! input {
        (@from [$src:expr] @rest) => {};
        (@from [$src:expr] @rest mut $($rest:tt)*) => {
            $crate::input! { @from [$src] @mut [mut] @rest $($rest)* }
        };
        (@from [$src:expr] @rest $($rest:tt)*) => {
            $crate::input! { @from [$src] @mut [] @rest $($rest)* }
        };
        (@from [$src:expr] @mut [$($m:tt)?] @rest $var:tt: $($rest:tt)*) => {
            $crate::input! { @from [$src] @mut [$($m)*] @var $var @kind [] @rest $($rest)* }
        };
        (@from [$src:expr] @mut [$($m:tt)?] @var $var:tt @kind [$($kind:tt)*] @rest) => {
            let $($m)* $var = $crate::read_value!(@source [$src] @kind [$($kind)*]);
        };
        (@from [$src:expr] @mut [$($m:tt)?] @var $var:tt @kind [$($kind:tt)*] @rest, $($rest:tt)*) => {
            $crate::input!(@from [$src] @mut [$($m)*] @var $var @kind [$($kind)*] @rest);
            $crate::input!(@from [$src] @rest $($rest)*);
        };
        (@from [$src:expr] @mut [$($m:tt)?] @var $var:tt @kind [] @rest [$($tt:tt)*] $($rest:tt)*) => {
            $crate::input!(@from [$src] @mut [$($m)*] @var $var @kind [[$($tt)*]] @rest $($rest)*);
        };
        (@from [$src:expr] @mut [$($m:tt)?] @var $var:tt @kind [] @rest ($($tt:tt)*) $($rest:tt)*) => {
            $crate::input!(@from [$src] @mut [$($m)*] @var $var @kind [($($tt)*)] @rest $($rest)*);
        };
        (@from [$src:expr] @mut [$($m:tt)?] @var $var:tt @kind [] @rest $ty:ty, $($rest:tt)*) => {
            $crate::input!(@from [$src] @mut [$($m)*] @var $var @kind [$ty] @rest, $($rest)*);
        };
        (@from [$src:expr] @mut [$($m:tt)?] @var $var:tt @kind [] @rest $ty:ty) => {
            $crate::input!(@from [$src] @mut [$($m)*] @var $var @kind [$ty] @rest);
        };
        (@from [$src:expr] @mut [$($m:tt)?] @var $var:tt @kind [] @rest with $($rest:tt)*) => {
            let $($m)* $var = $crate::read_value!(@source [$src] @dyn_kind [$($rest)*]);
        };
        (@from $($tt:tt)*) => {
            compile_error!("input! parse error (mini proconio)");
        };

        (from $src:expr, $($rest:tt)*) => {
            #[allow(unused_variables, unused_mut)]
            let mut __s = $src;
            $crate::input! { @from [&mut __s] @rest $($rest)* }
        };
        ($($rest:tt)*) => {
            let mut __lock = $crate::__mini_proconio::__acquire_global_stdin_lock();
            $crate::input! { @from [&mut *__lock] @rest $($rest)* }
            drop(__lock);
        };
    }

    #[macro_export]
    macro_rules! read_value {
        (@source [$src:expr] @kind [[$($kind:tt)*]]) => {
            $crate::read_value!(@array @source [$src] @kind [] @rest $($kind)*)
        };
        (@array @source [$src:expr] @kind [$($kind:tt)*] @rest) => {{
            let len = <usize as $crate::__mini_proconio::__Readable>::read($src);
            $crate::read_value!(@source [$src] @kind [[$($kind)*; len]])
        }};
        (@array @source [$src:expr] @kind [$($kind:tt)*] @rest ; $($rest:tt)*) => {
            $crate::read_value!(@array @source [$src] @kind [$($kind)*] @len [$($rest)*])
        };
        (@array @source [$src:expr] @kind [$($kind:tt)*] @rest $tt:tt $($rest:tt)*) => {
            $crate::read_value!(@array @source [$src] @kind [$($kind)* $tt] @rest $($rest)*)
        };
        (@array @source [$src:expr] @kind [$($kind:tt)*] @len [$($len:tt)*]) => {{
            let len = $($len)*;
            (0..len).map(|_| $crate::read_value!(@source [$src] @kind [$($kind)*])).collect::<Vec<_>>()
        }};

        (@source [$src:expr] @kind [($($kinds:tt)*)]) => {
            $crate::read_value!(@tuple @source [$src] @kinds [] @current [] @rest $($kinds)*)
        };
        (@tuple @source [$src:expr] @kinds [$([$($kind:tt)*])*] @current [] @rest) => {
            ( $($crate::read_value!(@source [$src] @kind [$($kind)*]),)* )
        };
        (@tuple @source [$src:expr] @kinds [$($kinds:tt)*] @current [$($curr:tt)*] @rest) => {
            $crate::read_value!(@tuple @source [$src] @kinds [$($kinds)* [$($curr)*]] @current [] @rest)
        };
        (@tuple @source [$src:expr] @kinds [$($kinds:tt)*] @current [$($curr:tt)*] @rest, $($rest:tt)*) => {
            $crate::read_value!(@tuple @source [$src] @kinds [$($kinds)* [$($curr)*]] @current [] @rest $($rest)*)
        };
        (@tuple @source [$src:expr] @kinds [$($kinds:tt)*] @current [$($curr:tt)*] @rest $tt:tt $($rest:tt)*) => {
            $crate::read_value!(@tuple @source [$src] @kinds [$($kinds)*] @current [$($curr)* $tt] @rest $($rest)*)
        };

        (@source [$src:expr] @kind [$ty:ty]) => {
            <$ty as $crate::__mini_proconio::__Readable>::read($src)
        };

        (@source [$src:expr] @dyn_kind [$dyn:expr]) => {
            $crate::__mini_proconio::RuntimeReadable::read($dyn, $src)
        };

        (from $src:expr, with $($rest:tt)*) => {{
            #[allow(unused_mut)]
            let mut __s = $src;
            $crate::read_value!(@source [&mut __s] @dyn_kind [$($rest)*])
        }};
        (from $src:expr, $($rest:tt)*) => {{
            #[allow(unused_mut)]
            let mut __s = $src;
            $crate::read_value!(@source [&mut __s] @kind [$($rest)*])
        }};
        ($($rest:tt)*) => {{
            let mut __lock = $crate::__mini_proconio::STDIN_SOURCE
                .get_or_init(|| {
                    std::sync::Mutex::new($crate::__mini_proconio::StdinSource::Normal(
                        $crate::__mini_proconio::source_pub::auto::AutoSource::new(
                            std::io::BufReader::new(std::io::stdin())
                        ),
                    ))
                })
                .lock()
                .expect("failed to lock stdin");
            let __res = $crate::read_value!(from &mut *__lock, $($rest)*);
            drop(__lock);
            __res
        }};
    }

    #[macro_export]
    macro_rules! input_interactive {
        ($($rest:tt)*) => {
            let mut __lock = $crate::__mini_proconio::__acquire_global_stdin_lock();
            $crate::input! { from &mut *__lock, $($rest)* }
            drop(__lock);
        };
    }
    #[macro_export]
    macro_rules! read_value_interactive {
        ($($rest:tt)*) => {{
            let mut __lock = $crate::__mini_proconio::__acquire_global_stdin_lock();
            let __res = $crate::read_value!(from &mut *__lock, $($rest)*);
            drop(__lock);
            __res
        }};
    }
}
#[allow(unused_imports)]
use __mini_proconio as proconio_;
#[allow(unused_imports)]
use proconio_::marker::*;

pub type ModInt1000000007 = StaticModInt<Mod1000000007>;
pub type ModInt998244353 = StaticModInt<Mod998244353>;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct StaticModInt<M> {
    val: u32,
    phantom: PhantomData<fn() -> M>,
}

pub trait Modulus: 'static + Copy + Eq {
    const VALUE: u32;
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum Mod1000000007 {}
impl Modulus for Mod1000000007 {
    const VALUE: u32 = 1_000_000_007;
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum Mod998244353 {}
impl Modulus for Mod998244353 {
    const VALUE: u32 = 998_244_353;
}

impl<M: Modulus> StaticModInt<M> {
    #[inline(always)]
    pub fn modulus() -> u32 {
        M::VALUE
    }

    #[inline]
    pub fn new<T: Into<i128>>(v: T) -> Self {
        let m = M::VALUE as i128;
        let mut x = v.into() % m;
        if x < 0 { x += m; }
        Self::raw(x as u32)
    }

    #[inline]
    pub fn raw(val: u32) -> Self {
        Self { val, phantom: PhantomData }
    }

    #[inline]
    pub fn val(self) -> u32 {
        self.val
    }

    #[inline]
    pub fn pow(self, mut n: u64) -> Self {
        let mut x = self;
        let mut r = if Self::modulus() > 1 { Self::raw(1) } else { Self::raw(0) };
        while n > 0 {
            if n & 1 == 1 { r *= x; }
            x *= x;
            n >>= 1;
        }
        r
    }

    #[inline]
    pub fn inv(self) -> Self {
        assert!(self.val() != 0, "attempt to divide by zero");
        self.pow((M::VALUE as u64) - 2)
    }
}
 
impl<M: Modulus> Neg for StaticModInt<M> {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        if self.val == 0 { self } else { Self::raw(M::VALUE - self.val) }
    }
}
 
impl<M: Modulus> Add for StaticModInt<M> {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        let mut v = self.val + rhs.val;
        let m = M::VALUE;
        if v >= m { v -= m; }
        Self::raw(v)
    }
}
 
impl<M: Modulus> Sub for StaticModInt<M> {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        let m = M::VALUE;
        let mut v = self.val.wrapping_sub(rhs.val);
        if v >= m { v = v.wrapping_add(m); }
        Self::raw(v)
    }
}
 
impl<M: Modulus> Mul for StaticModInt<M> {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        let m = M::VALUE as u64;
        let v = (self.val as u64) * (rhs.val as u64) % m;
        Self::raw(v as u32)
    }
}
 
impl<M: Modulus> Div for StaticModInt<M> {
    type Output = Self;
    #[inline]
    fn div(self, rhs: Self) -> Self {
        self * rhs.inv()
    }
}
 
impl<M: Modulus> AddAssign for StaticModInt<M> {
    #[inline]
    fn add_assign(&mut self, rhs: Self) { *self = *self + rhs; }
}
impl<M: Modulus> SubAssign for StaticModInt<M> {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) { *self = *self - rhs; }
}
impl<M: Modulus> MulAssign for StaticModInt<M> {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) { *self = *self * rhs; }
}
impl<M: Modulus> DivAssign for StaticModInt<M> {
    #[inline]
    fn div_assign(&mut self, rhs: Self) { *self = *self / rhs; }
}
impl<M: Modulus> From<i64> for StaticModInt<M> {
    #[inline]
    fn from(v: i64) -> Self { Self::new(v) }
}
impl<M: Modulus> From<u64> for StaticModInt<M> {
    #[inline]
    fn from(v: u64) -> Self { Self::new(v) }
}
impl<M: Modulus> From<usize> for StaticModInt<M> {
    #[inline]
    fn from(v: usize) -> Self { Self::new(v as u64) }
}
 
impl<M: Modulus> FromStr for StaticModInt<M> {
    type Err = <i128 as FromStr>::Err;
 
    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v = s.parse::<i128>()?;
        Ok(Self::new(v))
    }
}
 
macro_rules! impl_ops_with_prim {
    ($($t:ty),* $(,)?) => {
        $(
            impl<M: Modulus> Add<$t> for StaticModInt<M> {
                type Output = Self;
                #[inline] fn add(self, rhs: $t) -> Self { self + Self::new(rhs as i128) }
            }
            impl<M: Modulus> Sub<$t> for StaticModInt<M> {
                type Output = Self;
                #[inline] fn sub(self, rhs: $t) -> Self { self - Self::new(rhs as i128) }
            }
            impl<M: Modulus> Mul<$t> for StaticModInt<M> {
                type Output = Self;
                #[inline] fn mul(self, rhs: $t) -> Self { self * Self::new(rhs as i128) }
            }
            impl<M: Modulus> Div<$t> for StaticModInt<M> {
                type Output = Self;
                #[inline] fn div(self, rhs: $t) -> Self { self / Self::new(rhs as i128) }
            }
 
            impl<M: Modulus> AddAssign<$t> for StaticModInt<M> {
                #[inline] fn add_assign(&mut self, rhs: $t) { *self = *self + rhs; }
            }
            impl<M: Modulus> SubAssign<$t> for StaticModInt<M> {
                #[inline] fn sub_assign(&mut self, rhs: $t) { *self = *self - rhs; }
            }
            impl<M: Modulus> MulAssign<$t> for StaticModInt<M> {
                #[inline] fn mul_assign(&mut self, rhs: $t) { *self = *self * rhs; }
            }
            impl<M: Modulus> DivAssign<$t> for StaticModInt<M> {
                #[inline] fn div_assign(&mut self, rhs: $t) { *self = *self / rhs; }
            }
        )*
    }
}

impl_ops_with_prim!(i32, i64, u32, usize);

#[allow(unused_imports)]
use std::{
    convert::{Infallible, TryFrom, TryInto as _}, fmt::{self, Debug, Display, Formatter,},
    fs::File, hash::{Hash, Hasher, BuildHasherDefault}, iter::{Product, Sum}, marker::PhantomData,
    ops::{Add, AddAssign, Sub, SubAssign, Div, DivAssign, Mul, MulAssign, 
        Neg, RangeBounds, BitAnd, BitAndAssign, BitOr, BitXor, BitXorAssign, BitOrAssign, Index, IndexMut},
    str::FromStr, sync::{atomic::{self, AtomicU32, AtomicU64}, Once},
    collections::{*, btree_set::Range, btree_map::Range as BTreeRange}, mem::{take, swap},
    cmp::{self, Reverse, Ordering, Eq, PartialEq, PartialOrd},
    thread::LocalKey, f64::consts::PI, time::Instant, cell::RefCell,
    io::{self, stdin, Read, read_to_string, BufWriter, BufReader, stdout, Write},
    ptr::null_mut, println, print,debug_assert,debug_assert_eq,debug_assert_ne,
    matches,writeln
};
pub trait SortD{ fn sort_d(&mut self); }
impl<T: Ord> SortD for Vec<T>{ fn sort_d(&mut self) {
    self.sort_by(|u, v| v.cmp(&u));
} }
pub trait Mx{fn max(&self, rhs: Self)->Self;}
impl Mx for f64{ fn max(&self, rhs: Self)->Self{if *self < rhs{ rhs } else { *self } }}
pub trait Mi{ fn min(&self, rhs: Self)->Self; }
impl Mi for f64{ fn min(&self, rhs: Self)->Self{ if *self > rhs{ rhs } else { *self } } }
#[inline]
pub fn gcd(mut a: i64, mut b: i64)->i64{ if b==0{return a;}(a,b)=(a.abs(),b.abs());while b!=0{ let c = a;a = b;b = c%b; }a }
#[inline]
pub fn factorial_i64(n: usize)->(Vec<i64>, Vec<i64>){ let mut res = vec![1; n+1];let mut inv = vec![1; n+1];for i in 0..n{ res[i+1] = (res[i]*(i+1)as i64)%MOD; }inv[n] = mod_inverse(res[n], MOD);for i in (0..n).rev(){ inv[i] = inv[i+1]*(i+1) as i64%MOD; }(res, inv) }
#[inline]
pub fn floor(a:i64, b:i64)->i64{let res=(a%b+b)%b;(a-res)/b}
#[inline]
pub fn extended_gcd(a:i64,b:i64)->(i64,i64,i64)
{if b==0{(a,1,0)}else{let(g,x,y)=extended_gcd(b,a%b);(g,y,x-floor(a,b)*y)}}
#[inline]
pub fn mod_inverse(a:i64,m:i64)->i64{let(_,x,_) =extended_gcd(a,m);(x%m+m)%m}
#[inline]
pub fn comb(a: i64, b: i64, f: &Vec<(i64, i64)>)->i64{
    if a<b{return 0;}else if b==0 || a==b{ return 1; }
    else{let x=f[a as usize].0;
        let y=f[(a-b) as usize].1;let z=f[b as usize].1;return((x*y)%MOD)*z%MOD;}}
#[inline]
pub fn factorial(x: i64)->Vec<(i64, i64)>{
    let mut f=vec![(1i64,1i64),(1, 1)];let mut z = 1i64;
    let mut inv = vec![0; x as usize+10];inv[1] = 1;
    for i in 2..x+1{z=(z*i)%MOD;
        let w=(MOD-inv[(MOD%i)as usize]*(MOD/i)%MOD)%MOD;
        inv[i as usize] = w;
        f.push((z, (f[i as usize-1].1*w)%MOD));}return f;}
pub fn vec_out<T>(v: Vec<T>) where T: ToString{ println!("{}", v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(" ")); }
pub fn one_add_out(v: Vec<usize>){println!("{}", v.iter().map(|x| (x+1).to_string()).collect::<Vec<_>>().join(" "));}
pub trait Chmax{
    fn chmax(&mut self, rhs: Self);
}
impl Chmax for i64{
    fn chmax(&mut self, rhs: Self) {
        *self = (*self).max(rhs)
    }
}
impl Chmax for i32{
    fn chmax(&mut self, rhs: Self) {
        *self = (*self).max(rhs)
    }
}
impl Chmax for f64{
    fn chmax(&mut self, rhs: Self) {
        *self = (*self).max(rhs)
    }
}
impl Chmax for usize{
    fn chmax(&mut self, rhs: Self) {
        *self = (*self).max(rhs)
    }
}
pub trait Chmin {
    fn chmin(&mut self, rhs: Self);
}
impl Chmin for i64{
    fn chmin(&mut self, rhs: Self) {
        *self = (*self).min(rhs)
    }
}
impl Chmin for i32{
    fn chmin(&mut self, rhs: Self) {
        *self = (*self).min(rhs)
    }
}
impl Chmin for f64{
    fn chmin(&mut self, rhs: Self) {
        *self = (*self).min(rhs)
    }
}
impl Chmin for usize{
    fn chmin(&mut self, rhs: Self) {
        *self = (*self).min(rhs)
    }
}
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

#[derive(Debug, Clone)]
pub struct Counter<T: Ord>{
    c: usize,
    map: BTreeMap<T, usize>,
}

impl<T: Copy+Ord> Counter<T>{
    pub fn new()->Self{
        Counter{
            c: 0,
            map: BTreeMap::new(),
        }
    }

    #[inline(always)]
    pub fn range<R>(&self, range: R)->BTreeRange<'_, T, usize> where R: RangeBounds<T>{
        self.map.range(range)
    }

    #[inline(always)]
    pub fn mi(&self)->Option<T>{
        if let Some((x, _)) = self.range(..).next(){
            Some(*x)
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn mx(&self)->Option<T>{
        if let Some((x, _)) = self.range(..).next_back(){
            Some(*x)
        } else {
            None
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
    pub fn sub(&mut self, x: T, c: usize){
        let e = self.map.entry(x).or_insert(0);
        *e = e.saturating_sub(c);
        if self.map[&x] == 0{
            self.map.remove(&x);
        }
        self.c = self.c.saturating_sub(c);
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
    pub fn merge(&mut self, rhs: &mut Counter<T>){
        if self.len() < rhs.len(){
            swap(self, rhs);
        }
        for (&k, &v) in rhs.map.iter(){
            self.add(k, v);
        }
        rhs.clear();
    }
}

pub mod fxhash {
    use std::hash::BuildHasherDefault;

    const K: u64 = 0x517c_c1b7_2722_0a95;

    #[derive(Default)]
    pub struct FxHasher {
        pub hash: u64,
    }
    impl FxHasher {
        #[inline(always)]
        fn mix_u64(mut h: u64, x: u64) -> u64 {
            h = h.rotate_left(5) ^ x;
            h = h.wrapping_mul(K);

            let x2 = x ^ (x >> 33) ^ (x << 11);
            h = h.rotate_left(5) ^ x2;
            h = h.wrapping_mul(K);

            h
        }
        #[inline(always)]
        fn write_u64_impl(&mut self, x: u64) {
            self.hash = Self::mix_u64(self.hash, x);
        }
    }
    impl std::hash::Hasher for FxHasher {
        #[inline(always)]
        fn finish(&self) -> u64 {
            self.hash
        }
        #[inline(always)]
        fn write(&mut self, bytes: &[u8]) {
            let mut h = self.hash;
            for &b in bytes {
                h = h.rotate_left(5) ^ (b as u64);
                h = h.wrapping_mul(K);
            }
            self.hash = h;
        }
        #[inline(always)]
        fn write_u64(&mut self, i: u64) { self.write_u64_impl(i); }
        #[inline(always)]
        fn write_u32(&mut self, i: u32) { self.write_u64_impl(i as u64); }
        #[inline(always)]
        fn write_u16(&mut self, i: u16) { self.write_u64_impl(i as u64); }
        #[inline(always)]
        fn write_u8 (&mut self, i: u8 ) { self.write_u64_impl(i as u64); }
        #[inline(always)]
        fn write_usize(&mut self, i: usize) { self.write_u64_impl(i as u64); }
        #[inline(always)]
        fn write_i64(&mut self, i: i64) { self.write_u64_impl(i as u64); }
        #[inline(always)]
        fn write_i32(&mut self, i: i32) { self.write_u64_impl(i as u64); }
        #[inline(always)]
        fn write_i16(&mut self, i: i16) { self.write_u64_impl(i as u64); }
        #[inline(always)]
        fn write_i8 (&mut self, i: i8 ) { self.write_u64_impl(i as u64); }
        #[inline(always)]
        fn write_isize(&mut self, i: isize) { self.write_u64_impl(i as u64); }
    }

    pub type FxBuildHasher = BuildHasherDefault<FxHasher>;
    pub type FxMap<K, V> = std::collections::HashMap<K, V, FxBuildHasher>;
    pub type FxSet<K> = std::collections::HashSet<K, FxBuildHasher>;
}
#[allow(unused_imports)]
use fxhash::{FxSet, FxMap, FxBuildHasher};
#[allow(dead_code)]
const INF: i64 = 1<<60;
#[allow(dead_code)]
const I: i32 = 1<<30;
#[allow(dead_code)]
const MOD: i64 = 998244353;
#[allow(dead_code)]
const D: [(usize, usize); 4] = [(1, 0), (0, 1), (!0, 0), (0, !0)];
#[allow(dead_code)]
const D2: [(usize, usize); 8] = [(1, 0), (1, 1), (0, 1), (!0, 1), (!0, 0), (!0, !0), (0, !0), (1, !0)];
#[allow(dead_code)]
pub fn join2str<T: ToString>(v: &[T])->String{v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(" ")}
#[allow(dead_code)]
pub fn join2nospace<T: ToString>(v: &[T])->String{v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join("")}

pub struct Input {
    buf: Vec<u8>,
    pos: usize,
}

impl Input {
    #[inline]
    pub fn new() -> Self {
        let mut buf = Vec::new();
        io::stdin().read_to_end(&mut buf).unwrap();
        buf.push(0);
        Self { buf, pos: 0 }
    }

    #[inline(always)]
    fn skip_whitespace(&mut self) {
        while self.buf[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    #[inline(always)]
    fn token_range(&mut self) -> (usize, usize) {
        self.skip_whitespace();

        let l = self.pos;

        while self.buf[self.pos] > b' ' {
            self.pos += 1;
        }

        (l, self.pos)
    }

    #[inline(always)]
    pub fn byte(&mut self) -> u8 {
        self.skip_whitespace();
        let res = self.buf[self.pos];
        while self.buf[self.pos] > b' ' {
            self.pos += 1;
        }
        res
    }

    #[inline(always)]
    pub fn bytes(&mut self) -> &[u8] {
        let (l, r) = self.token_range();
        &self.buf[l..r]
    }

    #[inline(always)]
    pub fn string(&mut self) -> String {
        let (l, r) = self.token_range();

        unsafe {
            String::from_utf8_unchecked(self.buf[l..r].to_vec())
        }
    }

    #[inline(always)]
    pub fn char(&mut self) -> char {
        let (l, r) = self.token_range();

        unsafe {
            std::str::from_utf8_unchecked(&self.buf[l..r])
                .chars()
                .next()
                .unwrap()
        }
    }

    #[inline(always)]
    pub fn chars(&mut self) -> Vec<char> {
        let (l, r) = self.token_range();

        unsafe {
            std::str::from_utf8_unchecked(&self.buf[l..r])
                .chars()
                .collect()
        }
    }

    #[inline]
    pub fn f64(&mut self) -> f64 {
        let (l, r) = self.token_range();

        unsafe {
            std::str::from_utf8_unchecked(&self.buf[l..r])
                .parse()
                .unwrap()
        }
    }

    #[inline(always)]
    pub fn usize1(&mut self) -> usize {
        self.usize() - 1
    }

    #[inline]
    pub fn vec_byte(&mut self, n: usize) -> Vec<u8> {
        let mut res = Vec::with_capacity(n);

        for _ in 0..n {
            res.push(self.byte());
        }

        res
    }

    #[inline]
    pub fn vec_bytes(&mut self, n: usize) -> Vec<Vec<u8>> {
        let mut res = Vec::with_capacity(n);

        for _ in 0..n {
            res.push(self.bytes().to_vec());
        }
        res
    }

    #[inline]
    pub fn vec_string(&mut self, n: usize) -> Vec<String> {
        let mut res = Vec::with_capacity(n);

        for _ in 0..n {
            res.push(self.string());
        }

        res
    }

    #[inline]
    pub fn vec_char(&mut self, n: usize) -> Vec<char> {
        let mut res = Vec::with_capacity(n);

        for _ in 0..n {
            res.push(self.char());
        }

        res
    }

    #[inline]
    pub fn vec_chars(&mut self, n: usize) -> Vec<Vec<char>> {
        let mut res = Vec::with_capacity(n);

        for _ in 0..n {
            res.push(self.chars());
        }

        res
    }

    #[inline]
    pub fn byte_grid(&mut self, n: usize) -> Vec<Vec<u8>> {
        let mut res = Vec::with_capacity(n);

        for _ in 0..n {
            res.push(self.bytes().to_vec());
        }

        res
    }

    #[inline]
    pub fn vec_f64(&mut self, n: usize) -> Vec<f64> {
        let mut res = Vec::with_capacity(n);

        for _ in 0..n {
            res.push(self.f64());
        }

        res
    }

    #[inline]
    pub fn vec_usize1(&mut self, n: usize) -> Vec<usize> {
        let mut res = Vec::with_capacity(n);

        for _ in 0..n {
            res.push(self.usize1());
        }

        res
    }
}

macro_rules! impl_unsigned_input {
    ($($name:ident, $vec_name:ident, $t:ty);* $(;)?) => {
        impl Input {
            $(
                #[inline(always)]
                pub fn $name(&mut self) -> $t {
                    let buf = &self.buf;
                    let mut i = self.pos;

                    while buf[i].is_ascii_whitespace() {
                        i += 1;
                    }

                    let mut res: $t = 0;

                    while buf[i].is_ascii_digit() {
                        res = res * 10 + (buf[i] - b'0') as $t;
                        i += 1;
                    }

                    self.pos = i;
                    res
                }

                #[inline]
                pub fn $vec_name(&mut self, n: usize) -> Vec<$t> {
                    let mut res = Vec::with_capacity(n);

                    for _ in 0..n {
                        res.push(self.$name());
                    }

                    res
                }
            )*
        }
    };
}

macro_rules! impl_signed_input {
    ($($name:ident, $vec_name:ident, $t:ty);* $(;)?) => {
        impl Input {
            $(
                #[inline(always)]
                pub fn $name(&mut self) -> $t {
                    let buf = &self.buf;
                    let mut i = self.pos;
                    while buf[i].is_ascii_whitespace() {
                        i += 1;
                    }
                    let neg = buf[i] == b'-';
                    if neg {
                        i += 1;
                    }
                    let mut res: $t = 0;
                    if neg {
                        while buf[i].is_ascii_digit() {
                            res = res * 10 - (buf[i] - b'0') as $t;
                            i += 1;
                        }
                    } else {
                        while buf[i].is_ascii_digit() {
                            res = res * 10 + (buf[i] - b'0') as $t;
                            i += 1;
                        }
                    }

                    self.pos = i;
                    res
                }

                #[inline]
                pub fn $vec_name(&mut self, n: usize) -> Vec<$t> {
                    let mut res = Vec::with_capacity(n);

                    for _ in 0..n {
                        res.push(self.$name());
                    }

                    res
                }
            )*
        }
    };
}

impl_unsigned_input! {
    u8,    vec_u8,    u8;
    u16,   vec_u16,   u16;
    u32,   vec_u32,   u32;
    u64,   vec_u64,   u64;
    u128,  vec_u128,  u128;
    usize, vec_usize, usize;
}

impl_signed_input! {
    i8,   vec_i8,   i8;
    i16,  vec_i16,  i16;
    i32,  vec_i32,  i32;
    i64,  vec_i64,  i64;
    i128, vec_i128, i128;
}

#[derive(Clone, Debug)]
pub struct Predecessor64 {
    mx: usize,
    data: Vec<u64>,
    offset: [usize; 11],
    levels: usize,
}

impl Predecessor64 {
    pub fn new(mx: usize) -> Self {
        let (offset, levels, total) = Self::layout(mx.div_ceil(64));
        Self {
            mx,
            data: vec![0; total],
            offset,
            levels,
        }
    }

    fn layout(mut len: usize) -> ([usize; 11], usize, usize) {
        let mut offset = [0; 11];
        let mut levels = 0;
        let mut total = 0;
        loop {
            offset[levels] = total;
            levels += 1;
            total += len;
            if len <= 1 {
                break;
            }
            len = len.div_ceil(64);
        }
        (offset, levels, total)
    }

    pub fn from_vec_u64(mut base: Vec<u64>) -> Self {
        if base.is_empty() {
            return Self::new(1);
        }
        let mx = base.len() << 6;
        let (offset, levels, total) = Self::layout(base.len());
        base.reserve_exact(total - base.len());
        base.resize(total, 0);
        for i in 1..levels {
            let (lower, upper) = base.split_at_mut(offset[i]);
            let lower = &lower[offset[i - 1]..];
            for (chunk, word) in lower.chunks(64).zip(upper.iter_mut()) {
                let mut bits = 0;
                for (j, &v) in chunk.iter().enumerate() {
                    if v != 0 {
                        bits |= 1 << j;
                    }
                }
                *word = bits;
            }
        }
        Self {
            mx,
            data: base,
            offset,
            levels,
        }
    }

    pub fn from_vec_usize(base: Vec<usize>) -> Self {
        Self::from_vec_u64(base.into_iter().map(|v| v as u64).collect())
    }

    pub fn from_01_string(s: String) -> Self {
        let mut base = vec![0; s.len().div_ceil(64)];
        for (i, c) in s.bytes().enumerate() {
            if c == b'1' {
                base[i >> 6] |= 1 << (i & 63);
            }
        }
        Self::from_vec_u64(base)
    }

    #[inline(always)]
    pub fn insert(&mut self, mut p: usize) {
        for i in 0..self.levels {
            let word = &mut self.data[self.offset[i] + (p >> 6)];
            let old = *word;
            *word |= 1 << (p & 63);
            if old != 0 {
                return;
            }
            p >>= 6;
        }
    }

    #[inline(always)]
    pub fn remove(&mut self, mut p: usize) {
        for i in 0..self.levels {
            let word = &mut self.data[self.offset[i] + (p >> 6)];
            let old = *word;
            *word &= !(1 << (p & 63));
            if *word != 0 || old == 0 {
                return;
            }
            p >>= 6;
        }
    }

    #[inline(always)]
    pub fn include(&self, p: usize) -> bool {
        self.data[p >> 6] & (1 << (p & 63)) != 0
    }
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.data[self.offset[self.levels - 1]] == 0
    }
    #[inline(always)]
    pub fn min(&self) -> usize {
        self.extreme::<true>()
    }
    #[inline(always)]
    pub fn max(&self) -> usize {
        self.extreme::<false>()
    }
    #[inline(always)]
    fn extreme<const MIN: bool>(&self) -> usize {
        let leaf_index = if MIN { 0 } else { (self.mx - 1) >> 6 };
        let leaf = self.data[leaf_index];
        if leaf != 0 {
            let bit = if MIN {
                leaf.trailing_zeros() as usize
            } else {
                63 - leaf.leading_zeros() as usize
            };
            return (leaf_index << 6) | bit;
        }
        let mut word = self.data[self.offset[self.levels - 1]];
        if word == 0 {
            return !0;
        }
        let mut p = if MIN {
            word.trailing_zeros() as usize
        } else {
            63 - word.leading_zeros() as usize
        };
        for i in (0..self.levels - 1).rev() {
            word = self.data[self.offset[i] + p];
            let bit = if MIN {
                word.trailing_zeros() as usize
            } else {
                63 - word.leading_zeros() as usize
            };
            p = (p << 6) | bit;
        }
        p
    }
    #[inline(always)]
    pub fn prev(&self, p: usize) -> usize {
        self.search_prev::<false>(p)
    }
    #[inline(always)]
    pub fn inprev(&self, p: usize) -> usize {
        self.search_prev::<true>(p)
    }
    #[inline(always)]
    pub fn next(&self, p: usize) -> usize {
        self.search_next::<false>(p)
    }
    #[inline(always)]
    pub fn innext(&self, p: usize) -> usize {
        self.search_next::<true>(p)
    }

    #[inline(always)]
    fn search_prev<const INCLUSIVE: bool>(&self, mut p: usize) -> usize {
        for i in 0..self.levels {
            let mask = if i == 0 && INCLUSIVE {
                u64::MAX >> (63 - (p & 63))
            } else {
                (1u64 << (p & 63)) - 1
            };
            let word = self.data[self.offset[i] + (p >> 6)] & mask;
            if word != 0 {
                let mut res = (p & !63) | (63 - word.leading_zeros() as usize);
                for j in (0..i).rev() {
                    res = (res << 6)
                        | (63 - self.data[self.offset[j] + res].leading_zeros() as usize);
                }
                return res;
            }
            p >>= 6;
        }
        !0
    }

    #[inline(always)]
    fn search_next<const INCLUSIVE: bool>(&self, mut p: usize) -> usize {
        for i in 0..self.levels {
            let mask = if i == 0 && INCLUSIVE {
                u64::MAX << (p & 63)
            } else {
                (u64::MAX << (p & 63)) << 1
            };
            let word = self.data[self.offset[i] + (p >> 6)] & mask;
            if word != 0 {
                let mut res = (p & !63) | word.trailing_zeros() as usize;
                for j in (0..i).rev() {
                    res = (res << 6) | self.data[self.offset[j] + res].trailing_zeros() as usize;
                }
                return res;
            }
            p >>= 6;
        }
        !0
    }
}

#[allow(dead_code)]
type MI = ModInt998244353;
const MULTI: bool = false;

#[inline(always)]
fn append_answer(out: &mut Vec<u8>, answer: usize) {
    if answer == !0 { out.extend_from_slice(b"-1\n"); return; }
    let mut digits = [0u8; 16];
    let mut at = digits.len() - 1;
    digits[at] = b'\n';
    let mut value = answer;
    loop {
        at -= 1;
        digits[at] = b'0' + (value % 10) as u8;
        value /= 10;
        if value == 0 { break; }
    }
    out.extend_from_slice(&digits[at..]);
}

fn pack_binary(bytes: &[u8]) -> Vec<u64> {
    bytes.chunks(64).map(|chunk| {
        let mut word = 0;
        for (group, part) in chunk.chunks(8).enumerate() {
            let packed = if part.len() == 8 {
                let bits = u64::from_le_bytes(part.try_into().unwrap()) & 0x0101_0101_0101_0101;
                (bits.wrapping_mul(0x0102_0408_1020_4080) >> 56) as u64
            } else {
                part.iter().enumerate().fold(0u64, |w, (i, &c)| w | (((c - b'0') as u64) << i))
            };
            word |= packed << (group * 8);
        }
        word
    }).collect()
}
pub fn solve() {
    let mut o = BufWriter::new(stdout().lock());
    let start = std::time::Instant::now();
    let mut ip = Input::new();
    let read_time = start.elapsed();
    let build_start = std::time::Instant::now();
    let n = ip.usize();
    let q = ip.usize();
    let mut set = Predecessor64::from_vec_u64(pack_binary(ip.bytes()));
    let build_time = build_start.elapsed();
    let query_start = std::time::Instant::now();
    let mut res = Vec::with_capacity(q * 8);
    for _ in 0..q{
        let t = ip.u8();
        let k = ip.usize();
        if t==0 {set.insert(k);}
        else if t==1 {set.remove(k);}
        else if t==2 {
            if set.include(k){res.push(b'1')}else{res.push(b'0');}
            res.push(b'\n');
        } else if t==3 {
            append_answer(&mut res, set.innext(k));
        } else {
            append_answer(&mut res, set.inprev(k));
        }
    }
    let query_time = query_start.elapsed();
    let write_start = std::time::Instant::now();
    o.write_all(&res).unwrap();
    o.flush().unwrap();
    eprintln!("{:.6},{:.6},{:.6},{:.6}", read_time.as_secs_f64()*1000., build_time.as_secs_f64()*1000., query_time.as_secs_f64()*1000., write_start.elapsed().as_secs_f64()*1000.);
}

fn main() {
    if MULTI{
        input!{
            t: usize,
        }
        for _ in 0..t{
            solve();
        }
    } else {
        solve();
    }
}