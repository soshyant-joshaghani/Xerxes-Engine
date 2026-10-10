"""xerxes-ctrl cleanup: delete everything Git ignores across Xerxes-Engine.

Rust leaves a lot of output around the project (Cargo `target/` folders grow to many GB),
so cleanup removes every ignored file and folder, as Git reports them (`git ls-files
--others --ignored --exclude-standard --directory`): in the engine repository and in each
game's own repository (`games/<name>`). That covers `target/`, `dist/`, generated bundles
(`public/`), logs and caches, and whatever a `.gitignore` adds later. Tracked files are
never touched, and neither is what you would lose for good:

- secrets and local config: `.env`, `.env.local`, `.vscode/`, `letsencrypt/`
- your scratch and reference material: `__temp__/` (gitignored on purpose, e.g. the Unity dialogue
  package the plan studies; not build output, so cleanup never touches it)
- the control tool itself: `__ctrl__/.venv/` (its Python), `__ctrl__/.run/` (running servers)
- the games: the engine repository ignores `games/<name>` because each is its own repository;
  cleanup goes into each one and removes only what *its* Git ignores.

Everything removed is rebuilt on the next build (the first Bevy build takes a while).
"""

from __future__ import annotations

import argparse
import os
import shutil
import stat
import subprocess
import sys
from pathlib import Path

from lib.config import PROJECT

# Ignored, but never garbage (paths relative to the engine repository).
KEEP_PATHS = ("__ctrl__/.venv", "__ctrl__/.run", ".vscode", "letsencrypt", "__temp__")
# Ignored, but never garbage, in any repository.
KEEP_NAMES = {".env", ".env.local"}

GAMES = PROJECT / "games"
TEMPLATES = GAMES / "__templates__"


def _kept(repo: Path, rel: str) -> bool:
    if Path(rel).name in KEEP_NAMES:
        return True
    if repo == PROJECT:
        if any(rel == k or rel.startswith(k + "/") for k in KEEP_PATHS):
            return True
        # games/<name> are their own repositories (handled one by one); the templates are ours.
        if (rel == "games" or rel.startswith("games/")) and not rel.startswith("games/__templates__/"):
            return True
    return False


def ignored_in(repo: Path) -> list[Path] | None:
    """What Git ignores in `repo` (folders collapsed), minus what is kept. None: not a Git repo."""
    result = subprocess.run(
        ["git", "ls-files", "-z", "--others", "--ignored", "--exclude-standard", "--directory"],
        cwd=repo,
        capture_output=True,
    )
    if result.returncode != 0:
        return None
    found = []
    for rel in result.stdout.decode("utf-8", "replace").split("\0"):
        rel = rel.strip().rstrip("/")
        if rel and not _kept(repo, rel):
            found.append(repo / rel)
    return found


def _is_cargo_target(path: Path) -> bool:
    return (path / "CACHEDIR.TAG").is_file() or (path / ".rustc_info.json").is_file()


def _build_output(root: Path) -> list[Path]:
    """For a game folder without Git: its Cargo target/, dist/ and generated public/."""
    return [
        p
        for p in (root / "target", root / "dist", root / "public")
        if p.is_dir() and (p.name != "target" or _is_cargo_target(p))
    ]


def find_garbage() -> list[Path]:
    paths = ignored_in(PROJECT)
    if paths is None:
        print("error: Xerxes-Engine is not a Git repository; cleanup needs Git to know what is ignored.", file=sys.stderr)
        return []
    if GAMES.is_dir():
        for game in sorted(GAMES.iterdir()):
            if not game.is_dir() or game == TEMPLATES:
                continue
            own = ignored_in(game) if (game / ".git").exists() else None
            paths.extend(own if own is not None else _build_output(game))
    return sorted(set(paths))


def _size(path: Path) -> int:
    if path.is_file():
        return path.stat().st_size
    total = 0
    for current, _dirs, files in os.walk(path):
        for name in files:
            try:
                total += (Path(current) / name).stat().st_size
            except OSError:
                pass
    return total


def _human(n: float) -> str:
    for unit in ("B", "KB", "MB", "GB"):
        if n < 1024 or unit == "GB":
            return f"{n:.1f} {unit}" if unit != "B" else f"{int(n)} B"
        n /= 1024
    return f"{n:.1f} GB"


def _make_writable_and_retry(func, path, _exc):
    os.chmod(path, stat.S_IWRITE)
    func(path)


def _remove(path: Path) -> None:
    if path.is_dir() and not path.is_symlink():
        shutil.rmtree(path, onerror=_make_writable_and_retry)
    else:
        try:
            path.unlink()
        except PermissionError:
            os.chmod(path, stat.S_IWRITE)
            path.unlink()


def cmd_cleanup(args: argparse.Namespace) -> int:
    paths = find_garbage()
    if not paths:
        print("Nothing to clean: Git ignores nothing that is not kept.")
        return 0
    sizes = {p: _size(p) for p in paths}
    for p in sorted(paths, key=lambda p: -sizes[p]):
        slash = "/" if p.is_dir() else ""
        print(f"  {_human(sizes[p]):>10}  {p.relative_to(PROJECT).as_posix()}{slash}")
    print(f"  {_human(sum(sizes.values())):>10}  total ({len(paths)} ignored paths; kept: .env, .vscode, letsencrypt, __temp__, __ctrl__/.venv, __ctrl__/.run)")
    if args.dry_run:
        return 0
    if not args.yes:
        if not sys.stdin.isatty():
            print("Add --yes to delete without a prompt.", file=sys.stderr)
            return 1
        if input("Delete all of it? [y/N] ").strip().lower() not in ("y", "yes"):
            print("Cancelled.")
            return 0

    failed = []
    for p in paths:
        try:
            _remove(p)
        except OSError as err:
            failed.append(p)
            print(f"  could not remove {p.relative_to(PROJECT).as_posix()}: {err}", file=sys.stderr)
    print(f"Freed about {_human(sum(sizes[p] for p in paths if p not in failed))}.")
    if failed:
        print(
            "Some paths are in use. Stop running builds and servers (backend stop dev, dx, cargo, the engine) "
            "and the editor's rust-analyzer, then run cleanup again.",
            file=sys.stderr,
        )
        return 1
    return 0


def build_cleanup_subparser(sub: argparse._SubParsersAction) -> None:
    sp = sub.add_parser(
        "cleanup",
        help="Delete everything Git ignores (target/, dist/, public/, logs, caches) in the engine and every game",
    )
    sp.add_argument("--dry-run", action="store_true", help="List what would be deleted, with sizes")
    sp.add_argument("--yes", "-y", action="store_true", help="Do not ask for confirmation")
    sp.set_defaults(func=cmd_cleanup)
