#!/usr/bin/env python3
"""同梱MI/任意指定ACL rlibとの結合検証。外部取得なし。"""
from pathlib import Path
import argparse
import subprocess
import tempfile
root=Path(__file__).resolve().parents[1]
parser=argparse.ArgumentParser()
parser.add_argument('--acl-rlib',type=Path)
parser.add_argument('--full',action='store_true',help='同梱MIのreleaseで各入力2^24の最大長試験も実行')
args=parser.parse_args()
s=(root.parent/'librarychecker/src/main.rs').read_text()
start=s.index('pub type ModInt998244353 =')
end=s.index('#[allow(unused_imports)]\nuse std::{',start)
backends=[('bundled','use std::{marker::PhantomData,ops::*,str::FromStr};\n'+s[start:end],[])]
if args.acl_rlib:
    lib=args.acl_rlib.resolve()
    backends.append(('acl','use ac_library::*;\n',['--extern',f'ac_library={lib}','-L',f'dependency={lib.parent}']))
body='\n'.join((root/f).read_text() for f in ['src/Fps/convolution_mod998244353.rs','src/Fps/convolution.rs','tests/support/convolution_998244353_cases.rs'])
with tempfile.TemporaryDirectory(dir=root,prefix='conv-check-') as tmp:
    p=Path(tmp)
    for name,prefix,extra in backends:
        (p/'check.rs').write_text(prefix+body)
        for profile,flags in [('debug',[]),('release',['-O'])]:
            print(f'{name} / {profile}',flush=True)
            binary=p/f'{name}-{profile}'
            subprocess.run(['rustc','--edition=2021','--test',str(p/'check.rs'),'-o',str(binary),*extra,*flags],check=True)
            subprocess.run([str(binary)],check=True)
            if args.full and name == 'bundled' and profile == 'release':
                subprocess.run([str(binary), '--ignored', '--nocapture', '--test-threads=1'],check=True)
