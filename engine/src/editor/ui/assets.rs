//! The Project browser: the project's `assets/` folders on the left, the open folder's assets
//! on the right. Assets are typed by name (`Name.Type.rs`): each row shows its type and a small
//! preview (an image's thumbnail, a scene's 2D/3D, a type badge). A meta (`crate.image.rs`
//! beside `crate.png`) is hidden like Unity's `.meta`, so each asset appears once; it moves and
//! deletes with its file. Clicking a scene opens it. New Scene asks for a name and 2D or 3D, and
//! starts from the engine's empty scene template.

use dioxus::prelude::*;
use xerxes_build::types::AssetType;

use super::{Editor, FileEntry};
use crate::editor::project::browse::{self, Entry};
use crate::editor::project::codec::Dimension;
use crate::editor::project::{names, store};

fn name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

#[component]
pub fn Assets() -> Element {
    let editor = use_context::<Editor>();
    let files = editor.files.read().clone();
    let folder = editor.folder.read().clone();
    let folders: Vec<String> = files
        .iter()
        .filter(|f| f.dir)
        .map(|f| f.path.clone())
        .collect();
    let shown = browse::entries(&files, &folder);
    let open_scene = editor.scene_path.read().clone();
    let mut creating = use_signal(|| false);

    rsx! {
        div { class: "actions",
            button {
                disabled: editor.project.read().is_none(),
                class: if creating() { "on" } else { "" },
                onclick: move |_| creating.toggle(),
                "+ New Scene"
            }
        }
        if creating() {
            NewScene { on_done: move |_| creating.set(false) }
        }
        div { class: "split",
            div { class: "tree",
                FolderRow { path: String::new(), label: "assets".to_string(), depth: 0 }
                for path in folders {
                    FolderRow { key: "{path}", label: name(&path).to_string(), depth: path.matches('/').count() as u32 + 1, path: path.clone() }
                }
            }
            div { class: "files",
                if shown.is_empty() {
                    div { class: "hint", "Empty folder." }
                }
                for entry in shown {
                    FileRow { key: "{entry.path}", entry: entry.clone(), open: open_scene.as_deref() == Some(entry.path.as_str()) }
                }
            }
        }
    }
}

/// The scene files' names (`main` for `scenes/main.scene.rs`).
fn scene_stems(files: &[FileEntry]) -> Vec<String> {
    files
        .iter()
        .filter_map(|f| name(&f.path).strip_suffix(".scene.rs").map(str::to_string))
        .collect()
}

#[component]
fn NewScene(on_done: EventHandler<()>) -> Element {
    let editor = use_context::<Editor>();
    let mut typed = use_signal(|| None::<String>);
    let mut dimension = use_signal(|| Dimension::D3);
    let mut busy = use_signal(|| false);
    let stems = scene_stems(&editor.files.read());
    let taken: Vec<&str> = stems.iter().map(String::as_str).collect();
    let stem = typed
        .read()
        .clone()
        .unwrap_or_else(|| names::unique("level", '_', &taken));
    let problem = names::scene_problem(&stem, &taken);
    let wanted = stem.clone();
    let create = move |_| {
        let Some(project) = editor.project.peek().clone() else {
            return;
        };
        let (stem, dim) = (wanted.clone(), *dimension.peek());
        busy.set(true);
        spawn(async move {
            let path = format!("scenes/{stem}.scene.rs");
            let result = async {
                store::write(
                    &project,
                    &path,
                    store::new_scene_source(&stem, dim).into_bytes(),
                )
                .await?;
                store::files(&project).await
            }
            .await;
            busy.set(false);
            match result {
                Ok(files) => {
                    let (mut files_signal, mut folder, mut scene_path) =
                        (editor.files, editor.folder, editor.scene_path);
                    files_signal.set(files);
                    folder.set("scenes".into());
                    scene_path.set(Some(path.clone()));
                    let kind = if dim == Dimension::D2 { "2D" } else { "3D" };
                    editor.info(format!("new {kind} scene {path} (add it to a level in the project settings to play it)"));
                    on_done.call(());
                }
                Err(err) => editor.error(format!("new scene {path}: {err}")),
            }
        });
    };
    rsx! {
        div { class: "new-scene",
            span { class: "muted", "Name" }
            input {
                class: "num",
                value: "{stem}",
                oninput: move |e| typed.set(Some(names::normalize_scene(&e.value()))),
            }
            for (option, label) in [(Dimension::D2, "2D"), (Dimension::D3, "3D")] {
                button {
                    key: "{label}",
                    class: if dimension() == option { "on" } else { "" },
                    onclick: move |_| dimension.set(option),
                    "{label}"
                }
            }
            button { class: "play", disabled: busy() || problem.is_some(), onclick: create, "Create" }
            button { onclick: move |_| on_done.call(()), "Cancel" }
            if let Some(problem) = &problem {
                span { class: "manager-problem", "{problem}" }
            }
        }
    }
}

#[component]
fn FolderRow(path: String, label: String, depth: u32) -> Element {
    let editor = use_context::<Editor>();
    let on = *editor.folder.read() == path;
    let indent = 6 + depth * 14;
    rsx! {
        div {
            class: if on { "row sel" } else { "row" },
            style: "padding-left: {indent}px;",
            onclick: move |_| {
                let mut folder = editor.folder;
                folder.set(path.clone());
            },
            span { class: "icon", "D" }
            "{label}"
        }
    }
}

/// The small tile before a name: the type badge, with the image itself over it for an image
/// (a scene's tile says 2D or 3D). The badge stays if the image cannot be shown.
#[component]
fn Preview(entry: Entry) -> Element {
    let editor = use_context::<Editor>();
    let path = entry.path.clone();
    let kind = entry.kind;
    let size = entry.size;
    let detail = use_resource(move || {
        let path = path.clone();
        let project = editor.project.peek().clone();
        async move {
            let project = project?;
            match kind {
                Some(AssetType::Image) if size <= browse::PREVIEW_LIMIT => {
                    let bytes = store::read(&project, &path).await.ok()?;
                    browse::image_data_uri(&path, &bytes)
                }
                Some(AssetType::Scene) => {
                    let source = store::read_text(&project, &path).await.ok()?;
                    Some(browse::scene_dimension(&source).to_string())
                }
                _ => None,
            }
        }
    });
    let detail = detail.read().clone().flatten();
    let tile = match kind {
        Some(AssetType::Image) => "thumb t-image",
        Some(AssetType::Scene) => "thumb t-scene",
        Some(AssetType::Prefab) => "thumb t-prefab",
        Some(AssetType::Project) => "thumb t-project",
        Some(AssetType::Mesh) => "thumb t-mesh",
        Some(AssetType::Audio) => "thumb t-audio",
        Some(AssetType::Video) => "thumb t-video",
        None => "thumb t-folder",
        _ => "thumb",
    };
    rsx! {
        div { class: "{tile}",
            if kind == Some(AssetType::Scene) {
                if let Some(dimension) = detail.clone() {
                    "{dimension}"
                } else {
                    "{entry.badge()}"
                }
            } else {
                "{entry.badge()}"
            }
            if kind == Some(AssetType::Image) {
                if let Some(uri) = detail {
                    img { class: "thumb-img", src: "{uri}" }
                }
            }
        }
    }
}

#[component]
fn FileRow(entry: Entry, open: bool) -> Element {
    let editor = use_context::<Editor>();
    let path = entry.path.clone();
    let is_dir = entry.is_dir();
    let is_scene = entry.kind == Some(AssetType::Scene);
    let kind = entry.kind;
    let label = entry.label();
    let size = if is_dir {
        String::new()
    } else {
        browse::size_label(entry.size)
    };
    rsx! {
        div {
            class: if open { "row asset sel" } else { "row asset" },
            style: "padding-left: 8px;",
            onclick: move |_| {
                if is_dir {
                    let mut folder = editor.folder;
                    folder.set(path.clone());
                } else if is_scene {
                    let mut scene = editor.scene_path;
                    scene.set(Some(path.clone()));
                } else {
                    match kind {
                        Some(AssetType::Image) => super::imageeditor::open(editor, path.clone()),
                        Some(AssetType::Project) => {
                            super::show_editor(editor, crate::editor::layout::EditorKind::ProjectSettings)
                        }
                        Some(AssetType::Source | AssetType::Logic | AssetType::Prefab | AssetType::Other)
                            if super::texteditor::is_text(&path) =>
                        {
                            super::texteditor::open(editor, path.clone())
                        }
                        _ => {}
                    }
                }
            },
            Preview { entry: entry.clone() }
            span { class: "grow", "{entry.name}" }
            span { class: "muted", "{label}   {size}" }
        }
    }
}
