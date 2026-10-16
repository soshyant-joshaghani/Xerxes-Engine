"""Windows OpenSSH refuses PEM keys with ACL too open — harden before connect."""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path


def harden_private_key(path: str | Path) -> None:
    """Restrict private key ACLs so OpenSSH accepts the file (Windows)."""
    key = Path(path)
    if not key.is_file():
        return
    if sys.platform != "win32":
        try:
            os.chmod(key, 0o600)
        except OSError:
            pass
        return

    user = os.environ.get("USERNAME") or os.getlogin()
    # Disable inheritance, wipe inherited grants, grant only current user Read.
    cmds = [
        ["icacls", str(key), "/inheritance:r"],
        ["icacls", str(key), "/remove:g", "Authenticated Users"],
        ["icacls", str(key), "/remove:g", "BUILTIN\\Users"],
        ["icacls", str(key), "/remove:g", "Users"],
        ["icacls", str(key), "/grant:r", f"{user}:(R)"],
    ]
    for cmd in cmds:
        subprocess.run(cmd, capture_output=True, text=True, check=False)


def resolve_ssh_key(configured_path: str) -> str:
    """
    Prefer %USERPROFILE%\\.ssh\\<same-filename> when present (common on Windows),
    otherwise the configured __ctrl__/safe path. Always harden before return.
    """
    from .config import expand_path

    configured = Path(expand_path(configured_path))
    home_candidate = Path.home() / ".ssh" / configured.name
    key = home_candidate if home_candidate.is_file() else configured
    if not key.is_file():
        raise FileNotFoundError(str(configured))
    harden_private_key(key)
    return str(key.resolve())
