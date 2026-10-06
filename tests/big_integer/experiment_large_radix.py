#!/usr/bin/env python3
"""Generate full-storage large-radix prototypes for reproducible experiments."""
from pathlib import Path
import re, argparse
ROOT=Path(__file__).resolve().parents[2]
parser=argparse.ArgumentParser(description="Generate experimental full-storage radix 10^8/10^16 implementations; not the production implementation.")
parser.add_argument('--group',type=int,choices=[2,4],default=4)
parser.add_argument('--output',type=Path,required=True)
args=parser.parse_args()
s=(ROOT/'tests/support/big_integer_before_non_ntt.rs').read_text()
head=s[:s.index('mod big_integer_detail {')]
head=head.replace('Vec<u32>','Vec<u64>').replace('B as u128','wide_base::<B>() as u128').replace('as u32','as u64')
head=head.replace('let radix = if B == 10000 { 10 } else { 16 };','let radix: u64 = if B == 10000 { 10 } else { 16 };')
head=head.replace('(s.len() + 3) / 4','(s.len() + wide_group::<B>() * 4 - 1) / (wide_group::<B>() * 4)').replace('s.rchunks(4)','s.rchunks(wide_group::<B>() * 4)')
head=head.replace("let mut buf = [b'0'; 4];","let mut buf = [b'0'; 16];\n            let buf = &mut buf[..wide_group::<B>() * 4];")
head=head.replace('.unwrap_or(3)','.unwrap_or(wide_group::<B>() * 4 - 1)').replace('self.digits.len() * 4 + 1','self.digits.len() * wide_group::<B>() * 4 + 1')
orig=s[s.index('mod big_integer_detail {'):]
parts=orig[:orig.index('    pub(super) fn mul<const')]+orig[orig.index('    // 正規化済み b'):]
parts=parts.replace('const B: u32','const __PARAM: u32').replace('::<B>','::<__PARAM>')
parts=re.sub(r'\bB\b','wide_base::<__PARAM>()',parts)
parts=re.sub(r'\b(u32|u64|i64)\b',lambda m:({'u32':'u64','u64':'u128','i64':'i128'} if args.group==4 else {'u32':'u64','u64':'u64','i64':'i64'})[m[0]],parts)
if args.group==4: parts=parts.replace('0u64','0u128').replace('0i64','0i128')
parts=parts.replace('0u32','0u64')
parts=parts.replace('mod big_integer_detail {','mod big_integer_detail {\n use super::{wide_base,wide_group,ntt_detail};',1)
parts=parts.replace('const __PARAM: u64','const B: u32').replace('__PARAM','B')
# Counter widths should remain usize, wide limbs u64, temporaries u128/i128.
parts=parts.replace('if n <= 32 || q_len <= 32',f'if n <= {32//args.group} || q_len <= {32//args.group}').replace('if n <= 32',f'if n <= {32//args.group}')
mul=r'''
    pub(super) fn mul<const B:u32>(a:&[u64],b:&[u64])->Vec<u64>{
        if a.is_empty() || b.is_empty(){return vec![];}
        if a.len()==1{return mul_small::<B>(b,a[0]);}
        if b.len()==1{return mul_small::<B>(a,b[0]);}
        let base=wide_base::<B>() as u128;
        if a.len().min(b.len())<=12 {
            let mut c=vec![0u128;a.len()+b.len()-1];
            for (i,&x) in a.iter().enumerate(){for(j,&y) in b.iter().enumerate(){c[i+j]+=x as u128*y as u128;}}
            let mut out=Vec::with_capacity(c.len()+1);let mut carry=0u128;
            for x in c {let x=x+carry;out.push((x%base)as u64);carry=x/base;}
            while carry!=0 {out.push((carry%base)as u64);carry/=base;}trim(&mut out);return out;
        }
        fn unpack<const B:u32>(a:&[u64])->Vec<u32>{
            let mut v=Vec::with_capacity(a.len()*wide_group::<B>());
            for &x in a {let mut x=x;for _ in 0..wide_group::<B>() {v.push((x%B as u64)as u32);x/=B as u64;}}
            ntt_detail::trim(&mut v);v
        }
        let x=unpack::<B>(a);
        let product=if std::ptr::eq(a,b){ntt_detail::mul::<B>(&x,&x)}else{ntt_detail::mul::<B>(&x,&unpack::<B>(b))};
        let mut out=Vec::with_capacity((product.len()+wide_group::<B>()-1)/wide_group::<B>());
        for chunk in product.chunks(wide_group::<B>()) {let mut x=0u64;for &d in chunk.iter().rev(){x=x*B as u64+d as u64;}out.push(x);}
        out
    }
'''
mul=mul.replace('<=12',f'<={48//args.group}')
parts=parts.replace('    // 正規化済み b',mul+'\n    // 正規化済み b')
# Original NTT detail is embedded unchanged for splitting and repacking experiments.
args.output.write_text('''const fn wide_group<const B:u32>()->usize {if B==10000 {4}else{3}}
const fn wide_base<const B:u32>()->u64 {if B==10000 {10000000000000000}else{281474976710656}}
'''.replace('if B==10000 {4}else{3}', 'if B==10000 {2}else{2}' if args.group==2 else 'if B==10000 {4}else{3}').replace('10000000000000000', '100000000' if args.group==2 else '10000000000000000').replace('281474976710656', '4294967296' if args.group==2 else '281474976710656')+head+parts+orig.replace('mod big_integer_detail {','mod ntt_detail {',1))
