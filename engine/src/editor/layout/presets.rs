//! The layouts the editor ships with.

use super::editor_kind::EditorKind::*;
use super::tree::{Axis, Node, Workspace};

/// The names of the shipped layouts, in tab order.
pub const NAMES: [&str; 6] = [
    "Layout",
    "Assets",
    "Scripting",
    "Animation",
    "Phone",
    "Phone Portrait",
];

/// The first of [`NAMES`] that are tabs from the start (the phone layouts are added on a small
/// screen, or from the `+` menu).
const TABS: usize = 4;

/// The default workspace: Outliner on the left, the Viewport in the middle, Properties on the
/// right, the Asset Browser and the Console along the bottom.
pub fn layout() -> Workspace {
    Workspace::new(
        "Layout",
        Node::split(
            Axis::Y,
            0.74,
            Node::split(
                Axis::X,
                0.14,
                Node::area(Outliner),
                Node::split(Axis::X, 0.80, Node::area(Viewport), Node::area(Properties)),
            ),
            Node::split(Axis::X, 0.62, Node::area(Assets), Node::area(Console)),
        ),
    )
}

/// Working with files: a small Viewport, a big Asset Browser.
fn assets() -> Workspace {
    Workspace::new(
        "Assets",
        Node::split(
            Axis::Y,
            0.42,
            Node::split(
                Axis::X,
                0.2,
                Node::area(Outliner),
                Node::split(Axis::X, 0.75, Node::area(Viewport), Node::area(Properties)),
            ),
            Node::split(Axis::X, 0.8, Node::area(Assets), Node::area(Console)),
        ),
    )
}

/// Code next to the scene and the log.
fn scripting() -> Workspace {
    Workspace::new(
        "Scripting",
        Node::split(
            Axis::Y,
            0.72,
            Node::split(
                Axis::X,
                0.6,
                Node::area(TextEditor),
                Node::split(Axis::Y, 0.55, Node::area(Viewport), Node::area(Outliner)),
            ),
            Node::area(Console),
        ),
    )
}

/// The Viewport over the animation editors.
fn animation() -> Workspace {
    Workspace::new(
        "Animation",
        Node::split(
            Axis::Y,
            0.58,
            Node::split(
                Axis::X,
                0.2,
                Node::area(Outliner),
                Node::split(Axis::X, 0.78, Node::area(Viewport), Node::area(Properties)),
            ),
            Node::split(Axis::Y, 0.4, Node::area(Timeline), Node::area(DopeSheet)),
        ),
    )
}

/// A landscape phone: the Viewport with the Outliner over Properties beside it. Any area can be
/// maximized or switched to another editor.
fn phone() -> Workspace {
    Workspace::new(
        "Phone",
        Node::split(
            Axis::X,
            0.76,
            Node::area(Viewport),
            Node::split(Axis::Y, 0.5, Node::area(Outliner), Node::area(Properties)),
        ),
    )
}

/// A portrait phone: the Viewport over the Outliner and Properties.
fn phone_portrait() -> Workspace {
    Workspace::new(
        "Phone Portrait",
        Node::split(
            Axis::Y,
            0.5,
            Node::area(Viewport),
            Node::split(Axis::X, 0.5, Node::area(Outliner), Node::area(Properties)),
        ),
    )
}

/// Whether a window of this size (logical pixels) is a phone's: the shipped layouts do not
/// fit it.
pub fn is_phone(width: f32, height: f32) -> bool {
    height < 600.0 || width < 700.0
}

/// The phone layout for a window of this size.
pub fn for_phone(width: f32, height: f32) -> Workspace {
    if height > width {
        phone_portrait()
    } else {
        phone()
    }
}

/// A shipped layout by name.
pub fn by_name(name: &str) -> Option<Workspace> {
    match name {
        "Layout" => Some(layout()),
        "Assets" => Some(assets()),
        "Scripting" => Some(scripting()),
        "Animation" => Some(animation()),
        "Phone" => Some(phone()),
        "Phone Portrait" => Some(phone_portrait()),
        _ => None,
    }
}

/// All the shipped layouts, in tab order.
pub fn defaults() -> Vec<Workspace> {
    NAMES[..TABS].iter().filter_map(|n| by_name(n)).collect()
}
