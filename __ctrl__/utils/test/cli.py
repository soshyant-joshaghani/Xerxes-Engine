"""xerxes-ctrl test."""

from __future__ import annotations

import argparse
import subprocess
import sys

from lib.config import PROJECT
from lib.tooling import rust_docker_argv, rust_in_docker, setup_local, tool_argv


def _run(argv_name: str, args: list[str], cwd) -> int:
    argv = tool_argv(argv_name, *args)
    if not argv:
        return 1
    print(f"[xerxes] {' '.join(argv)}")
    return subprocess.run(argv, cwd=cwd).returncode


def _run_backend() -> int:
    print("[xerxes] Backend tests (cargo test)")
    if rust_in_docker():
        print("[xerxes] cargo not found on PATH: running cargo test in a rust:1 container")
        return subprocess.run(rust_docker_argv("test"), cwd=PROJECT).returncode
    return _run("cargo", ["test"], PROJECT / "backend")


def _run_contract(args: argparse.Namespace) -> int:
    script = PROJECT / "tests" / "contract" / "contract_test.py"
    if not script.is_file():
        print(f"error: missing {script}", file=sys.stderr)
        return 1
    cmd = [sys.executable, str(script), "--base", args.base, "--local"]
    if not args.no_jobs:
        cmd.append("--jobs")
    print("[contract] " + " ".join(cmd))
    return subprocess.run(cmd, cwd=PROJECT).returncode


def cmd_test(args: argparse.Namespace) -> int:
    if args.target == "contract":
        return _run_contract(args)
    target = args.target
    # Only the toolchains the chosen tests need: backend crates and/or dx + wasm32 + engine crates.
    code = setup_local(backend=target in ("backend", "all"), engine=target != "backend")
    if code != 0:
        return code
    if target in ("backend", "all"):
        code = _run_backend()
        if code != 0:
            print("Backend tests failed.", file=sys.stderr)
            return code
    if target in ("engine", "all"):
        from utils.game.cli import run_engine_tests

        code = run_engine_tests()
        if code != 0:
            print("Engine tests failed.", file=sys.stderr)
            return code
    if target in ("templates", "all"):
        from utils.game.cli import run_template_tests

        code = run_template_tests()
        if code != 0:
            print("Template tests failed.", file=sys.stderr)
            return code
    if target in ("games", "all"):
        from utils.game.cli import run_game_tests

        code = run_game_tests()
        if code != 0:
            print("Game tests failed.", file=sys.stderr)
            return code
    if target == "all":
        print("\nAll tests passed.")
    return 0


def build_test_subparser(sub: argparse._SubParsersAction) -> None:
    sp = sub.add_parser(
        "test",
        help="Run tests (all = backend + engine + templates + games; contract needs a running API)",
    )
    sp.add_argument("target", nargs="?", default="all", choices=("all", "backend", "engine", "templates", "games", "contract"))
    sp.add_argument("--base", default="http://localhost:8000", help="API origin for `test contract`")
    sp.add_argument(
        "--no-jobs",
        action="store_true",
        help="`test contract`: skip the Redis job check (slim mode: backend run dev --slim)",
    )
    sp.set_defaults(func=cmd_test)
