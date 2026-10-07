pub mod fps {
    use super::{convolution, Mint};
    #[inline]
    pub fn zero() -> Mint { Mint::new(0) }
    #[inline]
    pub fn one() -> Mint { Mint::new(1) }
    #[inline]
    pub fn two() -> Mint { Mint::new(2) }

    #[inline]
    pub fn trim(mut a: Vec<Mint>) -> Vec<Mint> {
        while a.last().map_or(false, |x| *x == zero()) {
            a.pop();
        }
        a
    }

    #[inline]
    fn prefix(mut a: Vec<Mint>, n: usize) -> Vec<Mint> {
        if a.len() > n { a.truncate(n); }
        else { a.resize(n, zero()); }
        a
    }

    pub fn add(a: &[Mint], b: &[Mint]) -> Vec<Mint> {
        let n = a.len().max(b.len());
        let mut c = vec![zero(); n];
        for i in 0..n {
            if i < a.len() { c[i] += a[i]; }
            if i < b.len() { c[i] += b[i]; }
        }
        trim(c)
    }

    pub fn sub(a: &[Mint], b: &[Mint]) -> Vec<Mint> {
        let n = a.len().max(b.len());
        let mut c = vec![zero(); n];
        for i in 0..n {
            if i < a.len() { c[i] += a[i]; }
            if i < b.len() { c[i] -= b[i]; }
        }
        trim(c)
    }

    pub fn mul(a: &[Mint], b: &[Mint]) -> Vec<Mint> {
        if a.is_empty() || b.is_empty() { return vec![]; }
        convolution(a, b)
    }

    pub fn inv(f: &[Mint], n: usize) -> Vec<Mint> {
        assert!(n > 0);
        assert!(!f.is_empty() && f[0] != zero());

        let mut g = vec![f[0].inv()];
        let mut m = 1usize;

        while m < n {
            let _m2 = (m << 1).min(n.next_power_of_two());
            let need = (m << 1).min(n);

            let f_tr = prefix(f.to_vec(), need);
            let mut t = mul(&f_tr, &g);
            t = prefix(t, need);

            for i in 0..need { t[i] = -t[i]; }
            t[0] += two();

            g = mul(&g, &t);
            g = prefix(g, need);

            m <<= 1;
        }

        prefix(g, n)
    }

    pub fn poly_mod(p: &[Mint], q: &[Mint]) -> Vec<Mint> {
        let mut p = trim(p.to_vec());
        let q = trim(q.to_vec());
        assert!(!q.is_empty());

        if p.len() < q.len() {
            return p;
        }

        let n = p.len() - 1;
        let m = q.len() - 1;
        let k = n - m + 1;

        let mut rp = p.clone(); rp.reverse();
        let mut rq = q.clone(); rq.reverse();

        assert!(rq[0] != zero());
        let inv_rq = inv(&rq, k);

        let rp_k = prefix(rp, k);
        let mut qrev = mul(&rp_k, &inv_rq);
        qrev = prefix(qrev, k);

        qrev.reverse();
        let quo = qrev;

        let mut prod = mul(&quo, &q);
        prod.resize(p.len(), zero());

        for i in 0..p.len() {
            p[i] -= prod[i];
        }
        p.truncate(m);
        trim(p)
    }

    #[inline]
    fn even_coeffs(a: &[Mint]) -> Vec<Mint> {
        let mut res = Vec::with_capacity((a.len() + 1) / 2);
        for i in (0..a.len()).step_by(2) { res.push(a[i]); }
        trim(res)
    }

    #[inline]
    fn odd_coeffs(a: &[Mint]) -> Vec<Mint> {
        let mut res = Vec::with_capacity(a.len() / 2);
        for i in (1..a.len()).step_by(2) { res.push(a[i]); }
        trim(res)
    }

    #[inline]
    fn negate_odd(mut q: Vec<Mint>) -> Vec<Mint> {
        for i in (1..q.len()).step_by(2) { q[i] = -q[i]; }
        q
    }

    pub fn bostan_mori(mut p: Vec<Mint>, mut q: Vec<Mint>, mut k: u64) -> Mint {
        p = trim(p);
        q = trim(q);
        assert!(!q.is_empty() && q[0] != zero());

        let inv_q0 = q[0].inv();
        for x in p.iter_mut() { *x *= inv_q0; }
        for x in q.iter_mut() { *x *= inv_q0; }

        if p.len() >= q.len() {
            p = poly_mod(&p, &q);
        }

        while k > 0 {
            let q_neg = negate_odd(q.clone());

            let u = mul(&p, &q_neg);
            let v = mul(&q, &q_neg);

            if (k & 1) == 0 {
                p = even_coeffs(&u);
            } else {
                p = odd_coeffs(&u);
            }
            q = even_coeffs(&v);

            assert!(!q.is_empty() && q[0] != zero());
            let inv_q0 = q[0].inv();
            for x in p.iter_mut() { *x *= inv_q0; }
            for x in q.iter_mut() { *x *= inv_q0; }

            k >>= 1;
            if p.is_empty() { return zero(); }
        }

        if p.is_empty() { zero() } else { p[0] }
    }
}
use fps::bostan_mori;
