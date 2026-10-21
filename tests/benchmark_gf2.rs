use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign};
include!("../src/DataStructure/bitset.rs");
#[allow(dead_code)]
mod old {
    include!("support/bit_matrix_gf2_baseline.rs");
}
use std::hint::black_box;
use std::time::Instant;
fn make(h: usize, w: usize, state: &mut u64, sparse: bool) -> BitMatrix {
    BitMatrix::from_rows(w, (0..h).map(|_| {
        let data = (0..w.div_ceil(64)).map(|_| {
            let mut x = u64::MAX;
            for _ in 0..if sparse { 6 } else { 1 } {
                *state ^= *state << 13; *state ^= *state >> 7; *state ^= *state << 17;
                x &= *state;
            }
            x
        }).collect();
        BitSet::build(w, data)
    }).collect())
}
fn rowxor(a: &BitMatrix, b: &BitMatrix) -> BitMatrix {
    let mut c = BitMatrix::new(a.height(), b.width());
    for (a, c) in a.rows.iter().zip(c.rows.iter_mut()) {
        for (block, &word) in a.data.iter().enumerate() {
            let mut word = word;
            while word != 0 {
                let k = block * 64 + word.trailing_zeros() as usize;
                for (c, b) in c.data.iter_mut().zip(&b.rows[k].data) { *c ^= b; }
                word &= word - 1;
            }
        }
    }
    c
}
fn run<T>(name: &str, mut f: impl FnMut() -> T) {
    black_box(f());
    let mut times = Vec::new();
    for _ in 0..3 {
        let start = Instant::now(); black_box(f());
        times.push(start.elapsed().as_secs_f64() * 1000.0);
    }
    times.sort_by(f64::total_cmp);
    println!("{name},{:.4}", times[1]);
}
#[test]
#[ignore]
fn tune() {
    let mut state = 20261020;
    for (n,k,m) in [(64,64,64),(256,256,256),(512,512,512),(1024,1024,1024),(4096,4096,4096),(16,4096,4096),(4096,4096,16),(4096,16,4096),(256,4096,4096)] {
        let a = make(n,k,&mut state,false); let b=make(k,m,&mut state,false);
        println!("shape,{n},{k},{m}");
        assert_eq!(old::prod_xor(&a,&b), a.prod_xor(&b));
        run("old",||old::prod_xor(&a,&b));
        run("rowxor",||rowxor(&a,&b));
        run("selected",||a.prod_xor(&b));
        macro_rules! bench { ($b:literal,$g:literal,$p:literal) => {assert_eq!(a.gf2_table_product::<$b,$g,$p>(&b), a.prod_xor(&b)); run(concat!(stringify!($b),"/",stringify!($g),"/",stringify!($p)),||a.gf2_table_product::<$b,$g,$p>(&b));} }
        bench!(4,1,8); bench!(4,4,8); bench!(4,4,16); bench!(4,4,32);
        bench!(8,1,8); bench!(8,1,16); bench!(8,4,4); bench!(8,4,8); bench!(8,4,16); bench!(8,4,32); bench!(8,4,64); bench!(8,8,16); bench!(8,8,32); bench!(8,8,64); bench!(8,16,64); bench!(4,8,32); bench!(4,8,64); bench!(4,16,64); bench!(2,4,64); bench!(6,4,64); bench!(10,4,16); bench!(10,4,32); bench!(10,4,64); bench!(12,4,16); bench!(12,4,32); bench!(12,4,64);
    }
}

#[test]
#[ignore]
fn density() {
    let n = 4096;
    let mut seed = 90210;
    let b = make(n,n,&mut seed,false);
    for mode in 0..4 {
        let mut a = make(n,n,&mut seed,true);
        if mode >= 2 { for row in &mut a.rows { for w in &mut row.data { *w = !*w; } } }
        if mode == 1 { for row in &mut a.rows { row.data.fill(0); } }
        if mode == 3 { for row in &mut a.rows { row.data.fill(u64::MAX); } }
        println!("density_mode,{mode}");
        let expected = rowxor(&a,&b);
        assert_eq!(a.prod_xor(&b),expected);
        run("rowxor",||rowxor(&a,&b));
        run("selected",||a.prod_xor(&b));
    }
}
