"""The engine's vendored crates: patched copies of crates Bevy builds against.

A vendored crate is upstream's source, trimmed to what the engine builds, plus one small patch
of our own — kept as a diff (`xerxes.patch`) so it can be read, reviewed and **re-applied to a
newer upstream**. That is the whole point of keeping the patch as a file rather than only in the
source: updating is `vendor update <crate> <version>`, not an afternoon of merging by hand.

    xerxes-ctrl vendor list                   what is vendored, and whether its patch still applies
    xerxes-ctrl vendor update winit 0.30.14   fetch that version, trim it, re-apply our patch
    xerxes-ctrl vendor patch cpal             regenerate xerxes.patch from the current source

See `engine/vendors/<crate>/XERXES.md` for what each patch does and when to drop it.
"""

from __future__ import annotations

import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

from lib.config import PROJECT

VENDORS = PROJECT / "engine" / "vendors"
PATCH_FILE = "xerxes.patch"

# What a vendored crate keeps. Everything else upstream ships (examples, tests, docs, changelogs,
# CI and packaging files) is not built here and only makes the tree harder to read. `assets` is
# what `include_str!`/`include_bytes!` read at build time (blitz-dom's default stylesheet and font).
KEEP = ("src", "assets", "Cargo.toml", "build.rs", "LICENSE", "LICENSE-APACHE", "LICENSE-MIT", "README.md")
# Ours, never upstream's.
OURS = ("XERXES.md", PATCH_FILE)
# Cargo sections a trimmed copy has no use for: their files are gone. Dev-dependencies are only
# the examples' and tests', and they are named at every depth (`target."cfg(…)".dev-dependencies.x`).
DROP_SECTIONS = ("example", "test", "bench")
DEV_SECTION = re.compile(r"(^|\.)dev-dependencies(\.|$)")
# Prose that upstream keeps **inside** `src/`, where the rule above cannot see it, with the lines
# that declare it. winit compiles its whole release history into the crate as doc comments: 165 KB
# of markdown behind `#[cfg(doc)]`, which no build here reads and GitHub already serves.
PROSE_IN_SRC = {
    "winit": [("src/changelog", "src/lib.rs", "#[cfg(any(doc, doctest, test))]\npub mod changelog;\n")],
}


def _crates() -> list[Path]:
    if not VENDORS.is_dir():
        return []
    return sorted(p for p in VENDORS.iterdir() if (p / "Cargo.toml").is_file())


def _version(crate: Path) -> str:
    text = (crate / "Cargo.toml").read_text(encoding="utf-8")
    match = re.search(r'^version\s*=\s*"([^"]+)"', text, re.M)
    return match.group(1) if match else "?"


def _registry_source(name: str, version: str) -> Path | None:
    """Upstream's own copy of `name-version`, as cargo unpacked it."""
    home = Path.home() / ".cargo" / "registry" / "src"
    for index in sorted(home.glob("*")):
        found = index / f"{name}-{version}"
        if found.is_dir():
            return found
    return None


def _fetch(name: str, version: str) -> Path | None:
    """Makes cargo unpack `name-version` into its registry, by asking for it from a scratch crate."""
    found = _registry_source(name, version)
    if found:
        return found
    print(f"[xerxes] fetching {name} {version} from crates.io")
    with tempfile.TemporaryDirectory() as tmp:
        scratch = Path(tmp) / "fetch"
        (scratch / "src").mkdir(parents=True)
        (scratch / "src" / "lib.rs").write_text("", encoding="utf-8")
        (scratch / "Cargo.toml").write_text(
            f'[package]\nname = "xerxes-vendor-fetch"\nversion = "0.0.0"\nedition = "2021"\n\n'
            f'[dependencies]\n{name} = "={version}"\n',
            encoding="utf-8",
        )
        code = subprocess.run(["cargo", "fetch"], cwd=scratch).returncode
        if code != 0:
            print(f"[xerxes] cargo could not fetch {name} {version}")
            return None
    return _registry_source(name, version)


def trim(crate: Path) -> None:
    """Leaves only what the engine builds, and takes the Cargo sections that named the rest."""
    for path in sorted(crate.iterdir()):
        if path.name in KEEP or path.name in OURS:
            continue
        shutil.rmtree(path) if path.is_dir() else path.unlink()

    cargo = crate / "Cargo.toml"
    lines, out, keep = cargo.read_text(encoding="utf-8").split("\n"), [], True
    for line in lines:
        stripped = line.strip()
        if stripped.startswith("[") and stripped.endswith("]"):
            section = stripped.strip("[]")
            keep = section not in DROP_SECTIONS and not DEV_SECTION.search(section)
        if keep:
            out.append(line)
    text = re.sub(r"\n{3,}", "\n\n", "\n".join(out)).rstrip() + "\n"
    # The packaging `include` list may name files a trimmed copy no longer has.
    for gone in ('    "/FEATURES.md",\n', '    "/docs",\n', '    "/examples",\n', '    "/tests",\n'):
        text = text.replace(gone, "")
    cargo.write_text(text, encoding="utf-8", newline="\n")

    # Prose buried in `src/`, and the `mod` that names it (see PROSE_IN_SRC).
    for folder, declared_in, declaration in PROSE_IN_SRC.get(crate.name, ()):
        prose = crate / folder
        if prose.is_dir():
            shutil.rmtree(prose)
        source = crate / declared_in
        text = source.read_text(encoding="utf-8")
        if declaration in text:
            source.write_text(text.replace(declaration, "", 1), encoding="utf-8", newline="\n")


def _trimmed_copy(upstream: Path, into: Path) -> Path:
    """Upstream as `trim` would leave it: what our patch is the difference from.

    `into` must be named after the crate: some trim rules are per crate (PROSE_IN_SRC).
    """
    shutil.copytree(upstream, into)
    for path in into.rglob("*"):
        if path.is_file():
            path.chmod(0o644)
    trim(into)
    return into


def make_patch(crate: Path, upstream: Path) -> str:
    """Our changes alone, as a patch with crate-relative paths.

    The difference is taken against a *trimmed* copy of upstream, because `vendor update` trims
    before it patches: a patch that also carried the trim's deletions would never apply twice.
    """
    with tempfile.TemporaryDirectory() as tmp:
        theirs_root = _trimmed_copy(upstream, Path(tmp) / crate.name)
        chunks = []
        for path in sorted(crate.rglob("*")):
            if not path.is_file() or path.name in OURS:
                continue
            relative = path.relative_to(crate).as_posix()
            theirs = theirs_root / relative
            if not theirs.is_file() or theirs.read_bytes() == path.read_bytes():
                continue
            diff = subprocess.run(
                ["git", "diff", "--no-index", "--", str(theirs), str(path)],
                capture_output=True,
                text=True,
            ).stdout
            # Point the headers at the crate, so the patch reads (and applies) anywhere.
            # Nothing else is touched: a hunk's trailing blank context line is part of its count.
            body = []
            for line in diff.split("\n"):
                if line.startswith("diff --git"):
                    body.append(f"diff --git a/{relative} b/{relative}")
                elif line.startswith("--- "):
                    body.append(f"--- a/{relative}")
                elif line.startswith("+++ "):
                    body.append(f"+++ b/{relative}")
                else:
                    body.append(line)
            chunks.append("\n".join(body))
        return "".join(chunks)


def apply_patch(crate: Path) -> bool:
    patch = crate / PATCH_FILE
    if not patch.is_file():
        print(f"[xerxes] {crate.name}: no {PATCH_FILE} to apply")
        return False
    done = subprocess.run(
        ["git", "apply", "--whitespace=nowarn", str(patch)],
        cwd=crate,
        capture_output=True,
        text=True,
    )
    if done.returncode != 0:
        print(done.stderr.strip())
    return done.returncode == 0


def patch_applies(crate: Path) -> bool:
    """Whether the stored patch is the difference the source actually carries."""
    patch = crate / PATCH_FILE
    if not patch.is_file():
        return False
    done = subprocess.run(
        ["git", "apply", "--reverse", "--check", str(patch)],
        cwd=crate,
        capture_output=True,
        text=True,
    )
    return done.returncode == 0


def cmd_list(_args) -> int:
    crates = _crates()
    if not crates:
        print("[xerxes] no vendored crates (engine/vendors/)")
        return 0
    print("[xerxes] engine/vendors: patched copies of crates the engine builds against")
    for crate in crates:
        state = "patch applied" if patch_applies(crate) else "PATCH DOES NOT MATCH THE SOURCE"
        size = sum(f.stat().st_size for f in crate.rglob("*") if f.is_file())
        print(f"  {crate.name:<10} {_version(crate):<10} {size / 1e6:5.1f} MB   {state}")
        notes = crate / "XERXES.md"
        if notes.is_file():
            first = next(
                (l for l in notes.read_text(encoding="utf-8").split("\n") if l.startswith("**What")),
                "",
            )
            if first:
                print(f"             {first[:100]}")
    return 0


def cmd_patch(args) -> int:
    crate = VENDORS / args.crate
    if not (crate / "Cargo.toml").is_file():
        print(f"[xerxes] no vendored crate `{args.crate}` (engine/vendors/)")
        return 1
    version = args.version or _version(crate)
    upstream = _fetch(args.crate, version)
    if upstream is None:
        return 1
    text = make_patch(crate, upstream)
    (crate / PATCH_FILE).write_text(text, encoding="utf-8", newline="\n")
    files = text.count("diff --git")
    print(f"[xerxes] {args.crate}: {PATCH_FILE} written from the difference against {version} ({files} file(s))")
    return 0


def cmd_update(args) -> int:
    name = args.crate
    crate = VENDORS / name
    if not (crate / "Cargo.toml").is_file():
        print(f"[xerxes] no vendored crate `{name}` (engine/vendors/)")
        return 1
    was = _version(crate)
    version = args.version or was
    upstream = _fetch(name, version)
    if upstream is None:
        return 1
    if not patch_applies(crate):
        print(f"[xerxes] {name}: the source does not match {PATCH_FILE}; run `vendor patch {name}` first")
        return 1

    keep = {f: (crate / f).read_bytes() for f in OURS if (crate / f).is_file()}
    shutil.rmtree(crate)
    shutil.copytree(upstream, crate)
    for path in crate.rglob("*"):
        path.chmod(0o644) if path.is_file() else None
    for name_, bytes_ in keep.items():
        (crate / name_).write_bytes(bytes_)
    trim(crate)
    if not apply_patch(crate):
        print(
            f"[xerxes] {name} {version} is in place but the patch did not apply: upstream moved.\n"
            f"         Fix the few lines by hand (they are marked `Xerxes patch`), then run\n"
            f"         `xerxes-ctrl vendor patch {name}` to store the new diff."
        )
        return 1
    print(f"[xerxes] {name}: {was} -> {version}, trimmed, patch re-applied.")
    print(f"         Check it builds (`template build cyrus web`) and read engine/vendors/{name}/XERXES.md.")
    return 0


def build_vendor_subparser(sub) -> None:
    sp = sub.add_parser(
        "vendor",
        help="The engine's patched copies of other crates (engine/vendors/): list, update, re-patch",
    )
    actions = sp.add_subparsers(dest="vendor_action", required=True)

    p = actions.add_parser("list", help="What is vendored, at which version, and whether its patch still matches")
    p.set_defaults(func=cmd_list)

    p = actions.add_parser("update", help="Fetch a version of the crate, trim it, and re-apply our patch")
    p.add_argument("crate", help="winit, cpal, ...")
    p.add_argument("version", nargs="?", help="the upstream version (default: the one vendored now)")
    p.set_defaults(func=cmd_update)

    p = actions.add_parser("patch", help="Regenerate xerxes.patch from the vendored source (after editing it by hand)")
    p.add_argument("crate")
    p.add_argument("version", nargs="?", help="the upstream version to diff against (default: the vendored one)")
    p.set_defaults(func=cmd_patch)
