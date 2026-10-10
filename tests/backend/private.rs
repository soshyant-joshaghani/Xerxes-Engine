mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::json;
use xerxes_backend::modules::api::base::users::repository::UserRepository;

#[tokio::test]
async fn private_ping_in_local() {
    let app = TestApp::new();
    let reply = app.get("/api/v1/private/ping", None).await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.body["message"], "private ok");
}

#[tokio::test]
async fn private_create_user_in_local() {
    let app = TestApp::new();
    let reply = app
        .post(
            "/api/v1/private/users",
            None,
            json!({"email": "dev@example.com", "password": "longenough1", "full_name": "Dev"}),
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.body["email"], "dev@example.com");
    assert_eq!(reply.body["is_superuser"], false);
    assert_eq!(reply.body["full_name"], "Dev");
    assert!(app
        .users
        .get_by_email("dev@example.com")
        .await
        .unwrap()
        .is_some());

    let duplicate = app
        .post(
            "/api/v1/private/users",
            None,
            json!({"email": "dev@example.com", "password": "longenough1"}),
        )
        .await;
    assert_eq!(duplicate.status, StatusCode::BAD_REQUEST);

    let short = app
        .post(
            "/api/v1/private/users",
            None,
            json!({"email": "other@example.com", "password": "short"}),
        )
        .await;
    assert_eq!(short.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(short.body["detail"].is_string());
}

#[tokio::test]
async fn private_job_ping_enqueues() {
    let app = TestApp::new();
    let reply = app
        .post("/api/v1/private/jobs/ping?message=hello", None, json!({}))
        .await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.body["message"], "hello");
    let job_id = reply.body["job_id"].as_str().unwrap().to_string();

    let jobs = app.jobs.jobs();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].id, job_id);
    assert_eq!(jobs[0].task, "ping");
    assert_eq!(jobs[0].args["message"], "hello");

    let default_message = app
        .send("POST", "/api/v1/private/jobs/ping", None, None)
        .await;
    assert_eq!(default_message.status, StatusCode::OK);
    assert_eq!(default_message.body["message"], "ping");
}

#[tokio::test]
async fn private_job_ping_is_503_when_redis_is_down() {
    let app = TestApp::new();
    app.jobs.fail_with("connection refused");
    let reply = app
        .send("POST", "/api/v1/private/jobs/ping", None, None)
        .await;
    assert_eq!(reply.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(reply.detail(), "Redis unavailable: connection refused");
}

#[tokio::test]
async fn private_routes_do_not_exist_outside_local() {
    for environment in ["staging", "production"] {
        let app = TestApp::with_environment(environment);
        assert_eq!(
            app.get("/api/v1/private/ping", None).await.status,
            StatusCode::NOT_FOUND,
            "{}",
            environment
        );
        let users = app
            .post(
                "/api/v1/private/users",
                None,
                json!({"email": "dev@example.com", "password": "longenough1"}),
            )
            .await;
        assert_eq!(users.status, StatusCode::NOT_FOUND);
        let jobs = app
            .send("POST", "/api/v1/private/jobs/ping", None, None)
            .await;
        assert_eq!(jobs.status, StatusCode::NOT_FOUND);
        assert!(app.jobs.jobs().is_empty());

        // Regular routes stay available.
        let health = app.get("/api/v1/utils/health-check", None).await;
        assert_eq!(health.status, StatusCode::OK);
    }
}
