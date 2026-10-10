//! Reading and changing the simple fields of a project settings file without disturbing the rest.

use xerxes_engine::editor::project::settings_text::{
    BackendOpts, WindowOpts, make_backend_explicit, make_window_explicit, parse, set_backend,
    set_title, set_window,
};

fn starter() -> String {
    std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("starter/assets/starter.project.rs"),
    )
    .unwrap()
}

#[test]
fn the_starter_settings_are_read() {
    let parsed = parse(&starter());
    assert_eq!(parsed.title.as_deref(), Some("Starter"));
    assert_eq!(
        parsed.window,
        Some(WindowOpts {
            width: 1280,
            height: 720,
            resizable: true
        })
    );
    assert_eq!(
        parsed.backend,
        Some(BackendOpts {
            dev: "http://127.0.0.1:8000".into(),
            publish: String::new()
        })
    );
    assert_eq!(parsed.levels.len(), 1);
    assert_eq!(parsed.levels[0].name, "Main");
    assert_eq!(parsed.levels[0].mode, "GameMode::sandbox()");
    let names: Vec<_> = parsed.actions.iter().map(|a| a.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "move_left",
            "move_right",
            "move_up",
            "move_down",
            "respawn",
            "pause",
            "exit_to_menu"
        ]
    );
    assert_eq!(parsed.actions[0].bindings, "KeyA, ArrowLeft");
    assert_eq!(parsed.actions[5].bindings, "Escape, KeyP");
    assert!(parsed.preload.is_empty());
}

#[test]
fn a_change_replaces_only_its_value() {
    let src = starter();
    let changed = set_title(&src, "My Game").unwrap();
    assert_eq!(parse(&changed).title.as_deref(), Some("My Game"));
    // The main menu's title (also "Starter") and everything else are untouched.
    assert_eq!(
        changed.matches("Starter").count(),
        src.matches("Starter").count() - 1
    );
    assert_eq!(
        changed.len() as i64 - src.len() as i64,
        "My Game".len() as i64 - "Starter".len() as i64
    );

    let window = WindowOpts {
        width: 800,
        height: 600,
        resizable: false,
    };
    let changed = set_window(&src, &window).unwrap();
    let parsed = parse(&changed);
    assert_eq!(parsed.window, Some(window));
    assert_eq!(parsed.title, parse(&src).title);
    assert_eq!(parsed.actions, parse(&src).actions);

    let backend = BackendOpts {
        dev: "http://localhost:9000".into(),
        publish: "https://play.example.com".into(),
    };
    let changed = set_backend(&src, &backend).unwrap();
    assert_eq!(parse(&changed).backend, Some(backend));
    // Comments survive.
    assert!(changed.contains("// The game's backend"));
}

#[test]
fn values_that_cannot_be_written_between_quotes_are_refused() {
    let src = starter();
    assert!(set_title(&src, "say \"hi\"").is_none());
    assert!(set_title(&src, "back\\slash").is_none());
    let bad = BackendOpts {
        dev: "a\"b".into(),
        publish: String::new(),
    };
    assert!(set_backend(&src, &bad).is_none());
}

#[test]
fn defaults_are_reported_missing_and_can_be_made_explicit() {
    let src = starter()
        .replace(
            "WindowSettings {\n            width: 1280,\n            height: 720,\n            resizable: true,\n        }",
            "WindowSettings::default()",
        )
        .replace(
            "BackendSettings {\n            dev: \"http://127.0.0.1:8000\",\n            publish: \"\",\n        }",
            "BackendSettings::default()",
        );
    let parsed = parse(&src);
    assert!(parsed.window.is_none() && parsed.backend.is_none());
    assert!(
        set_window(
            &src,
            &WindowOpts {
                width: 1,
                height: 1,
                resizable: true
            }
        )
        .is_none()
    );

    let explicit = make_backend_explicit(&make_window_explicit(&src).unwrap()).unwrap();
    let parsed = parse(&explicit);
    assert_eq!(
        parsed.window.map(|w| (w.width, w.height)),
        Some((1280, 720))
    );
    assert_eq!(
        parsed.backend.map(|b| b.dev).as_deref(),
        Some("http://127.0.0.1:8000")
    );
}

#[test]
fn a_file_that_is_not_settings_gives_nothing_and_never_panics() {
    for src in [
        "",
        "fn main() {}",
        "title:",
        "title: \"unterminated",
        "LevelDef {",
        ".action(",
        "WindowSettings { width: }",
    ] {
        let parsed = parse(src);
        assert!(parsed.levels.len() <= 1 && parsed.actions.len() <= 1);
        let _ = set_title(src, "x");
        let _ = set_window(
            src,
            &WindowOpts {
                width: 1,
                height: 1,
                resizable: true,
            },
        );
    }
}
