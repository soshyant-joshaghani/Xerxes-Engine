//! Bearer authentication helpers shared by every protected route.

pub mod router;

use axum::http::{header, HeaderMap};
use serde::Serialize;

use crate::core::error::{ApiError, ApiResult};
use crate::core::security;
use crate::core::state::AppState;
use crate::modules::api::base::users::schemas::User;

#[derive(Debug, Clone, Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct Token {
    pub access_token: String,
    pub token_type: String,
}

/// The raw token of `Authorization: Bearer <jwt>`, or `401 Not authenticated`.
fn bearer_token(headers: &HeaderMap) -> ApiResult<String> {
    let value = match headers.get(header::AUTHORIZATION) {
        Some(v) => v,
        None => return Err(ApiError::not_authenticated()),
    };
    let text = match value.to_str() {
        Ok(t) => t.trim(),
        Err(_) => return Err(ApiError::not_authenticated()),
    };
    let (scheme, token) = match text.split_once(' ') {
        Some(pair) => pair,
        None => return Err(ApiError::not_authenticated()),
    };
    if !scheme.eq_ignore_ascii_case("bearer") || token.trim().is_empty() {
        return Err(ApiError::not_authenticated());
    }
    Ok(token.trim().to_string())
}

/// Resolve the caller from the bearer token.
///
/// Missing header `401 Not authenticated`; bad/expired token or unknown user
/// `401 Could not validate credentials`; inactive user `400 Inactive user`.
pub async fn current_user(state: &AppState, headers: &HeaderMap) -> ApiResult<User> {
    let token = bearer_token(headers)?;
    let subject = match security::decode_access_token(&token, &state.config.secret_key) {
        Ok(sub) => sub,
        Err(_) => return Err(ApiError::invalid_credentials()),
    };
    let id = match uuid::Uuid::parse_str(&subject) {
        Ok(id) => id,
        Err(_) => return Err(ApiError::invalid_credentials()),
    };
    let user = match state.users.get_by_id(id).await? {
        Some(user) => user,
        None => return Err(ApiError::invalid_credentials()),
    };
    if !user.is_active {
        return Err(ApiError::bad_request("Inactive user"));
    }
    Ok(user)
}

pub async fn current_superuser(state: &AppState, headers: &HeaderMap) -> ApiResult<User> {
    let user = current_user(state, headers).await?;
    if !user.is_superuser {
        return Err(ApiError::forbidden(
            "The user doesn't have enough privileges",
        ));
    }
    Ok(user)
}
