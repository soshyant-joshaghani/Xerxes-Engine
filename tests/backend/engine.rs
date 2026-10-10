mod common;

use std::fs;
use std::path::PathBuf;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::TestApp;
use xerxes_backend::core::config::Settings;
use xerxes_backend::modules::api::engine::play_target;

/// A throwaway Xerxes repo: the engine, two games (unsorted on disk), one template,
/// and a published web build for `zeta`.
fn fake_repo(tag: &str) -> PathBuf {
    let root =
        std::env::temp_dir().join(format!("xerxes-engine-test-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let krate = |dir: PathBuf, title: &str| {
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("Cargo.toml"), "[package]\n").unwrap();
        fs::write(
            dir.join("Dioxus.toml"),
            format!("[application]\nname = \"x\"\n\n[web.app]\ntitle = \"{title}\"\n"),
        )
        .unwrap();
    };
    krate(root.join("engine"), "Xerxes Engine");
    krate(root.join("games").join("zeta"), "Zeta Game");
    fs::write(
        root.join("games").join("zeta").join("README.md"),
        "# Zeta\n\nA [space](x.md) shooter with `ships`. It has more.\n",
    )
    .unwrap();
    krate(root.join("games").join("alpha"), "Alpha Game");
    krate(
        root.join("games").join("__templates__").join("cyrus"),
        "Cyrus",
    );
    fs::create_dir_all(root.join("games").join("notes")).unwrap(); // no Cargo.toml: not a game
    let site = root.join("dist").join("zeta").join("web").join("public");
    fs::create_dir_all(&site).unwrap();
    fs::write(site.join("index.html"), "<title>zeta</title>").unwrap();
    root
}

fn app_for(root: &PathBuf, environment: &str) -> TestApp {
    let mut settings = Settings::for_tests();
    settings.xerxes_root = root.to_string_lossy().to_string();
    settings.environment = environment.to_string();
    TestApp::with_settings(settings)
}

#[tokio::test]
async fn catalog_lists_engine_then_games_alphabetically_and_templates() {
    let root = fake_repo("catalog");
    let reply = app_for(&root, "local")
        .get("/api/v1/engine/games", None)
        .await;
    assert_eq!(reply.status, StatusCode::OK);

    let body = &reply.body;
    // The engine app is listed on its own, outside the game and template indexes.
    assert!(body["engine"]["index"].is_null());
    assert_eq!(body["engine"]["title"], "Xerxes Engine");
    let games: Vec<(&str, u64)> = body["games"]
        .as_array()
        .unwrap()
        .iter()
        .map(|g| (g["name"].as_str().unwrap(), g["index"].as_u64().unwrap()))
        .collect();
    assert_eq!(games, vec![("alpha", 1), ("zeta", 2)]);
    assert_eq!(body["games"][1]["title"], "Zeta Game");
    assert_eq!(body["games"][1]["published"], true);
    assert_eq!(
        body["games"][1]["description"],
        "A space shooter with ships."
    );
    assert!(body["games"][0]["description"].is_null());
    assert_eq!(body["games"][1]["play_url"], "http://zeta.play.localhost/");
    assert_eq!(body["games"][0]["published"], false);
    assert!(body["games"][0]["play_url"].is_null());
    assert_eq!(body["templates"][0]["name"], "cyrus");
    assert_eq!(body["templates"][0]["kind"], "template");
    assert_eq!(body["templates"][0]["index"], 1);
}

#[tokio::test]
async fn published_build_is_served_on_its_play_host() {
    let root = fake_repo("play");
    let app = app_for(&root, "local");
    let request = |host: &str, path: &str| {
        Request::builder()
            .uri(path)
            .header("host", host)
            .body(Body::empty())
            .unwrap()
    };

    let page = app.dispatch(request("zeta.play.localhost", "/")).await;
    assert_eq!(page.status, StatusCode::OK);
    assert!(page.text.contains("<title>zeta</title>"));

    let missing = app.dispatch(request("alpha.play.localhost", "/")).await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
    assert!(missing.text.contains("game publish alpha"));

    let engine = app.dispatch(request("engine.play.localhost", "/")).await;
    assert_eq!(engine.status, StatusCode::NOT_FOUND);
    assert!(engine.text.contains("engine publish web"));

    // The API stays reachable on a play host (a published engine build reads its catalog there).
    let api_on_play = app
        .dispatch(request("zeta.play.localhost", "/api/v1/utils/health-check"))
        .await;
    assert_eq!(api_on_play.status, StatusCode::OK);

    // Other hosts still reach the API.
    let api = app
        .dispatch(request("api.localhost", "/api/v1/utils/health-check"))
        .await;
    assert_eq!(api.status, StatusCode::OK);
}

#[tokio::test]
async fn engine_services_are_local_only() {
    let root = fake_repo("prod");
    let mut settings = Settings::for_tests();
    settings.xerxes_root = root.to_string_lossy().to_string();
    settings.environment = "production".to_string();
    settings.secret_key = "a-real-secret".to_string();
    settings.first_superuser_password = "a-real-password".to_string();
    let app = TestApp::with_settings(settings);
    assert_eq!(
        app.get("/api/v1/engine/games", None).await.status,
        StatusCode::NOT_FOUND
    );
}

#[test]
fn play_host_names_are_strict() {
    assert_eq!(play_target("darius.play.localhost"), Some("darius"));
    assert_eq!(
        play_target("darius.play.localhost:80"),
        Some("darius")
    );
    assert_eq!(play_target("api.localhost"), None);
    assert_eq!(play_target(".play.localhost"), None);
    assert_eq!(play_target("Bad_Name.play.localhost"), None);
    assert_eq!(play_target("a.b.play.localhost"), None);
}
