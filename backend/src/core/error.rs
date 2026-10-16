use std::fmt;

use axum::body::Bytes;
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::de::DeserializeOwned;
use serde_json::json;
use uuid::Uuid;

/// The error body every route answers with (`{"detail": "..."}`), as documented.
#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
pub struct Detail {
    pub detail: String,
}

/// Every route's errors are documented as the default response: a [`Detail`] body.
impl aide::operation::OperationOutput for ApiError {
    type Inner = Detail;

    fn operation_response(
        ctx: &mut aide::generate::GenContext,
        operation: &mut aide::openapi::Operation,
    ) -> Option<aide::openapi::Response> {
        let mut response = Json::<Detail>::operation_response(ctx, operation)?;
        response.description = "Error: `{\"detail\": \"...\"}`".into();
        Some(response)
    }

    fn inferred_responses(
        ctx: &mut aide::generate::GenContext,
        operation: &mut aide::openapi::Operation,
    ) -> Vec<(Option<u16>, aide::openapi::Response)> {
        Self::operation_response(ctx, operation)
            .map(|res| vec![(None, res)])
            .unwrap_or_default()
    }
}

/// HTTP error rendered as `{"detail": "<message>"}`.
#[derive(Debug, Clone)]
pub struct ApiError {
    pub status: StatusCode,
    pub detail: String,
    pub bearer: bool,
}

pub type ApiResult<T> = Result<T, ApiError>;

impl ApiError {
    pub fn new(status: StatusCode, detail: impl Into<String>) -> Self {
        ApiError {
            status,
            detail: detail.into(),
            bearer: false,
        }
    }

    pub fn bad_request(detail: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, detail)
    }

    pub fn not_authenticated() -> Self {
        ApiError {
            status: StatusCode::UNAUTHORIZED,
            detail: "Not authenticated".to_string(),
            bearer: true,
        }
    }

    pub fn invalid_credentials() -> Self {
        ApiError {
            status: StatusCode::UNAUTHORIZED,
            detail: "Could not validate credentials".to_string(),
            bearer: true,
        }
    }

    pub fn forbidden(detail: impl Into<String>) -> Self {
        Self::new(StatusCode::FORBIDDEN, detail)
    }

    pub fn not_found(detail: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, detail)
    }

    pub fn conflict(detail: impl Into<String>) -> Self {
        Self::new(StatusCode::CONFLICT, detail)
    }

    pub fn validation(detail: impl Into<String>) -> Self {
        Self::new(StatusCode::UNPROCESSABLE_ENTITY, detail)
    }

    pub fn unavailable(detail: impl Into<String>) -> Self {
        Self::new(StatusCode::SERVICE_UNAVAILABLE, detail)
    }

    /// Log the real error, answer with a generic 500.
    pub fn internal<E: fmt::Display>(err: E) -> Self {
        tracing::error!("internal error: {}", err);
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.status.as_u16(), self.detail)
    }
}

impl std::error::Error for ApiError {}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let mut response = (self.status, Json(json!({ "detail": self.detail }))).into_response();
        if self.bearer {
            response
                .headers_mut()
                .insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
        }
        response
    }
}

/// Parse a JSON body; failures become `422 {"detail": "..."}`.
pub fn parse_json<T: DeserializeOwned>(body: &Bytes) -> ApiResult<T> {
    serde_json::from_slice::<T>(body.as_ref())
        .map_err(|e| ApiError::validation(format!("Invalid request body: {}", e)))
}

/// Parse an `application/x-www-form-urlencoded` body.
pub fn parse_form<T: DeserializeOwned>(body: &Bytes) -> ApiResult<T> {
    serde_urlencoded::from_bytes::<T>(body.as_ref())
        .map_err(|e| ApiError::validation(format!("Invalid form body: {}", e)))
}

/// Parse a raw query string into a struct.
pub fn parse_query<T: DeserializeOwned>(raw: &Option<String>) -> ApiResult<T> {
    let text = raw.as_deref().unwrap_or("");
    serde_urlencoded::from_str::<T>(text)
        .map_err(|e| ApiError::validation(format!("Invalid query parameters: {}", e)))
}

pub fn parse_uuid(raw: &str) -> ApiResult<Uuid> {
    Uuid::parse_str(raw).map_err(|_| ApiError::validation(format!("Invalid UUID: {}", raw)))
}

/// Length check in characters, like pydantic `min_length` / `max_length`.
pub fn check_len(field: &str, value: &str, min: usize, max: usize) -> ApiResult<()> {
    let len = value.chars().count();
    if len < min {
        return Err(ApiError::validation(format!(
            "{}: must have at least {} character(s)",
            field, min
        )));
    }
    if len > max {
        return Err(ApiError::validation(format!(
            "{}: must have at most {} characters",
            field, max
        )));
    }
    Ok(())
}

/// Light e-mail shape check (`a@b`, no whitespace) plus the 255 char limit.
pub fn check_email(field: &str, value: &str) -> ApiResult<()> {
    check_len(field, value, 3, 255)?;
    let ok = match value.split_once('@') {
        Some((local, domain)) => {
            !local.is_empty()
                && !domain.is_empty()
                && !domain.contains('@')
                && !value.chars().any(|c| c.is_whitespace())
        }
        None => false,
    };
    if !ok {
        return Err(ApiError::validation(format!(
            "{}: value is not a valid email address",
            field
        )));
    }
    Ok(())
}
