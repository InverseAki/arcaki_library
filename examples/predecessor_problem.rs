#![allow(dead_code)]
#[path = "../src/Basic/basic_io.rs"]
mod basic_io;
#[path = "../src/Basic/basic_output.rs"]
mod basic_output;
#[path = "../src/DataStructure/predecessor64.rs"]
mod predecessor64;

fn main() {
    let mut input = basic_io::Input::new();
    let n = input.usize();
    let q = input.usize();
    let mut set = predecessor64::Predecessor64::from_vec_u64(input.binary_u64_len(n));
    let mut out = basic_output::Output::from_writer(std::io::stdout().lock());
    for _ in 0..q {
        let t = input.u8();
        let k = input.usize();
        match t {
            0 => set.insert(k),
            1 => set.remove(k),
            2 => {
                out.u8_line(set.include(k) as u8);
            }
            3 => {
                out.i32_line(set.innext(k) as i32);
            }
            4 => {
                out.i32_line(set.inprev(k) as i32);
            }
            _ => unreachable!(),
        }
    }
    out.flush().unwrap();
}
