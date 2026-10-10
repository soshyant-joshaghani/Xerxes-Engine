//! Documented routing: the OpenAPI document is generated from the routes themselves, never
//! written by hand. Register routes on an [`aide::axum::ApiRouter`] with [`get`], [`post`],
//! [`put`], [`patch`] and [`delete`] from here, and use the extractors below in handlers:
//!
//! - request bodies, query strings and path params come from the handler's types
//!   (`Body<T>`, `Form<T>`, `Query<T>`, `Path<T>`; `T: JsonSchema`), responses from its
//!   return type (`Json<T>`, [`Created`], [`FileBytes`], `ApiResult<..>` adds the error body);
//! - [`CurrentUser`] / [`Superuser`] mark the operation as needing the bearer token;
//! - the summary, operation id and tag come from the handler's name and module, like FastAPI:
//!   `modules::api::base::users::router::read_users` is "Read Users" under "[BASE] Users".
//!
//! The extractors keep the wire contract: a bad body, form or query is a `422 {"detail"}`
//! (not axum's plain-text rejection), and auth errors are the usual `401`/`403`.

use aide::axum::routing::{self, ApiMethodRouter};
use aide::generate::GenContext;
use aide::openapi::{Operation, Response};
use aide::operation::{OperationHandler, OperationInput, OperationOutput};
use aide::transform::TransformOperation;
use axum::body::Bytes;
use axum::extract::{FromRequest, FromRequestParts, RawQuery, Request};
use axum::handler::Handler;
use axum::http::request::Parts;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response as HttpResponse};
use axum::Json;
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::core::error::{parse_form, parse_json, parse_query, ApiError};
use crate::core::state::AppState;
use crate::modules::api::base::auth::{current_superuser, current_user};
use crate::modules::api::base::users::schemas::User;

/// The bearer-token security scheme (OAuth2 password flow; Scalar and Swagger log in with it).
pub const BEARER: &str = "OAuth2PasswordBearer";

// ---- routing -----------------------------------------------------------------------------

/// "read_users" → "Read Users".
fn title(snake: &str) -> String {
    snake
        .split('_')
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Summary, operation id and tag from the handler's path
/// (`xerxes_backend::modules::api::<group>::<module>[::router]::<handler>`; other series, like
/// `multiplayer`, tag by the series).
pub fn describe_handler(type_name: &str) -> (String, String, String) {
    let parts: Vec<&str> = type_name.split("::").collect();
    let handler = parts.last().copied().unwrap_or("handler");
    let after = parts
        .iter()
        .position(|p| *p == "modules")
        .map(|i| &parts[i + 1..parts.len() - 1])
        .unwrap_or(&[]);
    // `api` is the default series and not part of the tag; any other series is its own group.
    let (series, after) = match after.split_first() {
        Some((&"api", rest)) => ("api", rest),
        Some((&series, _)) => (series, after),
        None => ("api", after),
    };
    let group = match series {
        "api" => after.first().copied().unwrap_or("api"),
        series => series,
    }
    .to_uppercase();
    let module = after
        .iter()
        .rev()
        .find(|m| !matches!(**m, "router" | "routes" | "handlers" | "mod"))
        .copied()
        .unwrap_or("api");
    // `[BASE] Users`, but a module that is its own group is just `[ENGINE]`.
    let tag = if group.eq_ignore_ascii_case(module) {
        format!("[{group}]")
    } else {
        format!("[{group}] {}", title(module))
    };
    (title(handler), handler.to_string(), tag)
}

fn describe<H>(op: TransformOperation<'_>) -> TransformOperation<'_> {
    let (summary, id, tag) = describe_handler(std::any::type_name::<H>());
    op.summary(&summary).id(&id).tag(&tag)
}

macro_rules! documented_method {
    ($name:ident, $with:ident) => {
        /// The handler, documented from its types, name and module.
        pub fn $name<H, I, O, T>(handler: H) -> ApiMethodRouter<AppState>
        where
            H: Handler<T, AppState> + OperationHandler<I, O>,
            I: OperationInput,
            O: OperationOutput,
            T: 'static,
        {
            routing::$with(handler, describe::<H>)
        }
    };
}

documented_method!(get, get_with);
documented_method!(post, post_with);
documented_method!(put, put_with);
documented_method!(patch, patch_with);
documented_method!(delete, delete_with);

// Several methods on one path: `get(read_users).merge(post(create_user))`.

// ---- extractors ------------------------------------------------------------------------

/// A JSON request body (`422 {"detail"}` when it does not parse).
pub struct Body<T>(pub T);

impl<T: DeserializeOwned, S: Send + Sync> FromRequest<S> for Body<T> {
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let bytes = Bytes::from_request(req, state)
            .await
            .map_err(|e| ApiError::validation(format!("Invalid request body: {e}")))?;
        parse_json(&bytes).map(Body)
    }
}

impl<T: JsonSchema> OperationInput for Body<T> {
    fn operation_input(ctx: &mut GenContext, operation: &mut Operation) {
        <Json<T> as OperationInput>::operation_input(ctx, operation);
    }
}

/// An `application/x-www-form-urlencoded` body (`422 {"detail"}` when it does not parse).
pub struct Form<T>(pub T);

impl<T: DeserializeOwned, S: Send + Sync> FromRequest<S> for Form<T> {
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let bytes = Bytes::from_request(req, state)
            .await
            .map_err(|e| ApiError::validation(format!("Invalid form body: {e}")))?;
        parse_form(&bytes).map(Form)
    }
}

impl<T: JsonSchema> OperationInput for Form<T> {
    fn operation_input(ctx: &mut GenContext, operation: &mut Operation) {
        <axum::extract::Form<T> as OperationInput>::operation_input(ctx, operation);
    }
}

/// Query parameters (`422 {"detail"}` when they do not parse).
pub struct Query<T>(pub T);

impl<T: DeserializeOwned, S: Send + Sync> FromRequestParts<S> for Query<T> {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let RawQuery(raw) = RawQuery::from_request_parts(parts, state)
            .await
            .unwrap_or(RawQuery(None));
        parse_query(&raw).map(Query)
    }
}

impl<T: JsonSchema> OperationInput for Query<T> {
    fn operation_input(ctx: &mut GenContext, operation: &mut Operation) {
        <axum::extract::Query<T> as OperationInput>::operation_input(ctx, operation);
    }
}

/// A raw request body (a file's bytes), documented as `application/octet-stream`.
pub struct RawBody(pub Bytes);

impl<S: Send + Sync> FromRequest<S> for RawBody {
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        Bytes::from_request(req, state)
            .await
            .map(RawBody)
            .map_err(|e| ApiError::validation(format!("Invalid request body: {e}")))
    }
}

impl OperationInput for RawBody {
    fn operation_input(ctx: &mut GenContext, operation: &mut Operation) {
        Bytes::operation_input(ctx, operation);
    }
}

/// The signed-in user (bearer token). `401` without a valid token, `400` when inactive.
pub struct CurrentUser(pub User);

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        current_user(state, &parts.headers).await.map(CurrentUser)
    }
}

/// The signed-in user, who must be a superuser (`403` otherwise).
pub struct Superuser(pub User);

impl FromRequestParts<AppState> for Superuser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        current_superuser(state, &parts.headers)
            .await
            .map(Superuser)
    }
}

fn require_bearer(operation: &mut Operation) {
    let already = operation.security.iter().any(|r| r.contains_key(BEARER));
    if !already {
        let _ = TransformOperation::new(operation).security_requirement(BEARER);
    }
}

impl OperationInput for CurrentUser {
    fn operation_input(_ctx: &mut GenContext, operation: &mut Operation) {
        require_bearer(operation);
    }
}

impl OperationInput for Superuser {
    fn operation_input(_ctx: &mut GenContext, operation: &mut Operation) {
        require_bearer(operation);
    }
}

// ---- outputs ---------------------------------------------------------------------------

/// `201 Created` with a JSON body.
pub struct Created<T>(pub T);

impl<T: Serialize> IntoResponse for Created<T> {
    fn into_response(self) -> HttpResponse {
        (StatusCode::CREATED, Json(self.0)).into_response()
    }
}

impl<T: JsonSchema> OperationOutput for Created<T> {
    type Inner = T;

    fn operation_response(ctx: &mut GenContext, operation: &mut Operation) -> Option<Response> {
        Json::<T>::operation_response(ctx, operation)
    }

    fn inferred_responses(
        ctx: &mut GenContext,
        operation: &mut Operation,
    ) -> Vec<(Option<u16>, Response)> {
        Json::<T>::operation_response(ctx, operation)
            .map(|res| vec![(Some(201), res)])
            .unwrap_or_default()
    }
}

/// A file's bytes with its content type, documented as `application/octet-stream`.
pub struct FileBytes {
    pub bytes: Vec<u8>,
    pub content_type: &'static str,
}

impl IntoResponse for FileBytes {
    fn into_response(self) -> HttpResponse {
        ([(header::CONTENT_TYPE, self.content_type)], self.bytes).into_response()
    }
}

impl OperationOutput for FileBytes {
    type Inner = Vec<u8>;

    fn operation_response(ctx: &mut GenContext, operation: &mut Operation) -> Option<Response> {
        Vec::<u8>::operation_response(ctx, operation)
    }

    fn inferred_responses(
        ctx: &mut GenContext,
        operation: &mut Operation,
    ) -> Vec<(Option<u16>, Response)> {
        Vec::<u8>::inferred_responses(ctx, operation)
    }
}
