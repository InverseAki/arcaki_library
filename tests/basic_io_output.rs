#![allow(dead_code)]
mod input {
    include!("../src/Basic/basic_io.rs");
    pub fn from_bytes(bytes: &[u8]) -> Input {
        let mut buf = bytes.to_vec();
        buf.push(0);
        Input { buf, pos: 0 }
    }
}
#[path = "../src/Basic/basic_output.rs"]
mod output;
use output::Output;

fn oracle_pack(bytes: &[u8]) -> Vec<u64> {
    let mut words = vec![0; bytes.len().div_ceil(64)];
    for (i, &c) in bytes.iter().enumerate() {
        if c == b'1' { words[i >> 6] |= 1 << (i & 63); }
    }
    words
}

#[test]
fn binary_all_byte_patterns_and_word_boundaries() {
    for pattern in 0..256u16 {
        let bytes: Vec<_> = (0..8).map(|i| b'0' + ((pattern >> i) & 1) as u8).collect();
        assert_eq!(input::from_bytes(&bytes).binary_u64(), vec![pattern as u64]);
        assert_eq!(input::from_bytes(&bytes).binary_u64_len(8), vec![pattern as u64]);
    }
    for n in (0..=140).chain([4095, 4096, 4097, 10000]) {
        for pattern in 0..8 {
            let bytes: Vec<_> = (0..n).map(|i| b'0' + if pattern == 0 { 0 } else if pattern == 1 { 1 }
                else { (((i * 37 + pattern) >> 3) & 1) as u8 }).collect();
            let expected = oracle_pack(&bytes);
            assert_eq!(input::from_bytes(&bytes).binary_u64(), expected);
            assert_eq!(input::from_bytes(&bytes).binary_u64_len(n), expected);
        }
    }
}

#[test]
fn binary_mixed_tokens_eof_and_zero_length() {
    for fixed in [false, true] {
        let mut inp = input::from_bytes(b"\r\n\t 9 2\n100000001\r\n-128 255 01");
        let n = inp.usize();
        assert_eq!(inp.usize(), 2);
        assert!(inp.binary_u64_len(0).is_empty());
        let words = if fixed { inp.binary_u64_len(n) } else { inp.binary_u64() };
        assert_eq!(words, vec![257]);
        assert_eq!(inp.i8(), -128);
        assert_eq!(inp.u8(), 255);
        assert_eq!(inp.binary_u64_len(2), vec![2]);
        assert!(inp.binary_u64().is_empty());
    }
    let mut inp = input::from_bytes(b"101 7");
    assert!(inp.binary_u64_len(0).is_empty());
    assert_eq!(inp.binary_u64_len(3), vec![5]);
    assert_eq!(inp.usize(), 7);
}

#[test]
fn binary_rejects_invalid_bytes_and_wrong_lengths() {
    for n in [1, 7, 8, 9, 63, 64, 65] {
        for p in [0, n / 2, n - 1] {
            for c in [b'2', b'9', b'A', b'/', 0x80, 0xff] {
                let mut bytes = vec![b'0'; n]; bytes[p] = c;
                assert!(std::panic::catch_unwind(|| input::from_bytes(&bytes).binary_u64()).is_err());
                assert!(std::panic::catch_unwind(|| input::from_bytes(&bytes).binary_u64_len(n)).is_err());
            }
        }
    }
    for c in 0u16..=255 {
        if c == b'0' as u16 || c == b'1' as u16 { continue; }
        let mut bytes = vec![b'0'; 64]; bytes[31] = c as u8;
        assert!(std::panic::catch_unwind(|| input::from_bytes(&bytes).binary_u64_len(64)).is_err());
    }
    for (bytes, n) in [(&b"101"[..], 4), (&b"1011"[..], 3), (&b"10 1"[..], 4), (&b"101"[..], usize::MAX)] {
        assert!(std::panic::catch_unwind(|| input::from_bytes(bytes).binary_u64_len(n)).is_err());
    }
}

fn random(seed: &mut u64) -> u128 {
    fn next(seed: &mut u64) -> u64 {
        *seed = seed.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = *seed;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
    ((next(seed) as u128) << 64) | next(seed) as u128
}

macro_rules! check_integer {
    ($out:ident, $expected:ident, $seed:ident, $t:ty, $name:ident, $line:ident) => {{
        let mut values = vec![<$t>::MIN, <$t>::MAX, 0, 1, 9];
        let mut power: $t = 10;
        loop {
            values.extend([power - 1, power, power + 1]);
            if let Some(next) = power.checked_mul(10) { power = next; } else { break; }
        }
        values.extend((0..1000).map(|_| random(&mut $seed) as $t));
        for v in values {
            $out.$name(v).space().$line(v);
            $expected.extend_from_slice(format!("{v} {v}\n").as_bytes());
        }
    }};
}

#[test]
fn output_all_integer_types_extrema_decimal_boundaries_and_random_values() {
    for capacity in [0, 1, 2, 7, 64, 65536] {
        let mut out = Output::with_writer_capacity(capacity, Vec::new());
        let mut expected = Vec::new();
        let mut seed = 12345;
        check_integer!(out, expected, seed, u8, u8, u8_line);
        check_integer!(out, expected, seed, u16, u16, u16_line);
        check_integer!(out, expected, seed, u32, u32, u32_line);
        check_integer!(out, expected, seed, u64, u64, u64_line);
        check_integer!(out, expected, seed, u128, u128, u128_line);
        check_integer!(out, expected, seed, usize, usize, usize_line);
        check_integer!(out, expected, seed, i8, i8, i8_line);
        check_integer!(out, expected, seed, i16, i16, i16_line);
        check_integer!(out, expected, seed, i32, i32, i32_line);
        check_integer!(out, expected, seed, i64, i64, i64_line);
        check_integer!(out, expected, seed, i128, i128, i128_line);
        check_integer!(out, expected, seed, isize, isize, isize_line);
        out.flush().unwrap();
        assert_eq!(out.into_inner().unwrap(), expected);
    }
}

#[test]
fn output_raw_text_flush_and_drop() {
    let mut bytes = Vec::new();
    {
        let mut out = Output::with_writer_capacity(7, &mut bytes);
        out.string("日本語").space().i64(-42).newline().bytes(b"\0\xff").byte(b'!');
    }
    let mut expected = "日本語 -42\n".as_bytes().to_vec();
    expected.extend_from_slice(b"\0\xff!");
    assert_eq!(bytes, expected);
}

#[derive(Debug, Default)]
struct ShortWriter { data: Vec<u8>, flushed: usize, interrupt: bool }
impl std::io::Write for ShortWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self.interrupt { self.interrupt = false; return Err(std::io::ErrorKind::Interrupted.into()); }
        let n = bytes.len().min(3);
        self.data.extend_from_slice(&bytes[..n]);
        Ok(n)
    }
    fn flush(&mut self) -> std::io::Result<()> { self.flushed += 1; Ok(()) }
}

#[test]
fn output_short_writes_interrupts_and_explicit_flush() {
    let mut out = Output::with_writer_capacity(5, ShortWriter { interrupt: true, ..Default::default() });
    out.u128_line(u128::MAX).i128_line(i128::MIN);
    out.flush().unwrap();
    let inner = out.into_inner().unwrap();
    assert_eq!(inner.flushed, 1);
    assert_eq!(inner.data, format!("{}\n{}\n", u128::MAX, i128::MIN).as_bytes());
}
