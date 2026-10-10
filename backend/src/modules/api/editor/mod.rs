//! Editor services (`ENVIRONMENT=local` only): the editor's only way to read and write a
//! project's files and to run the build pipeline, on every editor platform (web, windows,
//! android). Paths are confined to a project's `assets/` folder.
//!
//! The routes are in [`router`]; their documentation is generated from the handlers
//! (`/docs`, `/sdoc`).
//!
//! A project is a game (`games/<name>`) or a template game (`games/__templates__/<name>`).

use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{Arc, Mutex, OnceLock};

use aide::axum::ApiRouter;
use axum::extract::{DefaultBodyLimit, Path as UrlPath, State};
use axum::http::StatusCode;
use axum::Json;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::core::api::{delete, get, post, put, Body, Created, FileBytes, RawBody};
use crate::core::error::{ApiError, ApiResult};
use crate::core::state::AppState;

/// Art uploads (images, models, video) can be large.
const MAX_UPLOAD: usize = 512 * 1024 * 1024;
/// Lines of output kept per job.
const LOG_LINES: usize = 400;

pub fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/engine/projects/{name}/files", get(list_project_files))
        .api_route(
            "/engine/projects/{name}/files/{*path}",
            get(read_file)
                .merge(put(write_file))
                .merge(delete(delete_file))
                .layer(DefaultBodyLimit::max(MAX_UPLOAD)),
        )
        .api_route("/engine/projects/{name}/move", post(move_file))
        .api_route("/engine/projects/{name}/addons", post(add_addon_to_project))
        .api_route("/engine/addons", get(list_addons))
        .api_route("/engine/jobs", post(start_job))
        .api_route("/engine/projects", post(new_project))
        .api_route("/engine/jobs/{id}", get(job_status).merge(delete(stop_job)))
}

/// `{name}`: a game or template game.
#[derive(Debug, Deserialize, JsonSchema)]
struct ProjectName {
    name: String,
}

/// `{name}` and `{path}`: a file inside the project's `assets/` (slashes allowed).
#[derive(Debug, Deserialize, JsonSchema)]
struct ProjectFile {
    name: String,
    path: String,
}

/// `{id}`: a job.
#[derive(Debug, Deserialize, JsonSchema)]
struct JobId {
    id: u64,
}

fn root(state: &AppState) -> PathBuf {
    PathBuf::from(&state.config.xerxes_root)
}

/// `games/<name>` or `games/__templates__/<name>` (a crate folder).
pub fn project_dir(root: &Path, name: &str) -> ApiResult<PathBuf> {
    if !valid_name(name) {
        return Err(ApiError::bad_request(format!(
            "invalid project name `{name}`"
        )));
    }
    [
        root.join("games").join(name),
        root.join("games").join("__templates__").join(name),
    ]
    .into_iter()
    .find(|dir| dir.join("Cargo.toml").is_file())
    .ok_or_else(|| ApiError::new(StatusCode::NOT_FOUND, format!("no project `{name}`")))
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// A path inside `assets/`: forward slashes, no empty, `.` or `..` parts, no drive or
/// backslash. Returns it normalized.
pub fn safe_relative(path: &str) -> ApiResult<String> {
    let parts: Vec<&str> = path.trim_matches('/').split('/').collect();
    let bad =
        |p: &&str| p.is_empty() || *p == "." || *p == ".." || p.contains('\\') || p.contains(':');
    if parts.iter().any(bad) {
        return Err(ApiError::bad_request(format!("invalid path `{path}`")));
    }
    Ok(parts.join("/"))
}

fn assets_dir(state: &AppState, name: &str) -> ApiResult<PathBuf> {
    Ok(project_dir(&root(state), name)?.join("assets"))
}

/// The `assets/` of a project that may be changed. Template games (`games/__templates__`) are
/// kept as they are: they can be read and run, never edited through the editor services. A user
/// makes their own copy (New Project from the template, into `games/`) and edits that.
fn writable_assets_dir(state: &AppState, name: &str) -> ApiResult<PathBuf> {
    let dir = project_dir(&root(state), name)?;
    if dir.parent().is_some_and(|p| p.ends_with("__templates__")) {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            format!("`{name}` is a template game and is read-only: create a project from it and edit that"),
        ));
    }
    Ok(dir.join("assets"))
}

/// The meta beside an external file (`art/crate.png` → `art/crate.image.rs`), when its type has
/// one. It moves, renames and deletes with the file.
fn meta_of(path: &str) -> Option<String> {
    xerxes_build::types::meta_path(path)
}

fn io_error(err: std::io::Error) -> ApiError {
    match err.kind() {
        std::io::ErrorKind::NotFound => ApiError::new(StatusCode::NOT_FOUND, "not found"),
        _ => ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
    }
}

#[derive(Debug, Serialize, PartialEq, JsonSchema)]
pub struct FileEntry {
    pub path: String,
    pub dir: bool,
    pub size: u64,
}

/// Every file and folder under `dir`, sorted, as paths relative to it.
pub fn tree(dir: &Path) -> Vec<FileEntry> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<FileEntry>) {
        let Ok(read) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in read.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue;
            }
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            let meta = entry.metadata().ok();
            let dir = path.is_dir();
            out.push(FileEntry {
                path: rel,
                dir,
                size: if dir {
                    0
                } else {
                    meta.map(|m| m.len()).unwrap_or(0)
                },
            });
            if dir {
                walk(root, &path, out);
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, dir, &mut out);
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out
}

async fn list_project_files(
    State(state): State<AppState>,
    UrlPath(ProjectName { name }): UrlPath<ProjectName>,
) -> ApiResult<Json<Vec<FileEntry>>> {
    Ok(Json(tree(&assets_dir(&state, &name)?)))
}

async fn read_file(
    State(state): State<AppState>,
    UrlPath(ProjectFile { name, path }): UrlPath<ProjectFile>,
) -> ApiResult<FileBytes> {
    let file = assets_dir(&state, &name)?.join(safe_relative(&path)?);
    let bytes = tokio::fs::read(&file).await.map_err(io_error)?;
    let content_type = if path.ends_with(".rs") {
        "text/plain; charset=utf-8"
    } else {
        "application/octet-stream"
    };
    Ok(FileBytes {
        bytes,
        content_type,
    })
}

async fn write_file(
    State(state): State<AppState>,
    UrlPath(ProjectFile { name, path }): UrlPath<ProjectFile>,
    RawBody(body): RawBody,
) -> ApiResult<StatusCode> {
    let file = writable_assets_dir(&state, &name)?.join(safe_relative(&path)?);
    if file.is_dir() {
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            format!("`{path}` is a folder"),
        ));
    }
    if let Some(parent) = file.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(io_error)?;
    }
    tokio::fs::write(&file, &body).await.map_err(io_error)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_file(
    State(state): State<AppState>,
    UrlPath(ProjectFile { name, path }): UrlPath<ProjectFile>,
) -> ApiResult<StatusCode> {
    let rel = safe_relative(&path)?;
    let assets = writable_assets_dir(&state, &name)?;
    let target = assets.join(&rel);
    if target.is_dir() {
        tokio::fs::remove_dir_all(&target).await.map_err(io_error)?;
    } else {
        tokio::fs::remove_file(&target).await.map_err(io_error)?;
        if let Some(meta) = meta_of(&rel).map(|m| assets.join(m)) {
            if meta.is_file() {
                tokio::fs::remove_file(meta).await.map_err(io_error)?;
            }
        }
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MoveRequest {
    pub from: String,
    pub to: String,
}

/// The (from, to) pairs a move performs: the file, plus its typed meta when it is an external
/// file (`crate.png` → `crate.image.rs`; the meta follows the new name and type).
pub fn move_pairs(assets: &Path, from: &str, to: &str) -> Vec<(PathBuf, PathBuf)> {
    let mut pairs = vec![(assets.join(from), assets.join(to))];
    if let (Some(meta), Some(moved)) = (meta_of(from), meta_of(to)) {
        let meta = assets.join(meta);
        if meta.is_file() {
            pairs.push((meta, assets.join(moved)));
        }
    }
    pairs
}

async fn move_file(
    State(state): State<AppState>,
    UrlPath(ProjectName { name }): UrlPath<ProjectName>,
    Body(request): Body<MoveRequest>,
) -> ApiResult<StatusCode> {
    let (from, to) = (safe_relative(&request.from)?, safe_relative(&request.to)?);
    let assets = writable_assets_dir(&state, &name)?;
    if !assets.join(&from).exists() {
        return Err(ApiError::new(StatusCode::NOT_FOUND, format!("no `{from}`")));
    }
    let pairs = move_pairs(&assets, &from, &to);
    if let Some((_, taken)) = pairs.iter().find(|(_, dest)| dest.exists()) {
        let taken = taken
            .strip_prefix(&assets)
            .unwrap_or(taken)
            .to_string_lossy()
            .replace('\\', "/");
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            format!("`{taken}` already exists"),
        ));
    }
    for (src, dest) in pairs {
        if let Some(parent) = dest.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(io_error)?;
        }
        tokio::fs::rename(src, dest).await.map_err(io_error)?;
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Serialize, PartialEq, JsonSchema)]
pub struct AddonEntry {
    /// Its place in the engine's add-ons: `hud/hud.rs`.
    pub path: String,
    /// The first doc line.
    pub summary: String,
    /// Add-ons it uses (`//! needs: pause`), as library paths. Adding a module adds these too.
    pub needs: Vec<String>,
}

/// The engine's add-ons (`engine/addons/`, compiled into `xerxes_build`).
async fn list_addons() -> Json<Vec<AddonEntry>> {
    Json(
        xerxes_build::addons::all()
            .into_iter()
            .map(|l| AddonEntry {
                path: l.path,
                summary: l.summary,
                needs: l.needs,
            })
            .collect(),
    )
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddonRequest {
    /// A library path from `GET /engine/addons`.
    pub path: String,
}

#[derive(Debug, Serialize, PartialEq, JsonSchema)]
pub struct AddonCopy {
    /// Files written into the project's `assets/`.
    pub added: Vec<String>,
    /// Files the project already has (its own copies are never overwritten).
    pub kept: Vec<String>,
}

/// Adds add-on `path` and, transitively, what it needs into `assets/logic/`, never overwriting
/// a file the project already has (the game owns its copies).
pub fn add_addon(assets: &Path, path: &str) -> ApiResult<AddonCopy> {
    let files = xerxes_build::addons::files_for(path)
        .map_err(|err| ApiError::new(StatusCode::NOT_FOUND, err))?;
    let mut copy = AddonCopy {
        added: Vec::new(),
        kept: Vec::new(),
    };
    for (landing, bytes) in files {
        let dest = assets.join(&landing);
        if dest.exists() {
            copy.kept.push(landing);
            continue;
        }
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(io_error)?;
        }
        std::fs::write(&dest, bytes).map_err(io_error)?;
        copy.added.push(landing);
    }
    Ok(copy)
}

async fn add_addon_to_project(
    State(state): State<AppState>,
    UrlPath(ProjectName { name }): UrlPath<ProjectName>,
    Body(request): Body<AddonRequest>,
) -> ApiResult<Json<AddonCopy>> {
    let path = safe_relative(&request.path)?;
    Ok(Json(add_addon(
        &writable_assets_dir(&state, &name)?,
        &path,
    )?))
}

// ---- jobs ---------------------------------------------------------------------------------

#[derive(Debug, Deserialize, Clone, JsonSchema)]
pub struct JobRequest {
    /// dev | build | publish
    pub command: String,
    /// A game or template name, or `engine`.
    pub target: String,
    /// web | windows | android
    pub platform: String,
}

/// The `xerxes-ctrl` arguments for a job (after `main.py`). Mirrors the CLI, so a job does
/// exactly what the same command typed in a terminal does.
pub fn job_args(root: &Path, job: &JobRequest) -> ApiResult<Vec<String>> {
    if !["dev", "build", "publish"].contains(&job.command.as_str()) {
        return Err(ApiError::bad_request(format!(
            "unknown command `{}` (dev, build, publish)",
            job.command
        )));
    }
    if !["web", "windows", "win", "android"].contains(&job.platform.as_str()) {
        return Err(ApiError::bad_request(format!(
            "unknown platform `{}` (web, windows, android)",
            job.platform
        )));
    }
    if job.target == "engine" {
        // The engine app's dev command is `run`.
        let command = if job.command == "dev" {
            "run"
        } else {
            job.command.as_str()
        };
        return Ok(vec!["engine".into(), command.into(), job.platform.clone()]);
    }
    let dir = project_dir(root, &job.target)?;
    let group = if dir.parent().is_some_and(|p| p.ends_with("__templates__")) {
        "template"
    } else {
        "game"
    };
    Ok(vec![
        group.into(),
        job.command.clone(),
        job.target.clone(),
        job.platform.clone(),
    ])
}

#[derive(Debug, Clone, Serialize, PartialEq, JsonSchema)]
pub struct JobStatus {
    pub id: u64,
    pub args: Vec<String>,
    /// running | succeeded | failed | stopped
    pub status: String,
    pub exit_code: Option<i32>,
    pub log: Vec<String>,
}

struct Job {
    status: JobStatus,
    pid: Option<u32>,
}

type Jobs = Arc<Mutex<HashMap<u64, Job>>>;

fn jobs() -> &'static (Jobs, Mutex<u64>) {
    static JOBS: OnceLock<(Jobs, Mutex<u64>)> = OnceLock::new();
    JOBS.get_or_init(|| (Arc::new(Mutex::new(HashMap::new())), Mutex::new(0)))
}

/// The Python the pipeline runs with: `XERXES_PYTHON` when set; else the ctrl tool's own
/// virtual environment (`__ctrl__/.venv`, where its requirements are installed: what
/// `xerxes-ctrl` itself uses); else the system's.
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

#[derive(Debug, Deserialize, Clone, JsonSchema)]
pub struct NewProject {
    pub name: String,
    /// `starter` (the engine's built-in empty project, the default) or a template game in
    /// `games/__templates__`.
    pub template: Option<String>,
}

/// A project made by [`create_project`].
#[derive(Debug, Serialize, PartialEq, JsonSchema)]
pub struct CreatedProject {
    pub name: String,
    /// Relative to the repo: `games/<name>`.
    pub location: String,
}

/// Creates `games/<name>` from a template game with the shared scaffold
/// (`xerxes_build::scaffold`, the same code as the native editor and `game new`), then gives
/// it its own Git repository when Git is installed. The web editor's New Project.
pub fn create_project(root: &Path, request: &NewProject) -> ApiResult<CreatedProject> {
    use xerxes_build::scaffold;
    let name = request.name.trim();
    let games_dir = root.join("games");
    let templates_dir = games_dir.join("__templates__");
    let folders = |dir: &Path| -> Vec<String> {
        std::fs::read_dir(dir)
            .map(|entries| {
                entries
                    .flatten()
                    .filter(|e| e.path().is_dir())
                    .map(|e| e.file_name().to_string_lossy().into_owned())
                    .filter(|n| !n.starts_with("__"))
                    .collect()
            })
            .unwrap_or_default()
    };
    let (games, templates) = (folders(&games_dir), folders(&templates_dir));
    let taken: Vec<&str> = games.iter().map(String::as_str).collect();
    let reserved: Vec<&str> = templates.iter().map(String::as_str).collect();
    if let Some(problem) = scaffold::name_problem(name, &taken, &reserved) {
        let status = if taken.contains(&name) {
            StatusCode::CONFLICT
        } else {
            StatusCode::BAD_REQUEST
        };
        return Err(ApiError::new(status, problem));
    }
    let template = request
        .template
        .clone()
        .unwrap_or_else(|| scaffold::DEFAULT_TEMPLATE.into());
    let source = templates_dir.join(&template);
    let builtin = template == scaffold::STARTER;
    if !builtin && (!valid_name(&template) || !source.join("Cargo.toml").is_file()) {
        return Err(ApiError::bad_request(format!("no template `{template}`")));
    }
    let failed = |err: std::io::Error| {
        ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("new project {name}: {err}"),
        )
    };
    let files = if builtin {
        scaffold::starter_files()
    } else {
        scaffold::read_dir(&source).map_err(failed)?
    };
    let engine = scaffold::EngineSource::Path("../../engine".into());
    let files = scaffold::customize(files, &template, name, &engine).map_err(failed)?;
    let dest = games_dir.join(name);
    scaffold::write(&dest, &files).map_err(failed)?;
    // Each game is its own repository; without Git it still works.
    let _ = std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(&dest)
        .status();
    Ok(CreatedProject {
        name: name.to_string(),
        location: format!("games/{name}"),
    })
}

async fn new_project(
    State(state): State<AppState>,
    Body(request): Body<NewProject>,
) -> ApiResult<Created<CreatedProject>> {
    let root = root(&state);
    let created = tokio::task::spawn_blocking(move || create_project(&root, &request))
        .await
        .map_err(|err| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))??;
    Ok(Created(created))
}

async fn start_job(
    State(state): State<AppState>,
    Body(request): Body<JobRequest>,
) -> ApiResult<Created<JobStatus>> {
    let root = root(&state);
    let args = job_args(&root, &request)?;
    spawn_job(&root, args).await
}

/// The absolute repo root and `__ctrl__/main.py` in it. Both must be absolute: the job
/// runs with the root as its working directory, so a relative script path (the default
/// root is `..`) would resolve against the wrong folder.
pub fn ctrl_script(root: &Path) -> (PathBuf, PathBuf) {
    let root = std::path::absolute(root).unwrap_or_else(|_| root.to_path_buf());
    let main = root.join("__ctrl__").join("main.py");
    (root, main)
}

/// Runs `xerxes-ctrl <args>` in the background and tracks it as a job.
async fn spawn_job(root: &Path, args: Vec<String>) -> ApiResult<Created<JobStatus>> {
    let (root, main) = ctrl_script(root);
    let mut child = tokio::process::Command::new(ctrl_python(&root))
        .arg(&main)
        .args(&args)
        .current_dir(&root)
        .env("PYTHONUNBUFFERED", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| {
            ApiError::new(
                StatusCode::CONFLICT,
                format!("cannot run the pipeline here ({err}). Jobs need the API on the host, with Python, cargo and dx (`backend run dev`)."),
            )
        })?;

    let (all, counter) = jobs();
    let id = {
        let mut next = counter.lock().unwrap_or_else(|p| p.into_inner());
        *next += 1;
        *next
    };
    let status = JobStatus {
        id,
        args: args.clone(),
        status: "running".into(),
        exit_code: None,
        log: Vec::new(),
    };
    all.lock().unwrap_or_else(|p| p.into_inner()).insert(
        id,
        Job {
            status: status.clone(),
            pid: child.id(),
        },
    );

    let log: Arc<Mutex<VecDeque<String>>> = Arc::new(Mutex::new(VecDeque::new()));
    for stream in [
        child
            .stdout
            .take()
            .map(|s| Box::new(s) as Box<dyn tokio::io::AsyncRead + Send + Unpin>),
        child.stderr.take().map(|s| Box::new(s) as _),
    ]
    .into_iter()
    .flatten()
    {
        let (jobs, log) = (all.clone(), log.clone());
        tokio::spawn(async move {
            let mut lines = BufReader::new(stream).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let mut log = log.lock().unwrap_or_else(|p| p.into_inner());
                log.push_back(line);
                while log.len() > LOG_LINES {
                    log.pop_front();
                }
                if let Some(job) = jobs.lock().unwrap_or_else(|p| p.into_inner()).get_mut(&id) {
                    job.status.log = log.iter().cloned().collect();
                }
            }
        });
    }
    let jobs_handle = all.clone();
    tokio::spawn(async move {
        let exit = child.wait().await.ok().and_then(|s| s.code());
        if let Some(job) = jobs_handle
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .get_mut(&id)
        {
            job.pid = None;
            job.status.exit_code = exit;
            if job.status.status == "running" {
                job.status.status = if exit == Some(0) {
                    "succeeded"
                } else {
                    "failed"
                }
                .into();
            }
        }
    });
    Ok(Created(status))
}

async fn job_status(UrlPath(JobId { id }): UrlPath<JobId>) -> ApiResult<Json<JobStatus>> {
    let (all, _) = jobs();
    all.lock()
        .unwrap_or_else(|p| p.into_inner())
        .get(&id)
        .map(|job| Json(job.status.clone()))
        .ok_or_else(|| ApiError::new(StatusCode::NOT_FOUND, format!("no job {id}")))
}

/// Stops a job and everything it started (dx, cargo, the game process).
async fn stop_job(UrlPath(JobId { id }): UrlPath<JobId>) -> ApiResult<Json<JobStatus>> {
    let (all, _) = jobs();
    let pid = {
        let mut all = all.lock().unwrap_or_else(|p| p.into_inner());
        let job = all
            .get_mut(&id)
            .ok_or_else(|| ApiError::new(StatusCode::NOT_FOUND, format!("no job {id}")))?;
        if job.status.status == "running" {
            job.status.status = "stopped".into();
        }
        job.pid.take()
    };
    if let Some(pid) = pid {
        kill_tree(pid).await;
    }
    job_status(UrlPath(JobId { id })).await
}

async fn kill_tree(pid: u32) {
    let pid = pid.to_string();
    let result = if cfg!(windows) {
        tokio::process::Command::new("taskkill")
            .args(["/T", "/F", "/PID", &pid])
            .output()
            .await
    } else {
        tokio::process::Command::new("pkill")
            .args(["-TERM", "-P", &pid])
            .output()
            .await
            .and(
                tokio::process::Command::new("kill")
                    .args(["-TERM", &pid])
                    .output()
                    .await,
            )
    };
    if let Err(err) = result {
        tracing::warn!("could not stop job process {pid}: {err}");
    }
}
