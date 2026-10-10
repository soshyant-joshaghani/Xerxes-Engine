//! The editor UI: one set of Dioxus components for every platform (DOM on the web, Blitz
//! natively), laid out like Blender: an app bar over a screen of areas (`shell`), each showing
//! one editor (Viewport, Outliner, Properties, Asset Browser, Console...) chosen from its
//! header. The Viewport is Bevy, shining through its area.
//!
//! [`Editor`] holds the session state as signals; everything it reads and writes goes
//! through the project store (`project::store`: the disk natively, the backend on the web).

mod appmenu;
mod assets;
mod commands;
mod console;
pub mod edit;
mod gamepreview;
mod headers;
mod hierarchy;
mod historypanel;
mod imageeditor;
mod inspector;
mod manager;
mod palette;
mod preferences;
mod projectsettings;
mod shell;
mod spreadsheet;
pub mod texteditor;
mod toolbar;
mod workspaces;

use std::collections::HashMap;
use std::sync::Arc;

use super::bridge::UiPort;
use super::history::History;
use super::host::use_runtime_snapshot;
use super::layout::{AreaId, Layouts, Workspace};
use super::project::codec::{ObjectDoc, SceneDoc, parse_prefab, parse_scene, write_scene};
use super::project::store;
use super::protocol::{EditorCommand, EditorSnapshot, Space, Tool};
use super::theme;
use super::view;
use dioxus::prelude::*;

pub type Port = UiPort<EditorCommand, EditorSnapshot>;

pub use super::project::store::{FileEntry, Listing, ProjectEntry};

/// The project list, as the Project Manager last loaded it.
#[derive(Debug, Clone, PartialEq)]
pub enum Projects {
    Loading,
    Ready(Listing),
    /// The store could not list projects (on the web: the backend is not reachable).
    Failed(String),
}

/// Which screen the editor shows: the Project Manager (where it starts) or the editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Manager,
    Editor,
}

/// An open menu. Menus are drawn last, over every panel: Blitz (native) paints later
/// elements over earlier ones regardless of `z-index`, so this is the one order that is the
/// same on every platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Menu {
    Projects,
    /// The editor-type menu of an area header.
    EditorType(AreaId),
    /// The menu of a layout tab (by its index), and the one of the `+` button.
    Workspace(usize),
    NewWorkspace,
    /// File, Edit, Window or Help.
    App(appmenu::AppMenu),
    /// A menu of an area header (the area, and which menu).
    Header(AreaId, usize),
}

/// The editing session, shared by every panel.
#[derive(Clone, Copy)]
pub struct Editor {
    pub screen: Signal<Screen>,
    pub projects: Signal<Projects>,
    /// The open project.
    pub project: Signal<Option<ProjectEntry>>,
    pub files: Signal<Vec<FileEntry>>,
    /// The folder open in the Project browser (`""` = `assets/`).
    pub folder: Signal<String>,
    pub scene_path: Signal<Option<String>>,
    pub scene: Signal<Option<SceneDoc>>,
    /// The open scene's file as last read or saved (Save writes the document into it).
    pub scene_src: Signal<Option<String>>,
    /// The open scene has unsaved edits.
    pub dirty: Signal<bool>,
    /// Parsed prefabs by the path scenes write (`crate::assets::prefabs::x_prefab::prefab`).
    pub prefabs: Signal<HashMap<String, ObjectDoc>>,
    pub selected: Signal<Option<u32>>,
    pub tool: Signal<Tool>,
    /// Gizmo axes for move/rotate (scale always uses the object's).
    pub space: Signal<Space>,
    /// Every saved layout (the tabs), and which one is on screen.
    pub layouts: Signal<Layouts>,
    /// The layout tab being renamed.
    pub renaming: Signal<Option<usize>>,
    /// A press on a layout tab (a click, or a drag that reorders the tabs).
    pub tab_drag: Signal<Option<workspaces::TabDrag>>,
    /// The open scene's edits, for Undo, Redo and the History editor.
    pub history: Signal<History<edit::Snapshot>>,
    /// The command palette: `Some(what is typed)` while it is open.
    pub palette: Signal<Option<String>>,
    /// The file open in the Text Editor, and the image open in the Image Editor.
    pub text: Signal<Option<texteditor::TextDoc>>,
    pub image: Signal<Option<imageeditor::ImageDoc>>,
    /// The last Play, Build or Publish job (followed while it runs), a note when one could not
    /// start, and the platform the next one runs on.
    pub job: Signal<Option<crate::editor::project::jobs::JobStatus>>,
    pub job_note: Signal<Option<String>>,
    pub platform: Signal<String>,
    /// A drag or a pick over the screen (resizing an edge, choosing an area to split or swap).
    pub mode: Signal<Option<shell::Mode>>,
    /// The Area Edge Options menu, when open.
    pub edge_menu: Signal<Option<shell::EdgeMenu>>,
    pub menu: Signal<Option<Menu>>,
    /// Console lines: (is_error, text).
    pub log: Signal<Vec<(bool, String)>>,
}

impl Editor {
    /// The layout on screen: areas and the editor each shows.
    pub fn workspace(&self) -> Workspace {
        self.layouts.read().active_workspace().clone()
    }

    /// Changes the layout on screen.
    pub fn edit_layout<R>(self, change: impl FnOnce(&mut Workspace) -> R) -> R {
        let mut layouts = self.layouts;
        let mut layouts = layouts.write();
        change(layouts.active_mut())
    }

    /// A Console line (also in the process log, for headless runs).
    pub fn info(mut self, line: impl Into<String>) {
        let line = line.into();
        bevy::log::info!("editor: {line}");
        self.log.write().push((false, line));
    }

    pub fn error(mut self, line: impl Into<String>) {
        let line = line.into();
        bevy::log::error!("editor: {line}");
        self.log.write().push((true, line));
    }
}

/// Where a project's layouts are kept in the preferences (a file name natively, so only
/// letters, digits, `-` and `_`).
fn layouts_key(project: &ProjectEntry) -> String {
    let name: String = project
        .name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    format!("layouts-{name}")
}

/// Makes sure an editor is on screen: when the layout has none, an area that shows nothing
/// useful (an Empty one, else the Console, the History or the Properties) becomes it. It can
/// be switched back with the area's editor-type button.
pub fn show_editor(editor: Editor, kind: crate::editor::layout::EditorKind) {
    use crate::editor::layout::EditorKind;
    let workspace = editor.workspace();
    if workspace.count(kind) > 0 {
        return;
    }
    let areas = workspace.areas();
    // An Empty area; else one that shows another document (a file, an image, the settings:
    // they take turns in one area); else the Console, the History, the Properties.
    let documents = [
        EditorKind::TextEditor,
        EditorKind::ImageEditor,
        EditorKind::ProjectSettings,
    ];
    let victim = areas
        .iter()
        .find(|a| a.editor == EditorKind::Empty)
        .or_else(|| areas.iter().find(|a| documents.contains(&a.editor)))
        .or_else(|| {
            [
                EditorKind::Console,
                EditorKind::History,
                EditorKind::Properties,
            ]
            .iter()
            .find_map(|wanted| areas.iter().find(|a| a.editor == *wanted))
        })
        .map(|a| a.id);
    match victim {
        Some(id) => {
            let result = editor.edit_layout(|w| w.set_editor(id, kind));
            if let Err(err) = result {
                editor.error(format!("layout: {err}"));
            }
        }
        None => editor.error(format!(
            "No area to show the {} in: switch one with its editor-type button",
            kind.title()
        )),
    }
}

/// The path a scene writes for a prefab file (`prefabs/coin.prefab.rs` →
/// `crate::assets::prefabs::coin_prefab::prefab`), by the same rule `build.rs` names modules.
pub fn prefab_ref(path: &str) -> String {
    let module = xerxes_build::module_path(path);
    format!(
        "crate::assets::{}::prefab",
        module.trim_start_matches("self::")
    )
}

fn is_image(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    [".png", ".jpg", ".jpeg"]
        .iter()
        .any(|ext| lower.ends_with(ext))
}

/// The editor's root UI (see `host::Root`).
pub fn root(port: Port) -> Element {
    rsx! { EditorRoot { port } }
}

#[component]
fn EditorRoot(port: Port) -> Element {
    let editor = use_context_provider(|| Editor {
        screen: Signal::new(Screen::Manager),
        projects: Signal::new(Projects::Loading),
        project: Signal::new(None),
        files: Signal::new(Vec::new()),
        folder: Signal::new(String::new()),
        scene_path: Signal::new(None),
        scene: Signal::new(None),
        scene_src: Signal::new(None),
        dirty: Signal::new(false),
        prefabs: Signal::new(HashMap::new()),
        selected: Signal::new(None),
        tool: Signal::new(Tool::Move),
        space: Signal::new(Space::Global),
        layouts: Signal::new(Layouts::default()),
        renaming: Signal::new(None),
        tab_drag: Signal::new(None),
        history: Signal::new(History::default()),
        palette: Signal::new(None),
        text: Signal::new(None),
        image: Signal::new(None),
        job: Signal::new(None),
        job_note: Signal::new(None),
        platform: Signal::new("web".to_string()),
        mode: Signal::new(None),
        edge_menu: Signal::new(None),
        menu: Signal::new(None),
        log: Signal::new(Vec::new()),
    });
    use_context_provider(|| port.clone());
    let snapshot = use_runtime_snapshot(&port);

    // The projects, once (the Project Manager lists them).
    use_hook(move || {
        spawn(async move {
            manager::refresh(editor).await;
            // Dev/QA hook (native): `XERXES_OPEN=<project>` opens it at start (with
            // `XERXES_SCREENSHOT`, checks the editor layout without clicking).
            #[cfg(not(target_arch = "wasm32"))]
            if let Ok(name) = std::env::var("XERXES_OPEN") {
                let found = listed(&editor.projects.peek())
                    .into_iter()
                    .find(|p| p.name == name);
                match found {
                    Some(project) => {
                        manager::open(editor, project);
                        // `XERXES_PLAY=dev|build|publish` (and `XERXES_PLATFORM`) then starts
                        // that job, to check Play without clicking.
                        if let Ok(command) = std::env::var("XERXES_PLAY") {
                            if let Ok(platform) = std::env::var("XERXES_PLATFORM") {
                                let mut chosen = editor.platform;
                                chosen.set(platform);
                            }
                            let command = match command.as_str() {
                                "build" => "build",
                                "publish" => "publish",
                                _ => "dev",
                            };
                            gamepreview::run(editor, command);
                        }
                    }
                    None => editor.error(format!("XERXES_OPEN: no project `{name}`")),
                }
            }
        });
    });

    // The Project Manager covers the Scene view: the stage leaves the pointer alone.
    let modal_port = port.clone();
    use_effect(move || {
        let covered = *editor.screen.read() == Screen::Manager
            || editor.menu.read().is_some()
            || editor.mode.read().is_some()
            || editor.edge_menu.read().is_some()
            || editor.tab_drag.read().is_some()
            || editor.palette.read().is_some();
        modal_port.send(EditorCommand::Modal(covered));
    });

    // Opening a project: its files, prefabs and images; then its first scene.
    let project_port = port.clone();
    use_effect(move || {
        let Some(project) = editor.project.read().clone() else {
            return;
        };
        let port = project_port.clone();
        spawn(async move { open_project(editor, port, project).await });
    });

    // Opening a scene.
    use_effect(move || {
        let (Some(project), Some(path)) = (
            editor.project.read().clone(),
            editor.scene_path.read().clone(),
        ) else {
            return;
        };
        spawn(async move {
            let (mut scene, mut scene_src, mut dirty, mut selected) = (
                editor.scene,
                editor.scene_src,
                editor.dirty,
                editor.selected,
            );
            let loaded = store::read_text(&project, &path)
                .await
                .and_then(|src| parse_scene(&src).map(|doc| (src, doc)));
            match loaded {
                Ok((src, doc)) => {
                    selected.set(None);
                    scene_src.set(Some(src));
                    scene.set(Some(doc));
                    dirty.set(false);
                    let mut history = editor.history;
                    history.write().clear();
                    editor.info(format!("opened {path}"));
                }
                Err(err) => editor.error(format!("{path}: {err}")),
            }
        });
    });

    // The Scene view follows the document, its prefabs and the selection.
    let view_port = port.clone();
    use_effect(move || {
        let view = match (&*editor.scene.read(), &*editor.prefabs.read()) {
            (Some(doc), prefabs) => view::build(doc, prefabs),
            _ => view::SceneView::default(),
        };
        bevy::log::debug!("editor: showing {} objects", view.objects.len());
        view_port.send(EditorCommand::ShowScene(view));
    });
    let select_port = port.clone();
    use_effect(move || select_port.send(EditorCommand::Select(*editor.selected.read())));
    // A Play that was running when the page was reloaded: follow it again.
    use_effect(move || {
        if editor.project.read().is_some() {
            gamepreview::resume(editor);
        }
    });
    // The first time the editor runs (no saved layouts) on a phone-sized window, it starts with
    // a phone layout. The stage reports the window's real size a moment after start.
    let mut first_run = use_signal(|| false);
    // The layouts belong to the project: each project keeps its own (`layouts-<project>` in the
    // preferences). Opening a project loads them; a project without any starts from the layouts
    // saved before they were per project, or the shipped ones.
    let mut layouts_for = use_signal(|| None::<String>);
    use_effect(move || {
        let Some(project) = editor.project.read().clone() else {
            return;
        };
        let key = layouts_key(&project);
        if layouts_for.peek().as_deref() == Some(key.as_str()) {
            return;
        }
        let read =
            |key: &str| crate::editor::prefs::load(key).and_then(|text| Layouts::from_json(&text));
        let own = read(&key);
        first_run.set(own.is_none());
        let mut layouts = editor.layouts;
        layouts.set(own.or_else(|| read("layouts")).unwrap_or_default());
        layouts_for.set(Some(key));
    });
    use_effect(move || {
        let frame = snapshot().layout;
        // Until the stage reports the real window, the frame is a placeholder.
        if frame.window == theme::Layout::default().window || !*first_run.peek() {
            return;
        }
        first_run.set(false);
        let (width, height) = (frame.window.x, frame.window.y);
        if crate::editor::layout::is_phone(width, height) {
            let mut layouts = editor.layouts;
            layouts
                .write()
                .prefer(crate::editor::layout::for_phone(width, height));
        }
    });
    // A new selection ends the edit in progress: the next edit is a step of its own.
    use_effect(move || {
        let _ = editor.selected.read();
        let mut history = editor.history;
        if !history.peek().is_sealed() {
            history.write().seal();
        }
    });
    // The layouts are saved when they change (not on every step of a drag).
    use_effect(move || {
        let layouts = editor.layouts.read().clone();
        if editor.mode.read().is_some() {
            return;
        }
        if let Some(key) = layouts_for.read().clone() {
            crate::editor::prefs::save(&key, &layouts.to_json());
        }
    });
    let layout_port = port.clone();
    use_effect(move || layout_port.send(EditorCommand::SetLayout(Arc::new(editor.workspace()))));
    let tool_port = port.clone();
    use_effect(move || tool_port.send(EditorCommand::SetTool(*editor.tool.read())));
    let space_port = port.clone();
    use_effect(move || space_port.send(EditorCommand::SetSpace(*editor.space.read())));

    // Gizmo drags in the Scene view: the dragged transform goes into the document.
    let mut last_drag = use_signal(|| None::<(u32, super::project::codec::TransformData, u64)>);
    use_effect(move || {
        let dragged = snapshot().dragged;
        if dragged.is_some() && dragged != *last_drag.peek() {
            last_drag.set(dragged);
            if let Some((id, transform, _)) = dragged {
                edit::set_transform(editor, id, transform);
            }
        }
        // The drag is over: the next drag of the same object is a step of its own.
        if dragged.is_none() && last_drag.peek().is_some() {
            last_drag.set(None);
            let mut history = editor.history;
            history.write().seal();
        }
    });

    // Keys that fired an action in the Scene's input (see `editor::keymap`).
    let mut last_action = use_signal(|| 0u64);
    let action_port = port.clone();
    use_effect(move || {
        let Some((action, seq, area)) = snapshot().action else {
            return;
        };
        if seq == *last_action.peek() {
            return;
        }
        last_action.set(seq);
        let command = commands::from_action(action, area);
        if commands::enabled(editor, command) {
            commands::run(editor, &action_port, command);
        }
    });

    // Clicks and W/E/R in the Scene view.
    let mut last_pick = use_signal(|| 0u64);
    let mut last_keyed = use_signal(|| 0u64);
    use_effect(move || {
        let snapshot = snapshot();
        if let Some((picked, seq)) = snapshot.picked {
            if seq != *last_pick.peek() {
                last_pick.set(seq);
                let mut selected = editor.selected;
                selected.set(picked);
            }
        }
        // W/E/R/X pressed in the stage: each change applies once (by its sequence number).
        if let Some((tool, space, seq)) = snapshot.keyed {
            if seq != *last_keyed.peek() {
                last_keyed.set(seq);
                let (mut tool_signal, mut space_signal) = (editor.tool, editor.space);
                tool_signal.set(tool);
                space_signal.set(space);
            }
        }
    });

    let fps = snapshot().fps;
    // The frame the stage placed the Scene view with (safe area, panel sizes).
    let layout = snapshot().layout;
    let root = layout.root_style();
    if *editor.screen.read() == Screen::Manager {
        return rsx! {
            style { {theme::style()} }
            div { class: "ed", style: "{root}", manager::Manager {} }
        };
    }
    let content = layout.content();
    rsx! {
        style { {theme::style()} }
        div { class: "ed", style: "{root}",
            toolbar::Toolbar { fps }
            shell::Shell { x: content.x, y: content.y, width: content.w, height: content.h }
            if *editor.menu.read() == Some(Menu::Projects) {
                ProjectsMenu {}
            }
            if let Some(menu @ (Menu::Workspace(_) | Menu::NewWorkspace)) = *editor.menu.read() {
                workspaces::WorkspaceMenu { menu }
            }
            if let Some(Menu::App(menu)) = *editor.menu.read() {
                appmenu::AppMenuView { menu }
            }
            if editor.palette.read().is_some() {
                palette::Palette { window_width: layout.window.x }
            }
            if editor.renaming.read().is_some() {
                workspaces::RenameBackdrop {}
            }
            if let Some(drag) = *editor.tab_drag.read() {
                workspaces::TabDragLayer { drag, x: layout.insets.left }
            }
        }
    }
}

/// The project picker, under the toolbar's project button.
#[component]
fn ProjectsMenu() -> Element {
    let editor = use_context::<Editor>();
    let close = move || {
        let mut menu = editor.menu;
        menu.set(None);
    };
    rsx! {
        div { class: "menu", style: "left: 8px; top: {theme::toolbar() - 4.0}px;", "data-interactive": "true",
            for entry in listed(&editor.projects.read()) {
                div {
                    key: "{entry.id}",
                    class: "row",
                    onclick: move |_| {
                        close();
                        manager::open(editor, entry.clone());
                    },
                    span { class: "grow", "{entry.title}" }
                    span { class: "muted", "{entry.kind}" }
                }
            }
            div {
                class: "row",
                onclick: move |_| {
                    close();
                    let mut screen = editor.screen;
                    screen.set(Screen::Manager);
                },
                span { class: "grow", "Project Manager…" }
            }
        }
    }
}

/// Every project in a listing (games, then template games).
fn listed(projects: &Projects) -> Vec<ProjectEntry> {
    match projects {
        Projects::Ready(listing) => listing
            .games
            .iter()
            .chain(&listing.templates)
            .cloned()
            .collect(),
        _ => Vec::new(),
    }
}

/// Writes the open scene to its `.rs` file.
pub fn save(editor: Editor) {
    let (Some(project), Some(path), Some(src), Some(doc)) = (
        editor.project.peek().clone(),
        editor.scene_path.peek().clone(),
        editor.scene_src.peek().clone(),
        editor.scene.peek().clone(),
    ) else {
        return;
    };
    let text = match write_scene(&src, &doc) {
        Ok(text) => text,
        Err(err) => return editor.error(format!("cannot save {path}: {err}")),
    };
    dioxus::core::spawn_forever(async move {
        match store::write(&project, &path, text.clone().into_bytes()).await {
            Ok(()) => {
                let (mut scene_src, mut dirty) = (editor.scene_src, editor.dirty);
                scene_src.set(Some(text));
                dirty.set(false);
                let mut history = editor.history;
                history.write().mark_saved();
                editor.info(format!("saved {path}"));
            }
            Err(err) => editor.error(format!("cannot save {path}: {err}")),
        }
    });
}

/// Loads a project: its file list, every prefab (parsed), every image (sent to the Scene
/// view), then opens its first scene.
async fn open_project(editor: Editor, port: Port, project: ProjectEntry) {
    let files = match store::files(&project).await {
        Ok(files) => files,
        Err(err) => return editor.error(err),
    };
    let mut prefabs = HashMap::new();
    for file in files.iter().filter(|f| f.path.ends_with(".prefab.rs")) {
        match store::read_text(&project, &file.path)
            .await
            .and_then(|src| parse_prefab(&src))
        {
            Ok(root) => {
                prefabs.insert(prefab_ref(&file.path), root);
            }
            Err(err) => editor.error(format!("{}: {err}", file.path)),
        }
    }
    let first_scene = files
        .iter()
        .filter(|f| f.path.ends_with(".scene.rs"))
        .map(|f| f.path.clone())
        .min_by_key(|p| (!p.ends_with("main.scene.rs"), p.clone()));
    let images: Vec<String> = files
        .iter()
        .filter(|f| !f.dir && is_image(&f.path))
        .map(|f| f.path.clone())
        .collect();

    let (mut files_signal, mut prefabs_signal, mut folder, mut scene_path, mut scene) = (
        editor.files,
        editor.prefabs,
        editor.folder,
        editor.scene_path,
        editor.scene,
    );
    files_signal.set(files);
    prefabs_signal.set(prefabs);
    folder.set(String::new());
    scene.set(None);
    scene_path.set(first_scene);
    editor.info(format!("project {} ({})", project.title, project.location));

    for path in images {
        match store::read(&project, &path).await {
            Ok(bytes) => port.send(EditorCommand::Texture {
                path,
                bytes: Arc::new(bytes),
            }),
            Err(err) => editor.error(format!("{path}: {err}")),
        }
    }
}
