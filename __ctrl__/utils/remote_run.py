"""Shared helpers to run __ctrl__/remote scripts from the CLI."""

from __future__ import annotations

import subprocess
import sys

from lib.config import ROOT

REMOTE = ROOT / "remote"


def run_remote_script(
    stem: str,
    *,
    brand: str,
    extra: list[str] | None = None,
    linux_only: frozenset[str] | None = None,
) -> int:
    extra = list(extra or [])
    linux_only = linux_only or frozenset({"setup-ubuntu"})
    if sys.platform == "win32" and stem in linux_only:
        print(
            f"error: {stem} is a Linux VM script.\n"
            f"  On the VM:  bash __ctrl__/remote/{stem}.sh\n"
            f"  From laptop: {brand}-ctrl.bat setup",
            file=sys.stderr,
        )
        return 1
    if sys.platform == "win32":
        script = REMOTE / f"{stem}.bat"
        if not script.is_file():
            script_sh = REMOTE / f"{stem}.sh"
            if script_sh.is_file():
                cmd = ["bash", str(script_sh), *extra]
                print(f"[{brand}] {' '.join(cmd)}")
                return subprocess.run(cmd, cwd=ROOT).returncode
            print(f"error: missing {script}", file=sys.stderr)
            return 1
        cmd = ["cmd", "/c", str(script), *extra]
    else:
        script = REMOTE / f"{stem}.sh"
        if not script.is_file():
            print(f"error: missing {script}", file=sys.stderr)
            return 1
        cmd = ["bash", str(script), *extra]

    print(f"[{brand}] {' '.join(cmd)}")
    return subprocess.run(cmd, cwd=ROOT).returncode
