from __future__ import annotations

import json
import socket
import subprocess
import sys
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from datetime import datetime
from pathlib import Path
from threading import Lock
from typing import Any, Callable

import paramiko

from .config import ROOT, require_host
from .keys import resolve_ssh_key


# Windows PowerShell may expose a CP1252 stream. Docker Compose emits Unicode
# status symbols, so make console output loss-tolerant instead of crashing after
# a successful remote deployment.
for _stream in (sys.stdout, sys.stderr):
    if hasattr(_stream, "reconfigure"):
        _stream.reconfigure(errors="replace")

print_lock = Lock()
_log_file = None

MAX_WORKERS = 5
MAX_RETRIES = 3
RETRY_DELAY = 2
PING_TIMEOUT = 3
CONNECT_TIMEOUT = 15
DEFAULT_CMD_TIMEOUT = 120

LOGS_DIR = ROOT / "logs"


def _ensure_logs_dir() -> Path:
    LOGS_DIR.mkdir(parents=True, exist_ok=True)
    return LOGS_DIR


def log(message: str) -> None:
    global _log_file
    if _log_file is None:
        stamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        path = _ensure_logs_dir() / f"ssh_log_{stamp}.txt"
        _log_file = open(path, "w", encoding="utf-8")
    ts = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    _log_file.write(f"[{ts}] {message}\n")
    _log_file.flush()


def close_log() -> None:
    global _log_file
    if _log_file is not None:
        _log_file.close()
        _log_file = None


def echo(message: str, *, end: str = "\n") -> None:
    with print_lock:
        print(message, end=end, flush=True)


def ping_host(ip: str, timeout: int = PING_TIMEOUT) -> bool:
    try:
        if sys.platform == "win32":
            cmd = ["ping", "-n", "1", "-w", str(timeout * 1000), ip]
        else:
            cmd = ["ping", "-c", "1", "-W", str(timeout), ip]
        proc = subprocess.run(cmd, capture_output=True, timeout=timeout + 2)
        return proc.returncode == 0
    except Exception:
        return False


def connect(server: dict[str, Any]) -> paramiko.SSHClient:
    host = require_host(server)
    try:
        key_path = resolve_ssh_key(server["key"])
    except FileNotFoundError as exc:
        raise FileNotFoundError(f"SSH key not found: {exc}") from exc

    client = paramiko.SSHClient()
    client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
    client.connect(
        hostname=host,
        port=int(server.get("port", 22)),
        username=server.get("user", "ubuntu"),
        key_filename=key_path,
        timeout=CONNECT_TIMEOUT,
        banner_timeout=CONNECT_TIMEOUT,
        auth_timeout=CONNECT_TIMEOUT,
        allow_agent=False,
        look_for_keys=False,
    )
    transport = client.get_transport()
    if transport is not None:
        transport.set_keepalive(30)
    return client


def run_command(
    client: paramiko.SSHClient,
    command: str,
    *,
    timeout: int | None = DEFAULT_CMD_TIMEOUT,
    get_pty: bool = True,
    stream: bool = False,
) -> tuple[int, str, str]:
    """Run remote command; drain stdout/stderr concurrently.

    Blocking ``stdout.read()`` then ``stderr.read()`` can stall or drop the
    channel during long ``docker compose`` builds (exit -1). Streaming keeps
    keepalive traffic flowing and avoids stderr window deadlock.

    ``timeout=None`` means no wall-clock deadline (for ``docker compose logs -f``).
    Interrupt with Ctrl+C; the remote channel is closed on KeyboardInterrupt.
    """
    # paramiko exec_command timeout is the channel open timeout, not wall clock.
    open_timeout = DEFAULT_CMD_TIMEOUT if timeout is None else max(timeout, 1)
    stdin, stdout, stderr = client.exec_command(
        command, timeout=open_timeout, get_pty=get_pty
    )
    stdin.close()
    channel = stdout.channel
    # Idle recv timeout: long docker build steps may be quiet for minutes.
    # Overall wall clock is still bounded by the caller's timeout + keepalive
    # (unless timeout is None for follow-mode log streams).
    channel.settimeout(1.0)

    out_chunks: list[bytes] = []
    err_chunks: list[bytes] = []
    deadline = None if timeout is None else time.time() + max(timeout, 1)

    def _drain() -> None:
        while channel.recv_ready():
            chunk = channel.recv(65535)
            if not chunk:
                break
            out_chunks.append(chunk)
            if stream:
                echo(chunk.decode("utf-8", errors="ignore"), end="")
        while channel.recv_stderr_ready():
            chunk = channel.recv_stderr(65535)
            if not chunk:
                break
            err_chunks.append(chunk)
            if stream:
                echo(chunk.decode("utf-8", errors="ignore"), end="")

    try:
        while True:
            if deadline is not None and time.time() > deadline:
                channel.close()
                raise TimeoutError(f"remote command exceeded {timeout}s")
            try:
                _drain()
            except socket.timeout:
                pass
            if channel.exit_status_ready():
                break
            time.sleep(0.05)
    except KeyboardInterrupt:
        try:
            channel.close()
        except Exception:
            pass
        raise

    # Final drain after exit status is known
    try:
        _drain()
    except socket.timeout:
        pass
    while True:
        try:
            chunk = channel.recv(65535)
            if not chunk:
                break
            out_chunks.append(chunk)
            if stream:
                echo(chunk.decode("utf-8", errors="ignore"), end="")
        except socket.timeout:
            break
    while True:
        try:
            chunk = channel.recv_stderr(65535)
            if not chunk:
                break
            err_chunks.append(chunk)
            if stream:
                echo(chunk.decode("utf-8", errors="ignore"), end="")
        except socket.timeout:
            break

    code = channel.recv_exit_status()
    out = b"".join(out_chunks).decode("utf-8", errors="ignore")
    err = b"".join(err_chunks).decode("utf-8", errors="ignore")
    return code, out, err


def upload_file(
    client: paramiko.SSHClient,
    local_path: str | Path,
    remote_path: str,
) -> None:
    sftp = client.open_sftp()
    try:
        sftp.put(str(local_path), remote_path)
    finally:
        sftp.close()


def wait_exponential(attempt: int) -> float:
    return RETRY_DELAY * (2**attempt)


def with_retries(
    server: dict[str, Any],
    work: Callable[[paramiko.SSHClient], dict[str, Any]],
    *,
    skip_ping: bool = False,
) -> dict[str, Any]:
    host = require_host(server)
    result: dict[str, Any] = {
        "id": server["id"],
        "ip": host,
        "project": server.get("project"),
        "status": "pending",
        "output": "",
        "error": "",
        "attempts": 0,
        "ping_ok": False,
    }

    if not skip_ping:
        echo(f"\n[{server['id']}] ping {host}...")
        result["ping_ok"] = ping_host(host)
        if not result["ping_ok"]:
            result["status"] = "offline"
            result["error"] = "host offline / ping failed"
            log(f"[{server['id']}] offline")
            return result
        echo(f"[{server['id']}] online")

    last_error = ""
    for attempt in range(MAX_RETRIES):
        result["attempts"] = attempt + 1
        client: paramiko.SSHClient | None = None
        try:
            echo(f"[{server['id']}] SSH attempt {attempt + 1}/{MAX_RETRIES}")
            client = connect(server)
            payload = work(client)
            result.update(payload)
            if result.get("status") == "pending":
                result["status"] = "success"
            log(f"[{server['id']}] {result['status']}")
            return result
        except paramiko.AuthenticationException:
            result["status"] = "auth_failed"
            result["error"] = "authentication failed"
            log(f"[{server['id']}] auth failed")
            return result
        except Exception as exc:
            last_error = str(exc)
            result["error"] = last_error
            log(f"[{server['id']}] error: {exc}")
            if attempt < MAX_RETRIES - 1:
                time.sleep(wait_exponential(attempt))
        finally:
            if client is not None:
                client.close()

    result["status"] = "failed"
    result["error"] = last_error or "unknown error"
    return result


def run_parallel(
    servers: list[dict[str, Any]],
    work: Callable[[paramiko.SSHClient, dict[str, Any]], dict[str, Any]],
    *,
    skip_ping: bool = False,
) -> list[dict[str, Any]]:
    results: list[dict[str, Any]] = []

    def task(server: dict[str, Any]) -> dict[str, Any]:
        return with_retries(
            server,
            lambda client: work(client, server),
            skip_ping=skip_ping,
        )

    with ThreadPoolExecutor(max_workers=MAX_WORKERS) as pool:
        futures = {pool.submit(task, s): s for s in servers}
        for fut in as_completed(futures):
            results.append(fut.result())
    return results


def save_results_json(results: list[dict[str, Any]]) -> str:
    stamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    path = _ensure_logs_dir() / f"results_{stamp}.json"
    with open(path, "w", encoding="utf-8") as f:
        json.dump(results, f, indent=2, ensure_ascii=False)
    return str(path)


def print_result(result: dict[str, Any]) -> None:
    with print_lock:
        print(f"\n{'=' * 60}")
        print(f"server: {result.get('id')} ({result.get('ip')})")
        print(f"project: {result.get('project')}")
        print(f"status: {result.get('status')}")
        if result.get("attempts"):
            print(f"attempts: {result['attempts']}")
        if result.get("output"):
            print("-" * 40)
            print(result["output"].rstrip())
        if result.get("error") and result.get("status") != "success":
            print(f"error: {result['error']}")
        print("=" * 60)


def remote_home_path(path: str) -> str:
    """Turn ~/foo into $HOME/foo so it works inside double-quoted bash strings."""
    if path.startswith("~/"):
        return "$HOME/" + path[2:]
    if path == "~":
        return "$HOME"
    return path


def remote_cwd_cmd(server: dict[str, Any], command: str) -> str:
    """cd into the VM project dir (~/projects/<repo>) then run command."""
    remote_dir = remote_home_path(server.get("remote_dir") or "~/projects")
    sid = server.get("id", "?")
    # Fail clearly if clone was never run on this VM.
    inner = (
        f'target="{remote_dir}"; '
        f'if [[ ! -d "$target" ]]; then '
        f'echo "missing $target - run: xerxes-ctrl clone {sid}"; exit 1; '
        f'fi; '
        f'cd "$target" && pwd && {command}'
    )
    quoted = "'" + inner.replace("'", "'\"'\"'") + "'"
    return f"bash -lc {quoted}"
