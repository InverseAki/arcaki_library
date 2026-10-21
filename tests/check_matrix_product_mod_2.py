#!/usr/bin/env python3
import argparse
import hashlib
import json
from pathlib import Path
import platform
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
COUNTS = {"random": 3, "max_random": 3, "small": 10, "middle": 5, "many_1": 2}


def digest(data):
    return hashlib.sha256(data).hexdigest()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("problem", type=Path, help="library-checker-problems/linear_algebra/matrix_product_mod_2")
    parser.add_argument("--common", type=Path)
    parser.add_argument("--cxx", default="clang++")
    parser.add_argument("--result", type=Path, default=ROOT / "tests/matrix_product_mod_2_official_results.json")
    args = parser.parse_args()
    common = args.common or args.problem.parents[1] / "common"
    hashes = json.loads((args.problem / "hash.json").read_text())
    subprocess.run(["cargo", "build", "--offline", "--release", "--example", "matrix_product_mod_2"], cwd=ROOT, check=True)
    solver = ROOT / "target/release/examples/matrix_product_mod_2"
    results = []
    with tempfile.TemporaryDirectory(prefix="gf2-check-") as tmp:
        tmp = Path(tmp)
        for name in COUNTS:
            subprocess.run([args.cxx, "-O2", "-std=c++17", "-I", str(common), str(args.problem / "gen" / (name + ".cpp")), "-o", str(tmp / name)], check=True)
        cases = [(f"example_{i:02}", None, i) for i in range(3)]
        cases += [(f"{name}_{i:02}", name, i) for name, count in COUNTS.items() for i in range(count)]
        for label, generator, seed in cases:
            data = subprocess.check_output([str(tmp / generator), str(seed)]) if generator else (args.problem / "gen" / (label + ".in")).read_bytes()
            actual_input = digest(data)
            assert actual_input == hashes[label + ".in"], f"generator mismatch: {label}"
            (tmp / "input").write_bytes(data)
            with (tmp / "input").open("rb") as source, (tmp / "output").open("wb") as target:
                start = time.perf_counter()
                subprocess.run([str(solver)], stdin=source, stdout=target, timeout=10, check=True)
                seconds = time.perf_counter() - start
            actual_output = digest((tmp / "output").read_bytes())
            assert actual_output == hashes[label + ".out"], f"wrong answer: {label}"
            results.append(dict(case=label, shape=list(map(int, data.split(b"\n", 1)[0].split())), seconds=seconds, input_sha256=actual_input, output_sha256=actual_output))
            print(f"{label}: OK {seconds:.4f}s", flush=True)
    args.result.write_text(json.dumps(dict(timing="process startup, file stdin/stdout and computation; excludes generation and hashing", platform=platform.platform(), rustc=subprocess.check_output(["rustc", "-V"], text=True).strip(), cases=results), indent=2) + "\n")


if __name__ == "__main__":
    main()
