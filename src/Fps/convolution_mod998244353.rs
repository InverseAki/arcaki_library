// 前提: StaticModInt と Mod998244353（ACL版または同梱MI版）。
/// mod998244353の畳み込み。各入力長<=2^24、結果長<=2^25-1。
/// 998244353のradix-4 NTTを使用。長い入力はブロック分割し、変換済み配列を再利用。
/// 外部NTT/CRTに依存しない。空入力は空。入力は変更しない。
/// 最大長同士の配列領域ピークは入力込み約544MiB（実行環境の追加領域は別）。
/// 最大長時は4ブロックずつ、順変換8回・逆変換7回。変換長はいずれも2^23。
pub fn convolution_mod998244353(
    a: &[StaticModInt<Mod998244353>],
    b: &[StaticModInt<Mod998244353>],
) -> Vec<StaticModInt<Mod998244353>> {
    convolution_998244353_detail::check_lengths(a.len(), b.len());
    if a.is_empty() || b.is_empty() {
        return vec![];
    }
    let size = a.len() + b.len() - 1;
    if a.len().min(b.len()) <= 32 {
        let mut c = vec![StaticModInt::<Mod998244353>::raw(0); size];
        for (i, &x) in a.iter().enumerate() {
            for (j, &y) in b.iter().enumerate() {
                c[i + j] += x * y;
            }
        }
        return c;
    }
    let block_size = if size <= 1 << 23 {
        size.next_power_of_two()
    } else {
        (a.len().min(b.len()).next_power_of_two() * 4).min(1 << 22)
    };
    convolution_998244353_detail::blocked(a, b, block_size)
}

/// 入力Vecを消費する省メモリ版。同じ結果を返し、入力は呼出後に利用できない。
/// 大きい入力では各側の変換が済んだ時点で元Vecを解放する。
/// 最大長同士の配列領域ピークは約416MiB（元Vecの余分なcapacity等は別）。
pub fn convolution_mod998244353_owned(
    a: Vec<StaticModInt<Mod998244353>>,
    b: Vec<StaticModInt<Mod998244353>>,
) -> Vec<StaticModInt<Mod998244353>> {
    convolution_998244353_detail::check_lengths(a.len(), b.len());
    if a.is_empty() || b.is_empty() {
        return vec![];
    }
    let size = a.len() + b.len() - 1;
    if size <= 1 << 23 || a.len().min(b.len()) <= 1 << 22 {
        return convolution_mod998244353(&a, &b);
    }
    convolution_998244353_detail::blocked_owned(a, b, 1 << 22)
}

mod convolution_998244353_detail {
    type MI = super::StaticModInt<super::Mod998244353>;
    const P: u32 = 998244353;
    #[inline(always)]
    fn add(a: u32, b: u32) -> u32 {
        let x = a + b;
        if x >= P {
            x - P
        } else {
            x
        }
    }
    #[inline(always)]
    fn sub(a: u32, b: u32) -> u32 {
        if a >= b {
            a - b
        } else {
            a + P - b
        }
    }
    #[inline(always)]
    fn mul(a: u32, b: u32) -> u32 {
        (a as u64 * b as u64 % P as u64) as u32
    }
    fn pow(mut x: u32, mut n: u32) -> u32 {
        let mut r = 1;
        while n > 0 {
            if n & 1 != 0 {
                r = mul(r, x);
            }
            x = mul(x, x);
            n >>= 1;
        }
        r
    }
    pub(super) fn check_lengths(n: usize, m: usize) {
        assert!(
            n <= 1 << 24 && m <= 1 << 24,
            "each input length must be <= 2^24"
        );
    }
    // ACLのbutterflyと同じ段順序。独立したbit reversalは行わない。
    // https://github.com/atcoder/ac-library/blob/master/atcoder/convolution.hpp (CC0)
    struct Plan {
        root: [u32; 24],
        iroot: [u32; 24],
        rate2: [u32; 22],
        irate2: [u32; 22],
        rate3: [u32; 21],
        irate3: [u32; 21],
    }
    impl Plan {
        fn new() -> Self {
            let mut s = Self {
                root: [0; 24],
                iroot: [0; 24],
                rate2: [0; 22],
                irate2: [0; 22],
                rate3: [0; 21],
                irate3: [0; 21],
            };
            s.root[23] = pow(3, (P - 1) >> 23);
            s.iroot[23] = pow(s.root[23], P - 2);
            for i in (0..23).rev() {
                s.root[i] = mul(s.root[i + 1], s.root[i + 1]);
                s.iroot[i] = mul(s.iroot[i + 1], s.iroot[i + 1]);
            }
            let (mut p, mut ip) = (1, 1);
            for i in 0..22 {
                s.rate2[i] = mul(s.root[i + 2], p);
                s.irate2[i] = mul(s.iroot[i + 2], ip);
                p = mul(p, s.iroot[i + 2]);
                ip = mul(ip, s.root[i + 2]);
            }
            let (mut p, mut ip) = (1, 1);
            for i in 0..21 {
                s.rate3[i] = mul(s.root[i + 3], p);
                s.irate3[i] = mul(s.iroot[i + 3], ip);
                p = mul(p, s.iroot[i + 3]);
                ip = mul(ip, s.root[i + 3]);
            }
            s
        }
        #[inline]
        fn forward4<const UNIT: bool>(
            a0: &mut [u32],
            a1: &mut [u32],
            a2: &mut [u32],
            a3: &mut [u32],
            r1: u32,
            r2: u32,
            r3: u32,
            imag: u32,
        ) {
            for (((x0, x1), x2), x3) in a0.iter_mut().zip(a1).zip(a2).zip(a3) {
                let u = *x0;
                let v = if UNIT { *x1 } else { mul(*x1, r1) };
                let w = if UNIT { *x2 } else { mul(*x2, r2) };
                let z = if UNIT { *x3 } else { mul(*x3, r3) };
                let s02 = add(u, w);
                let d02 = sub(u, w);
                let s13 = add(v, z);
                let d13 = mul(sub(v, z), imag);
                *x0 = add(s02, s13);
                *x1 = sub(s02, s13);
                *x2 = add(d02, d13);
                *x3 = sub(d02, d13);
            }
        }
        #[inline]
        fn inverse4<const UNIT: bool>(
            a0: &mut [u32],
            a1: &mut [u32],
            a2: &mut [u32],
            a3: &mut [u32],
            r1: u32,
            r2: u32,
            r3: u32,
            imag: u32,
        ) {
            for (((x0, x1), x2), x3) in a0.iter_mut().zip(a1).zip(a2).zip(a3) {
                let u = *x0;
                let v = *x1;
                let w = *x2;
                let z = *x3;
                let s01 = add(u, v);
                let d01 = sub(u, v);
                let s23 = add(w, z);
                let d23 = mul(sub(w, z), imag);
                let t1 = add(d01, d23);
                let t2 = sub(s01, s23);
                let t3 = sub(d01, d23);
                *x0 = add(s01, s23);
                *x1 = if UNIT { t1 } else { mul(t1, r1) };
                *x2 = if UNIT { t2 } else { mul(t2, r2) };
                *x3 = if UNIT { t3 } else { mul(t3, r3) };
            }
        }
        fn forward(&self, a: &mut [u32]) {
            let h = a.len().trailing_zeros() as usize;
            let mut len = 0;
            while len < h {
                if h - len == 1 {
                    let p = 1 << (h - len - 1);
                    let blocks = 1 << len;
                    let mut rot = 1;
                    for (s, block) in a.chunks_exact_mut(2 * p).enumerate() {
                        let (l, r) = block.split_at_mut(p);
                        for (x, y) in l.iter_mut().zip(r) {
                            let u = *x;
                            let v = mul(*y, rot);
                            *x = add(u, v);
                            *y = sub(u, v);
                        }
                        if s + 1 < blocks {
                            rot = mul(rot, self.rate2[(!s).trailing_zeros() as usize]);
                        }
                    }
                    len += 1;
                } else {
                    let p = 1 << (h - len - 2);
                    let blocks = 1 << len;
                    let mut rot = 1;
                    let imag = self.root[2];
                    for (s, block) in a.chunks_exact_mut(4 * p).enumerate() {
                        let rot2 = mul(rot, rot);
                        let rot3 = mul(rot2, rot);
                        let (a0, rest) = block.split_at_mut(p);
                        let (a1, rest) = rest.split_at_mut(p);
                        let (a2, a3) = rest.split_at_mut(p);
                        if s == 0 {
                            Self::forward4::<true>(a0, a1, a2, a3, rot, rot2, rot3, imag);
                        } else {
                            Self::forward4::<false>(a0, a1, a2, a3, rot, rot2, rot3, imag);
                        }
                        if s + 1 < blocks {
                            rot = mul(rot, self.rate3[(!s).trailing_zeros() as usize]);
                        }
                    }
                    len += 2;
                }
            }
        }
        fn inverse(&self, a: &mut [u32]) {
            let h = a.len().trailing_zeros() as usize;
            let mut len = h;
            while len > 0 {
                if len == 1 {
                    let p = 1 << (h - len);
                    let blocks = 1 << (len - 1);
                    let mut rot = 1;
                    for (s, block) in a.chunks_exact_mut(2 * p).enumerate() {
                        let (l, r) = block.split_at_mut(p);
                        for (x, y) in l.iter_mut().zip(r) {
                            let u = *x;
                            let v = *y;
                            *x = add(u, v);
                            *y = mul(sub(u, v), rot);
                        }
                        if s + 1 < blocks {
                            rot = mul(rot, self.irate2[(!s).trailing_zeros() as usize]);
                        }
                    }
                    len -= 1;
                } else {
                    let p = 1 << (h - len);
                    let blocks = 1 << (len - 2);
                    let mut rot = 1;
                    let imag = self.iroot[2];
                    for (s, block) in a.chunks_exact_mut(4 * p).enumerate() {
                        let rot2 = mul(rot, rot);
                        let rot3 = mul(rot2, rot);
                        let (a0, rest) = block.split_at_mut(p);
                        let (a1, rest) = rest.split_at_mut(p);
                        let (a2, a3) = rest.split_at_mut(p);
                        if s == 0 {
                            Self::inverse4::<true>(a0, a1, a2, a3, rot, rot2, rot3, imag);
                        } else {
                            Self::inverse4::<false>(a0, a1, a2, a3, rot, rot2, rot3, imag);
                        }
                        if s + 1 < blocks {
                            rot = mul(rot, self.irate3[(!s).trailing_zeros() as usize]);
                        }
                    }
                    len -= 2;
                }
            }
        }
    }
    pub(super) fn blocked(a: &[MI], b: &[MI], block_size: usize) -> Vec<MI> {
        if a.is_empty() || b.is_empty() {
            return vec![];
        }
        let size = a.len() + b.len() - 1;
        let plan = Plan::new();
        let n = size.next_power_of_two().min(2 * block_size);
        let transform = |v: &[MI]| {
            let mut x = vec![0; n];
            for (x, v) in x.iter_mut().zip(v) {
                *x = v.val();
            }
            plan.forward(&mut x);
            x
        };
        if a.len() <= block_size && b.len() <= block_size {
            let mut x = transform(a);
            let y = transform(b);
            for (x, y) in x.iter_mut().zip(y) {
                *x = mul(*x, y);
            }
            plan.inverse(&mut x);
            let inv = pow(n as u32, P - 2);
            x.truncate(size);
            return x.into_iter().map(|v| MI::raw(mul(v, inv))).collect();
        }
        if a.len().min(b.len()) <= block_size {
            let (long, short) = if a.len() >= b.len() { (a, b) } else { (b, a) };
            let y = transform(short);
            let inv = pow(n as u32, P - 2);
            let mut result = vec![MI::raw(0); size];
            for (k, chunk) in long.chunks(block_size).enumerate() {
                let mut x = transform(chunk);
                for (x, &y) in x.iter_mut().zip(&y) {
                    *x = mul(*x, y);
                }
                plan.inverse(&mut x);
                for (out, &v) in result[k * block_size..]
                    .iter_mut()
                    .zip(&x[..chunk.len() + short.len() - 1])
                {
                    *out = MI::raw(add(out.val(), mul(v, inv)));
                }
            }
            return result;
        }
        let aa: Vec<_> = a.chunks(block_size).map(transform).collect();
        let bb: Vec<_> = b.chunks(block_size).map(transform).collect();
        combine(aa, bb, block_size, size, n, &plan)
    }
    pub(super) fn blocked_owned(a: Vec<MI>, b: Vec<MI>, block_size: usize) -> Vec<MI> {
        if a.is_empty() || b.is_empty() {
            return vec![];
        }
        let size = a.len() + b.len() - 1;
        let n = size.next_power_of_two().min(2 * block_size);
        let plan = Plan::new();
        let transform = |v: &[MI]| {
            let mut x = vec![0; n];
            for (x, v) in x.iter_mut().zip(v) {
                *x = v.val();
            }
            plan.forward(&mut x);
            x
        };
        let aa = a.chunks(block_size).map(transform).collect();
        drop(a);
        let bb = b.chunks(block_size).map(transform).collect();
        drop(b);
        combine(aa, bb, block_size, size, n, &plan)
    }
    fn combine(
        mut aa: Vec<Vec<u32>>,
        mut bb: Vec<Vec<u32>>,
        block_size: usize,
        size: usize,
        n: usize,
        plan: &Plan,
    ) -> Vec<MI> {
        // 出力を一度だけ確保。不要になったスペクトルは先に解放する。
        let mut out = vec![MI::raw(0); size];
        let mut work = vec![0; n];
        let inv = pow(n as u32, P - 2);
        for k in 0..aa.len() + bb.len() - 1 {
            let lo = k.saturating_sub(bb.len() - 1);
            let count = k.min(aa.len() - 1) - lo + 1;
            // 最大入力でも1対角線は4積以下。u64で加算して還元を1回にまとめる。
            macro_rules! diagonal {
                ($($j:expr),+) => {{
                    let xs=[$(&aa[lo+$j][..]),+];let ys=[$(&bb[k-lo-$j][..]),+];
                    for v in xs.iter().chain(ys.iter()){assert_eq!(v.len(),work.len());}
                    for (idx,w) in work.iter_mut().enumerate(){
                        let sum=0u64 $(+xs[$j][idx]as u64*ys[$j][idx]as u64)+;
                        *w=(sum%P as u64)as u32;
                    }
                }};
            }
            match count {
                1 => diagonal!(0),
                2 => diagonal!(0, 1),
                3 => diagonal!(0, 1, 2),
                4 => diagonal!(0, 1, 2, 3),
                _ => {
                    work.fill(0);
                    for i in lo..lo + count {
                        for ((w, &x), &y) in work.iter_mut().zip(&aa[i]).zip(&bb[k - i]) {
                            *w = add(*w, mul(x, y));
                        }
                    }
                }
            }
            if k >= bb.len() - 1 {
                let i = k - (bb.len() - 1);
                aa[i] = Vec::new();
            }
            if k >= aa.len() - 1 {
                let j = k - (aa.len() - 1);
                bb[j] = Vec::new();
            }
            plan.inverse(&mut work);
            let offset = k * block_size;
            let used = n.min(size - offset);
            for (o, &v) in out[offset..offset + used].iter_mut().zip(&work[..used]) {
                *o = MI::raw(add(o.val(), mul(v, inv)));
            }
        }
        out
    }
}
