"""Ensure the Rust toolchain, rustup targets, and the Dioxus CLI (dx).

No Node.js: the backend, the Xerxes engine and the games are all Rust. Games build for the web with dx.
"""

from __future__ import annotations

import os
import shutil
import subprocess
import sys
from pathlib import Path

# Keep in step with `dioxus` in engine/Cargo.toml (and each games/<name>/Cargo.toml).
DX_VERSION = "0.7.10"

# rustup targets per platform (windows uses the host target, already installed; android
# targets are added per device by the engine/game commands).
PLATFORM_TARGETS: dict[str, tuple[str, ...]] = {
    "web": ("wasm32-unknown-unknown",),
}


def _cargo_bin() -> Path:
    return Path(os.environ.get("CARGO_HOME", Path.home() / ".cargo")) / "bin"


def _refresh_path_windows() -> None:
    """Reload Machine+User PATH into this process (winget does not update it)."""
    try:
        import winreg
    except ImportError:
        return

    parts: list[str] = []
    for hive, subkey in (
        (
            winreg.HKEY_LOCAL_MACHINE,
            r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment",
        ),
        (winreg.HKEY_CURRENT_USER, r"Environment"),
    ):
        try:
            with winreg.OpenKey(hive, subkey) as key:
                value, _ = winreg.QueryValueEx(key, "Path")
                if value:
                    parts.append(os.path.expandvars(str(value)))
        except OSError:
            continue

    extras: list[str] = []
    local = os.environ.get("LOCALAPPDATA", "")
    for candidate in (_cargo_bin(), Path(local) / "Microsoft" / "WinGet" / "Links"):
        if candidate.is_dir():
            extras.append(str(candidate))

    merged = os.pathsep.join([*extras, *parts])
    if merged:
        os.environ["PATH"] = merged


def _refresh_path_unix() -> None:
    extras: list[str] = []
    for candidate in (_cargo_bin(), Path("/opt/homebrew/bin"), Path("/usr/local/bin"), Path.home() / ".local" / "bin"):
        if candidate.is_dir():
            extras.append(str(candidate))
    if extras:
        os.environ["PATH"] = os.pathsep.join([*extras, os.environ.get("PATH", "")])


def refresh_path() -> None:
    if sys.platform == "win32":
        _refresh_path_windows()
    else:
        _refresh_path_unix()


def _run(cmd: list[str], **kwargs) -> int:
    print(f"[runtimes] {' '.join(cmd)}")
    if sys.platform == "win32" and cmd and not cmd[0].lower().endswith(".exe"):
        exe = shutil.which(cmd[0])
        if exe and exe.lower().endswith((".cmd", ".bat")):
            cmd = ["cmd", "/c", exe, *cmd[1:]]
    return subprocess.run(cmd, check=False, **kwargs).returncode


def _install_rustup() -> int:
    print("[runtimes] cargo not found. Installing Rust (rustup, stable)...")
    if sys.platform == "win32":
        if not shutil.which("winget"):
            print("error: winget not found. Install Rust from https://rustup.rs/", file=sys.stderr)
            return 1
        code = _run(
            [
                "winget", "install", "-e", "--id", "Rustlang.Rustup",
                "--accept-package-agreements", "--accept-source-agreements",
            ]
        )
        if code != 0:
            return code
        print(
            "[runtimes] Rust on Windows links with MSVC. If the first build fails with 'link.exe not found',\n"
            "           install Visual Studio Build Tools with the 'Desktop development with C++' workload."
        )
        return 0
    if not shutil.which("curl"):
        print("error: curl not found. Install Rust from https://rustup.rs/", file=sys.stderr)
        return 1
    return subprocess.run(
        "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal",
        shell=True,
        check=False,
    ).returncode


def ensure_rust(*, install: bool = True) -> int:
    """Return 0 when cargo is on PATH (installing rustup if allowed), else 1."""
    refresh_path()
    if shutil.which("cargo"):
        return 0
    if not install:
        return 1
    code = _install_rustup()
    if code != 0:
        return code
    refresh_path()
    if shutil.which("cargo"):
        if shutil.which("rustup"):
            _run(["rustup", "default", "stable"])
        return 0
    print("error: cargo still not found after installing rustup. Open a new terminal, then retry.", file=sys.stderr)
    return 1


def ensure_targets(platform: str) -> int:
    """rustup target add for one platform (no-op for windows)."""
    targets = PLATFORM_TARGETS.get(platform, ())
    if not targets:
        return 0
    if not shutil.which("rustup"):
        print(f"warn: rustup not found; make sure these targets are installed: {', '.join(targets)}", file=sys.stderr)
        return 0
    installed = subprocess.run(
        ["rustup", "target", "list", "--installed"], capture_output=True, text=True, check=False
    ).stdout.split()
    missing = [t for t in targets if t not in installed]
    if not missing:
        return 0
    return _run(["rustup", "target", "add", *missing])


def dx_version() -> str | None:
    dx = shutil.which("dx")
    if not dx:
        return None
    proc = subprocess.run([dx, "--version"], capture_output=True, text=True, check=False)
    parts = (proc.stdout or "").split()
    return parts[1] if len(parts) >= 2 else None


def ensure_dx() -> int:
    """Install dx (the Dioxus CLI) pinned to DX_VERSION. Prefers cargo-binstall (prebuilt, fast)."""
    refresh_path()
    current = dx_version()
    if current == DX_VERSION:
        return 0
    if current:
        print(f"[runtimes] dx {current} found; this kit pins dx {DX_VERSION}. Reinstalling...")
    else:
        print(f"[runtimes] dx not found. Installing dioxus-cli {DX_VERSION}...")
    if shutil.which("cargo-binstall") is None:
        _run(["cargo", "install", "cargo-binstall", "--locked"])
        refresh_path()
    if shutil.which("cargo-binstall"):
        code = _run(["cargo", "binstall", "-y", "--force", f"dioxus-cli@{DX_VERSION}"])
    else:
        code = _run(["cargo", "install", "dioxus-cli", "--version", DX_VERSION, "--locked", "--force"])
    if code != 0:
        return code
    refresh_path()
    if dx_version():
        return 0
    print("error: dx still not found after install. Add ~/.cargo/bin to PATH, then retry.", file=sys.stderr)
    return 1


def ensure_client_toolchain(platform: str = "web") -> int:
    """Rust + dx + the rustup targets one platform needs."""
    for step in (ensure_rust, ensure_dx):
        code = step()
        if code != 0:
            return code
    return ensure_targets(platform)
