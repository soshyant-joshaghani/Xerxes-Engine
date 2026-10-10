"""xerxes-ctrl engine / game / template: run and publish the engine app and the games.

Games are Bevy only (UI in bevy_ui); the engine app adds the Dioxus editor UI over its Bevy
viewport, from the same source on every platform. Platforms (web | windows | android | ios):
  web      dx: one page with Bevy's canvas (engine app: Dioxus renders the page and the panel)
  windows  cargo: a Bevy window (engine app: the Dioxus panel drawn over it by Blitz; macOS and
           Linux behave the same); alias `win`
  android  cargo-apk: a NativeActivity APK (the `android` example) on the device adb sees
           (engine app: the Dioxus panel drawn over it by Blitz, touch forwarded)
  ios      needs macOS with Xcode: refused on other hosts, not wired yet on macOS

engine    the engine app (engine/), on its own:
            engine build <platform|all>                   compile the dev build so `run` starts fast
            engine run <platform> [--built] [--release]   live (hot reload on web) or the published build
            engine publish <platform|all>                 release build into dist/engine/<platform>/
game      the games users created with the engine (games/<name>, own Git repos), indexed 1.. A-Z
            game list | game build [target] [platform|all] | game dev|run [target] [platform]   (run = dev; --built runs the published build)
            game publish [target] [platform|all]
            game new <name> [--from starter]
template  finished games committed with the engine (games/__templates__/<name>), indexed 1.. A-Z
            template list | template build ... | template dev|run ... | template publish ...

`target` is an index or a name; without it the CLI asks. Everything shares one target dir
(engine/target), so Bevy is built once per feature set and platform.
"""

from __future__ import annotations

import argparse
import os
import re
import shutil
import socket
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

from lib.config import PROJECT
from lib.runtimes import ensure_targets
from lib.tooling import setup_local, tool_argv

ENGINE = PROJECT / "engine"
GAMES = PROJECT / "games"
TEMPLATES = GAMES / "__templates__"
DIST = PROJECT / "dist"
# XERXES_TARGET_DIR moves build output (a drive with room, say); the default is engine/target.
TARGET_DIR = Path(os.environ.get("XERXES_TARGET_DIR") or ENGINE / "target")
DEFAULT_TEMPLATE = "starter"
PLATFORMS = ("web", "windows", "android", "ios")
ALIASES = {"win": "windows"}
# Folder names a game may not take: the engine app's dist/engine/ and engine.play.localhost.
RESERVED_NAMES = {"engine"}
ANDROID_TRIPLES = {
    "arm64-v8a": "aarch64-linux-android",
    "x86_64": "x86_64-linux-android",
    "armeabi-v7a": "armv7-linux-androideabi",
    "x86": "i686-linux-android",
}
WEB_PORT = "5100"
# Games and templates serve web dev builds on their own port, so they run beside the engine
# app (the editor) on 5100. Traefik routes game.localhost here (at most one game runs at a time).
GAME_WEB_PORT = "5200"
WASM = "wasm32-unknown-unknown"


@dataclass(frozen=True)
class Target:
    name: str
    path: Path
    bin: str  # desktop executable name ([[bin]] name = package name)
    kind: str  # "engine app" | "game" | "template"


def _package_name(path: Path) -> str:
    match = re.search(r'^name\s*=\s*"([^"]+)"', (path / "Cargo.toml").read_text(encoding="utf-8"), re.M)
    return match.group(1) if match else path.name


def _crates_in(folder: Path, kind: str) -> list[Target]:
    if not folder.is_dir():
        return []
    paths = sorted((p for p in folder.iterdir() if (p / "Cargo.toml").is_file()), key=lambda p: p.name.lower())
    return [Target(p.name, p, _package_name(p), kind) for p in paths]


ENGINE_TARGET = Target("engine", ENGINE, "xerxes-engine", "engine app")


def game_index() -> list[Target]:
    """games/<name> (created with the engine), alphabetically. The engine has its own commands."""
    return _crates_in(GAMES, "game")


def template_index() -> list[Target]:
    return _crates_in(TEMPLATES, "template")


def _print_index(targets: list[Target]) -> None:
    if not targets:
        print("  (none)")
    for i, t in enumerate(targets, start=1):
        print(f"  {i:>2}  {t.name:<24} {t.kind}")


def _resolve(targets: list[Target], spec: str | None) -> Target | None:
    if spec is None:
        if not sys.stdin.isatty():
            print("error: name a target (index or name). See the list command.", file=sys.stderr)
            return None
        _print_index(targets)
        spec = input("Which one? [1] ").strip() or "1"
    if spec.isdigit() and 1 <= int(spec) <= len(targets):
        return targets[int(spec) - 1]
    for t in targets:
        if t.name == spec:
            return t
    print(f"error: unknown target {spec!r}. Available:", file=sys.stderr)
    _print_index(targets)
    return None


def _env() -> dict[str, str]:
    env = {**os.environ, "CARGO_TARGET_DIR": str(TARGET_DIR)}
    # Bevy at full parallelism can exhaust memory (Windows: "paging file is too small").
    env.setdefault("CARGO_BUILD_JOBS", str(max(2, (os.cpu_count() or 4) // 2)))
    return env


def _run(tool: str, args: list[str], cwd: Path) -> int:
    argv = tool_argv(tool, *args)
    if not argv:
        return 1
    print(f"[xerxes] ({cwd.relative_to(PROJECT).as_posix()}) {' '.join(argv)}")
    return subprocess.run(argv, cwd=cwd, env=_env()).returncode


# ---- tests -----------------------------------------------------------------


def run_engine_tests() -> int:
    """Engine library + app tests, the build crate (asset build step, project scaffold) and the
    simulation core (`xerxes_sim`), then
    the engine app must build for the web."""
    for label, folder in (
        ("Engine tests", ENGINE),
        ("Build crate (asset build step, scaffold)", ENGINE / "build"),
        ("Simulation core (xerxes_sim: physics, rules)", ENGINE / "sim"),
    ):
        print(f"[xerxes] {label} (cargo test)")
        code = _run("cargo", ["test"], folder)
        if code != 0:
            return code
    ensure_targets("web")
    print("[xerxes] Engine web check")
    return _run("cargo", ["check", "--features", "web", "--target", WASM], ENGINE)


def _test_crates(targets: list[Target], label: str) -> int:
    if not targets:
        print(f"[xerxes] No {label}.")
        return 0
    ensure_targets("web")
    for t in targets:
        print(f"[xerxes] {t.name}: cargo test, then web check")
        code = _run("cargo", ["test"], t.path)
        if code == 0:
            code = _run("cargo", ["check", "--features", "web", "--target", WASM], t.path)
        if code != 0:
            return code
    return 0


def run_template_tests() -> int:
    return _test_crates(template_index(), "templates in games/__templates__/")


def run_game_tests() -> int:
    return _test_crates(game_index(), "games under games/")


# ---- android ---------------------------------------------------------------


def _android_env() -> dict[str, str] | None:
    """cargo-apk needs the SDK and NDK; take them from the usual variables or the default SDK path."""
    env = _env()
    sdk = next(
        (Path(v) for v in (env.get("ANDROID_HOME"), env.get("ANDROID_SDK_ROOT"),
                           str(Path(env.get("LOCALAPPDATA", "")) / "Android" / "Sdk"),
                           # The slim setup without Android Studio: ~/Android/{Sdk,jdk-17}.
                           str(Path.home() / "Android" / "Sdk"))
         if v and Path(v).is_dir()),
        None,
    )
    if sdk is None:
        print("error: Android SDK not found. Set ANDROID_HOME (Android Studio > SDK Manager).", file=sys.stderr)
        return None
    ndk = next((Path(v) for v in (env.get("ANDROID_NDK_ROOT"), env.get("ANDROID_NDK_HOME"), env.get("NDK_HOME"))
                if v and Path(v).is_dir()), None)
    if ndk is None:
        versions = sorted((sdk / "ndk").iterdir()) if (sdk / "ndk").is_dir() else []
        ndk = versions[-1] if versions else None
    if ndk is None:
        print("error: Android NDK not found. Install it from the SDK Manager (SDK Tools > NDK).", file=sys.stderr)
        return None
    env.update({"ANDROID_HOME": str(sdk), "ANDROID_SDK_ROOT": str(sdk), "ANDROID_NDK_ROOT": str(ndk)})
    adb_dir = sdk / "platform-tools"
    env["PATH"] = f"{adb_dir}{os.pathsep}{env.get('PATH', '')}"
    # The APK signer and keytool are Java: JAVA_HOME, or the slim setup's JDK next to the SDK.
    jdk = next((Path(v) for v in (env.get("JAVA_HOME"), str(sdk.parent / "jdk-17")) if v and (Path(v) / "bin").is_dir()), None)
    if jdk is not None:
        env["JAVA_HOME"] = str(jdk)
        env["PATH"] = f"{jdk / 'bin'}{os.pathsep}{env['PATH']}"
    return env


def _cargo_apk_ready() -> bool:
    if shutil.which("cargo-apk"):
        return True
    print("[xerxes] Installing cargo-apk (packages Android APKs without Gradle)...")
    argv = tool_argv("cargo", "install", "cargo-apk")
    return bool(argv) and subprocess.run(argv).returncode == 0


def _android_device_triple(env: dict[str, str]) -> str | None:
    """The Rust target for the first device adb sees (phone: arm64, emulator: usually x86_64)."""
    adb = shutil.which("adb", path=env["PATH"])
    if not adb:
        return None
    devices = subprocess.run([adb, "devices"], capture_output=True, text=True, env=env).stdout
    if not any(line.endswith("\tdevice") for line in devices.splitlines()):
        return None
    abi = subprocess.run([adb, "shell", "getprop", "ro.product.cpu.abi"], capture_output=True, text=True, env=env)
    return ANDROID_TRIPLES.get(abi.stdout.strip())


def _apk(target: Target, args: list[str], env: dict[str, str]) -> int:
    argv = tool_argv("cargo", "apk", *args)
    if not argv:
        return 1
    print(f"[xerxes] ({target.path.relative_to(PROJECT).as_posix()}) {' '.join(argv)}")
    return subprocess.run(argv, cwd=target.path, env=env).returncode


def _dev_android(target: Target, release: bool) -> int:
    env = _android_env()
    if env is None or not _cargo_apk_ready():
        return 1
    triple = _android_device_triple(env)
    if triple is None:
        print("error: no Android device. Connect a phone with USB debugging, or start an emulator "
              "(Android Studio > Device Manager), then check `adb devices`.", file=sys.stderr)
        return 1
    subprocess.run(["rustup", "target", "add", triple], capture_output=True)
    flags = ["--release"] if release else []
    print(f"[xerxes] {target.name} on Android ({triple})")
    _reverse_backend(env)
    run = ["run", "--example", "android", "--features", "android", "--target", triple, *flags]
    if os.environ.get("XERXES_ALL_LOGS") == "1":
        return _apk(target, run, env)
    # The device's whole log is mostly other apps and system services (and keeps old runs):
    # clear it, start the app, then follow only the app's own output.
    adb = shutil.which("adb", path=env["PATH"]) or "adb"
    subprocess.run([adb, "logcat", "-c"], capture_output=True, env=env)
    code = _apk(target, [*run, "--no-logcat"], env)
    if code != 0:
        return code
    print("[xerxes] app log: Rust output, panics, crashes (Ctrl+C stops following; the app keeps "
          "running). Whole device log: set XERXES_ALL_LOGS=1.")
    return subprocess.run(
        [adb, "logcat", "-v", "time", *ANDROID_LOG_TAGS, "*:S"], env=env
    ).returncode


# The app's own log lines: Rust stdout/stderr (Bevy's log), Rust panics, Java crashes, native
# crash dumps.
ANDROID_LOG_TAGS = ["RustStdoutStderr:V", "RustPanic:V", "AndroidRuntime:E", "DEBUG:V", "libc:F"]


def _reverse_backend(env: dict) -> None:
    """Lets the phone reach the dev backend on this machine (`127.0.0.1:8000` on the device is
    forwarded to `:8000` here): a game's dev build uses it (`BackendSettings::dev`). The
    engine app's editor does not need it (projects are on the device)."""
    adb = shutil.which("adb", path=env["PATH"]) or "adb"
    done = subprocess.run([adb, "reverse", "tcp:8000", "tcp:8000"], capture_output=True, text=True, env=env)
    if done.returncode == 0:
        print("[xerxes] adb reverse tcp:8000 → the device reaches this machine's dev backend")
    else:
        print(f"warning: adb reverse failed ({done.stderr.strip()}); a game's dev build on the device cannot reach the backend", file=sys.stderr)


def _publish_android(target: Target) -> int:
    env = _android_env()
    if env is None or not _cargo_apk_ready():
        return 1
    subprocess.run(["rustup", "target", "add", "aarch64-linux-android"], capture_output=True)
    code = _apk(target, ["build", "--release", "--example", "android", "--features", "android"], env)
    if code != 0:
        return code
    apks = sorted((TARGET_DIR / "release").rglob("*.apk"), key=lambda p: p.stat().st_mtime)
    if not apks:
        print(f"error: no APK under {TARGET_DIR / 'release'}", file=sys.stderr)
        return 1
    out = DIST / target.name / "android"
    out.mkdir(parents=True, exist_ok=True)
    apk = out / f"{target.name}.apk"
    shutil.copy2(apks[-1], apk)
    print(f"[xerxes] {target.name} Android build: {apk.relative_to(PROJECT).as_posix()} "
          "(signed with the debug key until a release keystore is configured)")
    return 0


# ---- commands --------------------------------------------------------------


def _platform(value: str) -> str:
    return ALIASES.get(value, value)


def _ios_unavailable(target: Target) -> int:
    if sys.platform != "darwin":
        print(f"error: iOS builds of {target.name} need macOS with Xcode (Apple's toolchain and signing "
              "only run there). Run this command on a Mac.", file=sys.stderr)
    else:
        print("error: the iOS pipeline is not wired yet (it needs an Xcode project around the Bevy "
              "staticlib). It is on the plan.", file=sys.stderr)
    return 1


def _dev(target: Target, platform: str, release: bool) -> int:
    platform = _platform(platform)
    if platform == "ios":
        return _ios_unavailable(target)
    code = setup_local(backend=False)
    if code != 0:
        return code
    flags = ["--release"] if release else []
    if platform == "android":
        return _dev_android(target, release)
    if platform == "windows":
        return _run("cargo", ["run", *flags], target.path)
    ensure_targets("web")
    is_engine = target.name == ENGINE_TARGET.name
    wanted = WEB_PORT if is_engine else GAME_WEB_PORT
    port = _bindable_port(wanted)
    host = "engine.localhost" if is_engine else "game.localhost"
    if port != wanted:
        print(f"[xerxes] warning: port {wanted} cannot be bound on this machine; using {port}. "
              f"http://{host} (Traefik) points at {wanted}, so open http://localhost:{port} directly.")
    print(f"[xerxes] {target.name} on http://localhost:{port}  (http://{host} with `backend run dev` when the port is {wanted})")
    # 0.0.0.0 so Traefik (Docker) can route engine.localhost to it.
    return _run("dx", ["serve", "--platform", "web", "--port", port, "--addr", "0.0.0.0", *flags], target.path)


def _bindable_port(preferred: str) -> str:
    """`preferred` when a server can bind it, else the next port that can be bound.

    On Windows a bind can fail with error 10013 although nothing listens: Hyper-V, WinNAT,
    WSL and Docker reserve port ranges (see `netsh interface ipv4 show excludedportrange
    protocol=tcp`), and which ones changes. `XERXES_PORT_<port>` overrides the choice.
    """
    forced = os.environ.get(f"XERXES_PORT_{preferred}")
    if forced:
        return forced
    start = int(preferred)
    for port in range(start, start + 400):
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as probe:
            try:
                probe.bind(("0.0.0.0", port))
            except OSError:
                continue
        return str(port)
    return preferred


# A web build is a static site. Every web build (dev and publish, engine and games) gets a
# Dockerfile beside it, so it can run on another server: `docker build -t <name> <dir>` then
# `docker run -p 8080:80 <name>`. nginx serves the site (SPA fallback, wasm gzip); the engine
# app also proxies /api to the backend at XERXES_API.
_NGINX_TEMPLATE = """server {
    listen 80;
    root /usr/share/nginx/html;
    gzip on;
    gzip_types application/wasm application/javascript text/css application/zip;

    location / {
        try_files $uri $uri/ /index.html;
    }
%(api)s}
"""

_NGINX_API = """
    # The engine app's catalog and editor services (backend on another host or container).
    location /api/ {
        proxy_pass ${XERXES_API}/api/;
        proxy_set_header Host $host;
    }
"""


def _dockerize_web(target: Target, out: Path, site: Path) -> None:
    """Writes out/Dockerfile and out/nginx.conf.template serving `site` (a folder under out)."""
    engine = target.name == ENGINE_TARGET.name
    (out / "nginx.conf.template").write_text(_NGINX_TEMPLATE % {"api": _NGINX_API if engine else ""}, encoding="utf-8")
    lines = [
        f"# {target.name} web build, served by nginx. Written by xerxes-ctrl; rebuilt with the site.",
        f"#   docker build -t {target.name}-web .",
        f"#   docker run -p 8080:80 {target.name}-web" + (" -e XERXES_API=http://<backend>:8000" if engine else ""),
        "FROM nginx:1.27-alpine",
        f"COPY {site.relative_to(out).as_posix()}/ /usr/share/nginx/html/",
        # The nginx image renders /etc/nginx/templates/*.template with the container's env.
        "COPY nginx.conf.template /etc/nginx/templates/default.conf.template",
    ]
    if engine:
        lines.append("ENV XERXES_API=http://host.docker.internal:8000")
    lines.append("EXPOSE 80")
    (out / "Dockerfile").write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"[xerxes] {target.name}: Dockerfile in {out.relative_to(PROJECT).as_posix()} (docker build -t {target.name}-web .)")


def _publish_web(target: Target) -> int:
    out = DIST / target.name / "web"
    if out.exists():
        shutil.rmtree(out)
    ensure_targets("web")
    code = _run("dx", ["bundle", "--platform", "web", "--release", "--out-dir", str(out)], target.path)
    if code == 0:
        site = next((p.parent for p in out.rglob("index.html")), out)
        print(f"[xerxes] {target.name} web build: {site.relative_to(PROJECT).as_posix()} (serve it as a static site)")
        _dockerize_web(target, out, site)
    return code


def _publish_windows(target: Target) -> int:
    code = _run("cargo", ["build", "--release"], target.path)
    if code != 0:
        return code
    exe = target.bin + (".exe" if sys.platform == "win32" else "")
    built = TARGET_DIR / "release" / exe
    if not built.is_file():
        print(f"error: expected {built}", file=sys.stderr)
        return 1
    out = DIST / target.name / "windows"
    out.mkdir(parents=True, exist_ok=True)
    shutil.copy2(built, out / exe)
    # Asset-model games ship only their bundles (public/, packed by build.rs from assets/);
    # hand-written games ship assets/ as Bevy's default root.
    shipped = next((d for d in ("public", "assets") if (target.path / d).is_dir()), None)
    if shipped:
        shutil.rmtree(out / shipped, ignore_errors=True)
        shutil.copytree(target.path / shipped, out / shipped)
    print(f"[xerxes] {target.name} desktop build: {(out / exe).relative_to(PROJECT).as_posix()}")
    return 0


def _build(target: Target, platform: str) -> int:
    """Compile the dev build for one platform (or every platform this host can build), so the
    next `dev`/`run` only links and starts."""
    platform = _platform(platform)
    if platform == "ios":
        return _ios_unavailable(target)
    code = setup_local(backend=False)
    if code != 0:
        return code
    for name in [p for p in PLATFORMS if p != "ios"] if platform == "all" else (platform,):
        if name == "web":
            ensure_targets("web")
            code = _run("dx", ["build", "--platform", "web"], target.path)
            if code == 0:
                code = _stage_web_dev(target)
        elif name == "windows":
            code = _run("cargo", ["build"], target.path)
        else:
            env = _android_env()
            if env is None or not _cargo_apk_ready():
                return 1
            triple = _android_device_triple(env) or "aarch64-linux-android"
            subprocess.run(["rustup", "target", "add", triple], capture_output=True)
            code = _apk(target, ["build", "--example", "android", "--features", "android", "--target", triple], env)
        if code != 0:
            return code
        print(f"[xerxes] {target.name}: {name} build ready")
    return 0


def _stage_web_dev(target: Target) -> int:
    """Copies the web dev build (dx's debug site) to dist/<name>/web-dev/ with its Dockerfile,
    so a dev build can run on another server too."""
    site = TARGET_DIR / "dx" / target.bin / "debug" / "web" / "public"
    if not (site / "index.html").is_file():
        print(f"error: expected the dx web dev build in {site}", file=sys.stderr)
        return 1
    out = DIST / target.name / "web-dev"
    shutil.rmtree(out, ignore_errors=True)
    shutil.copytree(site, out / "public")
    _dockerize_web(target, out, out / "public")
    return 0


def _publish(target: Target, platform: str) -> int:
    platform = _platform(platform)
    if platform == "ios":
        return _ios_unavailable(target)
    code = setup_local(backend=False)
    if code != 0:
        return code
    steps = {"web": _publish_web, "windows": _publish_windows, "android": _publish_android}
    # `all` covers what this host can build; iOS only when asked for by name.
    for name in [p for p in PLATFORMS if p != "ios"] if platform == "all" else (platform,):
        code = steps[name](target)
        if code != 0:
            return code
    return 0


# ---- built runs ------------------------------------------------------------


def _serve_built_web(target: Target) -> int:
    """Serve dist/<name>/web on :5100 like a static host, with /api proxied to the backend on :8000
    (the engine's game catalog), and index.html for unknown paths."""
    import http.server
    import urllib.error
    import urllib.request

    site = next((p.parent for p in (DIST / target.name / "web").rglob("index.html")), None)
    if site is None:
        print(f"error: no web build. Run: xerxes-ctrl {'engine' if target == ENGINE_TARGET else 'game'} "
              f"publish {'' if target == ENGINE_TARGET else target.name + ' '}web", file=sys.stderr)
        return 1

    class Handler(http.server.SimpleHTTPRequestHandler):
        extensions_map = {**http.server.SimpleHTTPRequestHandler.extensions_map, ".wasm": "application/wasm"}

        def __init__(self, *args, **kwargs):
            super().__init__(*args, directory=str(site), **kwargs)

        def _proxy(self):
            body = self.rfile.read(int(self.headers.get("Content-Length") or 0)) or None
            request = urllib.request.Request(f"http://127.0.0.1:8000{self.path}", data=body, method=self.command)
            for key in ("Content-Type", "Authorization"):
                if self.headers.get(key):
                    request.add_header(key, self.headers[key])
            try:
                with urllib.request.urlopen(request, timeout=30) as reply:
                    status, headers, payload = reply.status, reply.headers, reply.read()
            except urllib.error.HTTPError as err:
                status, headers, payload = err.code, err.headers, err.read()
            except OSError:
                status, headers, payload = 502, {}, b'{"detail": "backend not running (backend run dev)"}'
            self.send_response(status)
            self.send_header("Content-Type", headers.get("Content-Type", "application/json"))
            self.send_header("Content-Length", str(len(payload)))
            self.end_headers()
            self.wfile.write(payload)

        def do_GET(self):
            if self.path.startswith("/api/"):
                return self._proxy()
            if not (site / self.path.split("?")[0].lstrip("/")).exists():
                self.path = "/index.html"
            return super().do_GET()

        def do_POST(self):
            return self._proxy() if self.path.startswith("/api/") else self.send_error(405)

    # 0.0.0.0 like `dx serve`, so Traefik (Docker) can route engine.localhost to it.
    served_port = int(_bindable_port(WEB_PORT))
    server = http.server.ThreadingHTTPServer(("0.0.0.0", served_port), Handler)
    print(f"[xerxes] {target.name} (built) on http://localhost:{served_port}  from {site.relative_to(PROJECT).as_posix()}  (Ctrl+C stops)")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    return 0


def _run_built(target: Target, platform: str) -> int:
    platform = _platform(platform)
    if platform == "ios":
        return _ios_unavailable(target)
    if platform == "web":
        return _serve_built_web(target)
    if platform == "windows":
        exe = DIST / target.name / "windows" / (target.bin + (".exe" if sys.platform == "win32" else ""))
        if not exe.is_file():
            print(f"error: no windows build at {exe.relative_to(PROJECT).as_posix()}. Publish it first.", file=sys.stderr)
            return 1
        print(f"[xerxes] {exe.relative_to(PROJECT).as_posix()}")
        return subprocess.run([str(exe)], cwd=exe.parent).returncode
    env = _android_env()
    if env is None:
        return 1
    apk = DIST / target.name / "android" / f"{target.name}.apk"
    if not apk.is_file():
        print(f"error: no Android build at {apk.relative_to(PROJECT).as_posix()}. Publish it first.", file=sys.stderr)
        return 1
    if _android_device_triple(env) is None:
        print("error: no Android device (check `adb devices`).", file=sys.stderr)
        return 1
    adb = shutil.which("adb", path=env["PATH"]) or "adb"
    match = re.search(r'^package\s*=\s*"([^"]+)"', (target.path / "Cargo.toml").read_text(encoding="utf-8"), re.M)
    _reverse_backend(env)
    if subprocess.run([adb, "install", "-r", str(apk)], env=env).returncode != 0:
        return 1
    if match:
        subprocess.run([adb, "shell", "am", "start", "-n", f"{match.group(1)}/android.app.NativeActivity"], env=env)
    return 0


def cmd_engine_run(args: argparse.Namespace) -> int:
    if args.built:
        return _run_built(ENGINE_TARGET, args.platform)
    return _dev(ENGINE_TARGET, args.platform, args.release)


def cmd_engine_build(args: argparse.Namespace) -> int:
    return _build(ENGINE_TARGET, args.platform)


def cmd_engine_publish(args: argparse.Namespace) -> int:
    return _publish(ENGINE_TARGET, args.platform)


def _commands(index):
    def cmd_list(_: argparse.Namespace) -> int:
        _print_index(index())
        return 0

    def cmd_dev(args: argparse.Namespace) -> int:
        target = _resolve(index(), args.target)
        if target is None:
            return 1
        if getattr(args, "built", False):
            return _run_built(target, args.platform)
        return _dev(target, args.platform, args.release)

    def cmd_publish(args: argparse.Namespace) -> int:
        target = _resolve(index(), args.target)
        return 1 if target is None else _publish(target, args.platform)

    def cmd_build(args: argparse.Namespace) -> int:
        target = _resolve(index(), args.target)
        return 1 if target is None else _build(target, args.platform)

    return cmd_list, cmd_dev, cmd_publish, cmd_build


def cmd_new(args: argparse.Namespace) -> int:
    """New game from a template game: the shared Rust scaffold (`xerxes_build::scaffold`,
    binary `xerxes-new`) does the copy and renames, the same code the editor and the backend
    use; then the game gets its own Git repository."""
    name = args.name
    dest = GAMES / name
    code = _run(
        "cargo",
        [
            "run", "-q", "--manifest-path", str(ENGINE / "build" / "Cargo.toml"), "--bin", "xerxes-new", "--",
            str(TEMPLATES), args.template, str(GAMES), name, "--engine-path", "../../engine",
        ],
        PROJECT,
    )
    if code != 0:
        if args.template != DEFAULT_TEMPLATE and not (TEMPLATES / args.template / "Cargo.toml").is_file():
            _print_index(template_index())
        return code
    subprocess.run(["git", "init", "-q"], cwd=dest)
    position = next(i for i, t in enumerate(game_index(), start=1) if t.name == name)
    print(f"[xerxes] Created games/{name} from {args.template} (own Git repository), index {position}.")
    print(f"        Next: xerxes-ctrl game dev {name} web")
    return 0


def _add_pipeline(sp: argparse.ArgumentParser, index, what: str) -> argparse._SubParsersAction:
    cmd_list, cmd_dev, cmd_publish, cmd_build = _commands(index)
    actions = sp.add_subparsers(dest=f"{what}_action", required=True)

    p = actions.add_parser("list", help=f"The {what} index")
    p.set_defaults(func=cmd_list)

    p = actions.add_parser("build", help="Compile the dev build so `dev` starts fast (web, windows, android or all)")
    p.add_argument("target", nargs="?", help="index or name (asks when omitted)")
    p.add_argument("platform", nargs="?", default="web", choices=(*PLATFORMS, *ALIASES, "all"))
    p.set_defaults(func=cmd_build)

    p = actions.add_parser("dev", help="web: dx serve on :5200 (hot reload); windows: cargo run; android: APK on the adb device")
    p.add_argument("target", nargs="?", help="index or name (asks when omitted)")
    p.add_argument("platform", nargs="?", default="web", choices=(*PLATFORMS, *ALIASES))
    p.add_argument("--release", action="store_true")
    p.set_defaults(func=cmd_dev)

    p = actions.add_parser("run", help=f"Same as `dev` (e.g. `{what} run 1 android`); --built runs the published build from dist/")
    p.add_argument("target", nargs="?", help="index or name (asks when omitted)")
    p.add_argument("platform", nargs="?", default="web", choices=(*PLATFORMS, *ALIASES))
    p.add_argument("--built", action="store_true", help="run the published build from dist/<name>/<platform>/")
    p.add_argument("--release", action="store_true")
    p.set_defaults(func=cmd_dev)

    p = actions.add_parser("publish", help="Release build into dist/<name>/<platform>/")
    p.add_argument("target", nargs="?", help="index or name (asks when omitted)")
    p.add_argument("platform", nargs="?", default="web", choices=(*PLATFORMS, *ALIASES, "all"))
    p.set_defaults(func=cmd_publish)
    return actions


def build_engine_subparser(sub: argparse._SubParsersAction) -> None:
    sp = sub.add_parser("engine", help="The engine app: build, run (live or --built) and publish per platform")
    actions = sp.add_subparsers(dest="engine_action", required=True)

    p = actions.add_parser("build", help="Compile the dev build so `run` starts fast (web, windows, android or all)")
    p.add_argument("platform", nargs="?", default="web", choices=(*PLATFORMS, *ALIASES, "all"))
    p.set_defaults(func=cmd_engine_build)

    p = actions.add_parser("run", help="web: dx serve :5100 (hot reload); windows: cargo run; android: APK on the adb device")
    p.add_argument("platform", nargs="?", default="web", choices=(*PLATFORMS, *ALIASES))
    p.add_argument("--built", action="store_true", help="run the published build from dist/engine/<platform>/")
    p.add_argument("--release", action="store_true")
    p.set_defaults(func=cmd_engine_run)

    p = actions.add_parser("publish", help="Release build into dist/engine/<platform>/")
    p.add_argument("platform", nargs="?", default="web", choices=(*PLATFORMS, *ALIASES, "all"))
    p.set_defaults(func=cmd_engine_publish)


def build_game_subparser(sub: argparse._SubParsersAction) -> None:
    """`game` and `template` (the engine app's `engine` command is build_engine_subparser)."""
    sp = sub.add_parser("game", help="Games in games/ (1.. A-Z): list, build, dev, publish, new")
    actions = _add_pipeline(sp, game_index, "game")
    p = actions.add_parser("new", help="New game from the engine's starter (or a finished template game) into games/<name>, and git init")
    p.add_argument("name")
    p.add_argument("--from", dest="template", default=DEFAULT_TEMPLATE, help=f"`{DEFAULT_TEMPLATE}` (the engine's built-in empty project) or a games/__templates__ name (default: {DEFAULT_TEMPLATE})")
    p.set_defaults(func=cmd_new)

    sp = sub.add_parser("template", help="Templates in games/__templates__/: list, build, dev, publish")
    _add_pipeline(sp, template_index, "template")
