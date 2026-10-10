//! The API series: everything that is plain request/response HTTP. One folder per module
//! (`base`, `system`, `engine`, `editor`, and each product module you add). It is served at
//! `settings.api_v1_str` (`/api/v1`). It must not import [`super::multiplayer`].

pub mod base;
pub mod editor;
pub mod engine;
pub mod system;

use aide::axum::ApiRouter;

use crate::core::config::Settings;
use crate::core::state::AppState;

/// Every API route: system (health, private dev routes), engine and editor (local dev services), base (auth, users) and the
/// product modules.
pub fn router(settings: &Settings) -> ApiRouter<AppState> {
    ApiRouter::new()
        .merge(system::router::router(settings))
        .merge(engine::router(settings))
        .merge(base::router())
}
