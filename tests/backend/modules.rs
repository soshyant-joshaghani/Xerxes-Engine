//! The backend's two series of modules (API and multiplayer) stay separable: neither imports
//! the other, and `SERVICES` picks which one a process serves.

mod common;

use std::fs;
use std::path::{Path, PathBuf};

use axum::http::StatusCode;
use common::TestApp;
use xerxes_backend::core::config::{Services, Settings};

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// Source files of one series that mention the other one.
fn imports(series: &str, other: &str) -> Vec<String> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/modules")
        .join(series);
    let mut files = Vec::new();
    rust_files(&dir, &mut files);
    let needles = [format!("modules::{other}"), format!("super::{other}")];
    files
        .into_iter()
        .filter(|f| {
            let text = fs::read_to_string(f).unwrap();
            // Doc comments may name the other series; code may not import it.
            text.lines()
                .filter(|l| !l.trim_start().starts_with("//"))
                .any(|l| needles.iter().any(|n| l.contains(n)))
        })
        .map(|f| f.display().to_string())
        .collect()
}

#[test]
fn neither_series_imports_the_other() {
    assert_eq!(imports("multiplayer", "api"), Vec::<String>::new());
    assert_eq!(imports("api", "multiplayer"), Vec::<String>::new());
}

fn app_serving(services: &str) -> TestApp {
    let mut settings = Settings::for_tests();
    settings.services = services.parse().unwrap();
    TestApp::with_settings(settings)
}

#[tokio::test]
async fn both_series_are_served_by_default() {
    assert_eq!(Settings::for_tests().services, Services::All);
    let app = TestApp::new();
    assert_eq!(
        app.get("/api/v1/utils/health-check", None).await.status,
        StatusCode::OK
    );
    let reply = app.get("/multiplayer/health-check", None).await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.text.trim(), "true");
}

#[tokio::test]
async fn an_api_only_process_does_not_serve_multiplayer() {
    let app = app_serving("api");
    assert_eq!(
        app.get("/api/v1/utils/health-check", None).await.status,
        StatusCode::OK
    );
    assert_eq!(
        app.get("/multiplayer/health-check", None).await.status,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn a_multiplayer_only_process_does_not_serve_the_api() {
    let app = app_serving("multiplayer");
    assert_eq!(
        app.get("/multiplayer/health-check", None).await.status,
        StatusCode::OK
    );
    assert_eq!(
        app.get("/api/v1/utils/health-check", None).await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(app.get("/docs", None).await.status, StatusCode::NOT_FOUND);
}

#[test]
fn services_parse_and_reject_unknown_values() {
    assert_eq!("All".parse(), Ok(Services::All));
    assert!("both".parse::<Services>().is_err());
    let mut map = std::collections::HashMap::new();
    map.insert("SERVICES".to_string(), "mp".to_string());
    assert!(Settings::from_map(&map).is_err());
}
