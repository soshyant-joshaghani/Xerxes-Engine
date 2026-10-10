//! The editors an area can show. One table holds everything about them (id, title, icon,
//! menu column, and what is not built yet), so the editor-type menu, the saved files and the
//! placeholder bodies cannot disagree.
//!
//! The menu is Blender's: four columns (General, Animation, Scripting, Data). Editors that do
//! not exist yet can still be picked: the area shows what is planned. That keeps saved layouts
//! stable (an id never disappears) and makes the roadmap visible.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A column of the editor-type menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    General,
    Animation,
    Scripting,
    Data,
}

impl Category {
    pub const ALL: [Category; 4] = [
        Category::General,
        Category::Animation,
        Category::Scripting,
        Category::Data,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Category::General => "General",
            Category::Animation => "Animation",
            Category::Scripting => "Scripting",
            Category::Data => "Data",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EditorKind {
    /// What a saved id that no longer exists becomes; never offered in the menu.
    Empty,
    Viewport,
    GamePreview,
    ImageEditor,
    NodeEditor,
    Timeline,
    DopeSheet,
    GraphEditor,
    TextEditor,
    Console,
    Outliner,
    Properties,
    Assets,
    Spreadsheet,
    ProjectSettings,
    Preferences,
    History,
    Warp,
    Quests,
    Dialogue,
}

/// The most Viewports one layout can show.
pub const MAX_VIEWPORTS: usize = 4;

struct Info {
    kind: EditorKind,
    id: &'static str,
    title: &'static str,
    /// Two or three characters (an icon font comes later; text renders the same everywhere).
    icon: &'static str,
    category: Category,
    /// Empty when the editor exists; otherwise what is planned and when.
    planned: &'static str,
}

use Category::*;
use EditorKind::*;

const fn info(
    kind: EditorKind,
    id: &'static str,
    title: &'static str,
    icon: &'static str,
    category: Category,
    planned: &'static str,
) -> Info {
    Info {
        kind,
        id,
        title,
        icon,
        category,
        planned,
    }
}

/// Menu order: column by column, top to bottom. `Empty` is first and not in the menu.
const TABLE: &[Info] = &[
    info(Empty, "empty", "Empty", "--", General, ""),
    info(Viewport, "viewport", "Viewport", "3D", General, ""),
    info(
        GamePreview,
        "game_preview",
        "Game Preview",
        "PL",
        General,
        "",
    ),
    info(
        ImageEditor,
        "image_editor",
        "Image Editor",
        "IM",
        General,
        "",
    ),
    info(
        NodeEditor,
        "node_editor",
        "Node Editor",
        "ND",
        General,
        "Node graphs (phase 10).",
    ),
    info(
        Timeline,
        "timeline",
        "Timeline",
        "TL",
        Animation,
        "Animation timeline.",
    ),
    info(
        DopeSheet,
        "dope_sheet",
        "Dope Sheet",
        "DS",
        Animation,
        "Keyframes of every object.",
    ),
    info(
        GraphEditor,
        "graph_editor",
        "Graph Editor",
        "GR",
        Animation,
        "Animation curves.",
    ),
    info(
        TextEditor,
        "text_editor",
        "Text Editor",
        "TX",
        Scripting,
        "",
    ),
    info(Console, "console", "Console", ">_", Scripting, ""),
    info(Outliner, "outliner", "Outliner", "OL", Data, ""),
    info(Properties, "properties", "Properties", "PR", Data, ""),
    info(Assets, "assets", "Asset Browser", "AB", Data, ""),
    info(Spreadsheet, "spreadsheet", "Spreadsheet", "SS", Data, ""),
    info(
        ProjectSettings,
        "project_settings",
        "Project Settings",
        "PS",
        Data,
        "",
    ),
    info(Preferences, "preferences", "Preferences", "PF", Data, ""),
    info(History, "history", "History", "HI", Data, ""),
    info(
        Warp,
        "warp",
        "Warp",
        "WP",
        Data,
        "The level manager (stage 3.7).",
    ),
    info(
        Quests,
        "quests",
        "Quests",
        "QS",
        Data,
        "Quest states and entries (stage 3.8).",
    ),
    info(
        Dialogue,
        "dialogue",
        "Dialogue",
        "DL",
        Data,
        "Dialogue graphs (stage 3.9).",
    ),
];

impl EditorKind {
    /// Every editor the menu offers, in menu order.
    pub fn menu() -> impl Iterator<Item = EditorKind> {
        TABLE.iter().map(|i| i.kind).filter(|k| *k != Empty)
    }

    fn info(self) -> &'static Info {
        TABLE
            .iter()
            .find(|i| i.kind == self)
            .expect("every EditorKind is in TABLE")
    }

    /// The stable name written to files.
    pub fn id(self) -> &'static str {
        self.info().id
    }

    pub fn from_id(id: &str) -> Option<EditorKind> {
        TABLE.iter().find(|i| i.id == id).map(|i| i.kind)
    }

    pub fn title(self) -> &'static str {
        self.info().title
    }

    /// The name on an area's header button, which has room for about ten characters.
    pub fn short_title(self) -> &'static str {
        match self {
            Assets => "Assets",
            ProjectSettings => "Settings",
            GamePreview => "Game",
            ImageEditor => "Image",
            NodeEditor => "Nodes",
            DopeSheet => "Dopesheet",
            GraphEditor => "Graph",
            TextEditor => "Text",
            Spreadsheet => "Sheet",
            Preferences => "Prefs",
            other => other.title(),
        }
    }

    pub fn icon(self) -> &'static str {
        self.info().icon
    }

    pub fn category(self) -> Category {
        self.info().category
    }

    /// What is planned for an editor that does not exist yet (empty when it exists).
    pub fn planned(self) -> &'static str {
        self.info().planned
    }

    pub fn exists(self) -> bool {
        self.info().planned.is_empty() && self != Empty
    }

    /// How many areas of a workspace may show this editor: every Viewport is a camera that
    /// renders the scene again, so there are four at most.
    pub fn max_instances(self) -> Option<usize> {
        match self {
            Viewport => Some(MAX_VIEWPORTS),
            _ => None,
        }
    }
}

impl Serialize for EditorKind {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.id())
    }
}

/// An id this build does not know (a file from a newer editor) becomes `Empty`: restoring a
/// layout never fails because of one editor.
impl<'de> Deserialize<'de> for EditorKind {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let id = String::deserialize(d)?;
        Ok(EditorKind::from_id(&id).unwrap_or(Empty))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_is_consistent() {
        for kind in EditorKind::menu() {
            assert_eq!(EditorKind::from_id(kind.id()), Some(kind));
        }
        let mut ids: Vec<_> = TABLE.iter().map(|i| i.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), TABLE.len(), "ids are unique");
        assert!(
            Category::ALL
                .iter()
                .all(|c| EditorKind::menu().any(|k| k.category() == *c))
        );
    }
}
