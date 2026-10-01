pub struct Input {
    buf: Vec<u8>,
    pos: usize,
}

impl Input {
    #[inline]
    pub fn new()->Self {
        let mut buf = Vec::new();
        stdin().read_to_end(&mut buf).unwrap();
        buf.push(b' ');
        Self{buf,pos:0}
    }

    #[inline(always)]
    pub fn u32(&mut self)->u32{
        let buf = &self.buf;
        let mut i = self.pos;
        while buf[i]<b'0' {
            i+=1;
        }
        let mut res = 0;
        while buf[i]>=b'0'{
            res=res*10+(buf[i]-b'0')as u32;i+=1;
        }
        self.pos=i;
        res
    }

    #[inline(always)]
    pub fn usize(&mut self)->usize{
        let buf = &self.buf;
        let mut i = self.pos;
        while buf[i]<b'0' {
            i+=1;
        }
        let mut res = 0;
        while buf[i]>=b'0'{
            res=res*10+(buf[i]-b'0')as usize;i+=1;
        }
        self.pos=i;
        res
    }

    #[inline(always)]
    pub fn u8(&mut self)->u8{
        let buf = &self.buf;
        let mut i = self.pos;
        while buf[i]<b'0' {
            i+=1;
        }
        let mut res = 0;
        while buf[i]>=b'0'{
            res=res*10+buf[i]-b'0';i+=1;
        }
        self.pos=i;
        res
    }
}
