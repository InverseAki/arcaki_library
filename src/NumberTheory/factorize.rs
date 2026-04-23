pub fn fast_mod_pow(x: usize, p: usize, m: usize)->usize{
    let mut res=1;
    let mut t=x;
    let mut z=p;
    while z > 0{
        if z%2==1{
            res = ((res as u128*t as u128)%m as u128)as usize;
        }
        t = ((t as u128*t as u128)%m as u128)as usize;
        z /= 2;
    }
    res
}

pub fn miller_rabin(x: usize)->bool{
    if x==2{
        return true;
    } else if x&1==0||x==1{
        return false;
    }
    let mut cnt = 0;
    let mut z = x-1;
    while z&1 == 0{
        cnt += 1;
        z >>= 1;
    }
    let test = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];
    for p in test{
        if p==x{continue}
        let mut y = fast_mod_pow(p, z, x);
        let mut r = 0;
        if y == 1{continue}
        while y != x-1{
            y = (y as u128*y as u128%x as u128)as usize;
            r += 1;
            if y==1||r==cnt{
                return false;
            }
        }
    }
    true
}

pub fn floor(a:i64, b:i64)->i64{
    let res=(a%b+b)%b;
    (a-res)/b
}
pub fn extended_gcd(a:i64,b:i64)->(i64,i64,i64) {
    if b==0{
        (a,1,0)
    } else {
        let(g,x,y)=extended_gcd(b,a-floor(a, b)*b);
        (g,y,x-floor(a,b)*y)
    }
}

pub fn prime_finder(n: usize)->usize{
    if n%2==0{return 2;}
    let b = (n as f64).powf(0.125) as usize+1;
    for c in 1..n{
        let mut x = 0;
        let mut ys = 0;
        let f = |a: usize|{((a as u128*a as u128%n as u128+c as u128)%n as u128)as usize};
        let (mut y, mut g, mut q, mut r, mut k) = (0, 1, 1, 1, 0);
        while g == 1{
            x = y;
            while k < 3*r/4{
                y = f(y); k += 1;
            }
            while k < r && g == 1{
                ys = y;
                for _ in 0..b.min(r-k){
                    y = f(y); q = (q as u128*(x.max(y) as u128-x.min(y) as u128)%n as u128)as usize;
                }
                g = extended_gcd(q as i64, n as i64).0 as usize;k += b;
            }
            k = r; r <<= 1;
        }
        if g==n{
            g = 1; y = ys;
            while g==1{
                y = f(y); g = extended_gcd((x as i64-y as i64).abs(), n as i64).0 as usize;
            }
        }
        if miller_rabin(g){return g}
        else if g==n{continue}
        else if miller_rabin(n/g){return n/g}
        else {
            return prime_finder(g);
        }
    }
    unreachable!();
}

pub fn factorize(mut n: usize)->HashMap<usize, usize>{
    let mut res = HashMap::new();
    while !miller_rabin(n) && n > 1{
        let p = prime_finder(n);
        let mut cnt = 0;
        while n%p==0{
            n /= p; cnt += 1;
        }
        res.insert(p, cnt);
    }
    if n > 1{
        *res.entry(n).or_insert(0) += 1;
    }
    res
}
