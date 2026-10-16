"""xerxes-ctrl backend — run/stop the backend (API + multiplayer): dev (host apps + compose.dev.yml) or prod (compose.yml)."""

from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
import time
import webbrowser
from pathlib import Path

from lib.config import PROJECT
from lib.tooling import rust_docker_argv, rust_in_docker, require_cmd, setup_local, tool_argv

RUN_ORDER = ("infra", "apps")
STOP_ORDER = ("apps", "infra")
TARGETS = ("infra", "apps", "all")

COMPOSE_PROJECT_NAME = "xerxes-dev"
COMPOSE_FILE = "compose.dev.yml"
APP_STARTUP_TIMEOUT_S = 180.0
FULL_INFRA_SERVICES = ("db", "redis", "proxy", "adminer")
SLIM_INFRA_SERVICES = ("db", "proxy", "adminer")

API_PORT = 8000

# (window_title, port) — host processes; port None = no listener (e.g. background worker).
# The dev stack is infra + backend (the API and multiplayer series share one process for now). The engine runs with `engine run <platform>`, games with
# `game dev <index|name>` (or Play in the engine for published builds).
APP_PORTS = (
    ("xerxes-backend", API_PORT),
    ("xerxes-worker", None),
)

STACK_OPEN_URLS: dict[str, tuple[str, ...]] = {
    "infra": (
        "http://adminer.localhost/",
        "http://localhost:8080/",
    ),
    "apps": (
        "http://api.localhost/docs",
        "http://api.localhost/sdoc",
    ),
}


def _resolve_targets(target: str, *, order: tuple[str, ...]) -> tuple[str, ...]:
    if target == "all":
        return order
    return (target,)


def _open_stack_urls(targets: tuple[str, ...]) -> None:
    urls: list[str] = []
    for t in targets:
        for url in STACK_OPEN_URLS.get(t, ()):
            if url not in urls:
                urls.append(url)
    for url in urls:
        print(f"opening {url}")
        try:
            webbrowser.open(url)
        except Exception as exc:
            print(f"warn: could not open {url}: {exc}", file=sys.stderr)


def _compose(*args: str) -> int:
    if not require_cmd("docker"):
        return 1
    cmd = ["docker", "compose", "-f", COMPOSE_FILE, *args]
    print(f"[xerxes] {' '.join(cmd)}")
    return subprocess.run(cmd, cwd=PROJECT).returncode


def _wait_postgres(service: str, *, timeout_s: float = 120.0) -> int:
    deadline = time.time() + timeout_s
    while time.time() < deadline:
        proc = subprocess.run(
            [
                "docker",
                "compose",
                "-f",
                COMPOSE_FILE,
                "exec",
                "-T",
                service,
                "pg_isready",
                "-U",
                "postgres",
            ],
            cwd=PROJECT,
            capture_output=True,
            check=False,
        )
        if proc.returncode == 0:
            return 0
        time.sleep(2)
    print(f"error: {service} not ready within {timeout_s:.0f}s", file=sys.stderr)
    return 1


def _wait_redis(*, timeout_s: float = 60.0) -> int:
    deadline = time.time() + timeout_s
    while time.time() < deadline:
        proc = subprocess.run(
            [
                "docker",
                "compose",
                "-f",
                COMPOSE_FILE,
                "exec",
                "-T",
                "redis",
                "redis-cli",
                "ping",
            ],
            cwd=PROJECT,
            capture_output=True,
            check=False,
        )
        if proc.returncode == 0 and b"PONG" in (proc.stdout or b""):
            return 0
        time.sleep(2)
    print(f"error: redis not ready within {timeout_s:.0f}s", file=sys.stderr)
    return 1



def _port_listening(port: int) -> bool:
    if sys.platform == "win32":
        return bool(_pids_on_port_windows(port))
    return bool(_pids_on_port_unix(port))


def _wait_ports(ports: tuple[int, ...], *, timeout_s: float = APP_STARTUP_TIMEOUT_S) -> list[int]:
    deadline = time.time() + timeout_s
    pending = set(ports)
    last_log = time.time()
    while pending and time.time() < deadline:
        ready = {p for p in pending if _port_listening(p)}
        pending -= ready
        if pending:
            now = time.time()
            if now - last_log >= 15.0:
                print(f"  still waiting for port(s) {sorted(pending)}...")
                last_log = now
            time.sleep(0.5)
    return sorted(pending)


def _pids_on_port_windows(port: int) -> set[int]:
    proc = subprocess.run(
        ["netstat", "-ano", "-p", "tcp"],
        capture_output=True,
        text=True,
        check=False,
    )
    pids: set[int] = set()
    for line in (proc.stdout or "").splitlines():
        parts = line.split()
        if len(parts) < 5 or parts[0].upper() != "TCP":
            continue
        if parts[3].upper() != "LISTENING":
            continue
        local = parts[1]
        if not (local.endswith(f":{port}") or local.endswith(f"]:{port}")):
            continue
        try:
            pid = int(parts[4])
        except ValueError:
            continue
        if pid > 0:
            pids.add(pid)
    return pids


def _pids_on_port_unix(port: int) -> set[int]:
    pids: set[int] = set()
    if shutil.which("lsof"):
        proc = subprocess.run(
            ["lsof", "-ti", f":{port}"],
            capture_output=True,
            text=True,
            check=False,
        )
        for line in (proc.stdout or "").splitlines():
            line = line.strip()
            if line.isdigit():
                pids.add(int(line))
        return pids
    if shutil.which("fuser"):
        proc = subprocess.run(
            ["fuser", f"{port}/tcp"],
            capture_output=True,
            text=True,
            check=False,
        )
        for token in (proc.stdout or "").replace("\n", " ").split():
            digits = "".join(c for c in token if c.isdigit())
            if digits:
                pids.add(int(digits))
    return pids


def _kill_pid(pid: int) -> None:
    if sys.platform == "win32":
        subprocess.run(
            ["taskkill", "/F", "/T", "/PID", str(pid)],
            capture_output=True,
            check=False,
        )
    else:
        try:
            os.kill(pid, 15)
        except ProcessLookupError:
            return
        except PermissionError:
            subprocess.run(["kill", "-9", str(pid)], check=False)


def _start_infra(*, slim: bool = False) -> int:
    services = SLIM_INFRA_SERVICES if slim else FULL_INFRA_SERVICES

    code = _compose("up", "-d", *services)
    if code != 0:
        return code
    code = _compose("up", "-d", "--force-recreate", "--no-deps", "proxy")
    if code != 0:
        return code

    print("[xerxes] Waiting for Postgres...")
    if _wait_postgres("db") != 0:
        return 1

    if not slim:
        print("[xerxes] Waiting for Redis...")
        if _wait_redis() != 0:
            return 1

    # SQL migrations run when the API process starts.
    if slim:
        print("Infra ready (slim): DB + Adminer + Traefik — no Redis.")
    else:
        print("Infra ready (full): DB + Redis + Adminer + Traefik.")
    return 0


def _quote_cmd_arg(arg: str) -> str:
    """Quote a token for cmd.exe if it contains whitespace/special chars."""
    if not arg or any(c in arg for c in ' \t"&|<>()^%'):
        return '"' + arg.replace('"', '""') + '"'
    return arg


def _start_named_console(
    title: str,
    args: list[str],
    *,
    cwd: Path,
    env: dict[str, str] | None = None,
) -> None:
    """Open a titled console (`start "title" cmd /k …`).

    Uses shell=True so cmd.exe parses the line; argv is joined with cmd-safe quoting
    (avoids Python list2cmdline turning quotes into \\\").
    """
    inner = " ".join(_quote_cmd_arg(a) for a in args)
    cmdline = f'start "{title}" /D {_quote_cmd_arg(str(cwd))} cmd /k {inner}'
    subprocess.Popen(cmdline, shell=True, env=env)


def _app_specs(*, slim: bool) -> list[tuple[str, list[str], Path, dict[str, str], int | None]]:
    env = dict(os.environ)
    if rust_in_docker():
        api = rust_docker_argv("run", "--bin", "api", name="xerxes-backend", publish="8000:8000")
    else:
        api = tool_argv("cargo", "run", "--bin", "api")
    if not api:
        return []
    specs: list[tuple[str, list[str], Path, dict[str, str], int | None]] = [
        ("xerxes-backend", api, PROJECT / "backend", env, API_PORT),
    ]
    if not slim:
        if rust_in_docker():
            worker = rust_docker_argv("run", "--bin", "worker", name="xerxes-worker")
        else:
            worker = tool_argv("cargo", "run", "--bin", "worker")
        if not worker:
            return []
        specs.append(("xerxes-worker", worker, PROJECT / "backend", env, None))
    return specs



def _spawn_apps_windows(*, slim: bool = False) -> int:
    specs = _app_specs(slim=slim)
    if not specs:
        print("error: could not build app commands. Run setup-local.", file=sys.stderr)
        return 1

    wait_ports: list[int] = []
    for title, args, cwd, env, port in specs:
        if port is not None and _port_listening(port):
            print(f"  {title}: already listening on :{port}")
            continue
        print(f"  {title}: starting (new console)...")
        _start_named_console(title, args, cwd=cwd, env=env)
        if port is not None:
            wait_ports.append(port)

    if wait_ports:
        print(f"  waiting for app port(s) {wait_ports}...")
        missing = _wait_ports(tuple(wait_ports))
        if missing:
            print(
                f"error: apps did not listen on port(s) {missing} within {int(APP_STARTUP_TIMEOUT_S)}s. "
                "Check the new console windows for errors.",
                file=sys.stderr,
            )
            return 1
    return 0


def _spawn_apps_unix(*, slim: bool = False) -> int:
    specs = _app_specs(slim=slim)
    if not specs:
        print("error: could not build app commands. Run setup-local.", file=sys.stderr)
        return 1

    wait_ports: list[int] = []
    for title, cmd, cwd, env, port in specs:
        if port is not None and _port_listening(port):
            print(f"  {title}: already listening on :{port}")
            continue
        from utils.logs.cli import open_host_log
        try:
            log_fh, log_path = open_host_log(title)
        except KeyError:
            log_fh, log_path = None, None
        if log_path is not None:
            print(f"  {title}: starting (log: __ctrl__/logs/{log_path.name})")
        else:
            print(f"  {title}: starting...")
        subprocess.Popen(
            cmd,
            cwd=cwd,
            env=env,
            stdout=log_fh if log_fh is not None else subprocess.DEVNULL,
            stderr=subprocess.STDOUT if log_fh is not None else subprocess.DEVNULL,
            start_new_session=True,
        )
        if log_fh is not None:
            log_fh.close()
        if port is not None:
            wait_ports.append(port)

    if wait_ports:
        print(f"  waiting for app port(s) {wait_ports}...")
        missing = _wait_ports(tuple(wait_ports))
        if missing:
            print(
                f"error: apps did not listen on port(s) {missing} within {int(APP_STARTUP_TIMEOUT_S)}s.",
                file=sys.stderr,
            )
            return 1
    return 0


def _start_apps(*, slim: bool = False) -> int:
    _stop_apps()

    profile = "slim" if slim else "full"
    print(f"[xerxes] Starting apps ({profile}): backend API" + ("" if slim else " + worker"))
    if sys.platform == "win32":
        code = _spawn_apps_windows(slim=slim)
    else:
        code = _spawn_apps_unix(slim=slim)
    if code != 0:
        return code

    print()
    print("Dev stack ready:")
    print("  Games:       http://<name>.play.localhost  (published web builds)")
    print("  API docs:    http://api.localhost/docs")
    print("  Scalar:      http://api.localhost/sdoc")
    print("  Adminer:     http://adminer.localhost")
    print("  Traefik:     http://localhost:8080")
    print()
    print(f"Direct: http://localhost:{API_PORT}/docs  http://localhost:{API_PORT}/sdoc")
    if not slim:
        print("Worker: xerxes-worker (Redis queue)")
    print("Engine: xerxes-ctrl.bat engine run web   (http://engine.localhost; or windows | android, --built for the published build)")
    print("Games:  xerxes-ctrl.bat game list | game dev <index|name> [web|windows|android] | game publish <index|name>")
    print("Stop with: xerxes-ctrl.bat backend stop dev")
    return 0


def _stop_apps() -> int:
    print("[xerxes] Stopping host apps (backend API / worker)...")
    killed: set[int] = set()
    in_docker = rust_in_docker()
    if shutil.which("docker"):
        subprocess.run(
            ["docker", "rm", "-f", "xerxes-backend", "xerxes-worker"],
            capture_output=True,
            check=False,
        )
    for _title, port in APP_PORTS:
        if port is None:
            continue
        if in_docker and port == API_PORT:
            continue  # published by Docker: never kill the Docker proxy process
        if sys.platform == "win32":
            pids = _pids_on_port_windows(port)
        else:
            pids = _pids_on_port_unix(port)
        for pid in pids:
            if pid in killed:
                continue
            print(f"  kill pid {pid} (port {port})")
            _kill_pid(pid)
            killed.add(pid)

    if sys.platform == "win32":
        for title, _port in APP_PORTS:
            subprocess.run(
                ["taskkill", "/F", "/FI", f"WINDOWTITLE eq {title}*"],
                capture_output=True,
                check=False,
            )

    if not killed:
        print("  (no listeners found on app ports; closed matching consoles if any)")
    else:
        print("Host apps stopped.")
    return 0


def _compose_stop() -> int:
    code = _compose("stop")
    if code != 0:
        return code
    print("Infra stopped (containers kept).")
    return 0


def _compose_down(*, volumes: bool = False) -> int:
    cmd = ["down", "--remove-orphans"]
    if volumes:
        cmd.append("-v")
        print("[xerxes] compose down -v  (WIPES VOLUMES)")
    code = _compose(*cmd)
    if code != 0:
        return code
    if volumes:
        _cleanup_leftover_volumes()
        print("Infra removed; named volumes wiped.")
    else:
        print("Infra removed (volumes kept).")
    return 0


def _list_volume_names() -> list[str]:
    proc = subprocess.run(
        ["docker", "volume", "ls", "-q"],
        capture_output=True,
        text=True,
        check=False,
    )
    if proc.returncode != 0:
        return []
    return [line.strip() for line in (proc.stdout or "").splitlines() if line.strip()]


def _cleanup_leftover_volumes() -> None:
    prefix_us = f"{COMPOSE_PROJECT_NAME}_"
    for name in _list_volume_names():
        if name.startswith(prefix_us):
            print(f"  removing leftover volume {name}")
            subprocess.run(
                ["docker", "volume", "rm", "-f", name],
                capture_output=True,
                check=False,
            )
    print("  pruning dangling anonymous volumes...")
    proc = subprocess.run(
        ["docker", "volume", "prune", "-f"],
        capture_output=True,
        text=True,
        check=False,
    )
    out = ((proc.stdout or "") + (proc.stderr or "")).strip()
    if out:
        for line in out.splitlines():
            print(f"  {line}")


def _run_one(target: str, *, slim: bool = False) -> int:
    if target == "infra":
        return _start_infra(slim=slim)
    if target == "apps":
        return _start_apps(slim=slim)
    print(f"error: unknown target {target!r}", file=sys.stderr)
    return 1


def _stop_one(target: str) -> int:
    if target == "infra":
        return _compose_stop()
    if target == "apps":
        return _stop_apps()
    print(f"error: unknown target {target!r}", file=sys.stderr)
    return 1


def _down_one(target: str) -> int:
    if target == "infra":
        return _compose_down(volumes=False)
    if target == "apps":
        return _stop_apps()
    print(f"error: unknown target {target!r}", file=sys.stderr)
    return 1


def _wipe_one(target: str) -> int:
    if target == "infra":
        return _compose_down(volumes=True)
    if target == "apps":
        return _stop_apps()
    print(f"error: unknown target {target!r}", file=sys.stderr)
    return 1


def _run_targets(targets: tuple[str, ...], *, slim: bool) -> int:
    """`.env` + backend toolchain once (the backend stack never needs the engine's), then each target."""
    code = setup_local(engine=False)
    if code != 0:
        return code
    for t in targets:
        code = _run_one(t, slim=slim)
        if code != 0:
            return code
    _open_stack_urls(targets)
    return 0


# ---- production (compose.yml on this machine, through __ctrl__/remote scripts) --------------


def _prod_script(stem: str, *extra: str) -> int:
    from utils.remote_run import run_remote_script

    return run_remote_script(stem, brand="xerxes", extra=list(extra))


def _prod_run(*, slim: bool) -> int:
    return _prod_script("start-prod", *(["--slim"] if slim else []))


# ---- commands: `backend <action> <dev|prod|production>` -------------------------------------

ENVIRONMENTS = {"dev": "dev", "prod": "prod", "production": "prod"}
ENV_CHOICES = list(ENVIRONMENTS)


def _environment(args: argparse.Namespace) -> str:
    return ENVIRONMENTS[args.environment]


def _dev_targets(args: argparse.Namespace, *, order: tuple[str, ...]) -> tuple[str, ...]:
    return _resolve_targets(getattr(args, "only", None) or "all", order=order)


def _reject_only(args: argparse.Namespace) -> bool:
    if getattr(args, "only", None) and _environment(args) == "prod":
        print("error: --only is for the dev stack (prod is one compose project)", file=sys.stderr)
        return True
    return False


def cmd_backend_run(args: argparse.Namespace) -> int:
    slim = bool(getattr(args, "slim", False))
    if _reject_only(args):
        return 2
    if _environment(args) == "prod":
        return _prod_run(slim=slim)
    return _run_targets(_dev_targets(args, order=RUN_ORDER), slim=slim)


def cmd_backend_stop(args: argparse.Namespace) -> int:
    if _reject_only(args):
        return 2
    if _environment(args) == "prod":
        return _prod_script("stop-prod")
    for t in _dev_targets(args, order=STOP_ORDER):
        code = _stop_one(t)
        if code != 0:
            return code
    return 0


def cmd_backend_down(args: argparse.Namespace) -> int:
    if _reject_only(args):
        return 2
    if _environment(args) == "prod":
        # Production `stop` is already a compose down that keeps volumes and SSL.
        return _prod_script("stop-prod")
    for t in _dev_targets(args, order=STOP_ORDER):
        code = _down_one(t)
        if code != 0:
            return code
    return 0


def cmd_backend_purge(args: argparse.Namespace) -> int:
    if _reject_only(args):
        return 2
    if _environment(args) == "prod":
        print("WARNING: backend purge prod wipes the Postgres and Redis volumes and the app images (SSL is kept).")
        return _prod_script("reset-prod")
    print(
        "WARNING: backend purge dev removes compose named volumes "
        "(Postgres data for infra). Stack will stay down."
    )
    for t in _dev_targets(args, order=STOP_ORDER):
        code = _wipe_one(t)
        if code != 0:
            return code
    return 0


def cmd_backend_reset(args: argparse.Namespace) -> int:
    slim = bool(getattr(args, "slim", False))
    if _reject_only(args):
        return 2
    if _environment(args) == "prod":
        code = cmd_backend_purge(args)
        return code if code != 0 else _prod_run(slim=slim)
    print(
        "WARNING: backend reset dev removes compose named volumes "
        "(Postgres data for infra)."
    )
    for t in _dev_targets(args, order=STOP_ORDER):
        code = _wipe_one(t)
        if code != 0:
            return code
    return _run_targets(_dev_targets(args, order=RUN_ORDER), slim=slim)


EXAMPLES = (
    "examples:\n"
    "  xerxes-ctrl.bat backend run dev\n"
    "  xerxes-ctrl.bat backend run dev --slim\n"
    "  xerxes-ctrl.bat backend run prod\n"
    "  xerxes-ctrl.bat backend run production --slim\n"
    "  xerxes-ctrl.bat backend stop dev\n"
    "  xerxes-ctrl.bat backend down dev\n"
    "  xerxes-ctrl.bat backend purge dev --only infra\n"
    "  xerxes-ctrl.bat backend reset dev\n"
    "  xerxes-ctrl.bat backend logs dev api\n"
    "\n"
    "dev:    compose infra (Postgres, Redis, Traefik, Adminer) + host API and worker\n"
    "prod:   compose.yml on this machine (also: production); the VM runs the same stack via SSH start/stop\n"
    "  --slim:         no Redis and no worker (CRUD and auth work; the lightweight mode)\n"
    "  --only infra|apps (dev): just the containers, or just the host processes\n"
    "stop:   stop the stack (containers kept; dev stops host apps)\n"
    "down:   remove containers and networks (volumes kept)\n"
    "purge:  wipe the data volumes and leave the stack down (prod: also the app images; SSL is kept)\n"
    "reset:  purge, then run again\n"
    "Engine: engine run; games: game dev. The API and multiplayer series share this backend until they are split (__docs__/backend.md)."
)


def _backend_help(args: argparse.Namespace) -> int:
    _ = args
    print("usage: xerxes-ctrl.bat backend {run,stop,down,purge,reset,logs} {dev,prod,production}\n")
    print(EXAMPLES)
    return 0


def build_backend_subparser(sub: argparse._SubParsersAction) -> None:
    sp = sub.add_parser(
        "backend",
        help="Run/stop/down/purge/reset/logs the backend (API + multiplayer), dev or prod",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=EXAMPLES,
    )
    sp.set_defaults(func=_backend_help)

    actions = sp.add_subparsers(dest="backend_action", required=False)

    for name, help_, fn in [
        ("run", "Start the backend stack", cmd_backend_run),
        ("stop", "Stop it (containers kept)", cmd_backend_stop),
        ("down", "Remove containers/networks (keep volumes)", cmd_backend_down),
        ("purge", "Wipe the data volumes and leave the stack down", cmd_backend_purge),
        ("reset", "Wipe the data volumes then start again", cmd_backend_reset),
    ]:
        action_sp = actions.add_parser(name, help=help_)
        action_sp.add_argument(
            "environment",
            choices=ENV_CHOICES,
            metavar="{dev,prod|production}",
            help="dev: host apps + dev compose; prod (production): compose.yml",
        )
        action_sp.add_argument(
            "--only",
            choices=["infra", "apps"],
            help="dev only: just the containers (infra) or just the host processes (apps)",
        )
        if name in ("run", "reset"):
            action_sp.add_argument(
                "--slim",
                action="store_true",
                help="slim runtime: skip Redis and the background worker (official lightweight mode)",
            )
        action_sp.set_defaults(func=fn)

    from utils.logs.cli import attach_backend_logs_subparser

    attach_backend_logs_subparser(actions, ENV_CHOICES)


def cmd_setup_local(args: argparse.Namespace) -> int:
    """Top-level: Rust toolchain, dx, and crates for the backend and the engine."""
    if getattr(args, "force", False):
        for build_dir in (PROJECT / "backend" / "target", PROJECT / "engine" / "target"):
            if build_dir.is_dir():
                print(f"[xerxes] Removing {build_dir.relative_to(PROJECT)} (--force)...")
                shutil.rmtree(build_dir, ignore_errors=True)
    return setup_local()
