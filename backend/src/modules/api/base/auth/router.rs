use aide::axum::ApiRouter;
use axum::extract::State;
use axum::Json;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::core::api::{get, post, CurrentUser, Form};
use crate::core::error::ApiResult;
use crate::core::security;
use crate::core::state::AppState;
use crate::modules::api::base::auth::Token;
use crate::modules::api::base::users::schemas::UserPublic;
use crate::modules::api::base::users::service::UserService;

/// OAuth2 password login (form fields).
#[derive(Debug, Deserialize, JsonSchema)]
struct LoginForm {
    username: String,
    password: String,
}

pub fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/login/access-token", post(login_access_token))
        .api_route("/login/me", get(read_users_me))
}

async fn login_access_token(
    State(state): State<AppState>,
    Form(form): Form<LoginForm>,
) -> ApiResult<Json<Token>> {
    let user = UserService::new(&state)
        .authenticate(&form.username, &form.password)
        .await?;
    let token = security::create_access_token(
        &user.id.to_string(),
        &state.config.secret_key,
        state.config.access_token_expire_minutes,
    )
    .map_err(crate::core::error::ApiError::internal)?;
    Ok(Json(Token {
        access_token: token,
        token_type: "bearer".to_string(),
    }))
}

async fn read_users_me(CurrentUser(user): CurrentUser) -> ApiResult<Json<UserPublic>> {
    Ok(Json(UserPublic::from(&user)))
}
