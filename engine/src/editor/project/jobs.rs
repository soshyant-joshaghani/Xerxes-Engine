//! Jobs: running the build pipeline (`game dev | build | publish`) from the editor.
//!
//! - The web editor asks the local backend, which runs `xerxes-ctrl` on the host and keeps the
//!   job's output (`/api/v1/engine/jobs`).
//! - The native editor runs the same pipeline itself when it lives inside the Xerxes repo
//!   (`store::repo_root`); a standalone editor has no pipeline to run and says so.
//!
//! The types and the reading of the output are plain data, tested.

use serde::{Deserialize, Serialize};

/// What to run: the same words the command line takes (`game dev my-game web`).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct JobRequest {
    /// `dev` (run it), `build` or `publish`.
    pub command: String,
    /// The project's name.
    pub target: String,
    /// `web`, `windows` or `android`.
    pub platform: String,
    /// A template game (the pipeline's `template` group, not `game`). The backend finds this
    /// out from the folder; the native runner needs to be told.
    #[serde(skip)]
    pub template: bool,
}

/// A job as the backend (or the local runner) reports it.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct JobStatus {
    pub id: u64,
    #[serde(default)]
    pub args: Vec<String>,
    /// `running`, `succeeded`, `failed` or `stopped`.
    pub status: String,
    #[serde(default)]
    pub exit_code: Option<i32>,
    #[serde(default)]
    pub log: Vec<String>,
}

impl JobStatus {
    pub fn running(&self) -> bool {
        self.status == "running"
    }

    /// The job in words: `game dev my-game web`.
    pub fn title(&self) -> String {
        self.args.join(" ")
    }

    /// Whether it is a `dev` job (it keeps running to serve the game).
    pub fn is_dev(&self) -> bool {
        self.args.get(1).is_some_and(|c| c == "dev" || c == "run")
    }

    /// The project the job is for.
    pub fn target(&self) -> Option<&str> {
        self.args.get(2).map(String::as_str)
    }

    /// Where a running web job serves the game, from its output.
    pub fn preview_url(&self) -> Option<String> {
        preview_url(&self.log)
    }

    /// Where a build or publish put its result (`dist/...`), from its output.
    pub fn output(&self) -> Option<String> {
        output_path(&self.log)
    }
}

/// Removes the colour codes a terminal program writes (`ESC [ ... letter`).
pub fn strip_ansi(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            if chars.peek() == Some(&'[') {
                chars.next();
                for next in chars.by_ref() {
                    if next.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
            continue;
        }
        out.push(c);
    }
    out
}

/// The address the pipeline says it serves the game on: its own line `[xerxes] <game> on
/// http://localhost:<port> ...` (the newest one), not any other address in the output (the
/// dev server also mentions the backend it proxies to).
pub fn preview_url(log: &[String]) -> Option<String> {
    log.iter().rev().find_map(|line| {
        let line = strip_ansi(line);
        if !line.contains("[xerxes]") {
            return None;
        }
        let start = line.find("http://localhost:")?;
        let rest = &line[start..];
        let end = rest
            .char_indices()
            .find(|(i, c)| *i > 17 && !c.is_ascii_digit())
            .map_or(rest.len(), |(i, _)| i);
        let url = &rest[..end];
        // `http://localhost:` alone is not an address.
        (url.len() > 17).then(|| url.to_string())
    })
}

/// The `dist/...` path a build or publish says it made: the newest `[xerxes]` line that says
/// `build:` and names one (`web build: dist/x/web/public (...)`, `desktop build:
/// dist/x/windows/x.exe`), else the newest `[xerxes]` line that names one at all (a Dockerfile
/// line, say).
pub fn output_path(log: &[String]) -> Option<String> {
    let path_in = |line: &str| -> Option<String> {
        let rest = &line[line.find("dist/")?..];
        let end = rest
            .find(|c: char| c.is_whitespace() || c == ')')
            .unwrap_or(rest.len());
        (end > "dist/".len()).then(|| rest[..end].to_string())
    };
    let lines: Vec<String> = log
        .iter()
        .map(|l| strip_ansi(l))
        .filter(|l| l.contains("[xerxes]"))
        .collect();
    lines
        .iter()
        .rev()
        .filter(|l| l.contains("build:"))
        .find_map(|l| path_in(l))
        .or_else(|| lines.iter().rev().find_map(|l| path_in(l)))
}

/// Opens an address in the user's browser (a new tab on the web).
pub fn open_in_browser(url: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            let _ = window.open_with_url_and_target(url, "_blank");
        }
    }
    #[cfg(all(not(target_arch = "wasm32"), target_os = "windows"))]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn();
    }
    #[cfg(all(not(target_arch = "wasm32"), target_os = "macos"))]
    {
        let _ = std::process::Command::new("open").arg(url).spawn();
    }
    #[cfg(all(
        not(target_arch = "wasm32"),
        not(target_os = "windows"),
        not(target_os = "macos"),
        not(target_os = "android")
    ))]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }
    #[cfg(target_os = "android")]
    let _ = url;
}

/// The pipeline run in a process on this machine: what the native editor does (the backend
/// does the same on the web's behalf).
#[cfg(not(target_arch = "wasm32"))]
pub mod local {
    use std::collections::HashMap;
    use std::io::{BufRead, BufReader, Read};
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};
    use std::sync::{Arc, Mutex, OnceLock};

    use super::{JobRequest, JobStatus};

    /// Lines of output kept per job.
    const LOG_LINES: usize = 400;

    struct Job {
        status: Arc<Mutex<JobStatus>>,
        pid: Arc<Mutex<Option<u32>>>,
    }

    fn jobs() -> &'static Mutex<(u64, HashMap<u64, Job>)> {
        static JOBS: OnceLock<Mutex<(u64, HashMap<u64, Job>)>> = OnceLock::new();
        JOBS.get_or_init(|| Mutex::new((0, HashMap::new())))
    }

    fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
        mutex.lock().unwrap_or_else(|p| p.into_inner())
    }

    /// The Python the pipeline runs with: `XERXES_PYTHON`, else the ctrl tool's own virtual
    /// environment, else the system's (the backend picks the same way).
    pub fn ctrl_python(root: &Path) -> PathBuf {
        if let Ok(path) = std::env::var("XERXES_PYTHON") {
            return PathBuf::from(path);
        }
        let venv = root.join("__ctrl__").join(".venv");
        let inside = if cfg!(windows) {
            venv.join("Scripts").join("python.exe")
        } else {
            venv.join("bin").join("python")
        };
        if inside.is_file() {
            return inside;
        }
        PathBuf::from(if cfg!(windows) { "python" } else { "python3" })
    }

    /// The pipeline's arguments (after `main.py`) for a request, checked like the backend does.
    pub fn args(request: &JobRequest) -> Result<Vec<String>, String> {
        if !["dev", "build", "publish"].contains(&request.command.as_str()) {
            return Err(format!("unknown command `{}`", request.command));
        }
        if !["web", "windows", "win", "android"].contains(&request.platform.as_str()) {
            return Err(format!("unknown platform `{}`", request.platform));
        }
        Ok(vec![
            if request.template { "template" } else { "game" }.to_string(),
            request.command.clone(),
            request.target.clone(),
            request.platform.clone(),
        ])
    }

    /// Runs `xerxes-ctrl` for a request, from the repo at `root`.
    pub fn start(root: &Path, request: &JobRequest) -> Result<JobStatus, String> {
        let main = root.join("__ctrl__").join("main.py");
        if !main.is_file() {
            return Err(format!("{} not found", main.display()));
        }
        let args = args(request)?;
        let mut command = Command::new(ctrl_python(root));
        command.arg(&main).args(&args).current_dir(root);
        run(command, args)
    }

    /// Runs a prepared command as a job: output collected line by line, status kept.
    pub fn run(mut command: Command, args: Vec<String>) -> Result<JobStatus, String> {
        command
            .env("PYTHONUNBUFFERED", "1")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            // No console window of its own.
            command.creation_flags(0x0800_0000);
        }
        let mut child = command
            .spawn()
            .map_err(|err| format!("cannot run the pipeline here ({err})"))?;

        let id = {
            let mut all = lock(jobs());
            all.0 += 1;
            all.0
        };
        let status = Arc::new(Mutex::new(JobStatus {
            id,
            args,
            status: "running".into(),
            exit_code: None,
            log: Vec::new(),
        }));
        let pid = Arc::new(Mutex::new(Some(child.id())));
        lock(jobs()).1.insert(
            id,
            Job {
                status: status.clone(),
                pid: pid.clone(),
            },
        );

        let mut readers = Vec::new();
        if let Some(out) = child.stdout.take() {
            readers.push(std::thread::spawn({
                let status = status.clone();
                move || pump(out, status)
            }));
        }
        if let Some(err) = child.stderr.take() {
            readers.push(std::thread::spawn({
                let status = status.clone();
                move || pump(err, status)
            }));
        }
        /// Collects a stream's lines into the job's log.
        fn pump(stream: impl Read + Send + 'static, status: Arc<Mutex<JobStatus>>) {
            for line in BufReader::new(stream).lines().map_while(Result::ok) {
                let mut status = lock(&status);
                status.log.push(line);
                if status.log.len() > LOG_LINES {
                    status.log.remove(0);
                }
            }
        }

        let waiting = (status.clone(), pid);
        std::thread::spawn(move || {
            let (status, pid) = waiting;
            let code = child.wait().ok().and_then(|s| s.code());
            // Everything it printed is in the log before the job is called finished.
            for reader in readers {
                let _ = reader.join();
            }
            *lock(&pid) = None;
            let mut status = lock(&status);
            status.exit_code = code;
            if status.status == "running" {
                status.status = if code == Some(0) {
                    "succeeded"
                } else {
                    "failed"
                }
                .into();
            }
        });
        let first = lock(&status).clone();
        Ok(first)
    }

    /// Stops every job still running (the editor is closing: its servers and builds must not
    /// outlive it).
    pub fn stop_all() {
        let running: Vec<u64> = lock(jobs())
            .1
            .iter()
            .filter(|(_, job)| lock(&job.status).status == "running")
            .map(|(id, _)| *id)
            .collect();
        for id in running {
            let _ = stop(id);
        }
    }

    pub fn status(id: u64) -> Result<JobStatus, String> {
        let status = lock(jobs())
            .1
            .get(&id)
            .map(|job| job.status.clone())
            .ok_or_else(|| format!("no job {id}"))?;
        let status = lock(&status).clone();
        Ok(status)
    }

    /// Stops a job and everything it started (dx, cargo, the game process).
    pub fn stop(id: u64) -> Result<(), String> {
        let (status, pid) = {
            let all = lock(jobs());
            let job = all.1.get(&id).ok_or_else(|| format!("no job {id}"))?;
            (job.status.clone(), job.pid.clone())
        };
        {
            let mut status = lock(&status);
            if status.status == "running" {
                status.status = "stopped".into();
            }
        }
        if let Some(pid) = lock(&pid).take() {
            kill_tree(pid);
        }
        Ok(())
    }

    fn kill_tree(pid: u32) {
        let pid = pid.to_string();
        if cfg!(windows) {
            let _ = Command::new("taskkill")
                .args(["/T", "/F", "/PID", &pid])
                .output();
        } else {
            let _ = Command::new("pkill").args(["-TERM", "-P", &pid]).output();
            let _ = Command::new("kill").args(["-TERM", &pid]).output();
        }
    }
}

/// Starts, follows and stops jobs: through the backend on the web, locally in the native editor.
pub mod api {
    use super::{JobRequest, JobStatus};

    #[cfg(not(target_arch = "wasm32"))]
    const NO_PIPELINE: &str = "Play, Build and Publish need the Xerxes repo: run the editor from inside it (or use the web editor, `xerxes-ctrl engine run web`), or run `game dev <name> windows` in a terminal.";

    #[cfg(target_arch = "wasm32")]
    pub async fn start(request: &JobRequest) -> Result<JobStatus, String> {
        super::super::client::post_json("jobs", request).await
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn status(id: u64) -> Result<JobStatus, String> {
        super::super::client::get_json(&format!("jobs/{id}")).await
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn stop(id: u64) -> Result<(), String> {
        super::super::client::delete(&format!("jobs/{id}")).await
    }

    /// Waits without blocking the page.
    #[cfg(target_arch = "wasm32")]
    pub async fn pause(ms: u32) {
        super::super::client::sleep(ms).await
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn start(request: &JobRequest) -> Result<JobStatus, String> {
        let root = super::super::store::repo_root().ok_or(NO_PIPELINE)?;
        super::local::start(&root, request)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn status(id: u64) -> Result<JobStatus, String> {
        super::local::status(id)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn stop(id: u64) -> Result<(), String> {
        super::local::stop(id)
    }

    /// Waits without blocking the window: a thread sleeps and wakes the task.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn pause(ms: u32) {
        let (wake, asleep) = futures_channel::oneshot::channel::<()>();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(u64::from(ms)));
            let _ = wake.send(());
        });
        let _ = asleep.await;
    }
}
