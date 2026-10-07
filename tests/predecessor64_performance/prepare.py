from pathlib import Path
import sys

root = Path(__file__).resolve().parent
base = (root / 'baseline.rs').read_text()
candidate = (root / 'candidates.rs').read_text().split('#[derive(Clone, Debug)]\npub struct LeafFirst')[0]

# Isolate the update optimization from the search optimization.
start = candidate.index('    #[inline(always)]\n    pub fn insert')
end = candidate.index('    #[inline(always)]\n    pub fn include')
updates = candidate[start:end]
start_base = base.index('    #[inline(always)]\n    pub fn insert')
end_base = base.index('    #[inline(always)]\n    fn ml')
(root / 'update_only.rs').write_text(base[:start_base] + updates + base[end_base:])

# Same algorithms with a single contiguous allocation and fixed-size offsets.
# Eleven levels suffice for a 64-bit usize; no unsafe accesses are used.
flat = candidate.replace('Incremental', 'Flat')
flat = flat.replace('    d: Vec<Vec<u64>>,', '    data: Vec<u64>,\n    offset: [usize; 11],\n    levels: usize,')
start = flat.index('        let mut d = Vec::new();')
end = flat.index('\n    #[inline(always)]\n    pub fn insert')
flat = flat[:start] + '''        let mut offset = [0; 11];
        let mut levels = 0;
        let mut len = (mx + 63) >> 6;
        let mut total = 0;
        loop {
            offset[levels] = total;
            levels += 1;
            total += len;
            if len <= 1 { break; }
            len = (len + 63) >> 6;
        }
        Self { mx, data: vec![0; total], offset, levels }
    }
''' + flat[end:]
flat = flat.replace('for level in &mut self.d {', 'for i in 0..self.levels {')
flat = flat.replace('&mut level[p >> 6]', '&mut self.data[self.offset[i] + (p >> 6)]')
flat = flat.replace('self.d[0][p >> 6]', 'self.data[p >> 6]')
flat = flat.replace('self.d.last().unwrap()[0]', 'self.data[self.offset[self.levels - 1]]')
flat = flat.replace('for (i, level) in self.d.iter().enumerate() {', 'for i in 0..self.levels {')
flat = flat.replace('level[p >> 6]', 'self.data[self.offset[i] + (p >> 6)]')
flat = flat.replace('self.d[j][res]', 'self.data[self.offset[j] + res]')
flat = flat.replace('self.innext(0)', 'self.extreme::<true>()')
flat = flat.replace('self.inprev(self.mx - 1)', 'self.extreme::<false>()')
flat = flat.replace('    #[inline(always)]\n    pub fn prev', '''    #[inline(always)]
    fn extreme<const MIN: bool>(&self) -> usize {
        let leaf_index = if MIN { 0 } else { (self.mx - 1) >> 6 };
        let leaf = self.data[leaf_index];
        if leaf != 0 {
            let bit = if MIN { leaf.trailing_zeros() as usize }
                else { 63 - leaf.leading_zeros() as usize };
            return (leaf_index << 6) | bit;
        }
        let mut word = self.data[self.offset[self.levels - 1]];
        if word == 0 { return !0; }
        let mut p = if MIN { word.trailing_zeros() as usize }
            else { 63 - word.leading_zeros() as usize };
        for i in (0..self.levels - 1).rev() {
            word = self.data[self.offset[i] + p];
            let bit = if MIN { word.trailing_zeros() as usize }
                else { 63 - word.leading_zeros() as usize };
            p = (p << 6) | bit;
        }
        p
    }
    #[inline(always)]
    pub fn prev''')
if '--root-only' in sys.argv:
    start = flat.index('        let leaf_index =')
    end = flat.index('        let mut word =', start)
    flat = flat[:start] + flat[end:]
(root / 'flat.rs').write_text(flat)

# Shifted searches with contiguous storage; update methods are identical to Flat.
source = (root / 'candidates.rs').read_text()
shifted = source[source.index('#[derive(Clone, Debug)]\npub struct Shifted'):]
shifted = shifted.replace('Shifted', 'FlatShifted').replace('pub Incremental', 'pub Flat')
shifted = shifted.replace('Incremental::new', 'Flat::new')
shifted = shifted.replace('self.0.d.len()', 'self.0.levels')
shifted = shifted.replace('self.0.d[0][p >> 6]', 'self.0.data[p >> 6]')
shifted = shifted.replace('self.0.d[i][p >> 6]', 'self.0.data[self.0.offset[i] + (p >> 6)]')
shifted = shifted.replace('self.0.d[j][res]', 'self.0.data[self.0.offset[j] + res]')
shifted = shifted.replace('self.innext(0)', 'self.0.min()')
shifted = shifted.replace('self.inprev(self.0.mx - 1)', 'self.0.max()')
(root / 'flat.rs').write_text(flat + '\n' + shifted)
