"""Remote VM ops: update, reset, disk prune, ACME + volume backup/restore."""

from __future__ import annotations

import shlex
from typing import Any

import paramiko

from .actions import start_stack
from .ssh import echo, remote_home_path, run_command
from .actions import _bash_quote


def _compose_args(server: dict[str, Any]) -> str:
    files = server.get("compose_files") or []
    if not files:
        return ""
    return " ".join(f"-f {shlex.quote(f)}" for f in files)


def _run_script(
    client: paramiko.SSHClient,
    server: dict[str, Any],
    script: str,
    *,
    timeout: int = 7200,
) -> dict[str, Any]:
    remote_dir = remote_home_path(
        server.get("remote_dir") or f"~/projects/{server.get('project')}"
    )
    wrapped = f"""
set -euo pipefail
export COMPOSE_PROGRESS=plain
export BUILDKIT_PROGRESS=plain
cd "{remote_dir}"
pwd
{script}
"""
    code, out, err = run_command(
        client,
        f"bash -lc {_bash_quote(wrapped)}",
        timeout=timeout,
        get_pty=False,
    )
    combined = (out or "").rstrip()
    if err and err.strip():
        combined = (combined + "\n" + err.strip()).strip()
    return {
        "status": "success" if code == 0 else "failed",
        "output": combined,
        "error": "" if code == 0 else f"exit code {code}",
    }


def backup_acme(
    client: paramiko.SSHClient, server: dict[str, Any]
) -> dict[str, Any]:
    if not server.get("has_acme"):
        return {
            "status": "success",
            "output": f"[{server.get('id')}] has_acme=false — skip",
            "error": "",
        }
    acme = server.get("acme_path", "./letsencrypt/acme.json")
    vol = server.get("acme_volume")  # optional docker volume name
    project = server.get("project", "project")
    script = f"""
BACKUP_ROOT="$HOME/projects/.foxg-ssl-backups/{project}"
mkdir -p "$BACKUP_ROOT"
"""
    if vol:
        script += f"""
if docker volume inspect {shlex.quote(vol)} >/dev/null 2>&1; then
  docker run --rm -v {shlex.quote(vol)}:/certificates:ro -v "$BACKUP_ROOT":/out alpine \\
    sh -c 'cp -a /certificates/acme.json /out/acme.json && chmod 600 /out/acme.json'
  echo "Backed up ACME from volume {vol} -> $BACKUP_ROOT/acme.json"
elif [[ -f {shlex.quote(acme)} ]] && [[ -s {shlex.quote(acme)} ]]; then
  cp -a {shlex.quote(acme)} "$BACKUP_ROOT/acme.json"
  chmod 600 "$BACKUP_ROOT/acme.json"
  echo "Backed up ACME from {acme} -> $BACKUP_ROOT/acme.json"
else
  echo "WARNING: no ACME source found (volume={vol} path={acme})"
  exit 1
fi
"""
    else:
        script += f"""
if [[ -f {shlex.quote(acme)} ]] && [[ -s {shlex.quote(acme)} ]]; then
  cp -a {shlex.quote(acme)} "$BACKUP_ROOT/acme.json"
  chmod 600 "$BACKUP_ROOT/acme.json"
  echo "Backed up ACME from {acme} -> $BACKUP_ROOT/acme.json ($(wc -c <"$BACKUP_ROOT/acme.json") bytes)"
else
  echo "WARNING: missing/empty {acme}"
  exit 1
fi
"""
    script += 'date -u +%Y-%m-%dT%H:%M:%SZ >"$BACKUP_ROOT/last-backup.txt" || true\n'
    return _run_script(client, server, script, timeout=300)


def restore_acme(
    client: paramiko.SSHClient, server: dict[str, Any]
) -> dict[str, Any]:
    if not server.get("has_acme"):
        return {
            "status": "success",
            "output": f"[{server.get('id')}] has_acme=false — skip",
            "error": "",
        }
    acme = server.get("acme_path", "./letsencrypt/acme.json")
    vol = server.get("acme_volume")
    project = server.get("project", "project")
    script = f"""
BACKUP_ROOT="$HOME/projects/.foxg-ssl-backups/{project}"
BACKUP="$BACKUP_ROOT/acme.json"
if [[ ! -f "$BACKUP" ]] || [[ ! -s "$BACKUP" ]]; then
  echo "No ACME backup at $BACKUP"
  exit 1
fi
"""
    if vol:
        script += f"""
docker volume create {shlex.quote(vol)} >/dev/null 2>&1 || true
docker run --rm -v {shlex.quote(vol)}:/certificates -v "$BACKUP_ROOT":/in alpine \\
  sh -c 'cp -a /in/acme.json /certificates/acme.json && chmod 600 /certificates/acme.json'
echo "Restored ACME into volume {vol}"
"""
    else:
        script += f"""
mkdir -p "$(dirname {shlex.quote(acme)})"
if [[ -f {shlex.quote(acme)} ]] && [[ -s {shlex.quote(acme)} ]]; then
  cp -a {shlex.quote(acme)} "{acme}.bak.$(date +%Y%m%d-%H%M%S)"
fi
cp -a "$BACKUP" {shlex.quote(acme)}
chmod 600 {shlex.quote(acme)}
echo "Restored ACME -> {acme}"
"""
    return _run_script(client, server, script, timeout=300)


def backup_data(
    client: paramiko.SSHClient, server: dict[str, Any]
) -> dict[str, Any]:
    volumes = server.get("data_volumes") or []
    if not volumes:
        return {
            "status": "success",
            "output": f"[{server.get('id')}] no data_volumes configured — skip",
            "error": "",
        }
    project = server.get("project", "project")
    loop = "\n".join(
        f"""
if docker volume inspect {shlex.quote(vol)} >/dev/null 2>&1; then
  echo "Backing up {vol}..."
  docker run --rm -v {shlex.quote(vol)}:/data:ro -v "$DEST":/out alpine \\
    sh -c 'tar czf /out/{vol}.tar.gz -C /data .'
  echo "  -> $DEST/{vol}.tar.gz"
else
  echo "SKIP missing volume {vol}"
fi
"""
        for vol in volumes
    )
    script = f"""
BACKUP_ROOT="$HOME/projects/.foxg-data-backups/{project}"
STAMP="$(date -u +%Y%m%d-%H%M%S)"
DEST="$BACKUP_ROOT/$STAMP"
mkdir -p "$DEST"
{loop}
ln -sfn "$DEST" "$BACKUP_ROOT/latest"
echo "Data backup: $DEST"
ls -lah "$DEST"
"""
    return _run_script(client, server, script, timeout=3600)


def restore_data(
    client: paramiko.SSHClient,
    server: dict[str, Any],
    *,
    stamp: str | None = None,
) -> dict[str, Any]:
    volumes = server.get("data_volumes") or []
    if not volumes:
        return {
            "status": "success",
            "output": f"[{server.get('id')}] no data_volumes configured — skip",
            "error": "",
        }
    project = server.get("project", "project")
    stamp_q = shlex.quote(stamp or "latest")
    loop = "\n".join(
        f"""
TAR="$SRC/{vol}.tar.gz"
if [[ -f "$TAR" ]]; then
  echo "Restoring {vol}..."
  docker volume rm -f {shlex.quote(vol)} >/dev/null 2>&1 || true
  docker volume create {shlex.quote(vol)} >/dev/null
  docker run --rm -v {shlex.quote(vol)}:/data -v "$SRC":/in:ro alpine \\
    sh -c 'rm -rf /data/..?* /data/.[!.]* /data/* 2>/dev/null; tar xzf /in/{vol}.tar.gz -C /data'
  echo "  OK {vol}"
else
  echo "SKIP missing $TAR"
fi
"""
        for vol in volumes
    )
    script = f"""
BACKUP_ROOT="$HOME/projects/.foxg-data-backups/{project}"
SRC="$BACKUP_ROOT"/{stamp_q}
if [[ ! -d "$SRC" ]]; then
  echo "Missing backup dir $SRC"
  exit 1
fi
echo "Restoring data from $SRC (stack should be stopped)"
{loop}
echo "DONE"
"""
    return _run_script(client, server, script, timeout=3600)


def disk_df(client: paramiko.SSHClient, server: dict[str, Any]) -> dict[str, Any]:
    return _run_script(
        client,
        server,
        """
df -h /
echo
docker system df || true
echo
docker system df -v 2>/dev/null | head -n 80 || true
""",
        timeout=120,
    )


def disk_prune(
    client: paramiko.SSHClient,
    server: dict[str, Any],
    *,
    builder: bool = False,
    images: bool = False,
    system: bool = False,
    volumes: bool = False,
) -> dict[str, Any]:
    parts: list[str] = ["echo '==> disk before';", "docker system df || true"]
    if builder:
        parts.append("echo '==> docker builder prune -af'")
        parts.append("docker builder prune -af")
    if images:
        parts.append("echo '==> docker image prune -af (dangling + unused)'")
        parts.append("docker image prune -af")
    if system:
        if volumes:
            parts.append("echo '==> WARNING: docker system prune -a --volumes -f'")
            parts.append("docker system prune -a --volumes -f")
        else:
            parts.append("echo '==> docker system prune -a -f (no volumes)'")
            parts.append("docker system prune -a -f")
    if not (builder or images or system):
        parts.append("echo 'Nothing selected — use --builder / --images / --system [--volumes]'")
    parts += ["echo '==> disk after'", "docker system df || true"]
    return _run_script(client, server, "\n".join(parts), timeout=1800)


def update_stack(
    client: paramiko.SSHClient,
    server: dict[str, Any],
) -> dict[str, Any]:
    """Safe deploy refresh: backup ACME, stop, remove app images, pull, start.
    Does not wipe BuildKit cache (the Rust release build is slow without it).
    """
    sid = server.get("id", "?")
    app_images = server.get("app_images") or []
    compose = _compose_args(server)
    stop = server.get("stop_cmd") or (
        f"docker compose {compose} down --remove-orphans".strip()
        if compose
        else "docker compose down --remove-orphans"
    )

    echo(f"[{sid}] update: backup ACME (if any)...")
    if server.get("has_acme"):
        acme_r = backup_acme(client, server)
        if acme_r["status"] != "success":
            echo(f"[{sid}] ACME backup warning: {acme_r.get('error') or acme_r.get('output')}")

    imgs = "\n".join(
        f'  if docker image inspect {shlex.quote(img)} >/dev/null 2>&1; then docker rmi {shlex.quote(img)} || true; echo "  removed {img}"; fi'
        for img in app_images
    )
    script = f"""
echo "==> [{sid}] UPDATE (keep volumes + base images + BuildKit cache)"
echo "==> stop"
{stop} || true
echo "==> backup ACME already attempted from controller"
echo "==> skip builder prune (keeps the cargo layer cache; use disk prune if low on space)"
echo "==> remove app images only"
{imgs or 'echo "  (no app_images listed)"'}
docker image prune -f >/dev/null || true
echo "==> git fetch + reset to origin (discard uncommitted VM edits)"
git fetch origin
branch="$(git rev-parse --abbrev-ref HEAD)"
git reset --hard "origin/${{branch}}"
"""
    if server.get("has_acme"):
        # restore into place before start (in case build/start wiped bind path)
        acme = server.get("acme_path", "./letsencrypt/acme.json")
        project = server.get("project", "project")
        if not server.get("acme_volume"):
            script += f"""
echo "==> restore ACME if local missing"
BACKUP="$HOME/projects/.foxg-ssl-backups/{project}/acme.json"
if [[ (! -f {shlex.quote(acme)} || ! -s {shlex.quote(acme)} ) && -s "$BACKUP" ]]; then
  mkdir -p "$(dirname {shlex.quote(acme)})"
  cp -a "$BACKUP" {shlex.quote(acme)}
  chmod 600 {shlex.quote(acme)}
  echo "  restored {acme}"
fi
"""
    r = _run_script(client, server, script, timeout=3600)
    if r["status"] != "success":
        return r
    # Refresh .env so the secrets match __ctrl__/safe/* before build.
    if server.get("env_file"):
        from .actions import push_env

        echo(f"[{sid}] update: refresh .env...")
        env_r = push_env(client, server)
        if env_r["status"] != "success":
            echo(f"[{sid}] env refresh warning: {env_r.get('error') or env_r.get('output')}")
    echo(f"[{sid}] update: starting stack...")
    start_r = start_stack(client, server)
    start_r["output"] = (
        (r.get("output") or "")
        + "\n\n===== start =====\n"
        + (start_r.get("output") or "")
    ).strip()
    return start_r


def reset_stack(
    client: paramiko.SSHClient,
    server: dict[str, Any],
    *,
    volumes: bool = False,
    app_images: bool = True,
    all_images: bool = False,
    build_cache: bool = True,
    pull: bool = True,
    start_after: bool = True,
) -> dict[str, Any]:
    """Destructive reset with selectable scopes. Volumes wipe DB data."""
    sid = server.get("id", "?")
    compose = _compose_args(server)
    stop = server.get("stop_cmd") or (
        f"docker compose {compose} down --remove-orphans".strip()
        if compose
        else "docker compose down --remove-orphans"
    )
    data_vols = server.get("data_volumes") or []
    apps = server.get("app_images") or []

    if server.get("has_acme"):
        backup_acme(client, server)

    script = f"""
echo "==> [{sid}] RESET volumes={volumes} app_images={app_images} all_images={all_images} build_cache={build_cache}"
echo "==> stop"
{stop} || true
"""
    if build_cache:
        script += """
echo "==> docker builder prune -af"
docker builder prune -af || true
"""
    if volumes and data_vols:
        for vol in data_vols:
            script += f"""
if docker volume inspect {shlex.quote(vol)} >/dev/null 2>&1; then
  docker volume rm {shlex.quote(vol)} && echo "  removed volume {vol}" || true
fi
"""
    elif volumes and not data_vols:
        script += """
echo "==> no data_volumes in servers.json — skip volume wipe (e.g. front)"
"""
    if all_images:
        script += """
echo "==> docker image prune -af (ALL unused images)"
docker image prune -af || true
"""
    elif app_images and apps:
        for img in apps:
            script += f"""
if docker image inspect {shlex.quote(img)} >/dev/null 2>&1; then
  docker rmi {shlex.quote(img)} && echo "  removed {img}" || true
fi
"""
        script += "docker image prune -f >/dev/null || true\n"

    if pull:
        script += """
echo "==> git fetch + hard reset to origin (required unless --no-pull)"
git fetch origin
branch="$(git rev-parse --abbrev-ref HEAD)"
echo "  branch=${branch} -> origin/${branch}"
git reset --hard "origin/${branch}"
git log -1 --oneline
"""
    else:
        script += """
echo "==> skip git pull (--no-pull)"
"""

    r = _run_script(client, server, script, timeout=3600)
    if r["status"] != "success":
        return r
    if server.get("has_acme"):
        restore_acme(client, server)
    if start_after:
        echo(f"[{sid}] reset: starting stack...")
        start_r = start_stack(client, server)
        start_r["output"] = (
            (r.get("output") or "")
            + "\n\n===== start =====\n"
            + (start_r.get("output") or "")
        ).strip()
        return start_r
    return r
