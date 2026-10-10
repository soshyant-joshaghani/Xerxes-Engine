//! The screen: the active workspace's areas, placed by the layout solver (the same one the
//! Scene view uses), each with a header and its editor. Areas are absolute boxes; flexbox is
//! used only inside them, so web and Blitz lay them out identically.
//!
//! Menus are drawn after every area (Blitz paints later elements on top, whatever the
//! z-index): they live inside the shell, last.
//!
//! Pointer work that must not lose events (dragging an edge, picking an area) puts a
//! transparent [`Capture`] layer over the screen, because Blitz has no pointer capture. Only
//! `client_coordinates` exist on Blitz mouse events (the element and page ones panic), so
//! positions are client coordinates minus the screen's corner.

use dioxus::html::input_data::MouseButton;
use dioxus::html::{InteractionLocation, PointerInteraction};
use dioxus::prelude::*;

use super::{Editor, Menu};
use crate::editor::layout::{
    Area, AreaId, Axis, Category, Edge, EditorKind, Path, Rect, Solved, Workspace,
    edge as edge_width, header, solve,
};
use crate::editor::protocol::{EditorCommand, Space, Tool};

/// CSS for an absolutely placed box (whole pixels: no seams between neighbours).
fn place(rect: Rect) -> String {
    format!(
        "left: {}px; top: {}px; width: {}px; height: {}px;",
        rect.x.round(),
        rect.y.round(),
        rect.w.round(),
        rect.h.round()
    )
}

/// What the pointer is doing over the screen.
#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    /// Dragging the edge at `path`. A press that never moves is a click: it opens the edge's
    /// menu, which is how touch (no right button) reaches it.
    Resize {
        path: Path,
        start: (f32, f32),
        moved: bool,
    },
    /// Choosing the area to split with a line along `axis`.
    Split(Axis),
    /// Choosing two areas to swap.
    Swap(Option<AreaId>),
}

/// The Area Edge Options menu: which edge, and where it opened (screen-local pixels).
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeMenu {
    pub path: Path,
    pub at: (f32, f32),
}

/// Moving less than this is a click, not a drag.
const DRAG_SLOP: f32 = 3.0;

#[component]
pub fn Shell(x: f32, y: f32, width: f32, height: f32) -> Element {
    let editor = use_context::<Editor>();
    let workspace = editor.workspace();
    let bounds = Rect::new(0.0, 0.0, width, height);
    let solved = solve(&workspace, bounds);
    let menu = *editor.menu.read();
    let anchor = match menu {
        Some(Menu::EditorType(id)) => solved.rect_of(id).map(|rect| (id, rect)),
        _ => None,
    };
    let header_menu = match menu {
        Some(Menu::Header(id, index)) => solved
            .areas
            .iter()
            .find(|p| p.area.id == id)
            .map(|p| (id, p.area.editor, index, p.rect)),
        _ => None,
    };
    let mode = editor.mode.read().clone();
    let edge_menu = editor.edge_menu.read().clone();
    let pointer = use_signal(|| (0.0f32, 0.0f32));
    rsx! {
        div { class: "shell",
            for placed in solved.areas.clone() {
                AreaView { key: "{placed.area.id.0}", area: placed.area, rect: placed.rect }
            }
            for edge in solved.edges.clone() {
                EdgeHandle { key: "{edge.path:?}", edge, x, y }
            }
            if let Some(mode) = mode {
                Capture { mode, solved: solved.clone(), pointer, x, y }
            }
            if let Some(menu) = edge_menu {
                EdgeMenuView { menu, width, height }
            }
            if let Some((id, rect)) = anchor {
                EditorTypeMenu { area: id, anchor: rect, width, height }
            }
            if let Some((area, kind, index, rect)) = header_menu {
                super::headers::HeaderMenuView { area, kind, index, anchor: rect }
            }
        }
    }
}

/// Where a mouse event is, relative to the screen's corner.
fn local(e: &Event<MouseData>, x: f32, y: f32) -> (f32, f32) {
    let p = e.client_coordinates();
    (p.x as f32 - x, p.y as f32 - y)
}

/// The grab zone on an edge: drag to resize, right click (or a plain click) for its menu.
#[component]
fn EdgeHandle(edge: Edge, x: f32, y: f32) -> Element {
    let editor = use_context::<Editor>();
    let path = edge.path.clone();
    rsx! {
        div {
            class: if edge.axis == Axis::X { "edge ew" } else { "edge ns" },
            style: place(edge.hit(edge_width())),
            "data-interactive": "true",
            onmousedown: move |e| {
                let at = local(&e, x, y);
                let (mut mode, mut edge_menu, mut menu) = (editor.mode, editor.edge_menu, editor.menu);
                menu.set(None);
                if e.trigger_button() == Some(MouseButton::Secondary) {
                    edge_menu.set(Some(EdgeMenu { path: path.clone(), at }));
                } else {
                    edge_menu.set(None);
                    mode.set(Some(Mode::Resize { path: path.clone(), start: at, moved: false }));
                }
            },
        }
    }
}

/// The area under a point.
fn area_at(solved: &Solved, at: (f32, f32)) -> Option<(AreaId, Rect)> {
    solved
        .areas
        .iter()
        .find(|p| p.rect.contains(at.0, at.1))
        .map(|p| (p.area.id, p.rect))
}

/// Covers the screen while a drag or a pick is going on, and does the work.
#[component]
fn Capture(mode: Mode, solved: Solved, pointer: Signal<(f32, f32)>, x: f32, y: f32) -> Element {
    let editor = use_context::<Editor>();
    let at = pointer();
    let hovered = area_at(&solved, at);

    let (move_mode, move_solved) = (mode.clone(), solved.clone());
    let on_move = move |e: Event<MouseData>| {
        let at = local(&e, x, y);
        let mut pointer = pointer;
        pointer.set(at);
        let Mode::Resize { path, start, moved } = &move_mode else {
            return;
        };
        let far = (at.0 - start.0).hypot(at.1 - start.1) >= DRAG_SLOP;
        if !*moved && !far {
            return;
        }
        if !*moved {
            let mut mode = editor.mode;
            mode.set(Some(Mode::Resize {
                path: path.clone(),
                start: *start,
                moved: true,
            }));
        }
        if let Some(edge) = move_solved.edges.iter().find(|edge| &edge.path == path) {
            let along = if edge.axis == Axis::X { at.0 } else { at.1 };
            let _ = editor.edit_layout(|w| w.resize(path, edge.ratio_at(along)));
        }
    };

    let up_mode = mode.clone();
    let on_up = move |_: Event<MouseData>| {
        let Mode::Resize { path, start, moved } = &up_mode else {
            return;
        };
        let mut mode = editor.mode;
        mode.set(None);
        // A click on an edge opens its menu (the way to it without a right button).
        if !*moved {
            let mut edge_menu = editor.edge_menu;
            edge_menu.set(Some(EdgeMenu {
                path: path.clone(),
                at: *start,
            }));
        }
    };

    let (down_mode, down_solved) = (mode.clone(), solved.clone());
    let on_down = move |e: Event<MouseData>| {
        let mut mode = editor.mode;
        // The right button cancels a pick.
        if e.trigger_button() == Some(MouseButton::Secondary) {
            mode.set(None);
            return;
        }
        let at = local(&e, x, y);
        let target = area_at(&down_solved, at);
        match (&down_mode, target) {
            (Mode::Split(axis), Some((id, rect))) => {
                if Workspace::can_split(rect, *axis) {
                    let ratio = match axis {
                        Axis::X => (at.0 - rect.x) / rect.w,
                        Axis::Y => (at.1 - rect.y) / rect.h,
                    };
                    let result = editor.edit_layout(|w| w.split(id, *axis, ratio));
                    if let Err(err) = result {
                        editor.error(format!("layout: {err}"));
                    }
                } else {
                    editor.error("layout: that area is too small to split that way");
                }
                mode.set(None);
            }
            (Mode::Swap(None), Some((id, _))) => mode.set(Some(Mode::Swap(Some(id)))),
            (Mode::Swap(Some(first)), Some((id, _))) => {
                if *first != id {
                    let result = editor.edit_layout(|w| w.swap(*first, id));
                    if let Err(err) = result {
                        editor.error(format!("layout: {err}"));
                    }
                }
                mode.set(None);
            }
            _ => {}
        }
    };

    let resizing = matches!(mode, Mode::Resize { .. });
    let hint = match &mode {
        Mode::Resize { .. } => None,
        Mode::Split(_) => Some("Click an area to split it there. Right click cancels."),
        Mode::Swap(None) => Some("Click the first area to swap. Right click cancels."),
        Mode::Swap(Some(_)) => Some("Click the area to swap it with. Right click cancels."),
    };
    // What a pick shows: the area under the pointer, the first area of a swap, and the line a
    // split would make.
    let first = match &mode {
        Mode::Swap(Some(id)) => solved.rect_of(*id),
        _ => None,
    };
    let line = match (&mode, hovered) {
        (Mode::Split(Axis::X), Some((_, r))) => {
            Some(Rect::new(at.0.clamp(r.x, r.right()) - 1.0, r.y, 2.0, r.h))
        }
        (Mode::Split(Axis::Y), Some((_, r))) => {
            Some(Rect::new(r.x, at.1.clamp(r.y, r.bottom()) - 1.0, r.w, 2.0))
        }
        _ => None,
    };
    rsx! {
        if !resizing {
            if let Some((_, rect)) = hovered {
                div { class: "pick", style: place(rect) }
            }
            if let Some(rect) = first {
                div { class: "pick on", style: place(rect) }
            }
            if let Some(rect) = line {
                div { class: "splitline", style: place(rect) }
            }
            if let Some(text) = hint {
                div { class: "hintbar", "{text}" }
            }
        }
        div {
            class: if resizing { "capture" } else { "capture pickmode" },
            "data-interactive": "true",
            onmousemove: on_move,
            onmouseup: on_up,
            onmousedown: on_down,
        }
    }
}

/// Blender's Area Edge Options.
#[component]
fn EdgeMenuView(menu: EdgeMenu, width: f32, height: f32) -> Element {
    let editor = use_context::<Editor>();
    let workspace = editor.workspace();
    let bounds = Rect::new(0.0, 0.0, width, height);
    let solved = solve(&workspace, bounds);
    let Some(edge) = solved.edges.iter().find(|e| e.path == menu.path) else {
        return rsx! {};
    };
    // The two areas this edge separates where it was clicked: `a` is the left or top one.
    // "Join Right" lets the left area grow over the right one; they must share a whole edge
    // (a tree built another way does not matter: lining two edges up is enough).
    let (side_a, side_b) = match edge.axis {
        Axis::X => ((edge.pos - 4.0, menu.at.1), (edge.pos + 4.0, menu.at.1)),
        Axis::Y => ((menu.at.0, edge.pos - 4.0), (menu.at.0, edge.pos + 4.0)),
    };
    let pair = area_at(&solved, side_a).zip(area_at(&solved, side_b));
    let ids = pair.map(|((a, _), (b, _))| (a, b));
    let can = |keep_first: bool| {
        ids.is_some_and(|(a, b)| {
            let (keep, gone) = if keep_first { (a, b) } else { (b, a) };
            workspace.can_join(keep, gone, bounds)
        })
    };
    let (can_a, can_b) = (can(true), can(false));
    let (keep_a, keep_b) = match edge.axis {
        Axis::X => ("Join Right", "Join Left"),
        Axis::Y => ("Join Down", "Join Up"),
    };
    let left = menu.at.0.min(width - 200.0).max(4.0);
    let top = menu.at.1.min(height - 170.0).max(4.0);
    let start = move |mode: Mode| {
        let (mut m, mut edge_menu) = (editor.mode, editor.edge_menu);
        m.set(Some(mode));
        edge_menu.set(None);
    };
    // Join the pair, keeping the first or the second of the two areas.
    let join = move |keep_first: bool| {
        let mut edge_menu = editor.edge_menu;
        if let Some((a, b)) = ids {
            let (keep, gone) = if keep_first { (a, b) } else { (b, a) };
            let result = editor.edit_layout(|w| w.join(keep, gone, bounds));
            if let Err(err) = result {
                editor.error(format!("layout: {err}"));
            }
        }
        edge_menu.set(None);
    };
    rsx! {
        div {
            class: "backdrop",
            "data-interactive": "true",
            onmousedown: move |_| {
                let mut edge_menu = editor.edge_menu;
                edge_menu.set(None);
            },
        }
        div {
            class: "edge-menu",
            "data-interactive": "true",
            style: "left: {left}px; top: {top}px;",
            div { class: "title", "Area Edge Options" }
            div { class: "rule" }
            div { class: "item", onclick: move |_| start(Mode::Split(Axis::X)), "Vertical Split" }
            div { class: "item", onclick: move |_| start(Mode::Split(Axis::Y)), "Horizontal Split" }
            div { class: "rule" }
            div {
                class: if can_a { "item" } else { "item off" },
                onclick: move |_| if can_a { join(true) },
                "{keep_a}"
            }
            div {
                class: if can_b { "item" } else { "item off" },
                onclick: move |_| if can_b { join(false) },
                "{keep_b}"
            }
            div { class: "rule" }
            div { class: "item", onclick: move |_| start(Mode::Swap(None)), "Swap Areas" }
        }
    }
}

/// One area: its header and its editor.
#[component]
fn AreaView(area: Area, rect: Rect) -> Element {
    let viewport = area.editor == EditorKind::Viewport;
    rsx! {
        // The Viewport's body belongs to Bevy: only its header takes the pointer.
        div {
            class: if viewport { "area clear" } else { "area" },
            style: place(rect),
            "data-interactive": if viewport { None } else { Some("true") },
            div { class: "area-head", "data-interactive": "true",
                EditorTypeButton { area: area.id, editor: area.editor }
                super::headers::HeaderMenus { area: area.id, kind: area.editor }
                if viewport {
                    ViewportHeader {}
                }
                div { class: "grow" }
                AreaButtons { area: area.id }
            }
            div { class: "area-body",
                match area.editor {
                    EditorKind::Viewport => rsx! {},
                    EditorKind::Outliner => rsx! { super::hierarchy::Hierarchy {} },
                    EditorKind::Properties => rsx! { super::inspector::Inspector {} },
                    EditorKind::Assets => rsx! { super::assets::Assets {} },
                    EditorKind::Console => rsx! { super::console::Console {} },
                    EditorKind::History => rsx! { super::historypanel::HistoryPanel {} },
                    EditorKind::Spreadsheet => rsx! { super::spreadsheet::SpreadsheetPanel {} },
                    EditorKind::Preferences => rsx! { super::preferences::PreferencesPanel {} },
                    EditorKind::GamePreview => rsx! { super::gamepreview::GamePreviewPanel {} },
                    EditorKind::TextEditor => rsx! { super::texteditor::TextEditorPanel {} },
                    EditorKind::ImageEditor => rsx! { super::imageeditor::ImageEditorPanel {} },
                    EditorKind::ProjectSettings => rsx! { super::projectsettings::ProjectSettingsPanel {} },
                    other => rsx! { Placeholder { editor: other } },
                }
            }
        }
    }
}

/// Maximize (Blender's Ctrl+Space) and close, at the header's right.
#[component]
fn AreaButtons(area: AreaId) -> Element {
    let editor = use_context::<Editor>();
    let workspace = editor.workspace();
    let maximized = workspace.maximized == Some(area);
    let several = workspace.areas().len() > 1;
    rsx! {
        button {
            class: "dim",
            onclick: move |_| {
                editor.edit_layout(|w| w.toggle_maximize(area));
            },
            if maximized { "Min" } else { "Max" }
        }
        button {
            class: "dim",
            disabled: !several,
            onclick: move |_| {
                let result = editor.edit_layout(|w| w.close(area));
                if let Err(err) = result {
                    editor.error(format!("layout: {err}"));
                }
            },
            "×"
        }
    }
}

/// The body of an editor that does not exist yet (or of an `Empty` area).
#[component]
fn Placeholder(editor: EditorKind) -> Element {
    rsx! {
        div { class: "hint",
            if editor == EditorKind::Empty {
                "Empty area. Choose an editor with the button at the top left of this area."
            } else {
                "{editor.title()}: {editor.planned()} Not built yet."
            }
        }
    }
}

/// The button at the header's left: shows the area's editor, opens the editor-type menu.
#[component]
fn EditorTypeButton(area: AreaId, editor: EditorKind) -> Element {
    let app = use_context::<Editor>();
    rsx! {
        button {
            class: "etype",
            onclick: move |_| {
                let mut menu = app.menu;
                let open = *menu.peek() == Some(Menu::EditorType(area));
                menu.set(if open { None } else { Some(Menu::EditorType(area)) });
            },
            span { class: "ico", "{editor.icon()}" }
            span { class: "grow", "{editor.short_title()}" }
            span { class: "caret" }
        }
    }
}

/// The Viewport's own toolbar: the transform tools and the camera.
#[component]
fn ViewportHeader() -> Element {
    let editor = use_context::<Editor>();
    let port = use_context::<super::Port>();
    let tool = *editor.tool.read();
    let space = *editor.space.read();
    let tool_button = move |which: Tool, label: &'static str| {
        rsx! {
            button {
                class: if tool == which { "on" } else { "" },
                onclick: move |_| {
                    let mut tool = editor.tool;
                    tool.set(which);
                },
                "{label}"
            }
        }
    };
    rsx! {
        div { class: "sep" }
        {tool_button(Tool::Move, "Move (W)")}
        {tool_button(Tool::Rotate, "Rotate (E)")}
        {tool_button(Tool::Scale, "Scale (R)")}
        button {
            class: if space == Space::Local { "on" } else { "" },
            onclick: move |_| {
                let mut space = editor.space;
                space.set(if space() == Space::Local { Space::Global } else { Space::Local });
            },
            if space == Space::Local { "Local (X)" } else { "Global (X)" }
        }
        button { onclick: move |_| port.send(EditorCommand::Frame), "Frame (F)" }
    }
}

/// Blender's editor-type menu: four columns of editors (stacked on a narrow screen).
#[component]
fn EditorTypeMenu(area: AreaId, anchor: Rect, width: f32, height: f32) -> Element {
    let editor = use_context::<Editor>();
    let workspace = editor.workspace();
    let current = workspace.area(area).map(|a| a.editor);

    let columns = if width >= 760.0 {
        4.0
    } else if width >= 420.0 {
        2.0
    } else {
        1.0
    };
    // Column width + gaps + padding + border.
    let menu_width = columns * 168.0 + (columns - 1.0) * 4.0 + 18.0;
    let left = (anchor.x + 4.0).min(width - menu_width - 4.0).max(4.0);
    let top = anchor.y + header() - 2.0;
    let max_height = (height - top - 6.0).max(120.0);
    rsx! {
        div {
            class: "backdrop",
            "data-interactive": "true",
            onclick: move |_| {
                let mut menu = editor.menu;
                menu.set(None);
            },
        }
        div {
            class: "etype-menu",
            "data-interactive": "true",
            style: "left: {left}px; top: {top}px; width: {menu_width}px; max-height: {max_height}px;",
            for category in Category::ALL {
                div { key: "{category.title()}", class: "etype-col",
                    div { class: "etype-cat", "{category.title()}" }
                    for kind in EditorKind::menu().filter(|k| k.category() == category) {
                        EditorTypeItem {
                            key: "{kind.id()}",
                            area,
                            kind,
                            on: current == Some(kind),
                            allowed: workspace.can_show(area, kind),
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn EditorTypeItem(area: AreaId, kind: EditorKind, on: bool, allowed: bool) -> Element {
    let editor = use_context::<Editor>();
    let class = if on {
        "etype-item on"
    } else if !allowed {
        "etype-item off"
    } else if !kind.exists() {
        "etype-item soon"
    } else {
        "etype-item"
    };
    rsx! {
        div {
            class,
            onclick: move |_| {
                if !allowed {
                    return;
                }
                let result = editor.edit_layout(|w| w.set_editor(area, kind));
                if let Err(err) = result {
                    editor.error(format!("layout: {err}"));
                }
                let mut menu = editor.menu;
                menu.set(None);
            },
            span { class: "ico", "{kind.icon()}" }
            span { class: "grow", "{kind.title()}" }
            if !kind.exists() {
                span { class: "muted", "soon" }
            }
        }
    }
}
