from __future__ import annotations

import shlex
import textwrap
from pathlib import Path
from typing import Any

import paramiko

from .config import GPG_PATH
from .ssh import echo, run_command, upload_file

NETPLAN_HELPER = Path(__file__).resolve().parent / "remote_netplan_dns.py"


def _remote_script_iran(
    *,
    server_id: str,
    registry_mirror: str,
    apt_mirror: str,
) -> str:
    return textwrap.dedent(
        f"""\
        #!/usr/bin/env bash
        set -euo pipefail

        SERVER_ID={shlex.quote(server_id)}
        REGISTRY_MIRROR={shlex.quote(registry_mirror)}
        APT_MIRROR={shlex.quote(apt_mirror)}

        echo "==> [$SERVER_ID] Iran VM bootstrap (apt/Docker mirrors; keep provider DNS)"

        # Do NOT inject Shecan on Bamdad/Arvan — it often breaks resolution
        # (Temporary failure in name resolution). Apt + Docker Iran mirrors
        # already avoid archive.ubuntu.com / docker.io.
        NETPLAN_FILE="$(ls /etc/netplan/*.yaml /etc/netplan/*.yml 2>/dev/null | head -n1 || true)"
        if [[ -n "$NETPLAN_FILE" ]]; then
          BACKUP="$HOME/$(basename "$NETPLAN_FILE").backup"
          if [[ -f "$BACKUP" ]] && grep -qE '178\\.22\\.122\\.100|185\\.51\\.200\\.2' "$NETPLAN_FILE" 2>/dev/null; then
            echo "==> restoring provider DNS from $BACKUP (removing Shecan)"
            sudo cp "$BACKUP" "$NETPLAN_FILE"
            sudo netplan apply || true
            sudo systemctl restart systemd-resolved || true
            sleep 2
          fi
          resolvectl status || true
        fi

        dns_ok() {{
          timeout 5 getent hosts "$1" >/dev/null 2>&1
        }}

        echo "==> rewriting apt sources -> $APT_MIRROR"
        # Ubuntu 24.04 uses DEB822 *.sources; older images use sources.list / *.list.
        # Pin Iran/Arvan apt mirrors so we do not need Shecan for archive.ubuntu.com.
        rewrite_apt_host() {{
          local file="$1"
          [[ -f "$file" ]] || return 0
          sudo sed -i.bak \\
            -e "s|https\\?://archive.ubuntu.com/ubuntu/*|$APT_MIRROR|g" \\
            -e "s|https\\?://security.ubuntu.com/ubuntu/*|$APT_MIRROR|g" \\
            -e "s|https\\?://mirror.iranserver.com/ubuntu/*|$APT_MIRROR|g" \\
            "$file"
        }}
        rewrite_apt_host /etc/apt/sources.list
        shopt -s nullglob
        for f in /etc/apt/sources.list.d/*.list /etc/apt/sources.list.d/*.sources; do
          rewrite_apt_host "$f"
        done
        shopt -u nullglob
        echo "==> apt sources now:"
        grep -hE '^URIs:|^deb ' /etc/apt/sources.list /etc/apt/sources.list.d/* 2>/dev/null | grep -v '^#' | head -n 20 || true

        echo "==> apt update / upgrade"
        # Official Docker apt repo returns 403 from many Iran IPs — remove stale list first
        sudo rm -f /etc/apt/sources.list.d/docker.list
        sudo DEBIAN_FRONTEND=noninteractive apt-get update -y
        sudo DEBIAN_FRONTEND=noninteractive apt-get upgrade -y

        echo "==> base packages"
        sudo DEBIAN_FRONTEND=noninteractive apt-get install -y \\
          git curl wget nano vim tree htop unzip ca-certificates gnupg

        if [[ ! -f "$HOME/.ssh/id_ed25519" ]]; then
          echo "==> Generating ed25519 key for GitHub"
          ssh-keygen -t ed25519 -C "$SERVER_ID" -f "$HOME/.ssh/id_ed25519" -N ""
        else
          echo "==> ed25519 key already exists"
        fi
        echo
        echo "========== PUBLIC KEY (add at https://github.com/settings/keys) =========="
        cat "$HOME/.ssh/id_ed25519.pub"
        echo "=========================================================================="
        echo

        # Docker: download.docker.com is blocked (403). Use Ubuntu packages + registry mirror.
        if command -v docker >/dev/null 2>&1 && docker compose version >/dev/null 2>&1; then
          echo "==> Docker already installed: $(docker --version)"
          docker compose version || true
        else
          echo "==> Installing Docker from Ubuntu repos (docker.io + docker-compose-v2)"
          echo "    (skipping download.docker.com — 403 from Iran)"
          sudo DEBIAN_FRONTEND=noninteractive apt-get install -y \\
            docker.io docker-compose-v2
        fi

        # Keep local GPG around for operators who later switch to Docker CE via a reachable mirror
        if [[ -f /tmp/docker.gpg ]]; then
          sudo install -m 0755 -d /etc/apt/keyrings
          sudo gpg --batch --yes --dearmor -o /etc/apt/keyrings/docker.gpg /tmp/docker.gpg || true
          sudo chmod a+r /etc/apt/keyrings/docker.gpg 2>/dev/null || true
        fi

        sudo usermod -aG docker "$USER" || true
        sudo systemctl enable docker
        sudo systemctl start docker

        # Prefer configured mirror, then Arvan (Bamdad), then other Iran mirrors.
        sudo mkdir -p /etc/docker
        sudo tee /etc/docker/daemon.json >/dev/null <<EOF
        {{
          "registry-mirrors": [
            "$REGISTRY_MIRROR",
            "https://docker.arvancloud.ir",
            "https://docker.iranserver.com",
            "https://registry.docker.ir"
          ]
        }}
        EOF
        sudo systemctl daemon-reload
        sudo systemctl restart docker
        sleep 2

        echo "==> Registry mirrors:"
        sudo docker info 2>/dev/null | grep -A8 "Registry Mirrors" || true

        echo "==> DNS check before docker pull"
        sudo systemctl restart systemd-resolved || true
        sleep 1
        if ! dns_ok docker.arvancloud.ir && ! dns_ok docker.iranserver.com; then
          echo "==> WARNING: cannot resolve Docker mirrors — check DNS"
          resolvectl status || true
          getent hosts docker.arvancloud.ir || true
        fi

        echo "==> docker pull tests"
        PULL_OK=0
        for attempt in 1 2 3; do
          if sudo docker pull hello-world; then
            PULL_OK=1
            break
          fi
          echo "==> hello-world pull failed (attempt $attempt) — retrying after DNS bounce"
          sudo systemctl restart systemd-resolved || true
          sudo systemctl restart docker || true
          sleep 3
        done
        if [[ "$PULL_OK" -ne 1 ]]; then
          echo "ERROR: docker pull hello-world failed after retries (DNS/mirror)"
          exit 1
        fi
        sudo docker pull redis:8-alpine || true

        mkdir -p "$HOME/projects"
        echo
        echo "==> Setup done on $SERVER_ID"
        echo "    Manual next: add pubkey to GitHub, clone into ~/projects, then xerxes-ctrl start"
        echo "    Re-login once so docker group applies"
        """
    )


def _remote_script_global(*, server_id: str) -> str:
    return textwrap.dedent(
        f"""\
        #!/usr/bin/env bash
        set -euo pipefail
        SERVER_ID={shlex.quote(server_id)}
        echo "==> [$SERVER_ID] global VM bootstrap"

        sudo DEBIAN_FRONTEND=noninteractive apt-get update -y
        sudo DEBIAN_FRONTEND=noninteractive apt-get upgrade -y
        sudo DEBIAN_FRONTEND=noninteractive apt-get install -y \\
          git curl wget nano vim tree htop unzip ca-certificates gnupg

        if [[ ! -f "$HOME/.ssh/id_ed25519" ]]; then
          ssh-keygen -t ed25519 -C "$SERVER_ID" -f "$HOME/.ssh/id_ed25519" -N ""
        fi
        echo
        echo "========== PUBLIC KEY (add at https://github.com/settings/keys) =========="
        cat "$HOME/.ssh/id_ed25519.pub"
        echo "=========================================================================="
        echo

        if command -v docker >/dev/null 2>&1 && docker compose version >/dev/null 2>&1; then
          echo "==> Docker already installed: $(docker --version)"
        else
          sudo install -m 0755 -d /etc/apt/keyrings
          if [[ -f /tmp/docker.gpg ]]; then
            sudo gpg --batch --yes --dearmor -o /etc/apt/keyrings/docker.gpg /tmp/docker.gpg
          else
            curl -fsSL https://download.docker.com/linux/ubuntu/gpg | \\
              sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg
          fi
          sudo chmod a+r /etc/apt/keyrings/docker.gpg
          ARCH="$(dpkg --print-architecture)"
          CODENAME="$(. /etc/os-release && echo "$VERSION_CODENAME")"
          echo "deb [arch=$ARCH signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu $CODENAME stable" | \\
            sudo tee /etc/apt/sources.list.d/docker.list >/dev/null
          sudo DEBIAN_FRONTEND=noninteractive apt-get update -y
          sudo DEBIAN_FRONTEND=noninteractive apt-get install -y \\
            docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
        fi

        sudo usermod -aG docker "$USER" || true
        sudo systemctl enable docker
        sudo systemctl start docker
        mkdir -p "$HOME/projects"

        echo "==> Setup done on $SERVER_ID"
        """
    )


def setup_server(client: paramiko.SSHClient, server: dict[str, Any]) -> dict[str, Any]:
    iran = bool(server.get("iran_setup"))
    echo(f"[{server['id']}] uploading Docker GPG → /tmp/docker.gpg")
    if not GPG_PATH.is_file():
        return {"status": "failed", "error": f"missing GPG file: {GPG_PATH}", "output": ""}
    upload_file(client, GPG_PATH, "/tmp/docker.gpg")

    if iran:
        script = _remote_script_iran(
            server_id=server["id"],
            registry_mirror=server.get(
                "registry_mirror", "https://docker.arvancloud.ir"
            ),
            apt_mirror=server.get(
                "apt_mirror", "http://mirror.arvancloud.ir/ubuntu"
            ),
        )
    else:
        script = _remote_script_global(server_id=server["id"])

    remote_path = f"/tmp/foxg-ctrl-setup-{server['id']}.sh"
    sftp = client.open_sftp()
    try:
        with sftp.file(remote_path, "w") as rf:
            rf.write(script)
    finally:
        sftp.close()

    run_command(client, f"chmod +x {shlex.quote(remote_path)}", timeout=30)
    echo(f"[{server['id']}] remote setup running (may take several minutes)...")
    code, out, err = run_command(
        client,
        f"bash {shlex.quote(remote_path)}",
        timeout=3600,
    )
    combined = (out or "") + (("\n" + err) if err else "")
    return {
        "status": "success" if code == 0 else "failed",
        "output": combined,
        "error": "" if code == 0 else f"exit code {code}",
    }


def show_github_pubkey(
    client: paramiko.SSHClient, server: dict[str, Any]
) -> dict[str, Any]:
    _ = server
    code, out, err = run_command(
        client,
        "bash -lc 'if [[ -f ~/.ssh/id_ed25519.pub ]]; then cat ~/.ssh/id_ed25519.pub; "
        "else echo MISSING; fi'",
        timeout=30,
    )
    text = (out or err).strip()
    missing = "MISSING" in text
    return {
        "status": "success" if code == 0 and not missing else "failed",
        "output": text,
        "error": "" if not missing else "no ~/.ssh/id_ed25519.pub — run setup first",
    }
