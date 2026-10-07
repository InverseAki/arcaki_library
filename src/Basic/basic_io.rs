use std::io::{self, Read};

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
    pub fn bytes(&mut self) -> Vec<u8> {
        let (l, r) = self.token_range();
        self.buf[l..r].to_vec()
    }

    #[inline]
    pub fn binary_u64(&mut self) -> Vec<u64> {
        let (l, r) = self.token_range();
        Self::pack_binary(&self.buf[l..r])
    }

    #[inline]
    pub fn binary_u64_len(&mut self, n: usize) -> Vec<u64> {
        if n == 0 {
            return Vec::new();
        }
        self.skip_whitespace();
        let l = self.pos;
        let r = l.checked_add(n).expect("binary token length overflow");
        assert!(r < self.buf.len(), "binary token shorter than requested");
        assert!(
            self.buf[r] == 0 || self.buf[r].is_ascii_whitespace(),
            "binary token longer than requested"
        );
        self.pos = r;
        Self::pack_binary(&self.buf[l..r])
    }

    fn pack_binary(bytes: &[u8]) -> Vec<u64> {
        let mut invalid = 0u64;
        let words = bytes
            .chunks(64)
            .map(|chunk| {
                let mut word = 0;
                let mut groups = chunk.chunks_exact(8);
                for (i, group) in groups.by_ref().enumerate() {
                    let raw = u64::from_le_bytes(group.try_into().unwrap());
                    invalid |= (raw & 0xfefe_fefe_fefe_fefe) ^ 0x3030_3030_3030_3030;
                    let bits = raw & 0x0101_0101_0101_0101;
                    let packed = bits.wrapping_mul(0x0102_0408_1020_4080) >> 56;
                    word |= packed << (i * 8);
                }
                let offset = (chunk.len() / 8) * 8;
                for (i, &c) in groups.remainder().iter().enumerate() {
                    invalid |= ((c & 0xfe) ^ b'0') as u64;
                    word |= ((c & 1) as u64) << (offset + i);
                }
                word
            })
            .collect();
        assert_eq!(invalid, 0, "binary token must contain only 0 and 1");
        words
    }

    #[inline(always)]
    pub fn string(&mut self) -> String {
        let (l, r) = self.token_range();

        unsafe { String::from_utf8_unchecked(self.buf[l..r].to_vec()) }
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
            res.push(self.bytes());
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
            res.push(self.bytes());
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
