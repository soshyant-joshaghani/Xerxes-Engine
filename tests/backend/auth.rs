mod common;

use axum::http::StatusCode;
use chrono::Utc;
use common::{TestApp, PASSWORD};
use xerxes_backend::core::security;

#[test]
fn bcrypt_hashes_are_2b_and_verify() {
    let hashed = security::hash_password("secret-pass", 4).unwrap();
    assert!(hashed.starts_with("$2b$"), "got {}", hashed);
    assert!(security::verify_password("secret-pass", &hashed));
    assert!(!security::verify_password("other-pass", &hashed));
    assert!(!security::verify_password("secret-pass", "not-a-hash"));
}

#[test]
fn jwt_roundtrip_and_rejections() {
    let token = security::create_access_token("user-1", "secret", 5).unwrap();
    assert_eq!(
        security::decode_access_token(&token, "secret").unwrap(),
        "user-1"
    );
    assert!(security::decode_access_token(&token, "other-secret").is_err());
    let expired =
        security::create_token_with_exp("user-1", "secret", Utc::now().timestamp() - 3600).unwrap();
    assert!(security::decode_access_token(&expired, "secret").is_err());
}

#[tokio::test]
async fn login_success_returns_bearer_token() {
    let app = TestApp::new();
    let user = app.add_regular("alice@example.com").await;

    let reply = app.login("alice@example.com", PASSWORD).await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.body["token_type"], "bearer");
    let token = reply.body["access_token"].as_str().unwrap();
    let subject = security::decode_access_token(token, &app.settings.secret_key).unwrap();
    assert_eq!(subject, user.id.to_string());
}

#[tokio::test]
async fn login_wrong_password_is_400() {
    let app = TestApp::new();
    app.add_regular("alice@example.com").await;
    let reply = app.login("alice@example.com", "wrong-password").await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(reply.detail(), "Incorrect email or password");
}

#[tokio::test]
async fn login_unknown_email_is_400() {
    let app = TestApp::new();
    let reply = app.login("nobody@example.com", PASSWORD).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(reply.detail(), "Incorrect email or password");
}

#[tokio::test]
async fn login_inactive_user_is_400() {
    let app = TestApp::new();
    app.add_user("sleepy@example.com", PASSWORD, false, false)
        .await;
    let reply = app.login("sleepy@example.com", PASSWORD).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(reply.detail(), "Inactive user");
}

#[tokio::test]
async fn login_missing_fields_is_422_with_string_detail() {
    let app = TestApp::new();
    let request = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/base/login/access-token")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(axum::body::Body::from("username=only-this"))
        .unwrap();
    let reply = app.dispatch(request).await;
    assert_eq!(reply.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(reply.body["detail"].is_string());
}

#[tokio::test]
async fn me_returns_public_user() {
    let app = TestApp::new();
    let user = app.add_regular("alice@example.com").await;
    let token = app.token_for(&user);

    let reply = app.get("/api/v1/base/login/me", Some(&token)).await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.body["id"], user.id.to_string());
    assert_eq!(reply.body["email"], "alice@example.com");
    assert_eq!(reply.body["is_active"], true);
    assert_eq!(reply.body["is_superuser"], false);
    assert!(reply.body["full_name"].is_null());
    assert!(reply.body.get("hashed_password").is_none());
}

#[tokio::test]
async fn missing_token_is_401_not_authenticated() {
    let app = TestApp::new();
    let reply = app.get("/api/v1/base/login/me", None).await;
    assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
    assert_eq!(reply.detail(), "Not authenticated");
}

#[tokio::test]
async fn garbage_token_is_401_with_www_authenticate() {
    let app = TestApp::new();
    let reply = app.get("/api/v1/base/login/me", Some("not.a.jwt")).await;
    assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
    assert_eq!(reply.detail(), "Could not validate credentials");
    assert_eq!(reply.headers.get("www-authenticate").unwrap(), "Bearer");
}

#[tokio::test]
async fn expired_token_is_401() {
    let app = TestApp::new();
    let user = app.add_regular("alice@example.com").await;
    let expired = security::create_token_with_exp(
        &user.id.to_string(),
        &app.settings.secret_key,
        Utc::now().timestamp() - 3600,
    )
    .unwrap();
    let reply = app.get("/api/v1/base/login/me", Some(&expired)).await;
    assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
    assert_eq!(reply.detail(), "Could not validate credentials");
}

#[tokio::test]
async fn token_for_unknown_user_is_401() {
    let app = TestApp::new();
    let ghost = uuid::Uuid::new_v4().to_string();
    let token = security::create_access_token(&ghost, &app.settings.secret_key, 5).unwrap();
    let reply = app.get("/api/v1/base/login/me", Some(&token)).await;
    assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
    assert_eq!(reply.detail(), "Could not validate credentials");
}

#[tokio::test]
async fn token_signed_with_other_secret_is_401() {
    let app = TestApp::new();
    let user = app.add_regular("alice@example.com").await;
    let token = security::create_access_token(&user.id.to_string(), "another-secret", 5).unwrap();
    let reply = app.get("/api/v1/base/login/me", Some(&token)).await;
    assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
}
