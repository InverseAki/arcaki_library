"""Run all registered Library Checker cases locally, including verifier/checker.

No download or submission. Uses an isolated copy, preserves inputs and outputs.
python3 tests/run_static_rectangle_add_rectangle_sum_official.py --lc-root /path/to/library-checker-problems
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import tempfile
import time
import tomllib


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--lc-root", type=Path, required=True)
    parser.add_argument("--work-dir", type=Path)
    args = parser.parse_args()
    repo = Path(__file__).resolve().parents[1]
    work = args.work_dir or Path(tempfile.mkdtemp(prefix="static_rectangle_official_"))
    work.mkdir(parents=True, exist_ok=True)
    official = work / "official"
    problem = official / "data_structure" / "static_rectangle_add_rectangle_sum"
    shutil.copytree(args.lc_root / "data_structure" / "static_rectangle_add_rectangle_sum", problem, dirs_exist_ok=True)
    shutil.copytree(args.lc_root / "common", official / "common", dirs_exist_ok=True)
    config = tomllib.loads((problem / "info.toml").read_text())
    hashes = json.loads((problem / "hash.json").read_text())
    (problem / "params.h").write_text("\n".join(f"#define {k} (long long){v}" for k, v in config["params"].items()) + "\n")
    cxx = os.environ.get("CXX", "clang++")

    def compile_cpp(src):
        binary = src.with_suffix("")
        subprocess.run([cxx, "-O2", "-std=c++17", "-I", str(official / "common"), str(src), "-o", str(binary)], check=True)
        return binary

    print(f"work directory: {work}", flush=True)
    checker = compile_cpp(problem / "checker.cpp")
    verifier = compile_cpp(problem / "verifier.cpp")
    correct = compile_cpp(problem / "sol" / "correct.cpp")
    implementation = (repo / "src" / "OfflineQuery" / "static_rectangle_add_rectangle_sum.rs").read_text()
    example = (repo / "examples" / "static_rectangle_add_rectangle_sum.rs").read_text()
    submission = work / "submission.rs"
    mint_source = (repo / "src" / "NumberTheory" / "mint.rs").read_text()
    submission.write_text(example.replace('include!("../src/OfflineQuery/static_rectangle_add_rectangle_sum.rs");', implementation)
        .replace('include!("../src/NumberTheory/mint.rs");', mint_source))
    solver = work / "solver"
    subprocess.run(["rustc", "--edition=2021", "-O", str(submission), "-o", str(solver)], check=True)
    cases = work / "cases"
    cases.mkdir(exist_ok=True)
    results = []
    for test in config["tests"]:
        gen = problem / "gen" / test["name"]
        binary = compile_cpp(gen) if gen.suffix == ".cpp" else None
        for seed in range(test["number"]):
            name = f"{gen.stem}_{seed:02}"
            inp = cases / f"{name}.in"
            answer = cases / f"{name}.ans"
            output = cases / f"{name}.out"
            if binary:
                with inp.open("wb") as f:
                    subprocess.run([str(binary), str(seed)], stdout=f, check=True)
            else:
                shutil.copyfile(gen.with_name(f"{name}.in"), inp)
            digest = hashlib.sha256(inp.read_bytes()).hexdigest()
            assert digest == hashes[inp.name], name + ": input hash mismatch"
            with inp.open("rb") as f:
                subprocess.run([str(verifier)], stdin=f, capture_output=True, check=True)
            with inp.open("rb") as f, answer.open("wb") as g:
                subprocess.run([str(correct)], stdin=f, stdout=g, timeout=30, check=True)
            start = time.perf_counter()
            with inp.open("rb") as f, output.open("wb") as g:
                p = subprocess.Popen([str(solver)], stdin=f, stdout=g, stderr=subprocess.PIPE)
                try:
                    p.wait(timeout=config["timelimit"])
                except subprocess.TimeoutExpired:
                    p.kill(); p.wait(); raise
                stderr = p.stderr.read().decode()
                p.stderr.close()
                (cases / f"{name}.stderr").write_text(stderr)
                assert p.returncode == 0, name + ": solver failure"
            elapsed = time.perf_counter() - start
            result = subprocess.run([str(checker), str(inp), str(output), str(answer)], capture_output=True, text=True)
            (cases / f"{name}.checker").write_text(result.stdout + result.stderr)
            assert result.returncode == 0, name + ": " + result.stdout + result.stderr
            results.append({"case": name, "seconds": elapsed, "input_sha256": digest, "output_sha256": hashlib.sha256(output.read_bytes()).hexdigest(), "checker": (result.stdout + result.stderr).strip(), "status": "AC"})
            print(f"{name}: AC {elapsed:.4f}s", flush=True)
    revision = subprocess.check_output(["git", "-C", str(args.lc_root), "rev-parse", "HEAD"], text=True).strip()
    data = {"problem": "static_rectangle_add_rectangle_sum", "official_revision": revision, "implementation_sha256": hashlib.sha256(implementation.encode()).hexdigest(), "platform": platform.platform(), "rustc": subprocess.check_output(["rustc", "-V"], text=True).strip(), "work_directory": str(work), "cases": results, "online_submission": False}
    (repo / "tests" / "static_rectangle_add_rectangle_sum_official_results.json").write_text(json.dumps(data, indent=2) + "\n")


if __name__ == "__main__":
    main()
