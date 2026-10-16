//! Where projects live: the editor's one interface for listing, creating, reading and
//! writing projects, the same on every platform. Only the adapter differs (like the
//! renderer glue in `host/`):
//!
//! - **Native** (Windows, Android, later iOS): the device's disk. The editor is a standalone
//!   app, like Godot or Unity: it needs no backend. Inside the Xerxes repo (development) the
//!   projects are the repo's games (`games/`, `games/__templates__/`); anywhere else they
//!   live in a projects folder (`XERXES_PROJECTS`, else `Documents/Xerxes Projects`, or the
//!   app's storage on Android), and new games depend on the engine's Git repository.
//! - **Web**: a page cannot touch the disk, so it uses the local backend's engine services.
//!
//! New projects come from the template games packed into the editor (`build.rs`), made into
//! a game by the shared scaffold (`xerxes_build::scaffold`): the same code the backend and
//! `xerxes-ctrl game new` run.

use std::sync::OnceLock;

use serde::Deserialize;
use xerxes_build::scaffold;

use super::codec::Dimension;

/// A project the editor can open (a game, or a template game inside the repo).
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ProjectEntry {
    /// How the store finds it: its folder natively, its name on the web.
    #[serde(default)]
    pub id: String,
    pub name: String,
    pub title: String,
    /// `game` or `template`.
    pub kind: String,
    /// The first sentence of its README.
    #[serde(default)]
    pub description: Option<String>,
    /// Where it is, for people (`games/x`, or a folder).
    #[serde(default)]
    pub location: String,
}

/// A file or folder under a project's `assets/`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub dir: bool,
    pub size: u64,
}

/// A template game a new project can start from (packed into the editor).
#[derive(Debug, Clone, PartialEq)]
pub struct Starter {
    pub name: String,
    pub title: String,
    pub description: Option<String>,
}

/// The projects, and where a new one goes.
#[derive(Debug, Clone, PartialEq)]
pub struct Listing {
    pub games: Vec<ProjectEntry>,
    pub templates: Vec<ProjectEntry>,
    /// Shown under New Project: the folder new projects are created in.
    pub new_projects_in: String,
}

static PACKED: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/templates.zip"));

fn packed() -> &'static [(String, Vec<scaffold::File>)] {
    static UNPACKED: OnceLock<Vec<(String, Vec<scaffold::File>)>> = OnceLock::new();
    UNPACKED.get_or_init(|| {
        // The engine's own starter first, then the finished template games packed in.
        let mut all = vec![(scaffold::STARTER.to_string(), scaffold::starter_files())];
        all.extend(scaffold::unpack(PACKED).unwrap_or_default());
        all
    })
}

fn text<'a>(files: &'a [scaffold::File], path: &str) -> Option<&'a str> {
    files
        .iter()
        .find(|(p, _)| p == path)
        .and_then(|(_, b)| std::str::from_utf8(b).ok())
}

/// What New Project can start from: the engine's starter, then the template games packed in.
pub fn starters() -> Vec<Starter> {
    packed()
        .iter()
        .map(|(name, files)| Starter {
            name: name.clone(),
            title: scaffold::template_title(files).unwrap_or_else(|| name.clone()),
            description: text(files, "README.md").and_then(scaffold::readme_summary),
        })
        .collect()
}

/// A new scene's source: the editor's empty 2D or 3D starter scene (`starters/`).
pub fn new_scene_source(stem: &str, dimension: Dimension) -> String {
    let template = match dimension {
        Dimension::D2 => include_str!("starters/2d.scene.rs"),
        Dimension::D3 => include_str!("starters/3d.scene.rs"),
    };
    template.replacen(
        "//! Template: an empty ",
        &format!("//! Scene `{stem}`, started as an empty "),
        1,
    )
}

/// A path inside `assets/`: `/`-separated, no empty, `.` or `..` parts.
fn checked(path: &str) -> Result<&str, String> {
    let bad = path.is_empty()
        || path.starts_with('/')
        || path.contains('\\')
        || path.contains(':')
        || path
            .split('/')
            .any(|p| p.is_empty() || p == "." || p == "..");
    if bad {
        Err(format!("bad path `{path}`"))
    } else {
        Ok(path)
    }
}

pub use imp::{create, files, list, read, write};

/// The Xerxes repo the native editor runs in (it has `__ctrl__`, so it can run the build
/// pipeline); `None` on the web and in a standalone editor.
#[cfg(not(target_arch = "wasm32"))]
pub fn repo_root() -> Option<std::path::PathBuf> {
    imp::repo_root()
}

#[cfg(target_arch = "wasm32")]
pub fn repo_root() -> Option<std::path::PathBuf> {
    None
}

/// Template games are kept as they are: they can be opened to look at and run, never changed
/// through the editor. Make a project from one (New Project) and edit that.
pub fn ensure_writable(project: &ProjectEntry) -> Result<(), String> {
    if project.kind == "template" {
        return Err(format!(
            "`{}` is a template game and is read-only: create a project from it and edit that",
            project.name
        ));
    }
    Ok(())
}

pub async fn read_text(project: &ProjectEntry, path: &str) -> Result<String, String> {
    let bytes = read(project, path).await?;
    String::from_utf8(bytes).map_err(|_| format!("{path} is not UTF-8 text"))
}

// ---- native: the device's disk -------------------------------------------------------

#[cfg(not(target_arch = "wasm32"))]
mod imp {
    use std::path::{Path, PathBuf};
    use std::sync::OnceLock;

    use xerxes_build::scaffold;

    use super::{FileEntry, Listing, ProjectEntry, checked, packed};

    /// Where the projects are.
    enum Workspace {
        /// The Xerxes repo: `games/` and `games/__templates__/`; new games use `../../engine`.
        Repo(PathBuf),
        /// A projects folder; new games use the engine's Git repository.
        Folder(PathBuf),
    }

    fn is_repo(dir: &Path) -> bool {
        dir.join("engine").join("Cargo.toml").is_file() && dir.join("games").is_dir()
    }

    fn projects_folder() -> PathBuf {
        #[cfg(target_os = "android")]
        if let Some(dir) = bevy::android::ANDROID_APP
            .get()
            .and_then(|app| app.internal_data_path())
        {
            return dir.join("projects");
        }
        let home = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME"));
        match home {
            Some(home) => PathBuf::from(home)
                .join("Documents")
                .join("Xerxes Projects"),
            None => PathBuf::from("Xerxes Projects"),
        }
    }

    pub fn repo_root() -> Option<PathBuf> {
        match workspace() {
            Workspace::Repo(root) => Some(root.clone()),
            Workspace::Folder(_) => None,
        }
    }

    fn workspace() -> &'static Workspace {
        static WORKSPACE: OnceLock<Workspace> = OnceLock::new();
        WORKSPACE.get_or_init(|| {
            // An explicit projects folder wins (also how a standalone setup is tested).
            if let Some(dir) = std::env::var_os("XERXES_PROJECTS") {
                let dir = PathBuf::from(dir);
                let _ = std::fs::create_dir_all(&dir);
                return Workspace::Folder(std::path::absolute(&dir).unwrap_or(dir));
            }
            let mut candidates: Vec<PathBuf> = std::env::var_os("XERXES_ROOT")
                .map(PathBuf::from)
                .into_iter()
                .collect();
            if let Ok(dir) = std::env::current_dir() {
                candidates.extend(dir.ancestors().map(Path::to_path_buf));
            }
            if let Ok(exe) = std::env::current_exe() {
                candidates.extend(exe.ancestors().skip(1).map(Path::to_path_buf));
            }
            match candidates.into_iter().find(|d| is_repo(d)) {
                Some(root) => Workspace::Repo(std::path::absolute(&root).unwrap_or(root)),
                None => {
                    let dir = projects_folder();
                    let _ = std::fs::create_dir_all(&dir);
                    Workspace::Folder(std::path::absolute(&dir).unwrap_or(dir))
                }
            }
        })
    }

    /// Folder names under `dir` that are projects (have a Cargo.toml).
    fn projects_in(dir: &Path) -> Vec<PathBuf> {
        let mut found: Vec<PathBuf> = std::fs::read_dir(dir)
            .map(|e| e.flatten().map(|e| e.path()).collect())
            .unwrap_or_default();
        found.retain(|p| {
            p.join("Cargo.toml").is_file()
                && !p
                    .file_name()
                    .is_some_and(|n| n.to_string_lossy().starts_with("__"))
        });
        found.sort();
        found
    }

    fn entry(dir: &Path, kind: &str, location: String) -> ProjectEntry {
        let name = dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let read = |file: &str| std::fs::read_to_string(dir.join(file)).ok();
        ProjectEntry {
            id: dir.to_string_lossy().into_owned(),
            title: read("Dioxus.toml")
                .and_then(|t| scaffold::dioxus_title(&t))
                .unwrap_or_else(|| name.clone()),
            description: read("README.md").and_then(|t| scaffold::readme_summary(&t)),
            name,
            kind: kind.to_string(),
            location,
        }
    }

    pub async fn list() -> Result<Listing, String> {
        Ok(match workspace() {
            Workspace::Repo(root) => {
                let games = root.join("games");
                Listing {
                    games: projects_in(&games)
                        .iter()
                        .map(|d| entry(d, "game", format!("games/{}", name_of(d))))
                        .collect(),
                    templates: projects_in(&games.join("__templates__"))
                        .iter()
                        .map(|d| {
                            entry(d, "template", format!("games/__templates__/{}", name_of(d)))
                        })
                        .collect(),
                    new_projects_in: games.display().to_string(),
                }
            }
            Workspace::Folder(dir) => Listing {
                games: projects_in(dir)
                    .iter()
                    .map(|d| entry(d, "game", d.display().to_string()))
                    .collect(),
                templates: Vec::new(),
                new_projects_in: dir.display().to_string(),
            },
        })
    }

    fn name_of(dir: &Path) -> String {
        dir.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    pub async fn create(name: &str, template: &str) -> Result<ProjectEntry, String> {
        let (dest_dir, engine, reserved) = match workspace() {
            Workspace::Repo(root) => {
                let templates: Vec<String> = projects_in(&root.join("games").join("__templates__"))
                    .iter()
                    .map(|d| name_of(d))
                    .collect();
                (
                    root.join("games"),
                    scaffold::EngineSource::Path("../../engine".into()),
                    templates,
                )
            }
            Workspace::Folder(dir) => (
                dir.clone(),
                scaffold::EngineSource::Git {
                    url: scaffold::ENGINE_GIT.into(),
                    branch: "main".into(),
                },
                Vec::new(),
            ),
        };
        let taken: Vec<String> = projects_in(&dest_dir).iter().map(|d| name_of(d)).collect();
        let taken: Vec<&str> = taken.iter().map(String::as_str).collect();
        let reserved: Vec<&str> = reserved.iter().map(String::as_str).collect();
        if let Some(problem) = scaffold::name_problem(name, &taken, &reserved) {
            return Err(problem);
        }
        let files = packed()
            .iter()
            .find(|(n, _)| n == template)
            .map(|(_, files)| files.clone())
            .ok_or_else(|| format!("no template `{template}` in this editor"))?;
        let files =
            scaffold::customize(files, template, name, &engine).map_err(|e| e.to_string())?;
        let dest = dest_dir.join(name);
        scaffold::write(&dest, &files).map_err(|e| e.to_string())?;
        // Each game is its own repository when Git is installed (desktop).
        #[cfg(not(target_os = "android"))]
        let _ = std::process::Command::new("git")
            .args(["init", "-q"])
            .current_dir(&dest)
            .status();
        let location = match workspace() {
            Workspace::Repo(_) => format!("games/{name}"),
            Workspace::Folder(_) => dest.display().to_string(),
        };
        Ok(entry(&dest, "game", location))
    }

    fn assets(project: &ProjectEntry) -> PathBuf {
        Path::new(&project.id).join("assets")
    }

    pub async fn files(project: &ProjectEntry) -> Result<Vec<FileEntry>, String> {
        fn walk(root: &Path, dir: &Path, out: &mut Vec<FileEntry>) -> std::io::Result<()> {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                let rel = path
                    .strip_prefix(root)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .replace('\\', "/");
                let meta = entry.metadata()?;
                out.push(FileEntry {
                    path: rel,
                    dir: meta.is_dir(),
                    size: if meta.is_dir() { 0 } else { meta.len() },
                });
                if meta.is_dir() {
                    walk(root, &path, out)?;
                }
            }
            Ok(())
        }
        let root = assets(project);
        let mut out = Vec::new();
        walk(&root, &root, &mut out).map_err(|e| format!("{}: {e}", root.display()))?;
        out.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(out)
    }

    pub async fn read(project: &ProjectEntry, path: &str) -> Result<Vec<u8>, String> {
        std::fs::read(assets(project).join(checked(path)?)).map_err(|e| format!("{path}: {e}"))
    }

    pub async fn write(project: &ProjectEntry, path: &str, bytes: Vec<u8>) -> Result<(), String> {
        super::ensure_writable(project)?;
        let target = assets(project).join(checked(path)?);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("{path}: {e}"))?;
        }
        std::fs::write(target, bytes).map_err(|e| format!("{path}: {e}"))
    }
}

// ---- web: the local backend ------------------------------------------------------------

#[cfg(target_arch = "wasm32")]
mod imp {
    use serde::{Deserialize, Serialize};

    use super::super::client;
    use super::{FileEntry, Listing, ProjectEntry, checked};

    #[derive(Deserialize)]
    struct Catalog {
        games: Vec<ProjectEntry>,
        templates: Vec<ProjectEntry>,
    }

    fn located(mut entry: ProjectEntry) -> ProjectEntry {
        entry.id = entry.name.clone();
        entry.location = if entry.kind == "template" {
            format!("games/__templates__/{}", entry.name)
        } else {
            format!("games/{}", entry.name)
        };
        entry
    }

    pub async fn list() -> Result<Listing, String> {
        let catalog: Catalog = client::get_json("games").await.map_err(|err| {
            format!("{err} The web editor reaches projects through the local backend: run `xerxes-ctrl backend run dev` on the machine that has them.")
        })?;
        Ok(Listing {
            games: catalog.games.into_iter().map(located).collect(),
            templates: catalog.templates.into_iter().map(located).collect(),
            new_projects_in: "games/ (on the machine running the backend)".into(),
        })
    }

    #[derive(Serialize)]
    struct NewProject<'a> {
        name: &'a str,
        template: &'a str,
    }

    pub async fn create(name: &str, template: &str) -> Result<ProjectEntry, String> {
        #[derive(Deserialize)]
        struct Created {
            name: String,
        }
        let created: Created =
            client::post_json("projects", &NewProject { name, template }).await?;
        let listing = list().await?;
        listing
            .games
            .into_iter()
            .find(|g| g.name == created.name)
            .ok_or_else(|| format!("created {} but it is not listed", created.name))
    }

    pub async fn files(project: &ProjectEntry) -> Result<Vec<FileEntry>, String> {
        client::get_json(&format!("projects/{}/files", project.id)).await
    }

    pub async fn read(project: &ProjectEntry, path: &str) -> Result<Vec<u8>, String> {
        client::get_bytes(&format!("projects/{}/files/{}", project.id, checked(path)?)).await
    }

    pub async fn write(project: &ProjectEntry, path: &str, bytes: Vec<u8>) -> Result<(), String> {
        super::ensure_writable(project)?;
        client::put_bytes(
            &format!("projects/{}/files/{}", project.id, checked(path)?),
            bytes,
        )
        .await
    }
}
