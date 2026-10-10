#![allow(dead_code)]
include!("../../src/NumberTheory/ratio.rs");
include!("../../src/NumberTheory/big_integer.rs");
include!("../../src/NumberTheory/big_ratio.rs");
use std::io::{self, Read, Write};
fn pair<T: RatioInteger>(r: Result<Rational<T>, RatioError>) -> String {
    match r {
        Ok(r) => format!("{} {}", r.numerator(), r.denominator()),
        Err(e) => match e {
            RatioError::Overflow => "overflow",
            RatioError::Indeterminate => "indeterminate",
            _ => "error",
        }
        .into(),
    }
}
fn run<T: RatioInteger>(a: &[&str]) -> String {
    let get = |n: usize| a[n].parse::<T>().ok().unwrap();
    let lhs = Rational::<T>::try_from_fraction(get(1), get(2));
    let l = match lhs {
        Ok(x) => x,
        Err(e) => return pair::<T>(Err(e)),
    };
    match a[0] {
        "new" => pair(Ok(l)),
        "neg" => pair(l.try_neg()),
        "inv" => pair(l.try_inv()),
        "round" => format!("{} {} {}", l.floor(), l.ceil(), l.trunc()),
        "pow" => pair(l.try_pow(a[3].parse().unwrap())),
        op => {
            let r = match Rational::<T>::try_from_fraction(get(3), get(4)) {
                Ok(x) => x,
                Err(e) => return pair::<T>(Err(e)),
            };
            match op {
                "add" => pair(l.try_add(&r)),
                "sub" => pair(l.try_sub(&r)),
                "mul" => pair(l.try_mul(&r)),
                "div" => pair(l.try_div(&r)),
                "cmp" => format!(
                    "{}",
                    match l.cmp(&r) {
                        std::cmp::Ordering::Less => -1,
                        std::cmp::Ordering::Equal => 0,
                        std::cmp::Ordering::Greater => 1,
                    }
                ),
                _ => panic!("unknown operation"),
            }
        }
    }
}
fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut out = io::BufWriter::new(io::stdout().lock());
    for line in input.lines() {
        let a: Vec<_> = line.split_whitespace().collect();
        let result = match a[0] {
            "i64" => run::<i64>(&a[1..]),
            "i128" => run::<i128>(&a[1..]),
            "big" => run::<BigInt>(&a[1..]),
            "gcd" => {
                let x: BigInt = a[1].parse().unwrap();
                let y: BigInt = a[2].parse().unwrap();
                x.gcd(&y).to_string()
            }
            _ => panic!(),
        };
        writeln!(out, "{result}").unwrap();
    }
}
