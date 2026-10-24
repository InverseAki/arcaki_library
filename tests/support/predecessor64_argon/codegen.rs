#![crate_type = "lib"]
#[no_mangle]
pub fn high_sub(word: u64) -> u32 { 63 - word.leading_zeros() }
#[no_mangle]
pub fn high_xor(word: u64) -> u32 { 63 ^ word.leading_zeros() }
#[no_mangle]
pub fn high_sub_nonzero(word: u64) -> u32 { if word == 0 { !0 } else { 63 - word.leading_zeros() } }
#[no_mangle]
pub fn high_xor_nonzero(word: u64) -> u32 { if word == 0 { !0 } else { 63 ^ word.leading_zeros() } }
#[no_mangle]
pub fn convert(v: Vec<usize>) -> Vec<u64> { v.into_iter().map(|x| x as u64).collect() }
