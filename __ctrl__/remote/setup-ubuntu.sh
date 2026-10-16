#!/usr/bin/env bash
# One-time production VM setup for xerxes (Ubuntu 22.04 / 24.04).
#
# Prefer laptop bootstrap when possible:
#   __ctrl__/xerxes-ctrl.bat setup
#
# Or run on a fresh VM after cloning the repo:
#   chmod +x __ctrl__/remote/setup-ubuntu.sh
#   bash __ctrl__/remote/setup-ubuntu.sh
#
# Then edit compose.yml (DOMAIN, CORS) and .env (secrets), point DNS, and:
#   bash __ctrl__/remote/start-prod.sh
#   # or: xerxes-ctrl start   (from laptop via SSH)

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

if [[ "${EUID:-$(id -u)}" -eq 0 ]]; then
  echo "Run as a normal user with sudo — not as root."
  exit 1
fi

if ! command -v sudo >/dev/null 2>&1; then
  echo "sudo is required."
  exit 1
fi

echo "==> xerxes production VM setup"
echo "    Project: $ROOT"
echo

# ── OS check ────────────────────────────────────────────────────────────────
if [[ ! -f /etc/os-release ]]; then
  echo "Unsupported OS (expected Ubuntu)."
  exit 1
fi

# shellcheck disable=SC1091
source /etc/os-release
if [[ "${ID:-}" != "ubuntu" && "${ID_LIKE:-}" != *"debian"* ]]; then
  echo "Warning: tested on Ubuntu. Continuing on ${PRETTY_NAME:-unknown}..."
fi

# ── Base packages ───────────────────────────────────────────────────────────
echo "==> Installing base packages..."
sudo apt-get update -qq
sudo DEBIAN_FRONTEND=noninteractive apt-get install -y \
  ca-certificates \
  curl \
  gnupg \
  git \
  openssh-server \
  ufw

sudo systemctl enable ssh
sudo systemctl start ssh

# ── Docker ──────────────────────────────────────────────────────────────────
install_docker() {
  if command -v docker >/dev/null 2>&1 && docker compose version >/dev/null 2>&1; then
    echo "==> Docker already installed: $(docker --version)"
    docker compose version
    return 0
  fi

  echo "==> Installing Docker Engine + Compose plugin..."
  sudo install -m 0755 -d /etc/apt/keyrings
  if [[ ! -f /etc/apt/keyrings/docker.gpg ]]; then
    curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg
    sudo chmod a+r /etc/apt/keyrings/docker.gpg
  fi

  if [[ ! -f /etc/apt/sources.list.d/docker.list ]]; then
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu ${VERSION_CODENAME} stable" \
      | sudo tee /etc/apt/sources.list.d/docker.list >/dev/null
  fi

  sudo apt-get update -qq
  sudo DEBIAN_FRONTEND=noninteractive apt-get install -y \
    docker-ce \
    docker-ce-cli \
    containerd.io \
    docker-buildx-plugin \
    docker-compose-plugin

  sudo systemctl enable docker
  sudo systemctl start docker
}

install_docker

if ! groups "$USER" | grep -q '\bdocker\b'; then
  echo "==> Adding $USER to the docker group..."
  sudo usermod -aG docker "$USER"
  NEED_RELOGIN=1
else
  NEED_RELOGIN=0
fi

docker_cmd() {
  if docker info >/dev/null 2>&1; then
    docker "$@"
  else
    sudo docker "$@"
  fi
}

# ── Firewall (HTTP/HTTPS + SSH) ─────────────────────────────────────────────
echo "==> Configuring UFW..."
sudo ufw --force reset
sudo ufw default deny incoming
sudo ufw default allow outgoing
sudo ufw allow OpenSSH
sudo ufw allow 80/tcp comment 'HTTP (Traefik + ACME)'
sudo ufw allow 443/tcp comment 'HTTPS'
# Traefik dashboard — localhost only; do not expose publicly
sudo ufw --force enable
echo "    UFW status:"
sudo ufw status numbered

# ── Docker network for Traefik ──────────────────────────────────────────────
echo "==> Creating traefik-public network (if missing)..."
if ! docker_cmd network inspect traefik-public >/dev/null 2>&1; then
  docker_cmd network create traefik-public
fi

# ── Let's Encrypt storage (bind mount — survives volume prune) ─────────────
echo "==> Preparing ./letsencrypt for Traefik ACME..."
mkdir -p letsencrypt
if [[ ! -f letsencrypt/acme.json ]]; then
  touch letsencrypt/acme.json
fi
chmod 600 letsencrypt/acme.json

# ── Project env file ────────────────────────────────────────────────────────
if [[ ! -f .env ]]; then
  echo "==> Creating .env from .env.example..."
  cp .env.example .env
  echo "    Edit .env and set SECRET_KEY, POSTGRES_PASSWORD, FIRST_SUPERUSER_PASSWORD."
else
  echo "==> .env already exists — left unchanged."
fi

# ── Done ────────────────────────────────────────────────────────────────────
echo
echo "============================================"
echo " VM setup complete"
echo "============================================"
echo
echo "Before first deploy:"
echo "  1. Edit compose.yml — set DOMAIN, FRONTEND_HOST (game origin, for CORS), BACKEND_CORS_ORIGINS"
echo "  2. Edit compose.traefik.yml — set Let's Encrypt email (acme.email=...)"
echo "  3. Edit .env — production secrets (POSTGRES_PASSWORD, SECRET_KEY, …)"
echo "  4. DNS A records → this VM public IP:"
echo "       api   → https://api.<domain>    (backend)"
echo "       adminer → optional DB UI"
echo
echo "Deploy:"
echo "  bash __ctrl__/remote/start-prod.sh"
echo
echo "Stop / reset (prod):"
echo "  bash __ctrl__/remote/stop-prod.sh   — stop containers, keep DB + SSL"
echo "  bash __ctrl__/remote/reset-prod.sh  — wipe DB/Redis + app images, keep SSL"
echo "  Avoid: docker system prune -a --volumes  (deletes ALL volumes; use reset script instead)"
echo

if [[ "$NEED_RELOGIN" -eq 1 ]]; then
  echo "NOTE: Log out and back in (or run 'newgrp docker') so docker runs without sudo."
  echo
fi
