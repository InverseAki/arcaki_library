#![allow(dead_code)]
#[path = "../src/Basic/basic_io.rs"]
mod basic_io;
#[path = "../src/Basic/basic_output.rs"]
mod basic_output;

fn main() {
    let mut input = basic_io::Input::new();
    let n = input.usize();
    let q = input.usize();
    let mut prefix = vec![0i64; n + 1];
    for i in 0..n {
        prefix[i + 1] = prefix[i] + input.i64();
    }
    let mut out = basic_output::Output::new();
    for _ in 0..q {
        let l = input.usize();
        let r = input.usize();
        out.i64_line(prefix[r] - prefix[l]);
    }
    out.flush().unwrap();
}
