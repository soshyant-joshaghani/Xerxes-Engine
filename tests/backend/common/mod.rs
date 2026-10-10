#![allow(dead_code)]
//! Shared test harness: the real router wired to in-memory fakes.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{HeaderMap, Request, StatusCode};
use axum::Router;
use serde_json::Value;
use tower::ServiceExt;
use uuid::Uuid;

use xerxes_backend::build_router;
use xerxes_backend::core::cache::{Cache, InMemoryCache, NoopCache};
use xerxes_backend::core::config::Settings;
use xerxes_backend::core::jobs::{InMemoryJobQueue, JobQueue};
use xerxes_backend::core::security;
use xerxes_backend::core::state::AppState;
use xerxes_backend::modules::api::base::users::repository::{
    InMemoryUserRepository, UserRepository,
};
use xerxes_backend::modules::api::base::users::schemas::User;

pub const PASSWORD: &str = "password123";

pub struct Reply {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Value,
    pub text: String,
}

impl Reply {
    pub fn detail(&self) -> String {
        self.body["detail"].as_str().unwrap_or("").to_string()
    }
}

pub struct TestApp {
    pub router: Router,
    pub state: AppState,
    pub settings: Arc<Settings>,
    pub users: Arc<InMemoryUserRepository>,
    pub cache: Arc<InMemoryCache>,
    pub jobs: Arc<InMemoryJobQueue>,
}

impl TestApp {
    pub fn new() -> TestApp {
        TestApp::build(Settings::for_tests(), true)
    }

    pub fn with_environment(environment: &str) -> TestApp {
        let mut settings = Settings::for_tests();
        settings.environment = environment.to_string();
        TestApp::build(settings, true)
    }

    /// Same app, but the cache is a no-op (what a dead Redis looks like).
    pub fn with_settings(settings: Settings) -> TestApp {
        TestApp::build(settings, true)
    }

    pub fn without_cache() -> TestApp {
        TestApp::build(Settings::for_tests(), false)
    }

    fn build(settings: Settings, use_cache: bool) -> TestApp {
        let settings = Arc::new(settings);
        let users = Arc::new(InMemoryUserRepository::new());
        let cache = Arc::new(InMemoryCache::new());
        let jobs = Arc::new(InMemoryJobQueue::new());

        let users_dyn: Arc<dyn UserRepository> = users.clone();
        let jobs_dyn: Arc<dyn JobQueue> = jobs.clone();
        let real_cache: Arc<dyn Cache> = cache.clone();
        let noop_cache: Arc<dyn Cache> = Arc::new(NoopCache);
        let cache_dyn: Arc<dyn Cache> = if use_cache { real_cache } else { noop_cache };

        let state = AppState::new(settings.clone(), users_dyn, cache_dyn, jobs_dyn);
        let router = build_router(state.clone());
        TestApp {
            router,
            state,
            settings,
            users,
            cache,
            jobs,
        }
    }

    /// Insert a user directly into the fake repository (cheap bcrypt cost).
    pub async fn add_user(
        &self,
        email: &str,
        password: &str,
        is_superuser: bool,
        is_active: bool,
    ) -> User {
        let hashed = security::hash_password(password, 4).unwrap();
        let user = User {
            id: Uuid::new_v4(),
            email: email.to_string(),
            is_active,
            is_superuser,
            full_name: None,
            hashed_password: hashed,
        };
        self.users.create(user.clone()).await.unwrap();
        user
    }

    pub async fn add_admin(&self) -> User {
        self.add_user("admin@example.com", PASSWORD, true, true)
            .await
    }

    pub async fn add_regular(&self, email: &str) -> User {
        self.add_user(email, PASSWORD, false, true).await
    }

    pub fn token_for(&self, user: &User) -> String {
        security::create_access_token(&user.id.to_string(), &self.settings.secret_key, 60).unwrap()
    }

    pub async fn dispatch(&self, request: Request<Body>) -> Reply {
        let response = self.router.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let headers = response.headers().clone();
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let text = String::from_utf8_lossy(&bytes).to_string();
        let body = serde_json::from_slice::<Value>(&bytes).unwrap_or(Value::Null);
        Reply {
            status,
            headers,
            body,
            text,
        }
    }

    pub async fn send(
        &self,
        method: &str,
        uri: &str,
        token: Option<&str>,
        json_body: Option<Value>,
    ) -> Reply {
        let mut builder = Request::builder().method(method).uri(uri);
        if let Some(token) = token {
            builder = builder.header("authorization", format!("Bearer {}", token));
        }
        let body = match json_body {
            Some(value) => {
                builder = builder.header("content-type", "application/json");
                Body::from(value.to_string())
            }
            None => Body::empty(),
        };
        self.dispatch(builder.body(body).unwrap()).await
    }

    pub async fn get(&self, uri: &str, token: Option<&str>) -> Reply {
        self.send("GET", uri, token, None).await
    }

    pub async fn post(&self, uri: &str, token: Option<&str>, body: Value) -> Reply {
        self.send("POST", uri, token, Some(body)).await
    }

    pub async fn patch(&self, uri: &str, token: Option<&str>, body: Value) -> Reply {
        self.send("PATCH", uri, token, Some(body)).await
    }

    pub async fn delete(&self, uri: &str, token: Option<&str>) -> Reply {
        self.send("DELETE", uri, token, None).await
    }

    /// `POST /api/v1/base/login/access-token` with a urlencoded form.
    pub async fn login(&self, username: &str, password: &str) -> Reply {
        let form =
            serde_urlencoded::to_string([("username", username), ("password", password)]).unwrap();
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/base/login/access-token")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from(form))
            .unwrap();
        self.dispatch(request).await
    }
}
