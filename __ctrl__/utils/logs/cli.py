"""xerxes-ctrl logs — production (SSH), local prod, and dev logs.

Command map:
  logs <svc>                  production VM via SSH (compose.yml)
  backend logs prod <svc>     local compose.yml stack
  backend logs dev <svc>      local host files + compose.dev.yml
"""

from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

from lib.config import ROOT, resolve_targets
from lib.ssh import with_retries

PROJECT = ROOT.parent
LOGS_DIR = ROOT / "logs"

BRAND = "xerxes"
CTRL = "xerxes-ctrl"
DEFAULT_SERVER = "xerxes"

DEV_COMPOSE_FILE = "compose.dev.yml"
LOCAL_PROD_COMPOSE_FILES = ("compose.yml",)

# Host process titles (from utils.backend.cli._app_specs) → log file basename
HOST_TITLE_TO_LOG: dict[str, str] = {
    f"{BRAND}-backend": "backend.log",
    f"{BRAND}-worker": "worker.log",
}

HOST_ALIASES: dict[str, str] = {
    "backend": "backend.log",
    "api": "backend.log",
    "uvicorn": "backend.log",
    "worker": "worker.log",
    "arq": "worker.log",
}

DEV_COMPOSE_ALIASES: dict[str, str] = {
    "db": "db",
    "postgres": "db",
    "redis": "redis",
    "minio": "minio",
    "minio-init": "minio-init",
    "proxy": "proxy",
    "traefik": "proxy",
    "adminer": "adminer",
    "blender": "blender-worker",
    "blender-worker": "blender-worker",
}

DEV_COMPOSE_SERVICES = (
    "db",
    "redis",
    "minio",
    "minio-init",
    "proxy",
    "adminer",
    "blender-worker",
)

# Production containers (compose.yml + traefik include; optional minio)
PROD_COMPOSE_ALIASES: dict[str, str] = {
    "backend": "backend",
    "api": "backend",
    "uvicorn": "backend",
    "worker": "worker",
    "arq": "worker",
    "db": "db",
    "postgres": "db",
    "redis": "redis",
    "adminer": "adminer",
    "prestart": "prestart",
    "proxy": "proxy",
    "traefik": "proxy",
    "blender": "blender-worker",
    "blender-worker": "blender-worker",
    "minio": "minio",
    "minio-init": "minio-init",
}

PROD_COMPOSE_SERVICES = (
    "backend",
    "worker",
    "db",
    "redis",
    "adminer",
    "prestart",
    "proxy",
    "blender-worker",
    "minio",
    "minio-init",
)


def ensure_logs_dir() -> Path:
    LOGS_DIR.mkdir(parents=True, exist_ok=True)
    return LOGS_DIR


def host_log_path_for_title(title: str) -> Path:
    name = HOST_TITLE_TO_LOG.get(title)
    if not name:
        raise KeyError(f"unknown host app title: {title}")
    return ensure_logs_dir() / name


def open_host_log(title: str):
    """Truncate + open a host log for subprocess stdout/stderr (caller closes)."""
    path = host_log_path_for_title(title)
    fh = path.open("w", encoding="utf-8", errors="replace", buffering=1)
    stamp = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M:%S UTC")
    fh.write(f"=== {title} started {stamp} ===\n")
    fh.flush()
    return fh, path


def _compose_env() -> dict[str, str]:
    env = dict(os.environ)
    env["COMPOSE_BAKE"] = "false"
    return env


def _tail_file(path: Path, *, lines: int, follow: bool) -> int:
    if not path.is_file():
        print(
            f"No log file yet: {path}\n"
            "Start host apps first, then retry:\n"
            f"  {CTRL} backend run dev --only apps   (or: backend run dev)\n"
            "On Windows, backend/worker open in separate consoles — "
            "file logs are written on macOS/Linux.",
            file=sys.stderr,
        )
        return 1

    try:
        text = path.read_text(encoding="utf-8", errors="replace")
    except OSError as exc:
        print(f"error: cannot read {path}: {exc}", file=sys.stderr)
        return 1

    chunk = text.splitlines()
    for line in chunk[-max(1, lines) :]:
        print(line)

    if not follow:
        return 0

    print(f"\n--- following {path.name} (Ctrl+C to stop) ---", flush=True)
    try:
        with path.open("r", encoding="utf-8", errors="replace") as fh:
            fh.seek(0, os.SEEK_END)
            while True:
                line = fh.readline()
                if line:
                    print(line, end="", flush=True)
                else:
                    time.sleep(0.2)
    except KeyboardInterrupt:
        print()
        return 0


def _local_compose_logs(
    compose_files: tuple[str, ...] | list[str],
    service: str | None,
    *,
    lines: int,
    follow: bool,
) -> int:
    if not shutil.which("docker"):
        print("error: 'docker' not found on PATH", file=sys.stderr)
        return 1

    cmd: list[str] = ["docker", "compose"]
    for f in compose_files:
        cmd.extend(["-f", f])
    cmd.extend(["logs", f"--tail={max(1, lines)}"])
    if follow:
        cmd.append("--follow")
    if service:
        cmd.append(service)

    label = service or "all"
    files = " ".join(compose_files)
    print(
        f"[logs] {files} {label}" + ("  (Ctrl+C to stop)" if follow else ""),
        flush=True,
    )
    try:
        proc = subprocess.run(cmd, cwd=PROJECT, env=_compose_env())
        return int(proc.returncode or 0)
    except KeyboardInterrupt:
        print()
        return 0


def _print_dev_catalog() -> None:
    print("Dev — host apps (written under __ctrl__/logs/ on macOS/Linux):")
    for alias, name in sorted(set(HOST_ALIASES.items())):
        print(f"  {alias:<14} -> logs/{name}")
    print()
    print(f"Dev — compose containers ({DEV_COMPOSE_FILE}):")
    for svc in DEV_COMPOSE_SERVICES:
        print(f"  {svc}")
    print()
    print("examples:")
    print(f"  {CTRL} backend logs dev backend")
    print(f"  {CTRL} backend logs dev api --tail 100")
    print(f"  {CTRL} backend logs dev db")
    print(f"  {CTRL} backend logs dev proxy --no-follow")
    print(f"  {CTRL} backend logs dev list")


def _print_prod_catalog(*, remote: bool) -> None:
    where = "production VM (SSH)" if remote else "local compose.yml smoke"
    print(f"Prod — {where}:")
    for svc in PROD_COMPOSE_SERVICES:
        print(f"  {svc}")
    print()
    print("aliases: api→backend, traefik→proxy, arq→worker, …")
    print()
    print("examples:")
    if remote:
        print(f"  {CTRL} logs api")
        print(f"  {CTRL} logs backend --tail 200")
        print(f"  {CTRL} logs db --no-follow")
        print(f"  {CTRL} logs --server {DEFAULT_SERVER} worker")
        print(f"  {CTRL} logs list")
    else:
        print(f"  {CTRL} backend logs prod api")
        print(f"  {CTRL} backend logs prod backend --tail 200")
        print(f"  {CTRL} backend logs prod list")


def cmd_dev_logs(args: argparse.Namespace) -> int:
    """Local development: host file logs + compose.dev.yml containers."""
    target = (getattr(args, "logs_target", None) or getattr(args, "target", None) or "").strip().lower()
    follow = not bool(getattr(args, "no_follow", False))
    lines = int(getattr(args, "tail", 80) or 80)

    if not target or target in {"list", "help", "?"}:
        _print_dev_catalog()
        return 0

    if target in {"infra", "compose", "containers"}:
        return _local_compose_logs(
            (DEV_COMPOSE_FILE,), None, lines=lines, follow=follow
        )

    if target in HOST_ALIASES:
        path = ensure_logs_dir() / HOST_ALIASES[target]
        if sys.platform == "win32" and not path.is_file():
            print(
                f"Host app '{target}' runs in a titled console on Windows "
                f"({BRAND}-{target if target not in {'api', 'uvicorn'} else 'backend'}).\n"
                "On macOS/Linux, logs are under __ctrl__/logs/ — use this command there.\n"
                f"For Docker infra: {CTRL} backend logs dev db|redis|proxy|…",
                file=sys.stderr,
            )
            return 1
        return _tail_file(path, lines=lines, follow=follow)

    if target in DEV_COMPOSE_ALIASES:
        return _local_compose_logs(
            (DEV_COMPOSE_FILE,),
            DEV_COMPOSE_ALIASES[target],
            lines=lines,
            follow=follow,
        )

    print(f"Unknown backend dev logs target: {target!r}\n", file=sys.stderr)
    _print_dev_catalog()
    return 2


def cmd_prod_local_logs(args: argparse.Namespace) -> int:
    """Local production smoke stack (compose.yml on this machine)."""
    target = (getattr(args, "logs_target", None) or getattr(args, "target", None) or "").strip().lower()
    follow = not bool(getattr(args, "no_follow", False))
    lines = int(getattr(args, "tail", 80) or 80)

    if not target or target in {"list", "help", "?"}:
        _print_prod_catalog(remote=False)
        return 0

    if target in {"infra", "compose", "containers", "all"}:
        return _local_compose_logs(
            LOCAL_PROD_COMPOSE_FILES, None, lines=lines, follow=follow
        )

    if target in PROD_COMPOSE_ALIASES:
        return _local_compose_logs(
            LOCAL_PROD_COMPOSE_FILES,
            PROD_COMPOSE_ALIASES[target],
            lines=lines,
            follow=follow,
        )

    print(f"Unknown backend prod logs target: {target!r}\n", file=sys.stderr)
    _print_prod_catalog(remote=False)
    return 2


def cmd_prod_remote_logs(args: argparse.Namespace) -> int:
    """Production VM logs via SSH (servers.json compose_files)."""
    from lib.actions import logs_stack

    target = (getattr(args, "logs_target", None) or "").strip().lower()
    follow = not bool(getattr(args, "no_follow", False))
    lines = int(getattr(args, "tail", 80) or 80)
    server_id = (getattr(args, "server", None) or DEFAULT_SERVER).strip()

    if not target or target in {"list", "help", "?"}:
        _print_prod_catalog(remote=True)
        return 0

    service: str | None
    if target in {"infra", "compose", "containers", "all"}:
        service = None
    elif target in PROD_COMPOSE_ALIASES:
        service = PROD_COMPOSE_ALIASES[target]
    else:
        print(f"Unknown production logs target: {target!r}\n", file=sys.stderr)
        _print_prod_catalog(remote=True)
        return 2

    try:
        targets = resolve_targets(server_id)
    except SystemExit as exc:
        print(exc, file=sys.stderr)
        return 1

    if len(targets) != 1:
        print(
            "error: production logs require exactly one server "
            f"(got {len(targets)}; use --server <id>)",
            file=sys.stderr,
        )
        return 1

    server = targets[0]
    label = service or "all"
    print(
        f"[logs] SSH {server.get('id')} → compose logs {label}"
        + ("  (Ctrl+C to stop)" if follow else ""),
        flush=True,
    )

    try:
        result = with_retries(
            server,
            lambda client, _s=server: logs_stack(
                client,
                _s,
                service,
                lines=lines,
                follow=follow,
            ),
        )
    except KeyboardInterrupt:
        print()
        return 0
    except TimeoutError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1

    status = result.get("status")
    if status != "success":
        err = result.get("error") or "failed"
        out = (result.get("output") or "").strip()
        if out and not follow:
            print(out)
        print(f"error: {err}", file=sys.stderr)
        return 1
    return 0


def _add_common_log_flags(sp: argparse.ArgumentParser) -> None:
    sp.add_argument(
        "--tail",
        type=int,
        default=80,
        metavar="N",
        help="lines to show before follow (default 80)",
    )
    sp.add_argument(
        "--no-follow",
        action="store_true",
        help="print recent lines and exit (do not stream)",
    )


def build_logs_subparser(sub: argparse._SubParsersAction) -> None:
    """Top-level: production VM logs over SSH."""
    sp = sub.add_parser(
        "logs",
        help="Follow production container logs on the VM (SSH)",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=(
            "examples:\n"
            f"  {CTRL} logs api\n"
            f"  {CTRL} logs backend --tail 200\n"
            f"  {CTRL} logs db --no-follow\n"
            f"  {CTRL} logs list\n"
            "\n"
            f"Dev (local):  {CTRL} backend logs dev api | backend logs dev db\n"
            f"Local prod:   {CTRL} backend logs prod api\n"
        ),
    )
    sp.add_argument(
        "logs_target",
        nargs="?",
        default="list",
        help="api|backend|worker|db|redis|proxy|…|list",
    )
    sp.add_argument(
        "--server",
        default=DEFAULT_SERVER,
        metavar="ID",
        help=f"servers.json id (default: {DEFAULT_SERVER})",
    )
    _add_common_log_flags(sp)
    sp.set_defaults(func=cmd_prod_remote_logs)


def attach_backend_logs_subparser(
    backend_actions: argparse._SubParsersAction, environments: list[str]
) -> None:
    """Nest under: backend logs <dev|prod|production> [target]"""
    sp = backend_actions.add_parser(
        "logs",
        help="Follow local dev (host apps, compose.dev.yml) or prod (compose.yml) logs",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=(
            "examples:\n"
            f"  {CTRL} backend logs dev backend\n"
            f"  {CTRL} backend logs dev api --tail 100\n"
            f"  {CTRL} backend logs dev db\n"
            f"  {CTRL} backend logs prod api --tail 200\n"
            f"  {CTRL} backend logs dev list\n"
        ),
    )
    sp.add_argument(
        "environment",
        choices=environments,
        metavar="{dev,prod|production}",
        help="dev: host apps + compose.dev.yml; prod (production): compose.yml",
    )
    sp.add_argument(
        "logs_target",
        nargs="?",
        default="list",
        help="backend|worker|db|redis|proxy|…|list",
    )
    _add_common_log_flags(sp)
    sp.set_defaults(func=cmd_backend_logs)


def cmd_backend_logs(args: argparse.Namespace) -> int:
    if args.environment == "dev":
        return cmd_dev_logs(args)
    return cmd_prod_local_logs(args)
