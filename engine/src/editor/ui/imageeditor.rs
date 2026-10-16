//! The Image Editor: looks at an image asset opened from the Asset Browser, at fit size or at a
//! zoom (when the size is known: PNG files say it in their header). Editing pixels is not
//! there yet.

use dioxus::prelude::*;

use super::Editor;
use crate::editor::layout::EditorKind;
use crate::editor::project::{browse, store};

#[derive(Debug, Clone, PartialEq)]
pub struct ImageDoc {
    pub path: String,
    /// The image as a data URI (what an `img` shows).
    pub uri: String,
    /// Its size in pixels, when the file says it.
    pub size: Option<(u32, u32)>,
    pub bytes: usize,
}

/// Opens an image in the Image Editor (making room for one if none is on screen).
pub fn open(editor: Editor, path: String) {
    let Some(project) = editor.project.peek().clone() else {
        return;
    };
    super::show_editor(editor, EditorKind::ImageEditor);
    dioxus::core::spawn_forever(async move {
        let mut slot = editor.image;
        let loaded = store::read(&project, &path).await;
        match loaded {
            Ok(bytes) => match browse::image_data_uri(&path, &bytes) {
                Some(uri) => slot.set(Some(ImageDoc {
                    size: browse::png_size(&bytes),
                    bytes: bytes.len(),
                    path,
                    uri,
                })),
                None => editor.error(format!("{path}: this image format cannot be shown here")),
            },
            Err(err) => editor.error(format!("cannot open {path}: {err}")),
        }
    });
}

#[component]
pub fn ImageEditorPanel() -> Element {
    let editor = use_context::<Editor>();
    // 0 fits the area; otherwise the share of the real size.
    let mut zoom = use_signal(|| 0.0f32);
    let Some(doc) = editor.image.read().clone() else {
        return rsx! {
            div { class: "hint", "Click an image in the Asset Browser to look at it here." }
        };
    };
    let info = match doc.size {
        Some((w, h)) => format!("{w} x {h}   {}", browse::size_label(doc.bytes as u64)),
        None => browse::size_label(doc.bytes as u64),
    };
    // Every style sets the same properties: the page patches an `img`'s style one property at a
    // time, so one that is left out would keep the last zoom's value.
    let style = match (zoom(), doc.size) {
        (z, Some((w, h))) if z > 0.0 => format!(
            "width: {}px; height: {}px; max-width: none;",
            (w as f32 * z).round(),
            (h as f32 * z).round()
        ),
        _ => "width: auto; height: auto; max-width: 100%;".to_string(),
    };
    rsx! {
        div { class: "actions",
            for (label , value) in [("Fit", 0.0f32), ("25%", 0.25), ("50%", 0.5), ("100%", 1.0), ("200%", 2.0), ("400%", 4.0)] {
                button {
                    key: "{label}",
                    class: if zoom() == value { "on" } else { "" },
                    disabled: value > 0.0 && doc.size.is_none(),
                    onclick: move |_| zoom.set(value),
                    "{label}"
                }
            }
            span { class: "muted", "{doc.path}   {info}" }
        }
        div { class: "imgview",
            img { src: "{doc.uri}", style: "{style}" }
        }
    }
}
