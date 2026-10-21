#![allow(dead_code)]
include!("../src/Basic/basic_io.rs");
include!("../src/Basic/basic_output.rs");
include!("../src/DataStructure/sparsetable.rs");

fn main() {
    let mut ip = Input::new();
    let mut out = Output::new();
    let n = ip.usize();
    let q = ip.usize();
    let st = SparseTableMI::build(ip.vec_i32(n));
    for _ in 0..q {
        let l = ip.usize();
        let r = ip.usize();
        out.i32_line(st.query(l, r));
    }
}
