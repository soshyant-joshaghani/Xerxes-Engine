//! The Text Editor: any text file of the project (Rust sources, logic, prefabs, the settings
//! file...) opened from the Asset Browser. Edit, Save (the button or Ctrl+S), Revert. It edits
//! the file as text, so a change to the open scene's file reloads the scene (and its Undo
//! history) when it is saved.

use dioxus::prelude::*;

use super::Editor;
use crate::editor::layout::EditorKind;
use crate::editor::project::store;

/// The file being edited, as it was read and as it is now.
#[derive(Debug, Clone, PartialEq)]
pub struct TextDoc {
    pub path: String,
    pub saved: String,
    pub text: String,
    /// A template game: it can be read, not changed.
    pub read_only: bool,
}

impl TextDoc {
    pub fn modified(&self) -> bool {
        self.text != self.saved
    }
}

/// Whether a file's name says it is text the editor can show.
pub fn is_text(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path);
    let ext = name.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase());
    matches!(
        ext.as_deref(),
        Some(
            "rs" | "toml"
                | "md"
                | "txt"
                | "json"
                | "ron"
                | "wgsl"
                | "html"
                | "css"
                | "csv"
                | "yml"
                | "yaml"
        )
    )
}

/// Opens a file in the Text Editor (making room for one if none is on screen).
pub fn open(editor: Editor, path: String) {
    let Some(project) = editor.project.peek().clone() else {
        return;
    };
    if let Some(doc) = editor.text.peek().as_ref()
        && doc.modified()
        && doc.path != path
    {
        editor.error(format!(
            "Save or revert {} before opening another file",
            doc.path
        ));
        return;
    }
    super::show_editor(editor, EditorKind::TextEditor);
    dioxus::core::spawn_forever(async move {
        let mut slot = editor.text;
        match store::read_text(&project, &path).await {
            Ok(text) => slot.set(Some(TextDoc {
                path,
                saved: text.clone(),
                text,
                read_only: store::ensure_writable(&project).is_err(),
            })),
            Err(err) => editor.error(format!("cannot open {path}: {err}")),
        }
    });
}

/// Writes the file, then refreshes what depends on it.
pub fn save(editor: Editor) {
    let (Some(project), Some(doc)) = (editor.project.peek().clone(), editor.text.peek().clone())
    else {
        return;
    };
    if doc.read_only || !doc.modified() {
        return;
    }
    dioxus::core::spawn_forever(async move {
        let result = async {
            store::write(&project, &doc.path, doc.text.clone().into_bytes()).await?;
            store::files(&project).await
        }
        .await;
        match result {
            Ok(files) => {
                let (mut slot, mut files_signal) = (editor.text, editor.files);
                if let Some(current) = slot.write().as_mut()
                    && current.path == doc.path
                {
                    current.saved = doc.text.clone();
                }
                files_signal.set(files);
                editor.info(format!("saved {}", doc.path));
                // The open scene's own file: read it again.
                if editor.scene_path.peek().as_deref() == Some(doc.path.as_str()) {
                    let mut scene_path = editor.scene_path;
                    let same = scene_path.peek().clone();
                    scene_path.set(same);
                }
            }
            Err(err) => editor.error(format!("cannot save {}: {err}", doc.path)),
        }
    });
}

#[component]
pub fn TextEditorPanel() -> Element {
    let editor = use_context::<Editor>();
    let Some(doc) = editor.text.read().clone() else {
        return rsx! {
            div { class: "hint",
                "Click a source file (.rs), prefab or settings file in the Asset Browser to edit it here. File > Edit Scene as Text opens the scene."
            }
        };
    };
    let modified = doc.modified();
    let name = doc.path.rsplit('/').next().unwrap_or(&doc.path).to_string();
    rsx! {
        div { class: "actions",
            button {
                disabled: !modified || doc.read_only,
                onclick: move |_| save(editor),
                if modified { "Save*" } else { "Save" }
            }
            button {
                disabled: !modified,
                onclick: move |_| {
                    let mut slot = editor.text;
                    if let Some(current) = slot.write().as_mut() {
                        current.text = current.saved.clone();
                    }
                },
                "Revert"
            }
            span { class: "muted", "{doc.path}" }
            if doc.read_only {
                span { class: "manager-problem", "template game: read only" }
            }
        }
        textarea {
            class: "code-edit",
            "aria-label": "{name}",
            readonly: doc.read_only,
            value: "{doc.text}",
            spellcheck: false,
            oninput: move |e| {
                let mut slot = editor.text;
                if let Some(current) = slot.write().as_mut() {
                    current.text = e.value();
                }
            },
            onkeydown: move |e| {
                if e.modifiers().ctrl()
                    && matches!(e.key(), Key::Character(ref c) if c.eq_ignore_ascii_case("s"))
                {
                    e.prevent_default();
                    save(editor);
                }
            },
        }
    }
}
