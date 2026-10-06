#!/usr/bin/env python3
"""Generate all registered official cases in an isolated copy and check Rust.

python3 tests/euclidean_mst/run_official.py --lc-root /path/to/library-checker-problems
Requires Python 3.11+, rustc, and a C++17 compiler. No download or submission.
Keeps the work directory (including failure input/output) for inspection.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import tempfile
import time
import tomllib


def run(cmd, **kwargs):
    return subprocess.run(cmd, check=True, **kwargs)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--lc-root", type=Path, required=True)
    parser.add_argument("--work-dir", type=Path)
    parser.add_argument("--results", type=Path, default=Path(__file__).with_name("results.json"))
    args = parser.parse_args()
    repo = Path(__file__).resolve().parents[2]
    work = args.work_dir or Path(tempfile.mkdtemp(prefix="arcaki_euclidean_mst_"))
    work.mkdir(parents=True, exist_ok=True)
    print(f"work directory: {work}", flush=True)
    official = work / "official"
    source = args.lc_root / "geo" / "euclidean_mst"
    problem = official / "geo" / "euclidean_mst"
    shutil.copytree(source, problem, dirs_exist_ok=True)
    shutil.copytree(args.lc_root / "common", official / "common", dirs_exist_ok=True)
    config = tomllib.loads((problem / "info.toml").read_text())
    hashes = json.loads((problem / "hash.json").read_text())
    (problem / "params.h").write_text("\n".join(
        f"#define {key} (long long){value}" for key, value in config["params"].items()) + "\n")
    cxx = os.environ.get("CXX", "clang++")

    def compile_cpp(src):
        binary = src.with_suffix("")
        run([cxx, "-O2", "-std=c++17", "-I", str(official / "common"), str(src), "-o", str(binary)])
        return binary

    checker = compile_cpp(problem / "checker.cpp")
    verifier = compile_cpp(problem / "verifier.cpp")
    correct = compile_cpp(problem / "sol" / "correct.cpp")
    solver = work / "euclidean_mst"
    run(["rustc", "--edition=2021", "-O", str(repo / "examples" / "euclidean_mst.rs"), "-o", str(solver)])
    # Also build a single-file submission, ensuring no module-path dependencies.
    implementation = (repo / "src" / "Gemetory" / "delaunay.rs").read_text()
    example = (repo / "examples" / "euclidean_mst.rs").read_text()
    main_source = example[example.index("use std::io") :].replace("delaunay::euclidean_mst", "euclidean_mst")
    submission = work / "submission.rs"
    submission.write_text(implementation + "\n" + main_source)
    run(["rustc", "--edition=2021", "-O", str(submission), "-o", str(work / "submission")])
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
                    run([str(binary), str(seed)], stdout=f)
            else:
                shutil.copyfile(gen.with_name(f"{name}.in"), inp)
            assert hashlib.sha256(inp.read_bytes()).hexdigest() == hashes[inp.name], name + ": input hash mismatch"
            with inp.open("rb") as f:
                run([str(verifier)], stdin=f, capture_output=True)
            with inp.open("rb") as f, answer.open("wb") as g:
                run([str(correct)], stdin=f, stdout=g, timeout=30)
            start = time.perf_counter()
            with inp.open("rb") as f, output.open("wb") as g:
                run([str(solver)], stdin=f, stdout=g, timeout=config["timelimit"])
            elapsed = time.perf_counter() - start
            n = int(inp.read_text().split("\n", 1)[0])
            assert len(output.read_text().split()) == 2 * (n - 1), name + ": wrong output length"
            checked = run([str(checker), str(inp), str(output), str(answer)], capture_output=True, text=True)
            results.append({"case": name, "n": n, "seconds": round(elapsed, 6), "status": "AC", "checker": checked.stderr.strip()})
            print(f"{name}: AC {elapsed:.3f}s", flush=True)
    # Recheck the flattened submission on the sample and a maximum random case.
    for name in ["example_00", "max_random_02"]:
        inp, output, answer = [cases / f"{name}.{ext}" for ext in ["in", "out", "ans"]]
        with inp.open("rb") as f, output.open("wb") as g:
            run([str(work / "submission")], stdin=f, stdout=g, timeout=config["timelimit"])
        run([str(checker), str(inp), str(output), str(answer)], capture_output=True)
    report = {
        "platform": platform.platform(),
        "rustc": run(["rustc", "--version"], capture_output=True, text=True).stdout.strip(),
        "official_input_hashes_verified": True,
        "official_verifier_passed": True,
        "standalone_submission_checked": ["example_00", "max_random_02"],
        "source_sha256": {str(f.relative_to(official)): hashlib.sha256(f.read_bytes()).hexdigest()
            for f in [problem / "info.toml", problem / "checker.cpp", problem / "sol" / "correct.cpp"]},
        "cases": results,
    }
    if platform.system() in ["Darwin", "Linux"]:
        # A fresh Python parent measures only this solver child, excluding C++
        # compilation. This also avoids macOS time -l's restricted sysctl call.
        measure = "import subprocess,resource,sys; subprocess.run(sys.argv[1:],check=True,stdout=subprocess.DEVNULL); print(resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss)"
        with (cases / "max_random_02.in").open("rb") as f:
            measured = run([sys.executable, "-c", measure, str(solver)], stdin=f, capture_output=True, text=True)
        peak_rss = int(measured.stdout.strip())
        report["max_random_02_peak_rss_bytes"] = peak_rss * (1024 if platform.system() == "Linux" else 1)
    args.results.parent.mkdir(parents=True, exist_ok=True)
    args.results.write_text(json.dumps(report, indent=2) + "\n")
    print(f"{len(results)} official cases accepted; results: {args.results}", flush=True)


if __name__ == "__main__":
    main()
