use aide::axum::ApiRouter;
use axum::extract::State;
use axum::Json;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::core::api::{get, post, Body, Query};
use crate::core::config::Settings;
use crate::core::error::{check_email, check_len, ApiError, ApiResult};
use crate::core::state::AppState;
use crate::modules::api::base::users::schemas::{Message, UserCreate, UserPublic};
use crate::modules::api::base::users::service::UserService;

/// `/utils/*` always; `/private/*` only when `ENVIRONMENT=local`.
pub fn router(settings: &Settings) -> ApiRouter<AppState> {
    let mut routes = ApiRouter::new().api_route("/utils/health-check", get(health_check));
    if settings.is_local() {
        routes = routes
            .api_route("/private/ping", get(private_ping))
            .api_route("/private/users", post(private_create_user))
            .api_route("/private/jobs/ping", post(enqueue_ping_job));
    }
    routes
}

async fn health_check() -> Json<bool> {
    Json(true)
}

async fn private_ping() -> Json<Message> {
    Json(Message {
        message: "private ok".to_string(),
    })
}

#[derive(Debug, Deserialize, JsonSchema)]
struct PrivateUserCreate {
    email: String,
    password: String,
    #[serde(default)]
    full_name: Option<String>,
}

async fn private_create_user(
    State(state): State<AppState>,
    Body(input): Body<PrivateUserCreate>,
) -> ApiResult<Json<UserPublic>> {
    check_email("email", &input.email)?;
    check_len("password", &input.password, 8, 128)?;
    let create = UserCreate {
        email: input.email,
        password: input.password,
        is_active: true,
        is_superuser: false,
        full_name: input.full_name,
    };
    let user = UserService::new(&state).create(create).await?;
    Ok(Json(UserPublic::from(&user)))
}

#[derive(Debug, Default, Deserialize, JsonSchema)]
struct PingQuery {
    message: Option<String>,
}

async fn enqueue_ping_job(
    State(state): State<AppState>,
    Query(query): Query<PingQuery>,
) -> ApiResult<Json<Value>> {
    let message = query.message.unwrap_or_else(|| "ping".to_string());
    let args = json!({ "message": message });
    match state.jobs.enqueue("ping", args).await {
        Ok(job_id) => Ok(Json(json!({ "job_id": job_id, "message": message }))),
        Err(err) => Err(ApiError::unavailable(format!("Redis unavailable: {}", err))),
    }
}
