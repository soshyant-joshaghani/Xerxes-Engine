pub mod auth;
pub mod users;

use aide::axum::ApiRouter;

use crate::core::state::AppState;

/// Routes under `/base`.
pub fn router() -> ApiRouter<AppState> {
    let routes = ApiRouter::new()
        .merge(auth::router::router())
        .merge(users::router::router());
    ApiRouter::new().nest("/base", routes)
}
