from __future__ import annotations

import json
import os
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parent.parent
# The Xerxes repo root (__ctrl__ lives directly under it): compose files, backend/, engine/, games/.
PROJECT = ROOT.parent
SERVERS_PATH = ROOT / "servers.json"
GPG_PATH = ROOT / "static" / "gpg"


def expand_path(path: str) -> str:
    """Expand ~ / env vars; resolve relative paths against __ctrl__ root."""
    expanded = os.path.expanduser(os.path.expandvars(path))
    p = Path(expanded)
    if not p.is_absolute():
        p = ROOT / p
    return str(p.resolve())


def load_config(path: Path | None = None) -> dict[str, Any]:
    cfg_path = path or SERVERS_PATH
    with cfg_path.open(encoding="utf-8") as f:
        return json.load(f)


def read_host_file(path: str | Path) -> str:
    """First non-empty, non-comment line from an address .txt file."""
    p = Path(expand_path(str(path)))
    if not p.is_file():
        return ""
    for line in p.read_text(encoding="utf-8").splitlines():
        text = line.strip()
        if text and not text.startswith("#"):
            return text
    return ""


def hydrate_server(server: dict[str, Any]) -> dict[str, Any]:
    """Resolve host from host_file (preferred) or inline host."""
    out = dict(server)
    host_file = out.get("host_file")
    if host_file:
        from_file = read_host_file(host_file)
        if from_file:
            out["host"] = from_file
    return out


def list_servers(
    cfg: dict[str, Any] | None = None,
    *,
    include_disabled: bool = False,
) -> list[dict[str, Any]]:
    data = cfg or load_config()
    defaults = data.get("defaults", {})
    out: list[dict[str, Any]] = []
    for raw in data.get("servers", []):
        server = hydrate_server({**defaults, **raw})
        if not include_disabled and server.get("enabled", True) is False:
            continue
        out.append(server)
    return out


def resolve_targets(
    selector: str,
    *,
    include_disabled: bool = False,
) -> list[dict[str, Any]]:
    """Resolve 'all', a server id, a project name, or a site name."""
    servers = list_servers(include_disabled=include_disabled)
    key = selector.strip().lower()
    if key in {"all", "*"}:
        return servers

    by_id = [s for s in servers if s["id"].lower() == key]
    if by_id:
        return by_id

    by_project = [s for s in servers if s.get("project", "").lower() == key]
    if by_project:
        return by_project

    by_site = [s for s in servers if s.get("site", "").lower() == key]
    if by_site:
        return by_site

    # Legacy alias: region → site
    by_region = [s for s in servers if s.get("region", "").lower() == key]
    if by_region:
        return by_region

    known = ", ".join(s["id"] for s in list_servers(include_disabled=True))
    raise SystemExit(f"Unknown target '{selector}'. Known ids: {known}")


def require_host(server: dict[str, Any]) -> str:
    host = (server.get("host") or "").strip()
    if not host:
        hint = server.get("host_file") or "servers.json host / host_file"
        raise SystemExit(
            f"Server '{server['id']}' has empty host — fill {hint} first."
        )
    return host
