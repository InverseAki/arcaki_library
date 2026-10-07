pub struct Output<W: std::io::Write = std::io::Stdout> {
    writer: std::io::BufWriter<W>,
}

impl Output<std::io::Stdout> {
    pub fn new() -> Self {
        Self::from_writer(std::io::stdout())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_writer_capacity(capacity, std::io::stdout())
    }
}

impl<W: std::io::Write> Output<W> {
    pub fn from_writer(writer: W) -> Self {
        Self::with_writer_capacity(1 << 16, writer)
    }

    pub fn with_writer_capacity(capacity: usize, writer: W) -> Self {
        Self {
            writer: std::io::BufWriter::with_capacity(capacity, writer),
        }
    }

    #[inline(always)]
    pub fn bytes(&mut self, bytes: &[u8]) -> &mut Self {
        std::io::Write::write_all(&mut self.writer, bytes).unwrap();
        self
    }

    #[inline(always)]
    pub fn string(&mut self, s: &str) -> &mut Self {
        self.bytes(s.as_bytes())
    }

    #[inline(always)]
    pub fn byte(&mut self, byte: u8) -> &mut Self {
        self.bytes(&[byte])
    }

    #[inline(always)]
    pub fn space(&mut self) -> &mut Self {
        self.byte(b' ')
    }

    #[inline(always)]
    pub fn newline(&mut self) -> &mut Self {
        self.byte(b'\n')
    }

    pub fn flush(&mut self) -> std::io::Result<()> {
        std::io::Write::flush(&mut self.writer)
    }

    pub fn into_inner(self) -> Result<W, std::io::IntoInnerError<std::io::BufWriter<W>>> {
        self.writer.into_inner()
    }
}

macro_rules! impl_unsigned_output {
    ($($name:ident, $line:ident, $helper:ident, $t:ty, $digits:expr);* $(;)?) => {
        impl<W: std::io::Write> Output<W> {
            $(
                #[inline(always)]
                fn $helper<const LINE: bool>(&mut self, mut value: $t) -> &mut Self {
                    let mut buf = [0u8; $digits + 1];
                    let mut at = buf.len();
                    if LINE {
                        at -= 1;
                        buf[at] = b'\n';
                    }
                    loop {
                        at -= 1;
                        buf[at] = b'0' + (value % 10) as u8;
                        value /= 10;
                        if value == 0 { break; }
                    }
                    self.bytes(&buf[at..])
                }

                #[inline(always)]
                pub fn $name(&mut self, value: $t) -> &mut Self {
                    self.$helper::<false>(value)
                }

                #[inline(always)]
                pub fn $line(&mut self, value: $t) -> &mut Self {
                    self.$helper::<true>(value)
                }
            )*
        }
    };
}

macro_rules! impl_signed_output {
    ($($name:ident, $line:ident, $helper:ident, $t:ty, $digits:expr);* $(;)?) => {
        impl<W: std::io::Write> Output<W> {
            $(
                #[inline(always)]
                fn $helper<const LINE: bool>(&mut self, value: $t) -> &mut Self {
                    let mut magnitude = value.unsigned_abs();
                    let mut buf = [0u8; $digits + 2];
                    let mut at = buf.len();
                    if LINE {
                        at -= 1;
                        buf[at] = b'\n';
                    }
                    loop {
                        at -= 1;
                        buf[at] = b'0' + (magnitude % 10) as u8;
                        magnitude /= 10;
                        if magnitude == 0 { break; }
                    }
                    if value < 0 {
                        at -= 1;
                        buf[at] = b'-';
                    }
                    self.bytes(&buf[at..])
                }

                #[inline(always)]
                pub fn $name(&mut self, value: $t) -> &mut Self {
                    self.$helper::<false>(value)
                }

                #[inline(always)]
                pub fn $line(&mut self, value: $t) -> &mut Self {
                    self.$helper::<true>(value)
                }
            )*
        }
    };
}

impl_unsigned_output! {
    u8,    u8_line,    write_u8,    u8,    3;
    u16,   u16_line,   write_u16,   u16,   5;
    u32,   u32_line,   write_u32,   u32,   10;
    u64,   u64_line,   write_u64,   u64,   20;
    u128,  u128_line,  write_u128,  u128,  39;
    usize, usize_line, write_usize, usize, 20;
}

impl_signed_output! {
    i8,    i8_line,    write_i8,    i8,    3;
    i16,   i16_line,   write_i16,   i16,   5;
    i32,   i32_line,   write_i32,   i32,   10;
    i64,   i64_line,   write_i64,   i64,   20;
    i128,  i128_line,  write_i128,  i128,  39;
    isize, isize_line, write_isize, isize, 20;
}
