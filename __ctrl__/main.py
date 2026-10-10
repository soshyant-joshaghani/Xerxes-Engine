#!/usr/bin/env python3
"""xerxes-ctrl: local dev stack, engine/game pipeline, tests, prod helpers and SSH VM ops.

Standalone (not foxg-ctrl). Run from this folder: xerxes-ctrl.bat …
"""

from __future__ import annotations

import argparse
import subprocess
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from lib.actions import clone_repo, push_env, run_raw, start_stack, status_stack, stop_stack
from lib.config import list_servers, require_host, resolve_targets
from lib.keys import resolve_ssh_key
from lib.ops import (
    backup_acme,
    backup_data,
    disk_df,
    disk_prune,
    reset_stack,
    restore_acme,
    restore_data,
    update_stack,
)
from lib.setup import setup_server, show_github_pubkey
from lib.ssh import (
    close_log,
    echo,
    print_result,
    run_command,
    run_parallel,
    save_results_json,
    with_retries,
)
from utils.backend.cli import build_backend_subparser, cmd_setup_local
from utils.cleanup.cli import build_cleanup_subparser
from utils.game.cli import build_engine_subparser, build_game_subparser
from utils.logs.cli import build_logs_subparser
from utils.gitflat.cli import build_flatten_subparser, build_restore_flat_subparser
from utils.prod.cli import build_prod_subparser
from utils.test.cli import build_test_subparser


def cmd_list(_: argparse.Namespace) -> int:
    servers = list_servers(include_disabled=True)
    print(f"{'ID':<18} {'PROJECT':<22} {'SITE':<10} {'HOST':<18} ON")
    print("-" * 80)
    for s in servers:
        on = "yes" if s.get("enabled", True) is not False else "no"
        host = s.get("host") or "(empty)"
        print(
            f"{s['id']:<18} {s.get('project', ''):<22} {s.get('site', ''):<10} "
            f"{host:<18} {on}"
        )
    return 0


def _dispatch(targets: list[dict], worker, *, parallel: bool = True) -> int:
    start = time.time()
    if parallel and len(targets) > 1:
        results = run_parallel(targets, worker)
    else:
        results = [
            with_retries(s, lambda client, _s=s: worker(client, _s)) for s in targets
        ]

    for r in results:
        print_result(r)

    ok = sum(1 for r in results if r["status"] == "success")
    offline = sum(1 for r in results if r["status"] == "offline")
    failed = len(results) - ok - offline
    path = save_results_json(results)
    echo(
        f"\nsummary: ok={ok} offline={offline} failed={failed}  ({time.time() - start:.1f}s)"
    )
    echo(f"results: {path}")
    close_log()
    return 0 if failed == 0 and offline == 0 else 1


def cmd_ping(args: argparse.Namespace) -> int:
    targets = resolve_targets(args.target, include_disabled=args.all_disabled)

    def worker(client, server):
        _ = server
        code, out, err = run_command(client, "hostname && uname -a", timeout=30)
        return {
            "status": "success" if code == 0 else "failed",
            "output": (out or err).strip(),
        }

    return _dispatch(targets, worker, parallel=True)


def cmd_setup(args: argparse.Namespace) -> int:
    targets = resolve_targets(args.target, include_disabled=args.all_disabled)

    def worker(client, server):
        return setup_server(client, server)

    return _dispatch(targets, worker, parallel=args.parallel)


def cmd_pubkey(args: argparse.Namespace) -> int:
    targets = resolve_targets(args.target, include_disabled=args.all_disabled)

    def worker(client, server):
        return show_github_pubkey(client, server)

    return _dispatch(targets, worker, parallel=True)


def cmd_env(args: argparse.Namespace) -> int:
    targets = resolve_targets(args.target, include_disabled=args.all_disabled)

    def worker(client, server):
        return push_env(client, server)

    return _dispatch(targets, worker, parallel=args.parallel)


def cmd_clone(args: argparse.Namespace) -> int:
    targets = resolve_targets(args.target, include_disabled=args.all_disabled)

    def worker(client, server):
        return clone_repo(client, server, pull_if_exists=not args.no_pull)

    return _dispatch(targets, worker, parallel=args.parallel)


def cmd_start(args: argparse.Namespace) -> int:
    targets = resolve_targets(args.target)

    def worker(client, server):
        return start_stack(client, server)

    return _dispatch(targets, worker, parallel=False)


def cmd_stop(args: argparse.Namespace) -> int:
    targets = resolve_targets(args.target)

    def worker(client, server):
        return stop_stack(client, server)

    return _dispatch(targets, worker, parallel=len(targets) > 1)


def cmd_status(args: argparse.Namespace) -> int:
    targets = resolve_targets(args.target)

    def worker(client, server):
        return status_stack(client, server)

    return _dispatch(targets, worker, parallel=True)


def cmd_exec(args: argparse.Namespace) -> int:
    targets = resolve_targets(args.target)

    def worker(client, server):
        return run_raw(
            client,
            server,
            args.cmd,
            in_project=args.in_project,
            timeout=args.timeout,
        )

    return _dispatch(targets, worker, parallel=args.parallel)


def cmd_connect(args: argparse.Namespace) -> int:
    targets = resolve_targets(args.target, include_disabled=args.all_disabled)
    if len(targets) != 1:
        raise SystemExit("connect requires exactly one server id (not all)")
    server = targets[0]
    host = require_host(server)
    try:
        key = resolve_ssh_key(server["key"])
    except FileNotFoundError as exc:
        raise SystemExit(str(exc)) from exc
    user = server.get("user", "ubuntu")
    port = int(server.get("port", 22))
    echo(f"ssh -i {key} -p {port} {user}@{host}")
    return subprocess.call(
        ["ssh", "-i", key, "-p", str(port), f"{user}@{host}"]
    )


def cmd_update(args: argparse.Namespace) -> int:
    targets = resolve_targets(args.target)

    def worker(client, server):
        return update_stack(client, server)

    return _dispatch(targets, worker, parallel=False)


def cmd_reset(args: argparse.Namespace) -> int:
    targets = resolve_targets(args.target)
    full = bool(getattr(args, "full", False))
    volumes = (full or bool(getattr(args, "volumes", False))) and not bool(
        getattr(args, "keep_volumes", False)
    )

    def worker(client, server):
        return reset_stack(
            client,
            server,
            volumes=volumes,
            app_images=not bool(getattr(args, "no_app_images", False)),
            all_images=full or bool(getattr(args, "all_images", False)),
            build_cache=not bool(getattr(args, "no_build_cache", False)),
            pull=not bool(getattr(args, "no_pull", False)),
            start_after=not bool(getattr(args, "no_start", False)),
        )

    return _dispatch(targets, worker, parallel=False)


def cmd_backup_acme(args: argparse.Namespace) -> int:
    targets = resolve_targets(args.target)

    def worker(client, server):
        return backup_acme(client, server)

    return _dispatch(targets, worker, parallel=True)


def cmd_restore_acme(args: argparse.Namespace) -> int:
    targets = resolve_targets(args.target)

    def worker(client, server):
        return restore_acme(client, server)

    return _dispatch(targets, worker, parallel=True)


def cmd_backup_data(args: argparse.Namespace) -> int:
    targets = resolve_targets(args.target)

    def worker(client, server):
        return backup_data(client, server)

    return _dispatch(targets, worker, parallel=False)


def cmd_restore_data(args: argparse.Namespace) -> int:
    targets = resolve_targets(args.target)

    def worker(client, server):
        return restore_data(client, server, stamp=getattr(args, "stamp", "latest"))

    return _dispatch(targets, worker, parallel=False)


def cmd_disk(args: argparse.Namespace) -> int:
    targets = resolve_targets(args.target)
    action = args.disk_action

    def worker(client, server):
        if action == "df":
            return disk_df(client, server)
        return disk_prune(
            client,
            server,
            action=action,
            volumes=bool(getattr(args, "volumes", False)),
        )

    return _dispatch(targets, worker, parallel=True)


def _add_target(p: argparse.ArgumentParser) -> None:
    p.add_argument(
        "target",
        nargs="?",
        default="xerxes",
        help="server id (default: xerxes) or 'all'",
    )
    p.add_argument(
        "--all-disabled",
        action="store_true",
        help="include servers with enabled=false",
    )


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="xerxes-ctrl",
        description=(
            "Single CLI for xerxes: local dev, tests, prod helpers, and SSH VM ops."
        ),
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=(
            "local:\n"
            "  xerxes-ctrl.bat setup-local\n"
            "  xerxes-ctrl.bat backend run dev             infra + backend (API, multiplayer, worker)\n"
            "  xerxes-ctrl.bat backend run dev --slim      no Redis, no worker\n"
            "  xerxes-ctrl.bat backend run prod|production [--slim]\n"
            "  xerxes-ctrl.bat test all\n"
            "  xerxes-ctrl.bat cleanup [--dry-run] [--yes]    delete everything Git ignores (keeps .env, .venv)\n"
            "\n"
            "platforms: web | windows (win) | android | ios (macOS + Xcode only); release builds go to dist/<name>/<platform>/\n"
            "\n"
            "engine app (engine/):\n"
            "  xerxes-ctrl.bat engine build <platform | all>          compile so run starts fast\n"
            "  xerxes-ctrl.bat engine run <platform> [--built]        live (web: hot reload, engine.localhost) or the published build\n"
            "  xerxes-ctrl.bat engine publish <platform | all>\n"
            "\n"
            "games (games/<name>, index 1.. A-Z):\n"
            "  xerxes-ctrl.bat game list\n"
            "  xerxes-ctrl.bat game build [index|name] [platform | all]\n"
            "  xerxes-ctrl.bat game dev|run [index|name] [platform]\n"
            "  xerxes-ctrl.bat game publish [index|name] [platform | all]\n"
            "  xerxes-ctrl.bat game new <name> [--from starter]\n"
            "\n"
            "templates (games/__templates__/, committed with the engine):\n"
            "  xerxes-ctrl.bat template list | build | dev|run | publish   (same arguments as game)\n"
            "  xerxes-ctrl.bat test engine | games | templates\n"
            "\n"
            "SSH (VM):\n"
            "  xerxes-ctrl.bat setup | clone | env | start | stop | update\n"
            "  xerxes-ctrl.bat backup-acme | status | connect"
        ),
    )
    sub = parser.add_subparsers(dest="command")

    p = sub.add_parser("list", help="List configured servers")
    p.set_defaults(func=cmd_list)

    p = sub.add_parser(
        "setup-local",
        help="Install Rust + dx + wasm32, cargo fetch backend and engine",
    )
    p.add_argument(
        "--force",
        action="store_true",
        help="Delete backend/target and engine/target first",
    )
    p.set_defaults(func=cmd_setup_local)

    for name, help_, fn in [
        ("ping", "Ping SSH hosts", cmd_ping),
        ("setup", "Bootstrap Docker on the VM", cmd_setup),
        ("pubkey", "Show GitHub deploy pubkey", cmd_pubkey),
        ("clone", "Clone / pull the xerxes repo on the VM", cmd_clone),
        ("env", "Upload safe/*.env to the VM", cmd_env),
        ("start", "Start production compose", cmd_start),
        ("stop", "Stop production compose", cmd_stop),
        ("status", "docker compose ps", cmd_status),
        ("update", "Safe update (keep volumes)", cmd_update),
        ("connect", "Open interactive SSH", cmd_connect),
        ("backup-acme", "Backup Let's Encrypt acme.json", cmd_backup_acme),
        ("restore-acme", "Restore acme.json", cmd_restore_acme),
        ("backup-data", "Backup data volumes", cmd_backup_data),
    ]:
        sp = sub.add_parser(name, help=help_)
        _add_target(sp)
        if name in {"setup", "env", "clone"}:
            sp.add_argument("--parallel", action="store_true", default=False)
        if name == "clone":
            sp.add_argument("--no-pull", action="store_true")
        sp.set_defaults(func=fn)

    sp = sub.add_parser("reset", help="Destructive reset (see --help)")
    _add_target(sp)
    sp.add_argument("--keep-volumes", action="store_true")
    sp.add_argument("--volumes", action="store_true")
    sp.add_argument("--all-images", action="store_true")
    sp.add_argument("--full", action="store_true")
    sp.add_argument("--no-app-images", action="store_true")
    sp.add_argument("--no-build-cache", action="store_true")
    sp.add_argument("--no-pull", action="store_true")
    sp.add_argument("--no-start", action="store_true")
    sp.set_defaults(func=cmd_reset)

    sp = sub.add_parser("restore-data", help="Restore data volumes")
    _add_target(sp)
    sp.add_argument("--stamp", default="latest")
    sp.set_defaults(func=cmd_restore_data)

    sp = sub.add_parser("exec", help="Run a remote shell command")
    _add_target(sp)
    sp.add_argument("cmd")
    sp.add_argument("--in-project", action="store_true")
    sp.add_argument("--timeout", type=int, default=120)
    sp.add_argument("--parallel", action="store_true")
    sp.set_defaults(func=cmd_exec)

    sp = sub.add_parser("disk", help="Disk / docker prune on the VM")
    _add_target(sp)
    sp.add_argument(
        "disk_action",
        choices=["df", "prune-builder", "prune-images", "prune-system", "prune-all"],
    )
    sp.add_argument("--volumes", action="store_true")
    sp.set_defaults(func=cmd_disk)

    build_backend_subparser(sub)
    build_logs_subparser(sub)
    build_flatten_subparser(sub)
    build_restore_flat_subparser(sub)
    build_test_subparser(sub)
    build_prod_subparser(sub)
    build_engine_subparser(sub)
    build_game_subparser(sub)
    build_cleanup_subparser(sub)
    return parser


def _repl() -> int:
    parser = build_parser()
    echo("xerxes-ctrl> (backend run dev | engine run web | game list | game dev | test all | cleanup | quit)")
    while True:
        try:
            line = input("xerxes-ctrl> ").strip()
        except (EOFError, KeyboardInterrupt):
            echo("")
            return 0
        if not line:
            continue
        if line.lower() in {"quit", "exit", "q"}:
            return 0
        try:
            args = parser.parse_args(line.split())
        except SystemExit:
            continue
        if not getattr(args, "func", None):
            parser.print_help()
            continue
        try:
            code = args.func(args)
            if code:
                echo(f"(exit {code})")
        except SystemExit as exc:
            if exc.code not in (0, None):
                echo(str(exc))


def main(argv: list[str] | None = None) -> int:
    argv = list(sys.argv[1:] if argv is None else argv)
    if not argv:
        return _repl()
    parser = build_parser()
    args = parser.parse_args(argv)
    if not getattr(args, "func", None):
        parser.print_help()
        return 2
    return int(args.func(args) or 0)


if __name__ == "__main__":
    raise SystemExit(main())
