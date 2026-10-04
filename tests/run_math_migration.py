#!/usr/bin/env python3
"""既存ファイルは変更せず、同梱MIを抽出して結合検証。外部依存なし。"""
from pathlib import Path
import argparse
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
WORKSPACE = ROOT.parent
parser = argparse.ArgumentParser()
parser.add_argument("--acl-rlib", type=Path, help="既存のACL rlibでも結合検証する")
args = parser.parse_args()
source = (WORKSPACE / "librarychecker/src/main.rs").read_text()
start = source.index("pub type ModInt1000000007 =")
end = source.index("#[allow(unused_imports)]\nuse std::{", start)
mi = source[start:end]
# 補助関数の検証用バックエンド。NTT本体の検証を行うものではない。
code = "use std::{marker::PhantomData, ops::*, str::FromStr};\n" + mi
code += "\ntype MI = ModInt998244353;\n"
for module in ["math", "barrett"]:
    code += f'mod {module} {{\n' + ("const MOD: i64 = 998244353;\n" if module == "math" else "") + (ROOT / f"src/Basic/{module}.rs").read_text() + "\n}\n"
for file in ["src/NumberTheory/mint_combination.rs", "tests/support/math_migration_cases.rs"]:
    code += (ROOT / file).read_text() + "\n"
code += "fn convolution(a: &[MI], b: &[MI]) -> Vec<MI> { naive_product(a,b) }\n"
with tempfile.TemporaryDirectory(prefix="math-check-", dir=ROOT) as d:
    src = Path(d) / "check.rs"
    src.write_text(code)
    for label, flags in [("debug", []), ("release", ["-O"])]:
        binary = Path(d) / label
        subprocess.run(["rustc", "--edition=2021", "--test", str(src), "-o", str(binary), *flags], check=True)
        subprocess.run([str(binary)], check=True)

if args.acl_rlib:
    acl_code = "use ac_library::*;\ntype MI = ModInt998244353;\n"
    for module in ["math", "barrett"]:
        acl_code += f'mod {module} {{\n' + ("const MOD: i64 = 998244353;\n" if module == "math" else "") + (ROOT / f"src/Basic/{module}.rs").read_text() + "\n}\n"
    for file in ["src/NumberTheory/mint_combination.rs", "tests/support/math_migration_cases.rs"]:
        acl_code += (ROOT / file).read_text() + "\n"
    lib = args.acl_rlib.resolve()
    with tempfile.TemporaryDirectory(prefix="acl-check-", dir=ROOT) as d:
        src = Path(d) / "check.rs"
        src.write_text(acl_code)
        for label, flags in [("debug", []), ("release", ["-O"])]:
            binary = Path(d) / label
            subprocess.run(["rustc", "--edition=2021", "--test", str(src), "-o", str(binary), "--extern", f"ac_library={lib}", "-L", f"dependency={lib.parent}", *flags], check=True)
            subprocess.run([str(binary)], check=True)
