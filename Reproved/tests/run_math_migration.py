#!/usr/bin/env python3
"""旧パスの互換入口。本体srcを検証する。"""
from pathlib import Path
import runpy
runpy.run_path(str(Path(__file__).resolve().parents[2] / "tests" / "run_math_migration.py"), run_name="__main__")
