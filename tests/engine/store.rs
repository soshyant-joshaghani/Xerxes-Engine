//! The native project store as a standalone editor sees it (no repo, no backend): a projects
//! folder, New Project from the template games packed into the editor, then opening,
//! reading and writing the new project.

use futures_executor::block_on;
use xerxes_engine::editor::project::browse;
use xerxes_engine::editor::project::codec::{Dimension, parse_scene};
use xerxes_engine::editor::project::store;

#[test]
fn a_standalone_editor_creates_and_opens_projects() {
    let dir = std::env::temp_dir().join(format!("xerxes-store-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    // Set before the first store call: the store decides its workspace once.
    unsafe { std::env::set_var("XERXES_PROJECTS", &dir) };

    let listing = block_on(store::list()).unwrap();
    assert!(listing.games.is_empty() && listing.templates.is_empty());
    assert!(listing.new_projects_in.contains("xerxes-store-"));

    let project = block_on(store::create("space-rush", "starter")).unwrap();
    assert_eq!(
        (
            project.name.as_str(),
            project.kind.as_str(),
            project.title.as_str()
        ),
        ("space-rush", "game", "space-rush")
    );
    assert!(
        block_on(store::create("space-rush", "starter"))
            .unwrap_err()
            .contains("exists")
    );
    assert!(block_on(store::create("engine", "starter")).is_err());

    // A project outside the repo builds against the engine's Git repository.
    let cargo = std::fs::read_to_string(dir.join("space-rush/Cargo.toml")).unwrap();
    assert!(
        cargo.contains("git = \"https://github.com/soshyant-joshaghani/Xerxes-Engine.git\""),
        "{cargo}"
    );
    assert!(!cargo.contains("path = \"../"), "{cargo}");
    let ignore = std::fs::read_to_string(dir.join("space-rush/.gitignore")).unwrap();
    assert!(ignore.contains("/target/") && ignore.contains("/public/"));

    let listing = block_on(store::list()).unwrap();
    assert_eq!(
        listing
            .games
            .iter()
            .map(|g| g.name.as_str())
            .collect::<Vec<_>>(),
        ["space-rush"]
    );

    // Open: files, the project settings, a scene; then New Scene (2D) and read it back.
    let files = block_on(store::files(&project)).unwrap();
    assert!(files.iter().any(|f| f.path == "space-rush.project.rs"));
    assert!(files.iter().any(|f| f.path == "scenes/main.scene.rs"));
    // The starter is the project settings and one scene; the browser shows each asset once.
    let root: Vec<_> = browse::entries(&files, "")
        .into_iter()
        .map(|e| e.name)
        .collect();
    assert_eq!(root, ["scenes", "space-rush"]);
    let settings = block_on(store::read_text(&project, "space-rush.project.rs")).unwrap();
    assert!(settings.contains("title: \"space-rush\""));
    let source = store::new_scene_source("arena", Dimension::D2);
    block_on(store::write(
        &project,
        "scenes/arena.scene.rs",
        source.into_bytes(),
    ))
    .unwrap();
    let back = block_on(store::read_text(&project, "scenes/arena.scene.rs")).unwrap();
    assert_eq!(parse_scene(&back).unwrap().dimension, Dimension::D2);
    assert!(block_on(store::read(&project, "../Cargo.toml")).is_err());

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn template_games_are_read_only() {
    use xerxes_engine::editor::project::store::{ProjectEntry, ensure_writable};
    let entry = |kind: &str| ProjectEntry {
        id: "x".into(),
        name: "cyrus".into(),
        title: "Cyrus".into(),
        description: None,
        kind: kind.into(),
        location: "games/__templates__/cyrus".into(),
    };
    assert!(ensure_writable(&entry("game")).is_ok());
    let err = ensure_writable(&entry("template")).unwrap_err();
    assert!(err.contains("read-only") && err.contains("create a project from it"));
}

#[test]
fn a_png_header_gives_the_image_size() {
    use xerxes_engine::editor::project::browse::png_size;
    // Signature, then the IHDR chunk: length, "IHDR", width, height.
    let mut png = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    png.extend([0, 0, 0, 13]);
    png.extend(b"IHDR");
    png.extend(640u32.to_be_bytes());
    png.extend(480u32.to_be_bytes());
    assert_eq!(png_size(&png), Some((640, 480)));
    // Not a PNG, or cut short.
    assert_eq!(png_size(b"GIF89a....................."), None);
    assert_eq!(png_size(&png[..20]), None);
    assert_eq!(png_size(&[]), None);
}
