//! Editor services: project files confined to `assets/`, metas that move and delete with
//! their files, add-ons added with their needs, job arguments, local-only gating.

mod common;

use std::fs;
use std::path::PathBuf;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::TestApp;
use serde_json::json;
use xerxes_backend::core::config::Settings;
use xerxes_backend::modules::api::editor::{
    create_project, ctrl_python, ctrl_script, job_args, safe_relative, JobRequest, NewProject,
};

/// A throwaway repo: game `alpha` with a scene, an image and its meta; template game
/// `cyrus`. (The engine's add-ons are compiled into the backend.)
fn fake_repo(tag: &str) -> PathBuf {
    let root =
        std::env::temp_dir().join(format!("xerxes-editor-test-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let put = |path: PathBuf, body: &str| {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    };
    let alpha = root.join("games/alpha");
    put(alpha.join("Cargo.toml"), "[package]\n");
    put(
        alpha.join("assets/scenes/main.scene.rs"),
        "pub fn scene() {}\n",
    );
    put(alpha.join("assets/art/crate.png"), "PNG");
    put(
        alpha.join("assets/art/crate.image.rs"),
        "AssetMeta::image(ImageMeta::default()).bundle(\"core\")",
    );
    put(
        root.join("games/__templates__/cyrus/Cargo.toml"),
        "[package]\n",
    );
    root
}

fn app_for(root: &PathBuf, environment: &str) -> TestApp {
    let mut settings = Settings::for_tests();
    settings.xerxes_root = root.to_string_lossy().to_string();
    settings.environment = environment.to_string();
    TestApp::with_settings(settings)
}

async fn put_bytes(app: &TestApp, uri: &str, body: &'static [u8]) -> StatusCode {
    let request = Request::builder()
        .method("PUT")
        .uri(uri)
        .body(Body::from(body))
        .unwrap();
    app.dispatch(request).await.status
}

#[tokio::test]
async fn lists_reads_and_writes_project_files() {
    let root = fake_repo("files");
    let app = app_for(&root, "local");

    let reply = app.get("/api/v1/engine/projects/alpha/files", None).await;
    assert_eq!(reply.status, StatusCode::OK);
    let paths: Vec<&str> = reply
        .body
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["path"].as_str().unwrap())
        .collect();
    assert_eq!(
        paths,
        [
            "art",
            "art/crate.image.rs",
            "art/crate.png",
            "scenes",
            "scenes/main.scene.rs"
        ]
    );

    let status = put_bytes(
        &app,
        "/api/v1/engine/projects/alpha/files/prefabs/coin.prefab.rs",
        b"pub fn prefab() {}\n",
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let reply = app
        .get(
            "/api/v1/engine/projects/alpha/files/prefabs/coin.prefab.rs",
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.text, "pub fn prefab() {}\n");

    // Template games are projects too.
    let reply = app
        .get("/api/v1/engine/projects/cyrus/files", None)
        .await;
    assert_eq!(reply.status, StatusCode::OK);
    let reply = app.get("/api/v1/engine/projects/nope/files", None).await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn paths_stay_inside_assets() {
    for bad in ["../Cargo.toml", "a/../../x", "C:/x", "a\\b", "./a", ""] {
        assert!(safe_relative(bad).is_err(), "accepted {bad:?}");
    }
    assert_eq!(safe_relative("/art/a.png/").unwrap(), "art/a.png");

    let root = fake_repo("escape");
    let app = app_for(&root, "local");
    let reply = app
        .get("/api/v1/engine/projects/alpha/files/..%2FCargo.toml", None)
        .await;
    assert_ne!(reply.status, StatusCode::OK);
    let reply = app
        .post(
            "/api/v1/engine/projects/alpha/move",
            None,
            json!({"from": "art/crate.png", "to": "../stolen.png"}),
        )
        .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert!(root.join("games/alpha/assets/art/crate.png").is_file());
}

#[tokio::test]
async fn template_games_are_read_only() {
    let root = fake_repo("readonly");
    let app = app_for(&root, "local");
    let base = "/api/v1/engine/projects/cyrus";

    // They can be looked at...
    assert_eq!(
        app.get(&format!("{base}/files"), None).await.status,
        StatusCode::OK
    );
    // ...never changed: write, delete, move and copying a template in are all refused.
    assert_eq!(
        put_bytes(&app, &format!("{base}/files/scenes/x.scene.rs"), b"x").await,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        app.delete(&format!("{base}/files/scenes/x.scene.rs"), None)
            .await
            .status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        app.post(
            &format!("{base}/move"),
            None,
            json!({"from": "a.png", "to": "b.png"})
        )
        .await
        .status,
        StatusCode::FORBIDDEN
    );
    let reply = app
        .post(
            &format!("{base}/addons"),
            None,
            json!({"path": "hud/hud.rs"}),
        )
        .await;
    assert_eq!(reply.status, StatusCode::FORBIDDEN);
    assert!(reply.detail().contains("read-only"));
    // A game of the user's is still writable.
    assert_eq!(
        put_bytes(
            &app,
            "/api/v1/engine/projects/alpha/files/scenes/x.scene.rs",
            b"x"
        )
        .await,
        StatusCode::NO_CONTENT
    );
}

#[tokio::test]
async fn moving_or_deleting_an_external_file_takes_its_meta() {
    let root = fake_repo("meta");
    let app = app_for(&root, "local");
    let assets = root.join("games/alpha/assets");

    let reply = app
        .post(
            "/api/v1/engine/projects/alpha/move",
            None,
            json!({"from": "art/crate.png", "to": "props/box.png"}),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT);
    assert!(assets.join("props/box.png").is_file());
    assert!(
        assets.join("props/box.image.rs").is_file(),
        "the meta moved with its file"
    );
    assert!(!assets.join("art/crate.image.rs").exists());

    // No overwrite: a move onto an existing file is refused and changes nothing.
    fs::write(assets.join("scenes/taken.png"), "x").unwrap();
    let reply = app
        .post(
            "/api/v1/engine/projects/alpha/move",
            None,
            json!({"from": "props/box.png", "to": "scenes/taken.png"}),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CONFLICT);
    assert!(assets.join("props/box.png").is_file());

    // The meta follows the file's type: a mesh next to it is not touched.
    fs::write(assets.join("props/box.mesh.rs"), "AssetMeta::mesh()").unwrap();

    let reply = app
        .delete("/api/v1/engine/projects/alpha/files/props/box.png", None)
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT);
    assert!(
        !assets.join("props/box.image.rs").exists(),
        "the meta was deleted with its file"
    );
    assert!(assets.join("props/box.mesh.rs").is_file());
}

#[tokio::test]
async fn an_addon_is_added_with_what_it_needs_and_never_overwrites() {
    let root = fake_repo("addons");
    let app = app_for(&root, "local");

    let reply = app.get("/api/v1/engine/addons", None).await;
    let hud = reply
        .body
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["path"] == "hud/hud.rs")
        .unwrap()
        .clone();
    assert!(hud["summary"]
        .as_str()
        .unwrap()
        .starts_with("HUD (bevy_ui)"));
    assert_eq!(hud["needs"], json!(["game/pause.rs"]));

    let reply = app
        .post(
            "/api/v1/engine/projects/alpha/addons",
            None,
            json!({"path": "hud/hud.rs"}),
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(
        reply.body["added"],
        json!(["logic/hud.rs", "logic/pause.rs"])
    );

    // The game's copy is its own: a second add keeps it.
    let ours = root.join("games/alpha/assets/logic/hud.rs");
    fs::write(&ours, "// changed by the game\n").unwrap();
    let reply = app
        .post(
            "/api/v1/engine/projects/alpha/addons",
            None,
            json!({"path": "hud/hud.rs"}),
        )
        .await;
    assert_eq!(
        reply.body["kept"],
        json!(["logic/hud.rs", "logic/pause.rs"])
    );
    assert_eq!(
        fs::read_to_string(ours).unwrap(),
        "// changed by the game\n"
    );

    let reply = app
        .post(
            "/api/v1/engine/projects/alpha/addons",
            None,
            json!({"path": "hud/nope.rs"}),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
}

#[test]
fn jobs_run_the_same_cli_commands() {
    let root = fake_repo("jobs");
    let job = |command: &str, target: &str, platform: &str| JobRequest {
        command: command.into(),
        target: target.into(),
        platform: platform.into(),
    };
    assert_eq!(
        job_args(&root, &job("dev", "alpha", "web")).unwrap(),
        ["game", "dev", "alpha", "web"]
    );
    assert_eq!(
        job_args(&root, &job("publish", "cyrus", "android")).unwrap(),
        ["template", "publish", "cyrus", "android"]
    );
    assert_eq!(
        job_args(&root, &job("dev", "engine", "windows")).unwrap(),
        ["engine", "run", "windows"]
    );
    assert!(job_args(&root, &job("rm", "alpha", "web")).is_err());
    assert!(job_args(&root, &job("dev", "alpha", "ps5")).is_err());
    assert!(job_args(&root, &job("dev", "ghost", "web")).is_err());
}

#[tokio::test]
async fn editor_services_are_local_only() {
    let root = fake_repo("gate");
    let app = app_for(&root, "production");
    assert_eq!(
        app.get("/api/v1/engine/projects/alpha/files", None)
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        app.get("/api/v1/engine/addons", None).await.status,
        StatusCode::NOT_FOUND
    );
    let reply = app
        .post(
            "/api/v1/engine/jobs",
            None,
            json!({"command": "build", "target": "alpha", "platform": "web"}),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
}

#[test]
fn new_projects_use_the_shared_scaffold() {
    let root = fake_repo("new");
    fs::write(
        root.join("games/__templates__/cyrus/Cargo.toml"),
        "[package]\nname = \"cyrus\"\n[dependencies]\nxerxes_engine = { path = \"../../../engine\" }\n",
    )
    .unwrap();
    let new = |name: &str, template: Option<&str>| NewProject {
        name: name.into(),
        template: template.map(Into::into),
    };
    let created = create_project(&root, &new("my-game", None)).unwrap();
    assert_eq!(created.location, "games/my-game");
    let cargo = fs::read_to_string(root.join("games/my-game/Cargo.toml")).unwrap();
    assert!(cargo.contains("name = \"my-game\""));
    assert!(cargo.contains("path = \"../../engine\""));
    // The default is the engine's built-in starter: no template folder is needed for it.
    assert!(root
        .join("games/my-game/assets/my-game.project.rs")
        .is_file());
    assert!(root
        .join("games/my-game/assets/scenes/main.scene.rs")
        .is_file());
    // A finished template game works by its (clear) name.
    create_project(&root, &new("coin-rush", Some("cyrus"))).unwrap();
    assert!(create_project(&root, &new("late", Some("starter"))).is_ok());
    for bad in [
        "My Game",
        "1game",
        "",
        "engine",
        "starter",
        "cyrus",
        "alpha",
        "my-game",
    ] {
        assert!(
            create_project(&root, &new(bad, None)).is_err(),
            "accepted {bad:?}"
        );
    }
    assert!(create_project(&root, &new("x", Some("nope"))).is_err());
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn jobs_run_the_ctrl_script_by_absolute_path() {
    // The default root is `..`; the job's working directory is the root, so a relative
    // script path would point one folder too high.
    let (root, main) = ctrl_script(std::path::Path::new(".."));
    assert!(root.is_absolute() && main.is_absolute());
    assert!(main.ends_with("__ctrl__/main.py"));
    assert_eq!(main.parent().unwrap().parent().unwrap(), root);
}

#[test]
fn the_pipeline_uses_the_ctrl_tools_own_python_when_it_has_one() {
    // XERXES_PYTHON is not set in the tests (it is process-wide, so they never set it).
    if std::env::var_os("XERXES_PYTHON").is_some() {
        return;
    }
    let root = fake_repo("python");
    // No virtual environment: the system's python.
    let plain = ctrl_python(&root);
    assert!(
        plain.ends_with("python") || plain.ends_with("python3"),
        "{plain:?}"
    );
    assert!(!plain.starts_with(&root));

    // With one, that one.
    let inside = if cfg!(windows) {
        root.join("__ctrl__/.venv/Scripts/python.exe")
    } else {
        root.join("__ctrl__/.venv/bin/python")
    };
    fs::create_dir_all(inside.parent().unwrap()).unwrap();
    fs::write(&inside, "").unwrap();
    assert_eq!(ctrl_python(&root), inside);
    let _ = fs::remove_dir_all(&root);
}
