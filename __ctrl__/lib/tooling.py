"""Host tooling shared by the dev, engine/game and test commands: cargo/dx launching, the
rust:1 Docker fallback for the backend, and `setup_local`."""

from __future__ import annotations

import shutil
import subprocess
import sys

from lib.config import PROJECT
from lib.runtimes import ensure_client_toolchain, ensure_rust


def require_cmd(name: str) -> str | None:
    path = shutil.which(name)
    if not path:
        print(f"error: '{name}' not found on PATH", file=sys.stderr)
        return None
    return path


def tool_argv(name: str, *args: str) -> list[str] | None:
    """Launch cargo/dx, including Windows *.cmd shims."""
    exe = require_cmd(name)
    if not exe:
        return None
    if sys.platform == "win32":
        return ["cmd", "/c", exe, *args]
    return [exe, *args]


def rust_in_docker() -> bool:
    """No host Rust toolchain: run the backend's cargo inside the official rust image instead."""
    return shutil.which("cargo") is None and shutil.which("docker") is not None


def rust_docker_argv(*cargo_args: str, name: str | None = None, publish: str | None = None) -> list[str]:
    argv = ["docker", "run", "--rm"]
    if name:
        argv += ["--name", name]
    if publish:
        argv += ["-p", publish]
    argv += [
        "--add-host", "host.docker.internal:host-gateway",
        "-e", "APP_HOST=0.0.0.0",
        "-e", "POSTGRES_SERVER=host.docker.internal",
        "-e", "REDIS_HOST=host.docker.internal",
        "-v", f"{PROJECT}:/work",
        "-v", "xerxes-cargo:/usr/local/cargo/registry",
        "-v", "xerxes-target:/work/backend/target",
        "-w", "/work/backend",
        "rust:1",
        "cargo", *cargo_args,
    ]
    return argv


def ensure_env() -> None:
    env = PROJECT / ".env"
    example = PROJECT / ".env.example"
    if env.is_file():
        return
    if example.is_file():
        shutil.copy(example, env)
        print(f"created {env} from .env.example")
    else:
        print(f"warn: no .env or .env.example in {PROJECT}", file=sys.stderr)


def _setup_backend() -> int:
    if rust_in_docker():
        print("[xerxes] cargo not found on PATH: the backend runs in a rust:1 container (no hot reload)")
        print("[xerxes] cargo build --bins (once, so the API and the worker do not race on the crate cache)")
        return subprocess.run(rust_docker_argv("build", "--bins"), cwd=PROJECT).returncode
    argv = tool_argv("cargo", "fetch")
    if argv is None:
        return 1
    print("[xerxes] cargo fetch in backend")
    return subprocess.run(argv, cwd=PROJECT / "backend").returncode


def _setup_engine() -> int:
    engine = PROJECT / "engine"
    if not (engine / "Cargo.toml").is_file():
        return 0
    if ensure_client_toolchain("web") != 0:
        print(
            "error: the Xerxes engine and games need Rust and dx on the host (https://rustup.rs/).",
            file=sys.stderr,
        )
        return 1
    argv = tool_argv("cargo", "fetch")
    if argv is None:
        return 1
    print("[xerxes] cargo fetch in engine")
    return subprocess.run(argv, cwd=engine).returncode


def setup_local(*, backend: bool = True, engine: bool = True) -> int:
    """Rust toolchain, then what each side needs: `cargo fetch` for the backend (or a rust:1
    container build when there is no host cargo), and dx + wasm32 + `cargo fetch` for the engine."""
    ensure_env()
    if ensure_rust() != 0 and not (backend and not engine and rust_in_docker()):
        return 1
    if backend:
        code = _setup_backend()
        if code != 0:
            return code
    if engine:
        code = _setup_engine()
        if code != 0:
            return code
    print("Local environment ready.")
    return 0
