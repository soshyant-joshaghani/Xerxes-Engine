#!/usr/bin/env bash
# Main CLI entry (pair with xerxes-ctrl.bat).
set -euo pipefail
cd "$(cd "$(dirname "$0")" && pwd)"

CTRL_NAME="xerxes-ctrl"

refresh_path() {
  # Homebrew (Apple Silicon / Intel)
  if [[ -x /opt/homebrew/bin/brew ]]; then
    eval "$(/opt/homebrew/bin/brew shellenv)"
  elif [[ -x /usr/local/bin/brew ]]; then
    eval "$(/usr/local/bin/brew shellenv)"
  fi
  # Common Linux user-local bins
  if [[ -d "$HOME/.local/bin" ]]; then
    export PATH="$HOME/.local/bin:$PATH"
  fi
}

find_python() {
  PY_BIN=""
  local candidates=("python3.13" "python3.12" "python3.11" "python3.10" "python3" "python")
  local c
  for c in "${candidates[@]}"; do
    if command -v "$c" >/dev/null 2>&1; then
      if "$c" -c "import sys; raise SystemExit(0 if sys.version_info>=(3,10) else 1)" >/dev/null 2>&1; then
        PY_BIN="$c"
        return 0
      fi
    fi
  done
  return 1
}

install_python() {
  echo "[$CTRL_NAME] Python 3.10+ not found. Installing..."
  local uname_s
  uname_s="$(uname -s 2>/dev/null || echo unknown)"
  case "$uname_s" in
    Darwin)
      if ! command -v brew >/dev/null 2>&1; then
        echo "[$CTRL_NAME] Homebrew not found. Install from https://brew.sh/ then retry."
        return 1
      fi
      brew install python@3.12
      ;;
    Linux)
      if ! command -v apt-get >/dev/null 2>&1; then
        echo "[$CTRL_NAME] apt-get not found. Install Python 3.10+ manually, then retry."
        return 1
      fi
      sudo apt-get update -y
      sudo DEBIAN_FRONTEND=noninteractive apt-get install -y python3 python3-venv python3-pip
      ;;
    *)
      echo "[$CTRL_NAME] Unsupported OS for auto-install. Install Python 3.10+ manually."
      return 1
      ;;
  esac
  refresh_path
  return 0
}

refresh_path

if ! find_python; then
  install_python || exit 1
  if ! find_python; then
    echo "[$CTRL_NAME] Python 3.10+ still not found after install. Open a new terminal and retry."
    exit 1
  fi
fi

# --- Create venv if missing --------------------------------------------
if [[ ! -x .venv/bin/python ]]; then
  echo "[$CTRL_NAME] Creating .venv and installing requirements..."
  "$PY_BIN" -m venv .venv
  .venv/bin/pip install --upgrade pip >/dev/null
  .venv/bin/pip install -r requirements.txt
fi

PY=(.venv/bin/python)

if [[ $# -eq 0 ]]; then
  exec "${PY[@]}" main.py
fi
exec "${PY[@]}" main.py "$@"
