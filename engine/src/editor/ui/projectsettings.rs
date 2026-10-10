//! The Project Settings editor: the title, the window and the backend of the project's
//! `<name>.project.rs`, as fields; the levels, the input actions and the preloaded bundles are
//! shown (change them in the Text Editor, one click away). A change replaces only the value it
//! names in the file (`project::settings_text`), so the rest of the file stays as written.

use dioxus::prelude::*;

use super::Editor;
use crate::editor::project::settings_text::{self as text, BackendOpts, Parsed, WindowOpts};
use crate::editor::project::store;

/// The settings file: `assets/<name>.project.rs`.
fn settings_path(editor: Editor) -> Option<String> {
    editor
        .files
        .read()
        .iter()
        .find(|f| !f.dir && f.path.ends_with(".project.rs") && !f.path.contains('/'))
        .map(|f| f.path.clone())
}

#[component]
pub fn ProjectSettingsPanel() -> Element {
    let editor = use_context::<Editor>();
    let Some(path) = settings_path(editor) else {
        return rsx! {
            div { class: "hint", "Open a project to edit its settings." }
        };
    };
    // Bumped after a change is written: the form starts again from the new file.
    let reload = use_signal(|| 0u32);
    let project = editor.project.read().clone();
    let source = use_resource({
        let path = path.clone();
        move || {
            let path = path.clone();
            let project = project.clone();
            let _ = reload();
            async move { store::read_text(&project?, &path).await.ok() }
        }
    });
    let loaded = source.read().clone().flatten();
    match loaded {
        None => rsx! {
            div { class: "hint", "Loading {path}..." }
        },
        Some(src) => rsx! {
            SettingsForm { key: "{reload}", path, src, reload }
        },
    }
}

#[component]
fn SettingsForm(path: String, src: String, reload: Signal<u32>) -> Element {
    let editor = use_context::<Editor>();
    let parsed: Parsed = text::parse(&src);
    let read_only = editor
        .project
        .peek()
        .as_ref()
        .is_none_or(|p| store::ensure_writable(p).is_err());

    let mut title = use_signal(|| parsed.title.clone().unwrap_or_default());
    let (w0, w1) = (parsed.window.clone(), parsed.window.clone());
    let mut width = use_signal(|| w0.as_ref().map_or(String::new(), |w| w.width.to_string()));
    let mut height = use_signal(|| w1.as_ref().map_or(String::new(), |w| w.height.to_string()));
    let mut resizable = use_signal(|| parsed.window.as_ref().is_some_and(|w| w.resizable));
    let mut dev = use_signal(|| {
        parsed
            .backend
            .as_ref()
            .map_or(String::new(), |b| b.dev.clone())
    });
    let mut publish = use_signal(|| {
        parsed
            .backend
            .as_ref()
            .map_or(String::new(), |b| b.publish.clone())
    });
    let mut problem = use_signal(|| None::<String>);

    // The file as these fields would write it.
    let changed = {
        let (src, parsed) = (src.clone(), parsed.clone());
        move || -> Result<String, String> {
            let mut out = src.clone();
            if parsed.title.is_some() && *title.peek() != parsed.title.clone().unwrap_or_default() {
                out = text::set_title(&out, &title.peek())
                    .ok_or("The title cannot have quotes or backslashes.")?;
            }
            if let Some(old) = &parsed.window {
                let (w, h) = (
                    width.peek().trim().parse::<u32>(),
                    height.peek().trim().parse::<u32>(),
                );
                let (Ok(w), Ok(h)) = (w, h) else {
                    return Err("Width and height are whole numbers.".into());
                };
                let new = WindowOpts {
                    width: w,
                    height: h,
                    resizable: *resizable.peek(),
                };
                if &new != old {
                    out = text::set_window(&out, &new)
                        .ok_or("The window settings could not be written.")?;
                }
            }
            if let Some(old) = &parsed.backend {
                let new = BackendOpts {
                    dev: dev.peek().trim().to_string(),
                    publish: publish.peek().trim().to_string(),
                };
                if &new != old {
                    out = text::set_backend(&out, &new)
                        .ok_or("The backend URLs cannot have quotes or backslashes.")?;
                }
            }
            Ok(out)
        }
    };
    let pending = changed().map(|out| out != src).unwrap_or(true);

    let apply = {
        let (path, changed) = (path.clone(), changed.clone());
        move |_| {
            let Some(project) = editor.project.peek().clone() else {
                return;
            };
            match changed() {
                Err(why) => problem.set(Some(why)),
                Ok(out) => {
                    problem.set(None);
                    let path = path.clone();
                    spawn(async move {
                        match store::write(&project, &path, out.into_bytes()).await {
                            Ok(()) => {
                                editor.info(format!("saved {path}"));
                                reload += 1;
                            }
                            Err(err) => editor.error(format!("cannot save {path}: {err}")),
                        }
                    });
                }
            }
        }
    };

    // The fields' text, or what to do when the file uses the defaults.
    let make_explicit = {
        let (path, src) = (path.clone(), src.clone());
        move |window: bool| {
            let Some(project) = editor.project.peek().clone() else {
                return;
            };
            let out = if window {
                text::make_window_explicit(&src)
            } else {
                text::make_backend_explicit(&src)
            };
            let (Some(out), path) = (out, path.clone()) else {
                return;
            };
            spawn(async move {
                match store::write(&project, &path, out.into_bytes()).await {
                    Ok(()) => reload += 1,
                    Err(err) => editor.error(format!("cannot save {path}: {err}")),
                }
            });
        }
    };
    let (explicit_window, explicit_backend) = (make_explicit.clone(), make_explicit);
    let open_path = path.clone();
    let shown_levels = parsed.levels.clone();
    let shown_actions = parsed.actions.clone();
    let preload = parsed.preload.join(", ");

    rsx! {
        div { class: "actions",
            button {
                class: "play",
                disabled: read_only || !pending,
                onclick: apply,
                if pending { "Apply*" } else { "Apply" }
            }
            button { onclick: move |_| super::texteditor::open(editor, open_path.clone()), "Edit as text" }
            span { class: "muted", "{path}" }
            if read_only {
                span { class: "manager-problem", "template game: read only" }
            }
            if let Some(why) = problem() {
                span { class: "manager-problem", "{why}" }
            }
        }
        div { class: "body",
            div { class: "section",
                div { class: "section-head", "Project" }
                div { class: "field",
                    span { class: "label", "Title" }
                    div { class: "value",
                        input {
                            class: "num",
                            value: "{title}",
                            disabled: read_only || parsed.title.is_none(),
                            oninput: move |e| title.set(e.value()),
                        }
                    }
                }
            }
            div { class: "section",
                div { class: "section-head", "Window" }
                if parsed.window.is_some() {
                    div { class: "field",
                        span { class: "label", "Size" }
                        div { class: "value",
                            input {
                                class: "num",
                                value: "{width}",
                                disabled: read_only,
                                oninput: move |e| width.set(e.value()),
                            }
                            span { class: "axis", "x" }
                            input {
                                class: "num",
                                value: "{height}",
                                disabled: read_only,
                                oninput: move |e| height.set(e.value()),
                            }
                        }
                    }
                    div { class: "field",
                        span { class: "label", "Resizable" }
                        div { class: "value",
                            button {
                                class: if resizable() { "on" } else { "" },
                                disabled: read_only,
                                onclick: move |_| resizable.set(!resizable()),
                                if resizable() { "Yes" } else { "No" }
                            }
                        }
                    }
                } else {
                    div { class: "field",
                        span { class: "label", "Default" }
                        div { class: "value",
                            button { disabled: read_only, onclick: move |_| explicit_window(true), "Customize" }
                        }
                    }
                }
            }
            div { class: "section",
                div { class: "section-head", "Backend" }
                if parsed.backend.is_some() {
                    div { class: "field",
                        span { class: "label", "Dev" }
                        div { class: "value",
                            input {
                                class: "num",
                                value: "{dev}",
                                disabled: read_only,
                                oninput: move |e| dev.set(e.value()),
                            }
                        }
                    }
                    div { class: "field",
                        span { class: "label", "Publish" }
                        div { class: "value",
                            input {
                                class: "num",
                                value: "{publish}",
                                placeholder: "empty: offline",
                                disabled: read_only,
                                oninput: move |e| publish.set(e.value()),
                            }
                        }
                    }
                } else {
                    div { class: "field",
                        span { class: "label", "Default" }
                        div { class: "value",
                            button { disabled: read_only, onclick: move |_| explicit_backend(false), "Customize" }
                        }
                    }
                }
            }
            div { class: "section",
                div { class: "section-head", "Levels ({shown_levels.len()})" }
                for (index , level) in shown_levels.into_iter().enumerate() {
                    div { key: "{index}", class: "field",
                        span { class: "label", "{level.name}" }
                        span { class: "value muted", "{level.mode}" }
                    }
                }
            }
            div { class: "section",
                div { class: "section-head", "Input actions ({shown_actions.len()})" }
                for action in shown_actions {
                    div { key: "{action.name}", class: "field",
                        span { class: "label", "{action.name}" }
                        span { class: "value muted", "{action.bindings}" }
                    }
                }
            }
            if !preload.is_empty() {
                div { class: "section",
                    div { class: "section-head", "Preloaded bundles" }
                    div { class: "field", span { class: "value muted", "{preload}" } }
                }
            }
        }
    }
}
