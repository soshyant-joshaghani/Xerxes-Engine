//! Darius and Cyrus as the editor sees them: their scenes must parse and the Scene view must get
//! every element, the canvases and their UI included. Guards against a game scene turning back into
//! code the editor cannot read (a helper function call in a scene fails the parse).

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use xerxes_engine::editor::bridge::ui_view::{ViewCanvasMode, ViewWidget};
use xerxes_engine::editor::project::codec::parse_scene;
use xerxes_engine::editor::view::{self, SceneView};

fn scenes_dir(game: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../games/__templates__")
        .join(game)
        .join("assets/scenes")
}

fn darius_scene(name: &str) -> SceneView {
    scene_at(&scenes_dir("darius").join(name))
}

fn scene_at(path: &std::path::Path) -> SceneView {
    let name = path.display().to_string();
    let src = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let doc = parse_scene(&src).unwrap_or_else(|e| panic!("{name}: {e}"));
    view::build(&doc, &HashMap::new())
}

#[test]
fn the_menu_is_the_taxonomy_grid() {
    let view = darius_scene("menu.scene.rs");
    let menu = view.objects.iter().find(|o| o.name == "Menu").unwrap();
    let ui = menu.ui.as_ref().expect("the menu is UI");
    assert_eq!(ui.canvas.as_ref().unwrap().mode, ViewCanvasMode::Overlay);
    assert!(
        ui.background.is_some(),
        "BACKDROP resolves from the file's const"
    );

    let title = view.objects.iter().find(|o| o.name == "Title").unwrap();
    let title = title.ui.as_ref().unwrap();
    assert_eq!(title.text.as_deref(), Some("Darius"));
    assert_eq!(title.text_size, Some(34.0));
    assert!(title.text_color.is_some());

    // The taxonomy grid: one tile (a button) per game, 21, Solo x Role structurally empty.
    let tiles: Vec<_> = view
        .objects
        .iter()
        .filter(|o| o.name.ends_with(" Tile"))
        .collect();
    assert_eq!(tiles.len(), 21);
    assert!(tiles.iter().all(|t| t.ui.as_ref().unwrap().button));
    let daric = view
        .objects
        .iter()
        .find(|o| o.name == "Ahriman's Horde Tile")
        .unwrap();
    assert!(daric.ui.as_ref().unwrap().node.is_some());
    assert!(view.objects.iter().any(|o| o.name == "Solo Role Empty"));
}

/// Every scene of both games parses, and every game scene is a 2D (Darius) or 3D (Cyrus) scene
/// with a HUD; the empty ones have their title and a Back button.
#[test]
fn every_game_scene_of_darius_and_cyrus_reaches_the_view() {
    for (game, count) in [("darius", 21), ("cyrus", 21)] {
        let mut files = Vec::new();
        for family in ["solo", "coop", "duel", "ngon", "outside"] {
            let dir = scenes_dir(game).join("games").join(family);
            for entry in fs::read_dir(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display())) {
                files.push(entry.unwrap().path());
            }
        }
        assert_eq!(files.len(), count, "{game} has a scene per game");
        for path in files {
            let view = scene_at(&path);
            let named = |n: &str| view.objects.iter().find(|o| o.name == n);
            assert!(named("HUD").is_some(), "{}: a HUD canvas", path.display());
            if let Some(back) = named("Back Button") {
                assert!(back.ui.as_ref().unwrap().button);
                assert!(named("Title").unwrap().ui.as_ref().unwrap().text.is_some());
            }
        }
        scene_at(&scenes_dir(game).join("menu.scene.rs"));
    }
}

#[test]
fn ahrimans_horde_is_a_camera_and_two_canvases() {
    let view = darius_scene("games/solo/ahrimans_horde.scene.rs");
    let named = |n: &str| view.objects.iter().find(|o| o.name == n);

    // Three objects at the root: the camera, the Game canvas and the HUD canvas.
    let roots: Vec<&str> = view
        .objects
        .iter()
        .filter(|o| o.parent.is_none())
        .map(|o| o.name.as_str())
        .collect();
    assert_eq!(roots, ["Camera", "Game", "HUD"]);

    // The camera shows 1031 x 580 of the world on a 1280 x 720 screen (AutoMin 880 x 580 at
    // 16:9): its frame in the Scene view, to place the level and the HUD against.
    let ortho = named("Camera")
        .unwrap()
        .ortho
        .expect("an orthographic camera");
    let [w, h] = ortho.visible([1280, 720]);
    assert!(
        (w - 1031.11).abs() < 0.5 && (h - 580.0).abs() < 0.01,
        "{w} x {h}"
    );

    // The Game canvas is a world-space layer holding the level.
    let game = named("Game").unwrap();
    assert_eq!(
        game.ui.as_ref().unwrap().canvas.as_ref().unwrap().mode,
        ViewCanvasMode::World {
            size: [800, 500],
            pixels_per_unit: 1.0
        }
    );
    for name in [
        "Floor",
        "Wall North",
        "Pillar East",
        "Player",
        "Daric 1",
        "Daric 8",
    ] {
        assert_eq!(
            named(name).unwrap().parent,
            Some(game.id),
            "{name} is in Game"
        );
    }
    assert!(named("Player").unwrap().mesh.is_some());
    assert_eq!(
        view.objects
            .iter()
            .filter(|o| o.name.starts_with("Daric "))
            .count(),
        8
    );

    // The HUD canvas: an overlay, anchored elements, the hidden banner.
    let hud = named("HUD").unwrap();
    assert_eq!(
        hud.ui.as_ref().unwrap().canvas.as_ref().unwrap().mode,
        ViewCanvasMode::Overlay
    );
    // One column of three bands. The header fills the top band (its height is driven at runtime;
    // the scene's initial anchor is the open 20%).
    let header = named("Header").unwrap().ui.as_ref().unwrap().rect.unwrap();
    assert_eq!(header.anchor_min, [0.0, 0.8], "top band");
    assert_eq!(header.anchor_max, [1.0, 1.0]);
    // The banner covers the whole screen while paused or finished.
    let banner = named("Banner").unwrap().ui.as_ref().unwrap();
    assert!(banner.hidden);
    assert_eq!(banner.rect.unwrap().anchor_min, [0.0, 0.0]);
    assert_eq!(banner.rect.unwrap().anchor_max, [1.0, 1.0]);
    // The controls fill the footer band (its initial height is the open 22%).
    let footer = named("Footer").unwrap().ui.as_ref().unwrap().rect.unwrap();
    assert_eq!(footer.anchor_min, [0.0, 0.0], "bottom band");
    assert_eq!(footer.anchor_max, [1.0, 0.22]);
    // Both movement controls live in the footer: the floating stick and the four-way D-pad.
    assert!(named("Dpad").is_some());
    for dir in ["Up", "Down", "Left", "Right"] {
        assert!(
            named(&format!("{dir} Button"))
                .unwrap()
                .ui
                .as_ref()
                .unwrap()
                .button,
            "{dir} is a D-pad button"
        );
    }
    let stick = named("Move Stick").unwrap().ui.as_ref().unwrap();
    assert_eq!(
        stick.widget,
        Some(ViewWidget::Joystick {
            radius: 52.0,
            dead_zone: 0.12
        })
    );
    // It fills the left part of the footer; the switch and toggle sit clear of it on the right.
    let zone = stick.rect.unwrap();
    assert_eq!(zone.anchor_min, [0.0, 0.0]);
    assert_eq!(zone.anchor_max, [0.42, 1.0]);
    assert!(named("Score").unwrap().ui.as_ref().unwrap().text.is_some());
    assert!(named("Pause Button").unwrap().ui.as_ref().unwrap().button);
}

/// Qanat's room is its scene: 63 cells (9 x 7) in the Game canvas, each drawn as a sprite, the
/// cursor beside them, and its HUD with the Back button.
#[test]
fn qanat_is_a_room_of_cells_in_the_game_canvas() {
    let view = darius_scene("games/solo/qanat.scene.rs");
    let named = |n: &str| view.objects.iter().find(|o| o.name == n);
    let roots: Vec<&str> = view
        .objects
        .iter()
        .filter(|o| o.parent.is_none())
        .map(|o| o.name.as_str())
        .collect();
    assert_eq!(roots, ["Camera", "Game", "HUD"]);
    let game = named("Game").unwrap().id;
    let cells: Vec<_> = view
        .objects
        .iter()
        .filter(|o| o.name.starts_with("Cell "))
        .collect();
    assert_eq!(cells.len(), 63);
    assert!(
        cells
            .iter()
            .all(|c| c.parent == Some(game) && c.mesh.is_some())
    );
    assert_eq!(named("Cursor").unwrap().parent, Some(game));
    assert!(named("Back Button").unwrap().ui.as_ref().unwrap().button);
}

#[test]
fn widgets_in_a_scene_reach_the_view() {
    let src = r#"
use xerxes_engine::prelude::*;
pub fn scene() -> SceneAsset {
    SceneAsset {
        dimension: Dimension::D2,
        objects: vec![
            Object::new("Settings")
                .with(Canvas::world(400, 300, 100.0))
                .child(Object::new("Quality").with(Dropdown::new(["Low", "High"]).selected(1)))
                .child(Object::new("Sound").with(Toggle::new(true)))
                .child(Object::new("Volume").with(Slider::new(0.0, 1.0, 0.5))),
        ],
    }
}
"#;
    let view = view::build(&parse_scene(src).unwrap(), &HashMap::new());
    let widget = |n: &str| {
        view.objects
            .iter()
            .find(|o| o.name == n)
            .and_then(|o| o.ui.as_ref())
            .and_then(|u| u.widget.clone())
    };
    assert_eq!(
        widget("Quality"),
        Some(ViewWidget::Dropdown {
            options: vec!["Low".into(), "High".into()],
            selected: 1
        })
    );
    assert_eq!(widget("Sound"), Some(ViewWidget::Toggle(true)));
    assert_eq!(
        widget("Volume"),
        Some(ViewWidget::Slider {
            min: 0.0,
            max: 1.0,
            value: 0.5
        })
    );
    let canvas = view.objects[0]
        .ui
        .as_ref()
        .unwrap()
        .canvas
        .as_ref()
        .unwrap();
    assert_eq!(
        canvas.mode,
        ViewCanvasMode::World {
            size: [400, 300],
            pixels_per_unit: 100.0
        }
    );
}
