mod common;

use axum::http::StatusCode;
use common::{TestApp, PASSWORD};
use serde_json::json;
use uuid::Uuid;
use xerxes_backend::modules::api::base::users::repository::UserRepository;
use xerxes_backend::modules::api::base::users::service::UserService;

const ADMIN_LIST: &str = "/api/v1/base/users/admin";

fn user_url(id: &Uuid) -> String {
    format!("/api/v1/base/users/{}/admin", id)
}

#[tokio::test]
async fn first_superuser_is_seeded_once() {
    let app = TestApp::new();
    let service = UserService::new(&app.state);
    service
        .ensure_first_superuser("root@example.com", "rootpassword1")
        .await
        .unwrap();
    service
        .ensure_first_superuser("root@example.com", "rootpassword1")
        .await
        .unwrap();
    assert_eq!(app.users.count().await.unwrap(), 1);

    let reply = app.login("root@example.com", "rootpassword1").await;
    assert_eq!(reply.status, StatusCode::OK);
    let token = reply.body["access_token"].as_str().unwrap().to_string();
    let me = app.get("/api/v1/base/login/me", Some(&token)).await;
    assert_eq!(me.body["is_superuser"], true);
}

#[tokio::test]
async fn list_users_requires_superuser() {
    let app = TestApp::new();
    let admin = app.add_admin().await;
    let regular = app.add_regular("bob@example.com").await;

    let anonymous = app.get(ADMIN_LIST, None).await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);

    let forbidden = app.get(ADMIN_LIST, Some(&app.token_for(&regular))).await;
    assert_eq!(forbidden.status, StatusCode::FORBIDDEN);
    assert_eq!(
        forbidden.detail(),
        "The user doesn't have enough privileges"
    );

    let ok = app.get(ADMIN_LIST, Some(&app.token_for(&admin))).await;
    assert_eq!(ok.status, StatusCode::OK);
    assert_eq!(ok.body["count"], 2);
}

#[tokio::test]
async fn list_users_is_ordered_by_email_and_paginated() {
    let app = TestApp::new();
    let admin = app.add_admin().await;
    app.add_regular("carol@example.com").await;
    app.add_regular("bob@example.com").await;
    let token = app.token_for(&admin);

    let all = app.get(ADMIN_LIST, Some(&token)).await;
    let emails: Vec<String> = all.body["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|u| u["email"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(
        emails,
        vec!["admin@example.com", "bob@example.com", "carol@example.com"]
    );
    assert_eq!(all.body["count"], 3);

    let page = app
        .get(&format!("{}?skip=1&limit=1", ADMIN_LIST), Some(&token))
        .await;
    assert_eq!(page.status, StatusCode::OK);
    assert_eq!(page.body["data"].as_array().unwrap().len(), 1);
    assert_eq!(page.body["data"][0]["email"], "bob@example.com");
    assert_eq!(page.body["count"], 3);

    let bad = app
        .get(&format!("{}?skip=abc", ADMIN_LIST), Some(&token))
        .await;
    assert_eq!(bad.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(bad.body["detail"].is_string());
}

#[tokio::test]
async fn create_user_as_superuser() {
    let app = TestApp::new();
    let admin = app.add_admin().await;
    let token = app.token_for(&admin);

    let reply = app
        .post(
            ADMIN_LIST,
            Some(&token),
            json!({"email": "new@example.com", "password": "longenough1", "full_name": "New One"}),
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.body["email"], "new@example.com");
    assert_eq!(reply.body["is_active"], true);
    assert_eq!(reply.body["is_superuser"], false);
    assert_eq!(reply.body["full_name"], "New One");
    assert!(reply.body.get("hashed_password").is_none());

    let stored = app
        .users
        .get_by_email("new@example.com")
        .await
        .unwrap()
        .unwrap();
    assert!(stored.hashed_password.starts_with("$2b$"));

    let login = app.login("new@example.com", "longenough1").await;
    assert_eq!(login.status, StatusCode::OK);
}

#[tokio::test]
async fn create_user_rejects_duplicates_and_bad_input() {
    let app = TestApp::new();
    let admin = app.add_admin().await;
    let regular = app.add_regular("bob@example.com").await;
    let token = app.token_for(&admin);

    let duplicate = app
        .post(
            ADMIN_LIST,
            Some(&token),
            json!({"email": "bob@example.com", "password": "longenough1"}),
        )
        .await;
    assert_eq!(duplicate.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        duplicate.detail(),
        "The user with this email already exists in the system."
    );

    let short = app
        .post(
            ADMIN_LIST,
            Some(&token),
            json!({"email": "x@example.com", "password": "short"}),
        )
        .await;
    assert_eq!(short.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(short.body["detail"].is_string());

    let missing = app
        .post(ADMIN_LIST, Some(&token), json!({"email": "x@example.com"}))
        .await;
    assert_eq!(missing.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(missing.body["detail"].is_string());

    let not_json = app.send("POST", ADMIN_LIST, Some(&token), None).await;
    assert_eq!(not_json.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(not_json.body["detail"].is_string());

    let forbidden = app
        .post(
            ADMIN_LIST,
            Some(&app.token_for(&regular)),
            json!({"email": "y@example.com", "password": "longenough1"}),
        )
        .await;
    assert_eq!(forbidden.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn read_user_self_or_superuser() {
    let app = TestApp::new();
    let admin = app.add_admin().await;
    let bob = app.add_regular("bob@example.com").await;
    let carol = app.add_regular("carol@example.com").await;

    let own = app
        .get(&user_url(&bob.id), Some(&app.token_for(&bob)))
        .await;
    assert_eq!(own.status, StatusCode::OK);
    assert_eq!(own.body["email"], "bob@example.com");

    let other = app
        .get(&user_url(&carol.id), Some(&app.token_for(&bob)))
        .await;
    assert_eq!(other.status, StatusCode::FORBIDDEN);
    assert_eq!(other.detail(), "The user doesn't have enough privileges");

    let as_admin = app
        .get(&user_url(&carol.id), Some(&app.token_for(&admin)))
        .await;
    assert_eq!(as_admin.status, StatusCode::OK);
    assert_eq!(as_admin.body["email"], "carol@example.com");

    let unknown = app
        .get(&user_url(&Uuid::new_v4()), Some(&app.token_for(&admin)))
        .await;
    assert_eq!(unknown.status, StatusCode::NOT_FOUND);
    assert_eq!(unknown.detail(), "User not found");

    let bad_id = app
        .get(
            "/api/v1/base/users/not-a-uuid/admin",
            Some(&app.token_for(&admin)),
        )
        .await;
    assert_eq!(bad_id.status, StatusCode::UNPROCESSABLE_ENTITY);

    let anonymous = app.get(&user_url(&bob.id), None).await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn update_user_changes_only_sent_fields_and_rehashes_password() {
    let app = TestApp::new();
    let admin = app.add_admin().await;
    let bob = app.add_regular("bob@example.com").await;
    let token = app.token_for(&admin);

    let reply = app
        .patch(
            &user_url(&bob.id),
            Some(&token),
            json!({"full_name": "Bob B", "is_superuser": true}),
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.body["full_name"], "Bob B");
    assert_eq!(reply.body["is_superuser"], true);
    assert_eq!(reply.body["email"], "bob@example.com");
    assert_eq!(reply.body["is_active"], true);

    let old_hash = app
        .users
        .get_by_id(bob.id)
        .await
        .unwrap()
        .unwrap()
        .hashed_password;
    assert_eq!(
        old_hash, bob.hashed_password,
        "hash untouched without password"
    );

    let reply = app
        .patch(
            &user_url(&bob.id),
            Some(&token),
            json!({"password": "brand-new-pass"}),
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK);
    let new_hash = app
        .users
        .get_by_id(bob.id)
        .await
        .unwrap()
        .unwrap()
        .hashed_password;
    assert_ne!(new_hash, old_hash);
    assert!(new_hash.starts_with("$2b$"));
    assert_eq!(
        app.login("bob@example.com", "brand-new-pass").await.status,
        StatusCode::OK
    );
    assert_eq!(
        app.login("bob@example.com", PASSWORD).await.status,
        StatusCode::BAD_REQUEST
    );

    let cleared = app
        .patch(&user_url(&bob.id), Some(&token), json!({"full_name": null}))
        .await;
    assert_eq!(cleared.status, StatusCode::OK);
    assert!(cleared.body["full_name"].is_null());
}

#[tokio::test]
async fn update_user_errors() {
    let app = TestApp::new();
    let admin = app.add_admin().await;
    let bob = app.add_regular("bob@example.com").await;
    let carol = app.add_regular("carol@example.com").await;
    let token = app.token_for(&admin);

    let clash = app
        .patch(
            &user_url(&bob.id),
            Some(&token),
            json!({"email": "carol@example.com"}),
        )
        .await;
    assert_eq!(clash.status, StatusCode::CONFLICT);
    assert_eq!(clash.detail(), "User with this email already exists");

    let same_email = app
        .patch(
            &user_url(&bob.id),
            Some(&token),
            json!({"email": "bob@example.com"}),
        )
        .await;
    assert_eq!(same_email.status, StatusCode::OK);

    let unknown = app
        .patch(
            &user_url(&Uuid::new_v4()),
            Some(&token),
            json!({"full_name": "x"}),
        )
        .await;
    assert_eq!(unknown.status, StatusCode::NOT_FOUND);
    assert_eq!(
        unknown.detail(),
        "The user with this id does not exist in the system"
    );

    let forbidden = app
        .patch(
            &user_url(&bob.id),
            Some(&app.token_for(&carol)),
            json!({"full_name": "x"}),
        )
        .await;
    assert_eq!(forbidden.status, StatusCode::FORBIDDEN);

    let invalid = app
        .patch(
            &user_url(&bob.id),
            Some(&token),
            json!({"password": "short"}),
        )
        .await;
    assert_eq!(invalid.status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn delete_user_removes_the_user() {
    let app = TestApp::new();
    let admin = app.add_admin().await;
    let bob = app.add_regular("bob@example.com").await;

    let reply = app
        .delete(&user_url(&bob.id), Some(&app.token_for(&admin)))
        .await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.body["message"], "User deleted successfully");
    assert!(app.users.get_by_id(bob.id).await.unwrap().is_none());
}

#[tokio::test]
async fn delete_user_guards() {
    let app = TestApp::new();
    let admin = app.add_admin().await;
    let bob = app.add_regular("bob@example.com").await;
    let token = app.token_for(&admin);

    let own = app.delete(&user_url(&admin.id), Some(&token)).await;
    assert_eq!(own.status, StatusCode::FORBIDDEN);
    assert_eq!(
        own.detail(),
        "Super users are not allowed to delete themselves"
    );
    assert!(app.users.get_by_id(admin.id).await.unwrap().is_some());

    let unknown = app.delete(&user_url(&Uuid::new_v4()), Some(&token)).await;
    assert_eq!(unknown.status, StatusCode::NOT_FOUND);
    assert_eq!(unknown.detail(), "User not found");

    let forbidden = app
        .delete(&user_url(&admin.id), Some(&app.token_for(&bob)))
        .await;
    assert_eq!(forbidden.status, StatusCode::FORBIDDEN);
    assert_eq!(
        forbidden.detail(),
        "The user doesn't have enough privileges"
    );

    let anonymous = app.delete(&user_url(&bob.id), None).await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
}
