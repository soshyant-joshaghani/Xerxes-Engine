from __future__ import annotations

import shlex
from typing import Any

import paramiko

from .ssh import remote_cwd_cmd, remote_home_path, run_command


def _bash_quote(command: str) -> str:
    return "'" + command.replace("'", "'\"'\"'") + "'"


def _run_in_project(
    client: paramiko.SSHClient,
    server: dict[str, Any],
    command: str,
    *,
    timeout: int | None = 3600,
    stream: bool = False,
) -> dict[str, Any]:
    # Plain progress + no PTY: docker compose TTY spinners can flood SSH and yield exit -1.
    # Stream drain keeps the channel alive during long docker builds.
    full = remote_cwd_cmd(
        server,
        f"export COMPOSE_PROGRESS=plain BUILDKIT_PROGRESS=plain; {command}",
    )
    code, out, err = run_command(
        client, full, timeout=timeout, get_pty=False, stream=stream
    )
    combined = (out or "").rstrip()
    if err and err.strip():
        combined = (combined + "\n" + err.strip()).strip()
    return {
        "status": "success" if code == 0 else "failed",
        # Streamed success already hit the console — skip re-print via print_result.
        "output": "" if stream and code == 0 else combined,
        "error": "" if code == 0 else f"exit code {code}",
    }


def _compose_file_args(server: dict[str, Any]) -> str:
    files = server.get("compose_files") or ["compose.yml"]
    return " ".join(f"-f {shlex.quote(str(f))}" for f in files)


def logs_stack(
    client: paramiko.SSHClient,
    server: dict[str, Any],
    service: str | None,
    *,
    lines: int = 80,
    follow: bool = True,
) -> dict[str, Any]:
    """Stream ``docker compose logs`` on the VM (follow has no wall-clock timeout)."""
    compose = _compose_file_args(server)
    cmd = f"docker compose {compose} logs --tail={max(1, int(lines))}"
    if follow:
        cmd += " --follow"
    if service:
        cmd += f" {shlex.quote(service)}"
    return _run_in_project(
        client,
        server,
        cmd,
        timeout=None if follow else 120,
        stream=True,
    )


def restore_flat_stack(
    client: paramiko.SSHClient,
    server: dict[str, Any],
    *,
    branch: str | None = None,
) -> dict[str, Any]:
    """Fetch origin and hard-reset the VM clone to the rewritten remote tip.

    Does not stop/rebuild compose — run ``update`` or ``start`` afterward.
    """
    sid = server.get("id", "?")
    if branch:
        br = shlex.quote(branch)
        script = f"""
echo "==> [{sid}] restore-flat (git only)"
git fetch origin
git reset --hard origin/{br}
git log -1 --oneline
"""
    else:
        script = f"""
echo "==> [{sid}] restore-flat (git only)"
git fetch origin
branch="$(git rev-parse --abbrev-ref HEAD)"
if [[ "$branch" == "HEAD" ]]; then
  echo "error: detached HEAD on VM — pass --branch <name>"
  exit 1
fi
git reset --hard "origin/${{branch}}"
git log -1 --oneline
"""
    return _run_in_project(client, server, script.strip(), timeout=600, stream=True)



def git_url_for(server: dict[str, Any]) -> str:
    if server.get("git_url"):
        return str(server["git_url"])
    project = server.get("project")
    org = server.get("github_org", "soshyant-joshaghani")
    if not project:
        raise ValueError(f"server {server.get('id')} missing project/git_url")
    return f"git@github.com:{org}/{project}.git"


def clone_repo(
    client: paramiko.SSHClient,
    server: dict[str, Any],
    *,
    pull_if_exists: bool = True,
) -> dict[str, Any]:
    """Clone git_url into remote_dir (~/projects/<project>). Pull if already cloned."""
    remote_dir = remote_home_path(
        server.get("remote_dir") or f"~/projects/{server.get('project')}"
    )
    parent = remote_home_path(server.get("remote_projects_root") or "~/projects")
    url = git_url_for(server)
    sid = server.get("id", "?")
    iran = bool(server.get("iran_setup"))

    # Iran often blocks github.com:22 — route git SSH via ssh.github.com:443.
    github_ssh_cfg = ""
    if iran and url.startswith("git@"):
        github_ssh_cfg = """
echo "==> [Iran] GitHub SSH via ssh.github.com:443 (port 22 often blocked)"
mkdir -p "$HOME/.ssh"
chmod 700 "$HOME/.ssh"
# Drop any prior Host github.com block, then write 443 relay config.
if [[ -f "$HOME/.ssh/config" ]]; then
  awk 'BEGIN{skip=0} /^[Hh]ost[ \\t]+github\\.com$/{skip=1; next} /^[Hh]ost[ \\t]/{skip=0} !skip{print}' \\
    "$HOME/.ssh/config" > "$HOME/.ssh/config.tmp" && mv "$HOME/.ssh/config.tmp" "$HOME/.ssh/config"
fi
cat >> "$HOME/.ssh/config" <<'EOF'
Host github.com
  HostName ssh.github.com
  Port 443
  User git
  IdentityFile ~/.ssh/id_ed25519
  IdentitiesOnly yes
EOF
chmod 600 "$HOME/.ssh/config"
ssh-keyscan -p 443 -t ed25519,rsa ssh.github.com >> "$HOME/.ssh/known_hosts" 2>/dev/null || true
timeout 8 bash -c 'echo >/dev/tcp/ssh.github.com/443' 2>/dev/null \\
  && echo "==> ssh.github.com:443 reachable" \\
  || echo "==> WARNING: cannot open ssh.github.com:443 — clone may still fail"
"""

    script = f"""
set -euo pipefail
mkdir -p "{parent}"
{github_ssh_cfg}
if [[ -d "{remote_dir}/.git" ]]; then
  echo "==> [{sid}] already cloned: {remote_dir}"
  git -C "{remote_dir}" remote -v | head -n 2 || true
  git -C "{remote_dir}" status -sb || true
"""
    if pull_if_exists:
        script += f"""
  echo "==> [{sid}] git pull --ff-only"
  git -C "{remote_dir}" pull --ff-only
"""
    else:
        script += """
  echo "==> skip pull (--no-pull)"
"""
    script += f"""
else
  echo "==> [{sid}] git clone {url} {remote_dir}"
  mkdir -p "$HOME/.ssh"
  chmod 700 "$HOME/.ssh"
  if [[ ! -f "$HOME/.ssh/config" ]] || ! grep -q 'HostName ssh.github.com' "$HOME/.ssh/config" 2>/dev/null; then
    ssh-keyscan -t ed25519,rsa github.com >> "$HOME/.ssh/known_hosts" 2>/dev/null || true
  fi
  git clone {url} "{remote_dir}"
fi
echo "==> [{sid}] ready: {remote_dir}"
ls -la "{remote_dir}" | head -n 20
"""
    code, out, err = run_command(
        client,
        f"bash -lc {_bash_quote(script)}",
        timeout=1800,
    )
    combined = (out or "").rstrip()
    if err and err.strip():
        combined = (combined + "\n" + err.strip()).strip()
    return {
        "status": "success" if code == 0 else "failed",
        "output": combined,
        "error": "" if code == 0 else f"exit code {code}",
    }


def push_env(
    client: paramiko.SSHClient,
    server: dict[str, Any],
) -> dict[str, Any]:
    """Upload safe/*-env.env to ~/projects/<project>/.env on the VM."""
    from pathlib import Path

    from .config import expand_path
    from .ssh import echo, upload_file

    env_rel = server.get("env_file")
    if not env_rel:
        return {
            "status": "failed",
            "error": "no env_file in servers.json",
            "output": "",
        }
    local = Path(expand_path(env_rel))
    if not local.is_file():
        return {
            "status": "failed",
            "error": f"missing local env file: {local}",
            "output": "",
        }

    remote_dir = remote_home_path(
        server.get("remote_dir") or f"~/projects/{server.get('project')}"
    )
    sid = server.get("id", "?")
    remote_tmp = f"/tmp/foxg-ctrl-{sid}.env"

    echo(f"[{sid}] upload {local.name} → {remote_dir}/.env")
    upload_file(client, local, remote_tmp)

    script = f"""
set -euo pipefail
mkdir -p "{remote_dir}"
install -m 600 {remote_tmp} "{remote_dir}/.env"
rm -f {remote_tmp}
echo "==> [{sid}] wrote {remote_dir}/.env ($(wc -l < "{remote_dir}/.env") lines)"
"""
    code, out, err = run_command(
        client,
        f"bash -lc {_bash_quote(script)}",
        timeout=120,
    )
    combined = (out or "").rstrip()
    if err and err.strip():
        combined = (combined + "\n" + err.strip()).strip()
    return {
        "status": "success" if code == 0 else "failed",
        "output": combined,
        "error": "" if code == 0 else f"exit code {code}",
    }


def start_stack(
    client: paramiko.SSHClient,
    server: dict[str, Any],
) -> dict[str, Any]:
    cmd = server.get("start_cmd")
    if not cmd:
        return {"status": "failed", "error": "no start_cmd in servers.json", "output": ""}
    # Stream docker build logs so a long Rust release build doesn't drop SSH (exit -1).
    return _run_in_project(client, server, cmd, timeout=7200, stream=True)


def stop_stack(
    client: paramiko.SSHClient, server: dict[str, Any]
) -> dict[str, Any]:
    cmd = server.get("stop_cmd")
    if not cmd:
        return {"status": "failed", "error": "no stop_cmd in servers.json", "output": ""}
    return _run_in_project(client, server, cmd, timeout=600)


def status_stack(
    client: paramiko.SSHClient, server: dict[str, Any]
) -> dict[str, Any]:
    cmd = server.get("status_cmd") or "docker compose ps"
    return _run_in_project(client, server, cmd, timeout=120)


def run_raw(
    client: paramiko.SSHClient,
    server: dict[str, Any],
    command: str,
    *,
    in_project: bool = False,
    timeout: int = 600,
) -> dict[str, Any]:
    if in_project:
        return _run_in_project(client, server, command, timeout=timeout)
    code, out, err = run_command(
        client,
        f"bash -lc {_bash_quote(command)}",
        timeout=timeout,
    )
    combined = (out or "").rstrip()
    if err and err.strip():
        combined = (combined + "\n" + err.strip()).strip()
    return {
        "status": "success" if code == 0 else "failed",
        "output": combined,
        "error": "" if code == 0 else f"exit code {code}",
    }
