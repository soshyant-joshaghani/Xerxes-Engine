#!/usr/bin/env python3
"""Prefer Shecan nameservers on the first dhcp4: true netplan interface.

Do NOT disable DHCP DNS (`use-dns: false`). On some Bamdad/Arvan VMs Shecan
times out and exclusive Shecan leaves systemd-resolved hanging (127.0.0.53 i/o
timeout), which breaks Docker pulls. Apt/Docker Iran mirrors cover blocked
archive.ubuntu.com / docker.io without needing exclusive Shecan.
"""

from __future__ import annotations

import pathlib
import sys

DNS_A = "178.22.122.100"
DNS_B = "185.51.200.2"


def _indent_of(line: str) -> int:
    return len(line) - len(line.lstrip())


def _find_dhcp(lines: list[str]) -> int:
    for i, line in enumerate(lines):
        stripped = line.lstrip()
        if stripped.startswith("dhcp4:") and "true" in stripped:
            return i
    return -1


def _rewrite(lines: list[str], dhcp_idx: int) -> list[str]:
    indent = _indent_of(lines[dhcp_idx])
    child = " " * (indent + 2)
    ns = " " * indent

    head = lines[: dhcp_idx + 1]
    i = dhcp_idx + 1
    tail_start = len(lines)
    while i < len(lines):
        line = lines[i]
        stripped = line.lstrip()
        if not stripped or stripped.startswith("#"):
            i += 1
            continue
        li = _indent_of(line)
        if li < indent:
            tail_start = i
            break
        if li == indent and (
            stripped.startswith("nameservers:")
            or stripped.startswith("dhcp4-overrides:")
        ):
            # Drop prior nameservers / overrides (including exclusive Shecan).
            i += 1
            while i < len(lines):
                nxt = lines[i]
                ns_txt = nxt.lstrip()
                if not ns_txt or ns_txt.startswith("#"):
                    i += 1
                    continue
                if _indent_of(nxt) <= indent:
                    break
                i += 1
            continue
        if li == indent:
            tail_start = i
            break
        tail_start = i
        break

    # Shecan first; DHCP DNS still allowed as fallback (no use-dns: false).
    block = [
        f"{ns}nameservers:",
        f"{child}addresses:",
        f"{child}  - {DNS_A}",
        f"{child}  - {DNS_B}",
    ]
    return head + block + lines[tail_start:]


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: remote_netplan_dns.py /etc/netplan/xx.yaml", file=sys.stderr)
        return 2
    path = pathlib.Path(sys.argv[1])
    lines = path.read_text(encoding="utf-8").splitlines()
    dhcp_idx = _find_dhcp(lines)
    if dhcp_idx < 0:
        print("could not find dhcp4: true to attach nameservers", file=sys.stderr)
        return 1

    updated = _rewrite(lines, dhcp_idx)
    path.write_text("\n".join(updated) + "\n", encoding="utf-8")
    print(f"updated {path} (Shecan preferred, DHCP DNS kept)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
