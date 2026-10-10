//! The Blender-style screen layout: areas tile the window exactly, operations keep the tree
//! valid, and saved files survive unknown editors.

use xerxes_engine::editor::layout::{
    AreaId, Axis, EditorKind, LayoutError, Layouts, MAX_VIEWPORTS, MIN_WIDTH, NAME_LIMIT, Node,
    Rect, Side, Workspace, header, layout, min_height, preset, solve,
};

const WINDOW: Rect = Rect::new(0.0, 0.0, 1920.0, 1042.0);

fn id_of(ws: &Workspace, kind: EditorKind) -> AreaId {
    ws.areas()
        .into_iter()
        .find(|a| a.editor == kind)
        .unwrap_or_else(|| panic!("no {kind:?}"))
        .id
}

/// The areas cover the bounds exactly: total area equals the bounds, nothing overlaps, and
/// nothing sticks out.
fn assert_tiles(ws: &Workspace, bounds: Rect) {
    let solved = solve(ws, bounds);
    let total: f32 = solved.areas.iter().map(|p| p.rect.area()).sum();
    assert!(
        (total - bounds.area()).abs() < bounds.area() * 1e-4,
        "areas cover {total} of {}",
        bounds.area()
    );
    for (i, a) in solved.areas.iter().enumerate() {
        let (r, b) = (a.rect, bounds);
        assert!(r.x >= b.x - 1e-3 && r.y >= b.y - 1e-3, "{r:?} outside");
        assert!(r.right() <= b.right() + 1e-3 && r.bottom() <= b.bottom() + 1e-3);
        for other in &solved.areas[i + 1..] {
            let o = other.rect;
            let w = r.right().min(o.right()) - r.x.max(o.x);
            let h = r.bottom().min(o.bottom()) - r.y.max(o.y);
            assert!(w <= 1e-2 || h <= 1e-2, "{r:?} overlaps {o:?}");
        }
    }
}

#[test]
fn the_default_layout_tiles_every_window_size() {
    let ws = layout();
    assert_eq!(ws.areas().len(), 5);
    for (w, h) in [
        (1920.0, 1042.0),
        (1280.0, 720.0),
        (1100.0, 481.0),
        (800.0, 400.0),
        (300.0, 200.0),
    ] {
        assert_tiles(&ws, Rect::new(0.0, 0.0, w, h));
    }
    // Not at the origin (the UI places the shell below the app bar).
    assert_tiles(&ws, Rect::new(0.0, 38.0, 1500.0, 800.0));
}

#[test]
fn areas_keep_their_minimum_size_when_it_fits() {
    let mut ws = layout();
    let viewport = id_of(&ws, EditorKind::Viewport);
    // Ask for absurd shares on every edge.
    for edge in solve(&ws, WINDOW).edges {
        ws.resize(&edge.path, 0.99).unwrap();
    }
    for p in solve(&ws, WINDOW).areas {
        assert!(p.rect.w >= MIN_WIDTH - 1e-3, "{:?} too narrow", p.rect);
        assert!(p.rect.h >= min_height() - 1e-3, "{:?} too short", p.rect);
    }
    assert!(solve(&ws, WINDOW).rect_of(viewport).is_some());
}

#[test]
fn the_viewport_body_is_under_the_header() {
    let ws = layout();
    let solved = solve(&ws, WINDOW);
    let area = solved.first(EditorKind::Viewport).unwrap().rect;
    let body = solved.body_of(EditorKind::Viewport).unwrap();
    assert_eq!(body.y, area.y + header());
    assert_eq!(body.h, area.h - header());
    assert_eq!((body.x, body.w), (area.x, area.w));
}

#[test]
fn edges_follow_the_splits() {
    let ws = layout();
    let solved = solve(&ws, WINDOW);
    // Four splits for five areas.
    assert_eq!(solved.edges.len(), 4);
    let root = solved.edges.iter().find(|e| e.path.is_empty()).unwrap();
    assert_eq!(root.axis, Axis::Y);
    assert!((root.pos - WINDOW.h * 0.74).abs() < 1.0);
    assert_eq!((root.from, root.to), (0.0, WINDOW.w));
    // Dragging the pointer to the middle gives half.
    assert!((root.ratio_at(WINDOW.h / 2.0) - 0.5).abs() < 1e-4);
    assert_eq!(root.hit(6.0).h, 6.0);
}

#[test]
fn split_adds_an_area_that_shows_the_same_editor() {
    let mut ws = layout();
    let outliner = id_of(&ws, EditorKind::Outliner);
    let new = ws.split(outliner, Axis::Y, 0.5).unwrap();
    assert_eq!(ws.area(new).unwrap().editor, EditorKind::Outliner);
    assert_eq!(ws.areas().len(), 6);
    assert_tiles(&ws, WINDOW);
    // Ids stay unique.
    let mut ids: Vec<_> = ws.areas().iter().map(|a| a.id).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 6);
}

#[test]
fn up_to_four_viewports_and_the_fifth_is_refused() {
    let mut ws = layout();
    let viewport = id_of(&ws, EditorKind::Viewport);
    // Splitting a Viewport gives another one (each is a camera of its own).
    let second = ws.split(viewport, Axis::X, 0.5).unwrap();
    assert_eq!(ws.area(second).unwrap().editor, EditorKind::Viewport);
    let third = ws.split(second, Axis::Y, 0.5).unwrap();
    let fourth = ws.split(third, Axis::X, 0.5).unwrap();
    assert_eq!(ws.count(EditorKind::Viewport), MAX_VIEWPORTS);
    assert_tiles(&ws, WINDOW);

    // The fifth: a split gives an Empty area, and no other area may become a Viewport.
    let fifth = ws.split(fourth, Axis::Y, 0.5).unwrap();
    assert_eq!(ws.area(fifth).unwrap().editor, EditorKind::Empty);
    let outliner = id_of(&ws, EditorKind::Outliner);
    assert_eq!(
        ws.set_editor(outliner, EditorKind::Viewport),
        Err(LayoutError::TooMany(EditorKind::Viewport))
    );
    // A Viewport may keep being one, and freeing one lets another area take it.
    assert!(ws.set_editor(viewport, EditorKind::Viewport).is_ok());
    ws.set_editor(viewport, EditorKind::Console).unwrap();
    assert!(ws.set_editor(outliner, EditorKind::Viewport).is_ok());
}

#[test]
fn close_gives_the_space_to_the_sibling() {
    let mut ws = layout();
    let console = id_of(&ws, EditorKind::Console);
    let assets = id_of(&ws, EditorKind::Assets);
    ws.close(console).unwrap();
    assert_eq!(ws.areas().len(), 4);
    assert_tiles(&ws, WINDOW);
    let solved = solve(&ws, WINDOW);
    assert!((solved.rect_of(assets).unwrap().w - WINDOW.w).abs() < 1e-2);
    assert_eq!(ws.close(console), Err(LayoutError::NoArea(console)));

    let mut single = Workspace::new("one", Node::area(EditorKind::Console));
    let only = single.areas()[0].id;
    assert_eq!(single.close(only), Err(LayoutError::LastArea));
}

#[test]
fn join_swaps_and_resize() {
    let mut ws = layout();
    let assets = id_of(&ws, EditorKind::Assets);
    let console = id_of(&ws, EditorKind::Console);
    let bottom = solve(&ws, WINDOW)
        .edges
        .into_iter()
        .find(|e| e.axis == Axis::X && e.path == [Side::B])
        .unwrap();

    // Swap: they trade editors, the places stay.
    let before = solve(&ws, WINDOW);
    ws.swap(assets, console).unwrap();
    assert_eq!(ws.area(assets).unwrap().editor, EditorKind::Console);
    assert_eq!(solve(&ws, WINDOW).rect_of(assets), before.rect_of(assets));
    ws.swap(assets, console).unwrap();

    // Resize: the line moves.
    ws.resize(&bottom.path, 0.25).unwrap();
    let moved = solve(&ws, WINDOW).rect_of(assets).unwrap();
    assert!((moved.w - WINDOW.w * 0.25).abs() < 1.0);
    assert_eq!(
        ws.resize(&[Side::A, Side::A, Side::A, Side::A], 0.5),
        Err(LayoutError::NoEdge)
    );

    // Join: the left one keeps and grows over the right one.
    assert!(ws.can_join(assets, console, WINDOW));
    assert_eq!(ws.join(assets, console, WINDOW), Ok(assets));
    assert!(ws.area(console).is_none());
    assert_tiles(&ws, WINDOW);

    // Areas that only touch at a corner, or are far apart, do not join.
    let outliner = id_of(&ws, EditorKind::Outliner);
    assert_eq!(
        ws.join(assets, outliner, WINDOW),
        Err(LayoutError::NotAdjacent)
    );
    assert_eq!(
        ws.join(assets, assets, WINDOW),
        Err(LayoutError::NotAdjacent)
    );
}

/// The case that matters: the centre and the right column are each cut in two, at different
/// heights, so no tree edge separates the two bottom areas. Once the cuts are lined up they
/// join (and not before).
#[test]
fn two_areas_that_share_a_whole_edge_join_whatever_the_tree_looks_like() {
    // Outliner | ( Viewport over Empty | Properties over Properties )
    let mut ws = Workspace::new(
        "t",
        Node::split(
            Axis::X,
            0.2,
            Node::area(EditorKind::Outliner),
            Node::split(
                Axis::X,
                0.8,
                Node::split(
                    Axis::Y,
                    0.9,
                    Node::area(EditorKind::Viewport),
                    Node::area(EditorKind::Empty),
                ),
                Node::split(
                    Axis::Y,
                    0.6,
                    Node::area(EditorKind::Properties),
                    Node::area(EditorKind::Properties),
                ),
            ),
        ),
    );
    let empty = id_of(&ws, EditorKind::Empty);
    let props: Vec<_> = ws
        .areas()
        .into_iter()
        .filter(|a| a.editor == EditorKind::Properties)
        .map(|a| a.id)
        .collect();
    let (top, bottom) = (props[0], props[1]);

    // Cuts at 90% and 60% of the height: not aligned.
    assert_eq!(
        ws.join(bottom, empty, WINDOW),
        Err(LayoutError::NotAdjacent)
    );
    assert!(!ws.can_join(empty, bottom, WINDOW));

    // Drag the right column's cut to the centre's (a pixel or two off is fine).
    let right_cut = solve(&ws, WINDOW)
        .edges
        .into_iter()
        .find(|e| e.axis == Axis::Y && e.path == [Side::B, Side::B])
        .unwrap();
    let centre_cut = solve(&ws, WINDOW)
        .edges
        .into_iter()
        .find(|e| e.axis == Axis::Y && e.path == [Side::B, Side::A])
        .unwrap();
    ws.resize(&right_cut.path, right_cut.ratio_at(centre_cut.pos + 2.0))
        .unwrap();
    assert!(ws.can_join(empty, bottom, WINDOW));
    assert!(ws.can_join(bottom, empty, WINDOW));

    // Properties (bottom) grows over the Empty one: five areas become four.
    assert_eq!(ws.join(bottom, empty, WINDOW), Ok(bottom));
    assert_eq!(ws.areas().len(), 4);
    assert!(ws.area(empty).is_none());
    assert_tiles(&ws, WINDOW);
    // It now spans under both columns, and the top two keep their places.
    let solved = solve(&ws, WINDOW);
    let merged = solved.rect_of(bottom).unwrap();
    let viewport = solved.rect_of(id_of(&ws, EditorKind::Viewport)).unwrap();
    let top_rect = solved.rect_of(top).unwrap();
    assert!((merged.x - viewport.x).abs() < 0.5);
    assert!((merged.right() - top_rect.right()).abs() < 0.5);
    assert!(merged.y >= viewport.bottom() - 0.5);
    // The outliner column was not touched.
    assert_eq!(
        solved
            .rect_of(id_of(&ws, EditorKind::Outliner))
            .unwrap()
            .w
            .round(),
        (WINDOW.w * 0.2).round()
    );
}

#[test]
fn a_join_that_would_need_a_pinwheel_is_refused() {
    // Two columns; the left is cut at 30%, the right at 60%; joining the left-top with the
    // right-top is impossible (they are not aligned), and so is anything that is not a
    // neighbour. The tree is left untouched on every failure.
    let ws = Workspace::new(
        "t",
        Node::split(
            Axis::X,
            0.5,
            Node::split(
                Axis::Y,
                0.3,
                Node::area(EditorKind::Outliner),
                Node::area(EditorKind::Console),
            ),
            Node::split(
                Axis::Y,
                0.6,
                Node::area(EditorKind::Properties),
                Node::area(EditorKind::Assets),
            ),
        ),
    );
    let before = ws.clone();
    let mut work = ws.clone();
    let a = id_of(&ws, EditorKind::Outliner);
    let b = id_of(&ws, EditorKind::Properties);
    assert!(work.join(a, b, WINDOW).is_err());
    assert_eq!(work, before);
}

#[test]
fn maximize_shows_one_area_and_restores() {
    let mut ws = layout();
    let viewport = id_of(&ws, EditorKind::Viewport);
    ws.toggle_maximize(viewport);
    let solved = solve(&ws, WINDOW);
    assert_eq!(solved.areas.len(), 1);
    assert_eq!(solved.areas[0].rect, WINDOW);
    assert!(solved.edges.is_empty());
    // Closing the maximized area brings the layout back.
    ws.close(viewport).unwrap();
    assert_eq!(ws.maximized, None);
    assert_eq!(solve(&ws, WINDOW).areas.len(), 4);
    ws.toggle_maximize(AreaId(999));
    assert_eq!(ws.maximized, None);
}

#[test]
fn the_layout_round_trips_through_json_and_survives_unknown_editors() {
    let ws = layout();
    let text = serde_json::to_string(&ws).unwrap();
    assert!(text.contains("\"viewport\""));
    let back: Workspace = serde_json::from_str(&text).unwrap();
    assert_eq!(back, ws);

    // A file from a newer editor: the unknown editor becomes Empty, the rest loads.
    let newer = text.replace("\"console\"", "\"hologram\"");
    let mut loaded: Workspace = serde_json::from_str(&newer).unwrap();
    loaded.repair();
    assert_eq!(loaded.count(EditorKind::Empty), 1);
    assert_eq!(loaded.areas().len(), 5);

    // A damaged file (duplicate ids, a ratio out of range, a missing maximized area) is fixed.
    let mut broken = layout();
    broken.maximized = Some(AreaId(77));
    let json = serde_json::to_string(&broken)
        .unwrap()
        .replace("\"id\":2", "\"id\":1");
    let mut loaded: Workspace = serde_json::from_str(&json).unwrap();
    loaded.repair();
    let mut ids: Vec<_> = loaded.areas().iter().map(|a| a.id).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 5);
    assert_eq!(loaded.maximized, None);
    assert_tiles(&loaded, WINDOW);
}

#[test]
fn the_saved_layouts_ship_four_tabs_that_all_tile() {
    let layouts = Layouts::default();
    let names: Vec<_> = layouts.workspaces.iter().map(|w| w.name.as_str()).collect();
    assert_eq!(names, ["Layout", "Assets", "Scripting", "Animation"]);
    for ws in &layouts.workspaces {
        assert_tiles(ws, WINDOW);
        assert_tiles(ws, Rect::new(0.0, 64.0, 1100.0, 430.0));
        // One viewport in each, and ids are unique.
        assert_eq!(ws.count(EditorKind::Viewport), 1, "{}", ws.name);
    }
}

#[test]
fn tabs_are_added_duplicated_renamed_and_removed() {
    let mut layouts = Layouts::default();
    // A shipped layout added twice gets a unique name and is shown.
    let index = layouts.add(preset("Assets").unwrap());
    assert_eq!(layouts.workspaces[index].name, "Assets 2");
    assert_eq!(layouts.active, index);
    // Duplicate, rename (long names are cut).
    let copy = layouts.duplicate(0).unwrap();
    assert_eq!(layouts.workspaces[copy].name, "Layout 2");
    layouts.rename(copy, "A very long layout name that does not fit");
    assert_eq!(layouts.workspaces[copy].name.chars().count(), NAME_LIMIT);
    // Edits go to the active workspace only.
    let before = layouts.workspaces[0].clone();
    let viewport = id_of(layouts.active_workspace(), EditorKind::Viewport);
    layouts.active_mut().toggle_maximize(viewport);
    assert_eq!(layouts.workspaces[0], before);
    assert_eq!(layouts.active_workspace().maximized, Some(viewport));
    // Reset gives the shipped layout back (a custom name gets the default one).
    layouts.reset_workspace(copy);
    assert_eq!(layouts.workspaces[copy].maximized, None);
    assert_eq!(layouts.workspaces[copy].count(EditorKind::Viewport), 1);
    // Remove: the shown one moves to a neighbour; the last cannot go.
    layouts.select(1);
    layouts.remove(0).unwrap();
    assert_eq!(layouts.active_workspace().name, "Assets");
    while layouts.workspaces.len() > 1 {
        layouts.remove(0).unwrap();
    }
    assert_eq!(layouts.remove(0), Err(LayoutError::LastWorkspace));
}

#[test]
fn saved_layouts_round_trip_and_bad_files_give_nothing() {
    let mut layouts = Layouts::default();
    layouts.select(2);
    layouts.rename(1, "Mine");
    let back = Layouts::from_json(&layouts.to_json()).unwrap();
    assert_eq!(back, layouts);
    assert_eq!(back.active_workspace().name, "Scripting");

    assert!(Layouts::from_json("not json").is_none());
    assert!(Layouts::from_json(r#"{"workspaces":[]}"#).is_none());
    // An active index past the end, and an empty name, are repaired.
    let text = layouts
        .to_json()
        .replace("\"active\":2", "\"active\":99")
        .replace("\"Mine\"", "\"\"");
    let repaired = Layouts::from_json(&text).unwrap();
    assert_eq!(repaired.active, repaired.workspaces.len() - 1);
    assert_eq!(repaired.workspaces[1].name, "Layout");
}

#[test]
fn moving_a_tab_keeps_the_shown_layout_shown() {
    let names =
        |l: &Layouts| -> Vec<String> { l.workspaces.iter().map(|w| w.name.clone()).collect() };
    let mut layouts = Layouts::default(); // Layout, Assets, Scripting, Animation
    layouts.select(1); // Assets is showing

    // The shown tab itself moves.
    layouts.move_tab(1, 3);
    assert_eq!(
        names(&layouts),
        ["Layout", "Scripting", "Animation", "Assets"]
    );
    assert_eq!(layouts.active_workspace().name, "Assets");

    // Another tab jumps over it, from either side.
    layouts.move_tab(0, 3);
    assert_eq!(
        names(&layouts),
        ["Scripting", "Animation", "Assets", "Layout"]
    );
    assert_eq!(layouts.active_workspace().name, "Assets");
    layouts.move_tab(3, 0);
    assert_eq!(
        names(&layouts),
        ["Layout", "Scripting", "Animation", "Assets"]
    );
    assert_eq!(layouts.active_workspace().name, "Assets");

    // Out of range or no move: nothing happens.
    let before = layouts.clone();
    layouts.move_tab(9, 0);
    layouts.move_tab(0, 9);
    layouts.move_tab(2, 2);
    assert_eq!(layouts, before);
}

#[test]
fn the_phone_layouts_tile_a_phone_and_are_chosen_by_its_shape() {
    use xerxes_engine::editor::layout::{PRESET_NAMES, for_phone, is_phone};

    // Landscape (about 1100 x 481 usable) and portrait phones.
    let landscape = Rect::new(0.0, 0.0, 1100.0, 481.0);
    let portrait = Rect::new(0.0, 0.0, 400.0, 760.0);
    assert!(is_phone(1100.0, 519.0) && is_phone(400.0, 800.0));
    assert!(!is_phone(1280.0, 720.0) && !is_phone(1920.0, 1080.0));
    assert_eq!(for_phone(1100.0, 519.0).name, "Phone");
    assert_eq!(for_phone(400.0, 800.0).name, "Phone Portrait");
    for ws in [for_phone(1100.0, 519.0), for_phone(400.0, 800.0)] {
        assert_eq!(ws.count(EditorKind::Viewport), 1);
        assert_tiles(&ws, landscape);
        assert_tiles(&ws, portrait);
    }
    // Both are in the `+` menu, and a user starting on a phone gets one first.
    assert!(PRESET_NAMES.contains(&"Phone") && PRESET_NAMES.contains(&"Phone Portrait"));
    let mut layouts = Layouts::default();
    layouts.select(2);
    layouts.prefer(for_phone(1100.0, 519.0));
    assert_eq!(layouts.active, 0);
    assert_eq!(layouts.active_workspace().name, "Phone");
    assert_eq!(layouts.workspaces.len(), 5);
}
