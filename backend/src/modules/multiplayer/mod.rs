//! The multiplayer series (the M track of Phase 3): a Rust port of Colyseus (see `__docs__/multiplayer.md`).
//! WebSocket, Room, Client, Join, Leave, Message, State.
//! Nothing is built here yet; this is the seam. Put each feature in `multiplayer/<name>/` with
//! its own router, service and schemas, and merge it in [`router`].
//!
//! It is mounted at `/multiplayer` (WebSocket upgrades at `/multiplayer/ws`, later), outside
//! the `/api/v1` prefix. Only `crate::core` is shared with the API: this series must not import
//! `modules::api` (see `tests/backend/modules.rs`). Needing a user or a rule from the API means
//! moving it into `core`, or calling the API over HTTP, so the two can live in separate
//! containers.

use aide::axum::ApiRouter;
use axum::Json;

use crate::core::api::get;
use crate::core::state::AppState;

/// The URL prefix of the whole series.
pub const PREFIX: &str = "/multiplayer";

/// Routes of the multiplayer series, nested under [`PREFIX`].
pub fn router() -> ApiRouter<AppState> {
    ApiRouter::new().api_route("/health-check", get(health_check))
}

/// So a container that serves only this series still has a probe.
async fn health_check() -> Json<bool> {
    Json(true)
}
