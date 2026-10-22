use aide::axum::ApiRouter;
use axum::extract::{Path, State};
use axum::Json;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::core::api::{delete, get, patch, post, Body, CurrentUser, Query, Superuser};
use crate::core::error::{parse_uuid, ApiResult};
use crate::core::state::AppState;
use crate::modules::api::base::users::schemas::{
    Message, UserCreate, UserPublic, UserUpdate, UsersPublic,
};
use crate::modules::api::base::users::service::UserService;

#[derive(Debug, Default, Deserialize, JsonSchema)]
struct Pagination {
    skip: Option<i64>,
    limit: Option<i64>,
}

/// `{id}`: a user id (UUID; anything else is a 422).
#[derive(Debug, Deserialize, JsonSchema)]
struct UserId {
    id: String,
}

pub fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/users/admin", get(read_users).merge(post(create_user)))
        .api_route(
            "/users/{id}/admin",
            get(read_user_by_id)
                .merge(patch(update_user))
                .merge(delete(delete_user)),
        )
}

async fn read_users(
    State(state): State<AppState>,
    _admin: Superuser,
    Query(page): Query<Pagination>,
) -> ApiResult<Json<UsersPublic>> {
    let skip = page.skip.unwrap_or(0).max(0);
    let limit = page.limit.unwrap_or(100).max(0);
    let result = UserService::new(&state).list(skip, limit).await?;
    Ok(Json(result))
}

async fn create_user(
    State(state): State<AppState>,
    _admin: Superuser,
    Body(input): Body<UserCreate>,
) -> ApiResult<Json<UserPublic>> {
    let user = UserService::new(&state).create(input).await?;
    Ok(Json(UserPublic::from(&user)))
}

async fn read_user_by_id(
    State(state): State<AppState>,
    CurrentUser(current): CurrentUser,
    Path(UserId { id }): Path<UserId>,
) -> ApiResult<Json<UserPublic>> {
    let id = parse_uuid(&id)?;
    let user = UserService::new(&state).get_for(&current, id).await?;
    Ok(Json(UserPublic::from(&user)))
}

async fn update_user(
    State(state): State<AppState>,
    _admin: Superuser,
    Path(UserId { id }): Path<UserId>,
    Body(input): Body<UserUpdate>,
) -> ApiResult<Json<UserPublic>> {
    let id = parse_uuid(&id)?;
    let user = UserService::new(&state).update(id, input).await?;
    Ok(Json(UserPublic::from(&user)))
}

async fn delete_user(
    State(state): State<AppState>,
    Superuser(current): Superuser,
    Path(UserId { id }): Path<UserId>,
) -> ApiResult<Json<Message>> {
    let id = parse_uuid(&id)?;
    UserService::new(&state).delete(&current, id).await?;
    Ok(Json(Message {
        message: "User deleted successfully".to_string(),
    }))
}
