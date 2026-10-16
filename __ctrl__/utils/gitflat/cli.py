"""xerxes-ctrl flatten / restore-flat — rewrite git history to a single root.

  flatten --yes          orphan commit message \"__init__\" + force-push
  restore-flat           local: fetch + reset --hard origin/<branch>
  restore-flat --server  production VM via SSH (git only; no compose rebuild)
"""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path

from lib.config import PROJECT

BRAND = "xerxes"
CTRL = "xerxes-ctrl"
DEFAULT_SERVER = "xerxes"

COMMIT_MSG = "__init__"
ORPHAN_TMP = "__flatten_tmp__"


def _git(args: list[str], *, cwd: Path | None = None, check: bool = False) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["git", *args],
        cwd=cwd or PROJECT,
        text=True,
        capture_output=True,
        check=check,
    )


def _git_out(args: list[str], *, cwd: Path | None = None) -> str:
    proc = _git(args, cwd=cwd)
    if proc.returncode != 0:
        err = (proc.stderr or proc.stdout or "").strip()
        raise RuntimeError(err or f"git {' '.join(args)} failed ({proc.returncode})")
    return (proc.stdout or "").strip()


def _ensure_repo() -> None:
    if not (PROJECT / ".git").exists():
        raise SystemExit(f"error: not a git repo: {PROJECT}")
    inside = _git(["rev-parse", "--is-inside-work-tree"])
    if inside.returncode != 0 or inside.stdout.strip() != "true":
        raise SystemExit(f"error: not a git work tree: {PROJECT}")


def _is_dirty() -> bool:
    proc = _git(["status", "--porcelain"])
    return bool((proc.stdout or "").strip())


def _current_branch() -> str:
    branch = _git_out(["rev-parse", "--abbrev-ref", "HEAD"])
    if not branch or branch == "HEAD":
        raise SystemExit(
            "error: detached HEAD — checkout a branch, or pass --branch <name>"
        )
    return branch


def cmd_flatten(args: argparse.Namespace) -> int:
    """Rewrite history to a single __init__ commit and force-push."""
    if not bool(getattr(args, "yes", False)):
        print(
            "error: flatten rewrites history and force-pushes.\n"
            f"  Re-run with:  {CTRL} flatten --yes\n"
            "  Optional:     --allow-dirty   --force  (plain --force instead of --force-with-lease)",
            file=sys.stderr,
        )
        return 2

    try:
        _ensure_repo()
    except SystemExit:
        raise
    except RuntimeError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1

    if _is_dirty() and not bool(getattr(args, "allow_dirty", False)):
        print(
            "error: working tree is dirty. Commit/stash first, or pass --allow-dirty\n"
            "  (dirty files will be included in the new __init__ root commit).",
            file=sys.stderr,
        )
        return 1

    try:
        branch = _current_branch()
        old_sha = _git_out(["rev-parse", "HEAD"])
    except (RuntimeError, SystemExit) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1

    print("=" * 60)
    print(f"WARNING: flatten will rewrite {PROJECT}")
    print(f"  branch:     {branch}")
    print(f"  old HEAD:   {old_sha}")
    print(f"  new commit: {COMMIT_MSG!r} (orphan root)")
    print(f"  push:       {'--force' if args.force else '--force-with-lease'} origin/{branch}")
    print("=" * 60)

    steps = [
        ["checkout", "--orphan", ORPHAN_TMP],
        ["add", "-A"],
        ["commit", "-m", COMMIT_MSG],
        ["branch", "-M", branch],
    ]
    for step in steps:
        proc = _git(step)
        if proc.returncode != 0:
            err = (proc.stderr or proc.stdout or "").strip()
            print(f"error: git {' '.join(step)} failed:\n{err}", file=sys.stderr)
            print(
                f"\nRecovery: old tip was {old_sha} (see: git reflog)\n"
                f"  If stuck on {ORPHAN_TMP}: git checkout -f {branch}",
                file=sys.stderr,
            )
            return 1
        out = (proc.stdout or "").strip()
        if out:
            print(out)

    push_args = ["push"]
    if bool(getattr(args, "force", False)):
        push_args.append("--force")
    else:
        push_args.append("--force-with-lease")
    push_args.extend(["origin", branch])

    proc = _git(push_args)
    if proc.returncode != 0:
        err = (proc.stderr or proc.stdout or "").strip()
        print(f"error: git {' '.join(push_args)} failed:\n{err}", file=sys.stderr)
        print(
            f"\nLocal history is already flattened (old tip {old_sha}).\n"
            f"  Retry push:  git push --force-with-lease origin {branch}\n"
            f"  Or hard:     {CTRL} flatten --yes --force   (after fixing remotes)",
            file=sys.stderr,
        )
        return 1
    print((proc.stdout or proc.stderr or "").strip() or f"pushed origin/{branch}")

    try:
        new_sha = _git_out(["rev-parse", "HEAD"])
        log1 = _git_out(["log", "-1", "--oneline"])
    except RuntimeError:
        new_sha, log1 = "?", ""
    print()
    print(f"Flattened: {old_sha[:12]} -> {new_sha}")
    if log1:
        print(f"  {log1}")
    print(f"Other clones / VMs:  {CTRL} restore-flat")
    print(f"Production VM:       {CTRL} restore-flat --server {DEFAULT_SERVER}")
    return 0


def _restore_flat_local(*, branch: str | None, yes: bool) -> int:
    try:
        _ensure_repo()
        br = branch or _current_branch()
    except SystemExit as exc:
        print(exc, file=sys.stderr)
        return 1
    except RuntimeError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1

    if _is_dirty() and not yes:
        print(
            "error: working tree is dirty; restore-flat will discard local changes.\n"
            f"  Re-run with:  {CTRL} restore-flat --yes",
            file=sys.stderr,
        )
        return 1

    print(f"[restore-flat] local {PROJECT} → origin/{br}")
    for step in (
        ["fetch", "origin"],
        ["reset", "--hard", f"origin/{br}"],
    ):
        proc = _git(step)
        if proc.returncode != 0:
            err = (proc.stderr or proc.stdout or "").strip()
            print(f"error: git {' '.join(step)} failed:\n{err}", file=sys.stderr)
            return 1
        out = (proc.stdout or proc.stderr or "").strip()
        if out:
            print(out)

    try:
        print(_git_out(["log", "-1", "--oneline"]))
    except RuntimeError:
        pass
    return 0


def _restore_flat_remote(*, server_id: str, branch: str | None, yes: bool) -> int:
    # Lazy: flatten is local-only and must not require paramiko.
    from lib.actions import restore_flat_stack
    from lib.config import resolve_targets
    from lib.ssh import with_retries

    try:
        targets = resolve_targets(server_id)
    except SystemExit as exc:
        print(exc, file=sys.stderr)
        return 1

    if len(targets) != 1:
        print(
            "error: restore-flat requires exactly one server "
            f"(got {len(targets)}; use --server <id>)",
            file=sys.stderr,
        )
        return 1

    server = targets[0]
    print(
        f"[restore-flat] SSH {server.get('id')} → fetch + reset --hard origin/<branch>"
        + (f" (forced branch={branch})" if branch else ""),
        flush=True,
    )
    if not yes:
        # Remote always discards uncommitted VM edits — require --yes for clarity.
        print(
            "error: production restore-flat discards uncommitted VM edits.\n"
            f"  Re-run with:  {CTRL} restore-flat --server {server_id} --yes",
            file=sys.stderr,
        )
        return 2

    try:
        result = with_retries(
            server,
            lambda client, _s=server: restore_flat_stack(
                client, _s, branch=branch
            ),
        )
    except TimeoutError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1

    out = (result.get("output") or "").strip()
    if out:
        print(out)
    if result.get("status") != "success":
        print(f"error: {result.get('error') or 'failed'}", file=sys.stderr)
        return 1

    print()
    print("Git tree adopted. Rebuild containers when ready:")
    print(f"  {CTRL} update")
    print(f"  {CTRL} start")
    return 0


def cmd_restore_flat(args: argparse.Namespace) -> int:
    """Adopt rewritten remote history on laptop or production VM."""
    yes = bool(getattr(args, "yes", False))
    branch = (getattr(args, "branch", None) or "").strip() or None
    server = (getattr(args, "server", None) or "").strip()

    # --server without value shouldn't happen (argparse); empty = local
    if server:
        return _restore_flat_remote(server_id=server, branch=branch, yes=yes)
    return _restore_flat_local(branch=branch, yes=yes)


def build_flatten_subparser(sub: argparse._SubParsersAction) -> None:
    sp = sub.add_parser(
        "flatten",
        help="Rewrite git history to a single __init__ commit and force-push",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=(
            "DANGER: rewrites published history. Collaborators and VMs must run "
            "restore-flat afterward (plain git pull will fail).\n"
            "\n"
            "examples:\n"
            f"  {CTRL} flatten --yes\n"
            f"  {CTRL} flatten --yes --allow-dirty\n"
            f"  {CTRL} flatten --yes --force\n"
        ),
    )
    sp.add_argument(
        "--yes",
        action="store_true",
        help="required confirmation (force-push rewrite)",
    )
    sp.add_argument(
        "--allow-dirty",
        action="store_true",
        help="include uncommitted/untracked files in the new __init__ commit",
    )
    sp.add_argument(
        "--force",
        action="store_true",
        help="use git push --force instead of --force-with-lease",
    )
    sp.set_defaults(func=cmd_flatten)


def build_restore_flat_subparser(sub: argparse._SubParsersAction) -> None:
    sp = sub.add_parser(
        "restore-flat",
        help="Adopt flattened remote history (local or production VM)",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=(
            "After someone ran flatten + force-push, use this instead of git pull.\n"
            "\n"
            "examples:\n"
            f"  {CTRL} restore-flat\n"
            f"  {CTRL} restore-flat --yes\n"
            f"  {CTRL} restore-flat --server {DEFAULT_SERVER} --yes\n"
            f"  {CTRL} restore-flat --branch main --yes\n"
        ),
    )
    sp.add_argument(
        "--server",
        default="",
        metavar="ID",
        help=f"SSH servers.json id (omit for local; e.g. {DEFAULT_SERVER})",
    )
    sp.add_argument(
        "--branch",
        default="",
        metavar="NAME",
        help="branch to reset to (default: current branch)",
    )
    sp.add_argument(
        "--yes",
        action="store_true",
        help="discard dirty local/VM changes (required for --server)",
    )
    sp.set_defaults(func=cmd_restore_flat)
