"""xerxes-ctrl prod — local compose.yml helpers."""

from __future__ import annotations

import argparse

from utils.remote_run import run_remote_script

BRAND = "xerxes"
CTRL = "xerxes-ctrl"

# Running, stopping and wiping the stack is `backend run|stop|purge|reset prod` (utils/backend).
# What is left here are the one-off production chores.
ACTIONS: dict[str, str] = {
    "backup-acme": "backup-acme",
    "restore-acme": "restore-acme",
    "prune-build": "prune-docker-build",
    "migrate-acme": "migrate-letsencrypt-from-volume",
    "setup-ubuntu": "setup-ubuntu",
}


def cmd_prod(args: argparse.Namespace) -> int:
    action = args.prod_action
    stem = ACTIONS.get(action)
    if not stem:
        print(f"error: unknown prod action {action!r}")
        return 1
    extra: list[str] = []
    if action == "migrate-acme" and getattr(args, "volume", None):
        extra.append(args.volume)
    return run_remote_script(stem, brand=BRAND, extra=extra)


def _prod_help(_: argparse.Namespace) -> int:
    print(
        f"usage: {CTRL}.bat prod {{backup-acme,restore-acme,"
        "prune-build,migrate-acme,setup-ubuntu}\n"
        "\n"
        f"examples:\n"
        f"  {CTRL}.bat prod backup-acme\n"
        f"  {CTRL}.bat prod prune-build\n"
        f"\nrun / stop / reset the stack: {CTRL}.bat backend run|stop|reset prod\n"
    )
    return 0


def build_prod_subparser(sub: argparse._SubParsersAction) -> None:
    sp = sub.add_parser(
        "prod",
        help="Production chores: SSL backup/restore, build-cache prune, Ubuntu VM setup (run the stack: backend run prod)",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=(
            f"examples:\n"
            f"  {CTRL}.bat prod backup-acme\n"
            f"  {CTRL}.bat prod prune-build\n"
            f"\n"
            f"Run the stack:  {CTRL}.bat backend run prod [--slim]\n"
            "SSH: start | stop | update | backup-acme"
        ),
    )
    sp.set_defaults(func=_prod_help)
    actions = sp.add_subparsers(dest="prod_action", required=False)

    for name, help_ in [
        ("backup-acme", "Copy acme.json to parent .foxg-ssl-backups"),
        ("restore-acme", "Restore acme.json from parent backup"),
        ("prune-build", "Backup SSL then docker builder prune -af"),
        ("migrate-acme", "Copy acme.json from legacy Docker volume"),
        ("setup-ubuntu", "One-time Ubuntu VM bootstrap (Linux only)"),
    ]:
        action_sp = actions.add_parser(name, help=help_)
        if name == "migrate-acme":
            action_sp.add_argument(
                "volume",
                nargs="?",
                default=None,
                help="legacy volume name (optional)",
            )
        action_sp.set_defaults(func=cmd_prod, prod_action=name)

