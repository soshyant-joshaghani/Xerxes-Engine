"""`weigh`: where a web build's megabytes go.

A web game is one page, one `.wasm` and its art bundles. This says how big each is, what the
player actually downloads (gzipped, as nginx serves it), and — from the module that still has a
name section — **which crate wrote the code**. That last number is the one that decides whether
splitting the module per scene would be worth anything: see `__docs__/web-progressive.md`.

No dependencies: the wasm binary format is simple enough to walk for this.
"""

from __future__ import annotations

import gzip
import io
import re
from collections import defaultdict
from pathlib import Path

# A section's kind byte. 0 is a custom section, which carries its own name (`name`, `.debug_*`).
SECTION_KINDS = {
    0: "custom",
    1: "type",
    2: "import",
    3: "function",
    4: "table",
    5: "memory",
    6: "global",
    7: "export",
    8: "start",
    9: "element",
    10: "code",
    11: "data",
    12: "data count",
}

# Rust's v0 mangling writes a crate as `Cs<disambiguator>_<len><name>` (or `C<len><name>`). The
# first one in a symbol is the crate the item is defined in, which is the one that pays for it.
CRATE = re.compile(r"C(?:s[0-9A-Za-z]+_)?([0-9]+)")

# What to add up, in the order it is printed. A crate counts for a group when its name is the
# entry or starts with it and `_` (`bevy` takes `bevy_ecs`, not `bevyish`).
GROUPS = (
    (
        "the game's own code",
        ("game", "cyrus", "darius", "cyrus_rules", "darius_rules", "rules"),
    ),
    ("the engine (xerxes)", ("xerxes_engine", "xerxes_sim", "xerxes_build")),
    ("bevy", ("bevy",)),
    ("the renderer (wgpu, naga)", ("wgpu", "naga", "naga_oil")),
    ("physics (rapier, parry)", ("rapier2d", "rapier3d", "parry2d", "parry3d")),
)


def _uleb(data: bytes, i: int) -> tuple[int, int]:
    result, shift = 0, 0
    while True:
        byte = data[i]
        i += 1
        result |= (byte & 0x7F) << shift
        if not byte & 0x80:
            return result, i
        shift += 7


def _sections(data: bytes):
    if data[:4] != b"\0asm":
        raise ValueError("not a wasm module")
    i = 8
    while i < len(data):
        kind = data[i]
        size, i = _uleb(data, i + 1)
        yield kind, i, size
        i += size


def _section_name(data: bytes, kind: int, start: int) -> str:
    if kind != 0:
        return SECTION_KINDS.get(kind, str(kind))
    length, i = _uleb(data, start)
    return 'custom "%s"' % data[i : i + length].decode("utf-8", "replace")


def _imported_functions(data: bytes, start: int) -> int:
    """Function indices start with the imports, so the code section's n-th body is n + this."""
    count, i = _uleb(data, start)
    functions = 0
    for _ in range(count):
        for _ in range(2):  # module and name
            length, i = _uleb(data, i)
            i += length
        kind = data[i]
        i += 1
        if kind == 0:  # function
            _, i = _uleb(data, i)
            functions += 1
        elif kind in (1, 2):  # table, memory
            if kind == 1:
                i += 1  # reference type
            limited = data[i]
            i += 1
            _, i = _uleb(data, i)
            if limited == 1:
                _, i = _uleb(data, i)
        elif kind == 3:  # global
            i += 2
        else:
            raise ValueError("unknown import kind %d" % kind)
    return functions


def _body_sizes(data: bytes, start: int) -> list[int]:
    count, i = _uleb(data, start)
    sizes = []
    for _ in range(count):
        size, i = _uleb(data, i)
        sizes.append(size)
        i += size
    return sizes


def _function_names(data: bytes, start: int, size: int) -> dict[int, str]:
    length, i = _uleb(data, start)
    if data[i : i + length] != b"name":
        return {}
    i += length
    end, names = start + size, {}
    while i < end:
        sub = data[i]
        sub_size, i = _uleb(data, i + 1)
        if sub == 1:  # the function-name subsection
            count, j = _uleb(data, i)
            for _ in range(count):
                index, j = _uleb(data, j)
                length, j = _uleb(data, j)
                names[index] = data[j : j + length].decode("utf-8", "replace")
                j += length
        i += sub_size
    return names


def crate_of(symbol: str) -> str:
    """The crate a mangled symbol belongs to."""
    for match in CRATE.finditer(symbol):
        length, at = int(match.group(1)), match.end()
        crate = symbol[at : at + length]
        if crate and (crate[0].isalpha() or crate[0] == "_"):
            return crate
    head = symbol.lstrip("_").lstrip("ZN").lstrip("0123456789")  # the legacy mangling
    for char in "< >&*[:.":
        head = head.split(char)[0]
    return head or "(unnamed)"


def code_by_crate(path: Path) -> tuple[dict[str, int], int]:
    """How many bytes of code each crate wrote, and the code section's size."""
    data = path.read_bytes()
    imports, bodies, names = 0, [], {}
    for kind, start, size in _sections(data):
        if kind == 2:
            imports = _imported_functions(data, start)
        elif kind == 10:
            bodies = _body_sizes(data, start)
        elif kind == 0:
            names = _function_names(data, start, size) or names
    by_crate: dict[str, int] = defaultdict(int)
    for n, body in enumerate(bodies):
        name = names.get(imports + n)
        by_crate["(no name section)" if name is None else crate_of(name)] += body
    return dict(by_crate), sum(bodies)


def sections_of(path: Path) -> list[tuple[str, int]]:
    """Each section of a module, largest first."""
    data = path.read_bytes()
    found = [
        (_section_name(data, kind, start), size)
        for kind, start, size in _sections(data)
    ]
    return sorted(found, key=lambda row: -row[1])


def over_the_wire(path: Path, level: int = 6) -> int:
    """What the player downloads: the file as nginx gzips it (`gzip_comp_level 6`)."""
    buffer = io.BytesIO()
    with gzip.GzipFile(fileobj=buffer, mode="wb", compresslevel=level) as out:
        out.write(path.read_bytes())
    return len(buffer.getvalue())


def _mb(size: float) -> str:
    return "%8.2f MB" % (size / 1e6)


def report(shipped: Path | None, named: Path | None, bundles: list[Path]) -> None:
    """Prints the whole picture: what ships, what it costs over the wire, and who wrote it."""
    if shipped is not None:
        print("What ships (%s)" % shipped.name)
        size = shipped.stat().st_size
        print(
            "  the module          %s   %s gzipped (what the player downloads)"
            % (_mb(size), _mb(over_the_wire(shipped)))
        )
        for name, section in sections_of(shipped)[:6]:
            print("    %-20s %s  %5.1f%%" % (name, _mb(section), 100 * section / size))
        print()
    if bundles:
        print("Art bundles (downloaded when a scene that draws them is opened)")
        for bundle in sorted(bundles):
            print("  %-20s %s" % (bundle.name, _mb(bundle.stat().st_size)))
        print()
    if named is None:
        print("No module with a name section was found: build it first, then weigh it.")
        return
    by_crate, total = code_by_crate(named)
    if total == 0 or "(no name section)" in by_crate:
        print(
            "The module has no name section, so it cannot say which crate wrote what."
        )
        return
    print(
        "Who wrote the code (%s, %s in %d crates)"
        % (named.name, _mb(total), len(by_crate))
    )
    for label, crates in GROUPS:
        size = sum(
            bytes_
            for name, bytes_ in by_crate.items()
            if any(name == crate or name.startswith(crate + "_") for crate in crates)
        )
        print("  %-28s %s  %5.1f%%" % (label, _mb(size), 100 * size / total))
    print("  largest crates:")
    for name, size in sorted(by_crate.items(), key=lambda kv: -kv[1])[:10]:
        print("    %-26s %s  %5.1f%%" % (name, _mb(size), 100 * size / total))
