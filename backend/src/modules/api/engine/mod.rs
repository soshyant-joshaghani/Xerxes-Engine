//! Engine dev services (`ENVIRONMENT=local` only). They let the engine app run games:
//!
//! - `GET /engine/games`: the catalog, in the same order as `__ctrl__ game list`
//!   and `template list`: `games/<name>` A-Z from 1, `games/__templates__/<name>` A-Z from 1,
//!   plus the engine app itself (its own `engine` field, not part of either index: `index` is
//!   null), with each one's title and whether it has a published web build.
//! - `http://<name>.play.localhost/`: serves the published web build of `<name>` from
//!   `dist/<name>/web/public/` at the site root (the build expects to live at `/`). `/api/...`
//!   still reaches the API, so a published engine build can read this catalog.
//! - the editor services (project files, templates, jobs): see [`super::editor`].

use std::path::{Path, PathBuf};

use aide::axum::ApiRouter;
use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::header::HOST;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use schemars::JsonSchema;
use serde::Serialize;
use tower::ServiceExt;
use tower_http::services::{ServeDir, ServeFile};

use crate::core::config::Settings;
use crate::core::state::AppState;

const PLAY_DOMAIN: &str = ".play.localhost";

#[derive(Debug, Clone, Serialize, PartialEq, JsonSchema)]
pub struct CatalogEntry {
    /// Position in `game list` / `template list` (from 1); `None` for the engine app.
    pub index: Option<usize>,
    pub name: String,
    pub title: String,
    /// "engine app" | "game" | "template"
    pub kind: String,
    pub published: bool,
    /// The first sentence of the project's README (for the Project Manager).
    pub description: Option<String>,
    /// Where the published web build plays, when there is one.
    pub play_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, JsonSchema)]
pub struct Catalog {
    pub engine: CatalogEntry,
    pub games: Vec<CatalogEntry>,
    pub templates: Vec<CatalogEntry>,
}

pub fn router(settings: &Settings) -> ApiRouter<AppState> {
    if !settings.is_local() {
        return ApiRouter::new();
    }
    ApiRouter::new()
        .api_route("/engine/games", crate::core::api::get(list_games))
        .merge(super::editor::router())
}

async fn list_games(State(state): State<AppState>) -> Json<Catalog> {
    Json(catalog(Path::new(&state.config.xerxes_root)))
}

/// Reads the engine, games and templates from the repo root `root`.
pub fn catalog(root: &Path) -> Catalog {
    let engine = entry(root, None, "engine", &root.join("engine"), "engine app");
    let games = crate_dirs(&root.join("games"))
        .into_iter()
        .enumerate()
        .map(|(i, (name, dir))| entry(root, Some(i + 1), &name, &dir, "game"))
        .collect();
    let templates = crate_dirs(&root.join("games").join("__templates__"))
        .into_iter()
        .enumerate()
        .map(|(i, (name, dir))| entry(root, Some(i + 1), &name, &dir, "template"))
        .collect();
    Catalog {
        engine,
        games,
        templates,
    }
}

/// Folders directly inside `dir` that hold a `Cargo.toml`, sorted case-insensitively.
fn crate_dirs(dir: &Path) -> Vec<(String, PathBuf)> {
    let Ok(read) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut found: Vec<(String, PathBuf)> = read
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.join("Cargo.toml").is_file())
        .filter_map(|p| Some((p.file_name()?.to_str()?.to_string(), p)))
        .collect();
    found.sort_by_key(|(name, _)| name.to_lowercase());
    found
}

fn entry(root: &Path, index: Option<usize>, name: &str, dir: &Path, kind: &str) -> CatalogEntry {
    let published = published_site(root, name).join("index.html").is_file();
    CatalogEntry {
        index,
        name: name.to_string(),
        title: dioxus_title(dir).unwrap_or_else(|| name.to_string()),
        kind: kind.to_string(),
        published,
        description: readme_summary(dir),
        play_url: published.then(|| format!("http://{name}{PLAY_DOMAIN}/")),
    }
}

/// The README's first sentence (the shared rule in `xerxes_build::scaffold`).
fn readme_summary(dir: &Path) -> Option<String> {
    xerxes_build::scaffold::readme_summary(&std::fs::read_to_string(dir.join("README.md")).ok()?)
}

/// `[web.app] title = "..."` from the crate's Dioxus.toml.
fn dioxus_title(dir: &Path) -> Option<String> {
    let text = std::fs::read_to_string(dir.join("Dioxus.toml")).ok()?;
    let mut in_web_app = false;
    for line in text.lines().map(str::trim) {
        if line.starts_with('[') {
            in_web_app = line == "[web.app]";
        } else if in_web_app {
            if let Some(value) = line.strip_prefix("title") {
                let value = value.trim_start().strip_prefix('=')?.trim();
                return Some(value.trim_matches('"').to_string());
            }
        }
    }
    None
}

fn published_site(root: &Path, name: &str) -> PathBuf {
    root.join("dist").join(name).join("web").join("public")
}

/// The `<name>` in `<name>.play.localhost[:port]`, when it is a plain crate-style name.
pub fn play_target(host: &str) -> Option<&str> {
    let host = host.split(':').next()?;
    let name = host.strip_suffix(PLAY_DOMAIN)?;
    let valid = !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
    valid.then_some(name)
}

/// Middleware: requests for `<name>.play.localhost` get the published build; everything
/// else continues to the API. Only installed when `ENVIRONMENT=local`.
pub async fn serve_play(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let host = request
        .headers()
        .get(HOST)
        .and_then(|h| h.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let Some(name) = play_target(&host) else {
        return next.run(request).await;
    };
    if request.uri().path().starts_with("/api/") {
        return next.run(request).await;
    }
    let site = published_site(Path::new(&state.config.xerxes_root), name);
    if !site.join("index.html").is_file() {
        let body = if name == "engine" {
            "The engine app has no published web build. Run: xerxes-ctrl engine publish web"
                .to_string()
        } else {
            format!(
                "{name} has no published web build. Run: xerxes-ctrl game publish {name} web  (or template publish {name} web)"
            )
        };
        return (axum::http::StatusCode::NOT_FOUND, body).into_response();
    }
    let index = site.join("index.html");
    match ServeDir::new(&site)
        .fallback(ServeFile::new(index))
        .oneshot(request)
        .await
    {
        Ok(response) => response.map(Body::new),
        Err(err) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            err.to_string(),
        )
            .into_response(),
    }
}
