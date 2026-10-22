//! The Project browser: the project's `assets/` folders on the left, the open folder's assets
//! on the right. Assets are typed by name (`Name.Type.rs`): each row shows its type and a small
//! preview (an image's thumbnail, a scene's 2D/3D, a type badge). A meta (`crate.image.rs`
//! beside `crate.png`) is hidden like Unity's `.meta`, so each asset appears once; it moves and
//! deletes with its file. Clicking a scene or a prefab opens it as the document the Outliner, the
//! Scene view and the Properties show. New Scene asks for a name and 2D or 3D, and starts from the
//! engine's empty scene template; New Prefab makes one in the open folder, and the Variant
//! button on a prefab makes a variant of it (that prefab, with changes of its own).

use dioxus::prelude::*;
use xerxes_build::types::AssetType;

use super::{Editor, FileEntry};
use crate::editor::project::browse::{self, Entry};
use crate::editor::project::codec::Dimension;
use crate::editor::project::{names, prefab_ref, store};

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
    // `Some(None)`: a new prefab in this folder. `Some(Some(base))`: a variant of that prefab.
    let mut new_prefab = use_signal(|| None::<Option<String>>);

    rsx! {
        div { class: "actions",
            button {
                disabled: editor.project.read().is_none(),
                class: if creating() { "on" } else { "" },
                onclick: move |_| {
                    new_prefab.set(None);
                    creating.toggle();
                },
                "+ New Scene"
            }
            button {
                disabled: editor.project.read().is_none(),
                class: if new_prefab.read().is_some() { "on" } else { "" },
                title: "A prefab in this folder: the module a scene drops in",
                onclick: move |_| {
                    creating.set(false);
                    let open = new_prefab.peek().is_some();
                    new_prefab.set(if open { None } else { Some(None) });
                },
                "+ New Prefab"
            }
        }
        if creating() {
            NewScene { on_done: move |_| creating.set(false) }
        }
        if let Some(base) = new_prefab.read().clone() {
            NewPrefab { base, on_done: move |_| new_prefab.set(None) }
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
                    FileRow {
                        key: "{entry.path}",
                        entry: entry.clone(),
                        open: open_scene.as_deref() == Some(entry.path.as_str()),
                        on_variant: move |base: String| {
                            creating.set(false);
                            new_prefab.set(Some(Some(base)));
                        },
                    }
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
fn FileRow(entry: Entry, open: bool, on_variant: EventHandler<String>) -> Element {
    let editor = use_context::<Editor>();
    let path = entry.path.clone();
    let variant_of = entry.path.clone();
    let is_dir = entry.is_dir();
    let is_scene = entry.kind == Some(AssetType::Scene);
    let is_prefab_asset = entry.kind == Some(AssetType::Prefab);
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
                } else if is_scene || is_prefab_asset {
                    // A scene and a prefab are both documents: they open in the Outliner, the
                    // Scene view and the Properties. Opening one leaves prefab mode.
                    super::open_document(editor, path.clone());
                } else {
                    match kind {
                        Some(AssetType::Image) => super::imageeditor::open(editor, path.clone()),
                        Some(AssetType::Project) => {
                            super::show_editor(editor, crate::editor::layout::EditorKind::ProjectSettings)
                        }
                        Some(AssetType::Source | AssetType::Other)
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
            // A variant is this prefab with changes of its own (Unity's Prefab Variant).
            if is_prefab_asset {
                button {
                    title: "A variant of this prefab: it, with changes of its own",
                    onclick: move |event| {
                        event.stop_propagation();
                        on_variant.call(prefab_ref(&variant_of));
                    },
                    "Variant"
                }
            }
            span { class: "muted", "{label}   {size}" }
        }
    }
}

/// Makes a prefab, or a variant of one, in the open folder.
#[component]
fn NewPrefab(base: Option<String>, on_done: EventHandler<()>) -> Element {
    let editor = use_context::<Editor>();
    let mut typed = use_signal(|| None::<String>);
    let mut busy = use_signal(|| false);
    let folder = editor.folder.read().clone();
    // The prefab names already in this folder: a module name is unique per folder.
    let taken_names: Vec<String> = editor
        .files
        .read()
        .iter()
        .filter(|f| !f.dir && f.path.ends_with(".prefab.rs") && parent_of(&f.path) == folder)
        .map(|f| name(&f.path).trim_end_matches(".prefab.rs").to_string())
        .collect();
    let taken: Vec<&str> = taken_names.iter().map(String::as_str).collect();
    let base_name = base
        .as_deref()
        .map(crate::editor::project::codec::prefab_label);
    let suggestion = match &base_name {
        Some(from) => names::unique(&format!("{}_variant", from.replace(' ', "_")), '_', &taken),
        None => names::unique("prefab", '_', &taken),
    };
    let stem = typed.read().clone().unwrap_or(suggestion);
    let problem = names::prefab_problem(&stem, &taken);
    let wanted = stem.clone();
    let from = base_name.clone();
    let create = move |_| {
        let Some(project) = editor.project.peek().clone() else {
            return;
        };
        let (stem, folder, base, base_name) = (
            wanted.clone(),
            folder.clone(),
            base.clone(),
            from.clone().unwrap_or_default(),
        );
        busy.set(true);
        spawn(async move {
            let path = if folder.is_empty() {
                format!("{stem}.prefab.rs")
            } else {
                format!("{folder}/{stem}.prefab.rs")
            };
            let source = match &base {
                Some(base) => store::new_variant_source(&stem, base, &base_name),
                None => store::new_prefab_source(&stem),
            };
            let result = async {
                store::write(&project, &path, source.into_bytes()).await?;
                store::files(&project).await
            }
            .await;
            busy.set(false);
            match result {
                Ok(files) => {
                    let mut files_signal = editor.files;
                    files_signal.set(files);
                    super::open_document(editor, path.clone());
                    editor.info(match &base {
                        Some(_) => format!("new variant {path} (of {base_name})"),
                        None => format!("new prefab {path}"),
                    });
                    on_done.call(());
                }
                Err(err) => editor.error(format!("new prefab {path}: {err}")),
            }
        });
    };
    rsx! {
        div { class: "new-scene",
            span { class: "muted",
                if let Some(from) = &base_name { "Variant of {from}" } else { "Prefab" }
            }
            input {
                class: "num",
                value: "{stem}",
                oninput: move |e| typed.set(Some(names::normalize_scene(&e.value()))),
            }
            button { class: "play", disabled: busy() || problem.is_some(), onclick: create, "Create" }
            button { onclick: move |_| on_done.call(()), "Cancel" }
            if let Some(problem) = &problem {
                span { class: "manager-problem", "{problem}" }
            }
        }
    }
}

/// The folder a path is in (`""` for the root of `assets/`).
fn parent_of(path: &str) -> &str {
    path.rsplit_once('/').map(|(dir, _)| dir).unwrap_or("")
}
