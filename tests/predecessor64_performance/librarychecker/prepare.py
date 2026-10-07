"""Isolate solve() costs in the pasted submission; do not edit the library."""
from pathlib import Path
import hashlib
import json

ROOT = Path(__file__).resolve().parent
attachment = Path('/Users/tenp/.codex/attachments/8d286c61-edd2-47a4-b0d4-be23627a21fa/貼り付けたテキスト.txt')
cached = ROOT / 'pasted.rs'
original = attachment.read_text() if attachment.exists() else cached.read_text()
cached.write_text(original)
start = original.index('pub fn solve() {')
end = original.index('\nfn main()', start)
solve = original[start:end]
helpers = '''
#[inline(always)]
fn append_answer(out: &mut Vec<u8>, answer: usize) {
    if answer == !0 { out.extend_from_slice(b"-1\\n"); return; }
    let mut digits = [0u8; 16];
    let mut at = digits.len() - 1;
    digits[at] = b'\\n';
    let mut value = answer;
    loop {
        at -= 1;
        digits[at] = b'0' + (value % 10) as u8;
        value /= 10;
        if value == 0 { break; }
    }
    out.extend_from_slice(&digits[at..]);
}

fn pack_binary(bytes: &[u8]) -> Vec<u64> {
    bytes.chunks(64).map(|chunk| {
        let mut word = 0;
        for (group, part) in chunk.chunks(8).enumerate() {
            let packed = if part.len() == 8 {
                let bits = u64::from_le_bytes(part.try_into().unwrap()) & 0x0101_0101_0101_0101;
                (bits.wrapping_mul(0x0102_0408_1020_4080) >> 56) as u64
            } else {
                part.iter().enumerate().fold(0u64, |w, (i, &c)| w | (((c - b'0') as u64) << i))
            };
            word |= packed << (group * 8);
        }
        word
    }).collect()
}
'''

old_build = '''    let mut s = vec![0;(n+63)>>6];
    for (i, &x) in ip.bytes().iter().enumerate(){
        s[i>>6] |= ((x-b'0')as usize)<<(i&63);
    }
    let mut set = Predecessor64::from_vec_usize(s);'''
assert old_build in solve
for name in ['original', 'fmt', 'digits', 'pack', 'pack_digits']:
    body = solve
    if name == 'fmt':
        body = body.replace('    let mut res = String::new();', '    use std::fmt::Write as _;\n    let mut res = String::with_capacity(q * 8);')
        for method in ['innext', 'inprev']:
            body = body.replace(f'''            res.push_str(&(set.{method}(k)as i32).to_string());
            res.push('\\n');''', f'''            writeln!(&mut res, "{{}}", set.{method}(k) as i32).unwrap();''')
    if name in ['digits', 'pack_digits']:
        body = body.replace('let mut res = String::new();', 'let mut res = Vec::with_capacity(q * 8);')
        body = body.replace("res.push('1')", "res.push(b'1')").replace("res.push('0')", "res.push(b'0')").replace("res.push('\\n');", "res.push(b'\\n');")
        for method in ['innext', 'inprev']:
            body = body.replace(f'''            res.push_str(&(set.{method}(k)as i32).to_string());
            res.push(b'\\n');''', f'''            append_answer(&mut res, set.{method}(k));''')
        body = body.replace('write!(o, "{}", res).ok();', 'o.write_all(&res).unwrap();')
    if name in ['pack', 'pack_digits']:
        body = body.replace(old_build, '    let mut set = Predecessor64::from_vec_u64(pack_binary(ip.bytes()));')
    if name == 'pack_digits':
        (ROOT / 'submission.rs').write_text(original[:start] + helpers + body + original[end:] + '''
#[cfg(test)]
mod submission_tests {
    use super::*;
    #[test]
    fn pack_all_eight_bit_patterns() {
        for bits in 0..256u64 {
            let bytes: Vec<_> = (0..8).map(|i| b'0' + ((bits >> i) & 1) as u8).collect();
            assert_eq!(pack_binary(&bytes), vec![bits]);
        }
    }
    #[test]
    fn pack_partial_words_and_multiple_words() {
        for len in 0usize..=140 {
            for seed in 0..64 {
                let bytes: Vec<_> = (0..len).map(|i| b'0' + (((i * 37 + seed) >> 3) & 1) as u8).collect();
                let mut expected = vec![0u64; len.div_ceil(64)];
                for (i, &v) in bytes.iter().enumerate() { expected[i >> 6] |= ((v - b'0') as u64) << (i & 63); }
                assert_eq!(pack_binary(&bytes), expected);
            }
        }
    }
    #[test]
    fn decimal_answers() {
        for value in [0, 1, 9, 10, 99, 100, 9999, 10000, 9999999, !0usize] {
            let mut bytes = vec![];
            append_answer(&mut bytes, value);
            assert_eq!(bytes, format!("{}\\n", value as i32).as_bytes());
        }
    }
}
''')
    # Stage profiling only surrounds phases, without timing each operation.
    body = body.replace('    let mut ip = Input::new();', '    let start = std::time::Instant::now();\n    let mut ip = Input::new();\n    let read_time = start.elapsed();\n    let build_start = std::time::Instant::now();')
    body = body.replace('    let mut res =', '    let build_time = build_start.elapsed();\n    let query_start = std::time::Instant::now();\n    let mut res =', 1)
    pos = body.index('    write!(o,') if name not in ['digits','pack_digits'] else body.index('    o.write_all(')
    body = body[:pos] + '    let query_time = query_start.elapsed();\n    let write_start = std::time::Instant::now();\n' + body[pos:]
    body = body.rsplit('}', 1)[0] + '''    o.flush().unwrap();
    eprintln!("{:.6},{:.6},{:.6},{:.6}", read_time.as_secs_f64()*1000., build_time.as_secs_f64()*1000., query_time.as_secs_f64()*1000., write_start.elapsed().as_secs_f64()*1000.);
}
'''
    (ROOT / (name + '.rs')).write_text(original[:start] + helpers + body + original[end:])

metadata = {'attachment': str(attachment), 'attachment_sha256': hashlib.sha256(original.encode()).hexdigest()}
(ROOT / 'source.json').write_text(json.dumps(metadata, indent=2) + '\n')
