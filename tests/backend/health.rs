mod common;

use std::collections::HashMap;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::TestApp;
use xerxes_backend::core::config::{parse_env_file, Settings};

#[tokio::test]
async fn health_check_returns_true() {
    let app = TestApp::new();
    let reply = app.get("/api/v1/utils/health-check", None).await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.text.trim(), "true");
}

#[tokio::test]
async fn unknown_route_is_json_404() {
    let app = TestApp::new();
    let reply = app.get("/api/v1/nope", None).await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
    assert_eq!(reply.detail(), "Not Found");
}

#[tokio::test]
async fn docs_and_openapi_are_served() {
    let app = TestApp::new();

    let docs = app.get("/docs", None).await;
    assert_eq!(docs.status, StatusCode::OK);
    assert!(docs.text.contains("swagger"));

    let sdoc = app.get("/sdoc", None).await;
    assert_eq!(sdoc.status, StatusCode::OK);
    assert!(sdoc.text.contains("elysiajs"));
    assert!(sdoc.text.contains("OAuth2PasswordBearer"));

    let spec = app.get("/api/v1/openapi.json", None).await;
    assert_eq!(spec.status, StatusCode::OK);
    assert_eq!(spec.body["openapi"], "3.1.0");
    assert!(spec.body["paths"]["/base/login/access-token"]["post"].is_object());
    assert!(spec.body["paths"]["/base/users/{id}/admin"]["delete"].is_object());
    assert!(spec.body["paths"].get("/sample/notes").is_none());
    // Generated from the handlers (core::api): summary, id and tag from the name and module,
    // the bearer requirement from the auth extractor, the body schema from its type.
    let read_users = &spec.body["paths"]["/base/users/admin"]["get"];
    assert_eq!(read_users["summary"], "Read Users");
    assert_eq!(read_users["operationId"], "read_users");
    assert_eq!(read_users["tags"][0], "[BASE] Users");
    assert!(read_users["security"][0]
        .get("OAuth2PasswordBearer")
        .is_some());
    assert!(spec.body["paths"]["/utils/health-check"]["get"]["security"].is_null());
    let create = &spec.body["paths"]["/base/users/admin"]["post"];
    assert!(create["requestBody"]["content"]["application/json"]["schema"].is_object());
    assert!(
        spec.body["paths"]["/base/login/access-token"]["post"]["requestBody"]["content"]
            ["application/x-www-form-urlencoded"]
            .is_object()
    );
    // The engine services (local only) are documented, catch-all paths as plain params.
    let paths = &spec.body["paths"];
    assert!(paths["/engine/games"]["get"].is_object());
    assert!(paths["/engine/projects"]["post"].is_object());
    assert!(paths["/engine/jobs/{id}"]["delete"].is_object());
    let file = &paths["/engine/projects/{name}/files/{path}"];
    assert!(file["put"]["requestBody"]["content"]["application/octet-stream"].is_object());
    let params: Vec<&str> = file["get"]["parameters"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["name"].as_str().unwrap())
        .collect();
    assert_eq!(params, ["name", "path"]);
    assert_eq!(
        spec.body["components"]["securitySchemes"]["OAuth2PasswordBearer"]["flows"]["password"]
            ["tokenUrl"],
        "/api/v1/base/login/access-token"
    );
}

#[tokio::test]
async fn cors_allows_frontend_host_with_credentials() {
    let app = TestApp::new();
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/utils/health-check")
        .header("origin", "http://dashboard.localhost")
        .body(Body::empty())
        .unwrap();
    let reply = app.dispatch(request).await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(
        reply.headers.get("access-control-allow-origin").unwrap(),
        "http://dashboard.localhost"
    );
    assert_eq!(
        reply
            .headers
            .get("access-control-allow-credentials")
            .unwrap(),
        "true"
    );
}

#[tokio::test]
async fn cors_ignores_unknown_origin() {
    let app = TestApp::new();
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/utils/health-check")
        .header("origin", "http://evil.example")
        .body(Body::empty())
        .unwrap();
    let reply = app.dispatch(request).await;
    assert!(reply.headers.get("access-control-allow-origin").is_none());
}

#[test]
fn production_refuses_placeholder_secrets() {
    let mut map = HashMap::new();
    map.insert("ENVIRONMENT".to_string(), "production".to_string());
    let settings = Settings::from_map(&map).unwrap();
    assert!(settings.validate().is_err());

    map.insert("SECRET_KEY".to_string(), "a-real-secret".to_string());
    let settings = Settings::from_map(&map).unwrap();
    assert!(
        settings.validate().is_err(),
        "superuser password is still changethis"
    );

    map.insert(
        "FIRST_SUPERUSER_PASSWORD".to_string(),
        "strong-password".to_string(),
    );
    let settings = Settings::from_map(&map).unwrap();
    assert!(settings.validate().is_ok());

    map.insert("ENVIRONMENT".to_string(), "local".to_string());
    map.insert("SECRET_KEY".to_string(), "changethis".to_string());
    assert!(Settings::from_map(&map).unwrap().validate().is_ok());
}

#[test]
fn settings_defaults_and_cors_list() {
    let mut map = HashMap::new();
    map.insert(
        "BACKEND_CORS_ORIGINS".to_string(),
        "http://a.test, http://b.test/".to_string(),
    );
    map.insert("REDIS_PASSWORD".to_string(), "p@ss".to_string());
    let settings = Settings::from_map(&map).unwrap();
    assert_eq!(settings.api_v1_str, "/api/v1");
    assert_eq!(settings.access_token_expire_minutes, 11520);
    assert_eq!(settings.app_port, 8000);
    assert_eq!(
        settings.all_cors_origins(),
        vec![
            "http://a.test".to_string(),
            "http://b.test".to_string(),
            "http://dashboard.localhost".to_string()
        ]
    );
    assert_eq!(settings.redis_url(), "redis://:p%40ss@localhost:6379/0");
}

#[test]
fn env_file_parser_handles_quotes_and_comments() {
    let parsed = parse_env_file(
        "# comment\n\nA=1\nexport B=\"two words\"\nC='x' \nD=value # trailing\nbroken line\n",
    );
    let map: HashMap<String, String> = parsed.into_iter().collect();
    assert_eq!(map["A"], "1");
    assert_eq!(map["B"], "two words");
    assert_eq!(map["C"], "x");
    assert_eq!(map["D"], "value");
    assert_eq!(map.len(), 4);
}
