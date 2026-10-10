//! Xerxes backend library (Axum, on the FoxG wire contract). The `api` and `worker` binaries and the
//! integration tests in `../tests/backend` all use this crate.
#![recursion_limit = "256"]

pub mod core;
pub mod modules;

use axum::http::HeaderValue;
use axum::Router;
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tower_http::LatencyUnit;
use tracing::Level;

use crate::core::config::Settings;
use crate::core::error::ApiError;
use crate::core::state::AppState;

async fn not_found() -> ApiError {
    ApiError::not_found("Not Found")
}

fn cors_layer(settings: &Settings) -> CorsLayer {
    let origins: Vec<HeaderValue> = settings
        .all_cors_origins()
        .iter()
        .filter_map(|o| HeaderValue::from_str(o).ok())
        .collect();
    CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_credentials(true)
        .allow_methods(AllowMethods::mirror_request())
        .allow_headers(AllowHeaders::mirror_request())
}

/// The API series under `api_v1_str`, with its OpenAPI document and Swagger/Scalar pages. Every
/// route is documented from its handler (core::api); the spec is never hand-written.
pub fn api_router(settings: &Settings) -> Router<AppState> {
    let (api, doc) = crate::core::openapi::finish(modules::api::router(settings), settings);
    Router::new()
        .nest(&settings.api_v1_str, api)
        .merge(crate::core::openapi::router::<AppState>(settings, doc))
}

/// The multiplayer series under `/multiplayer`.
pub fn multiplayer_router() -> Router<AppState> {
    Router::new().nest(
        modules::multiplayer::PREFIX,
        Router::from(modules::multiplayer::router()),
    )
}

/// Build the full application router from a prepared [`AppState`]: the series `SERVICES` names
/// (both by default), with CORS and the access log.
pub fn build_router(state: AppState) -> Router {
    let settings = state.config.clone();

    // The two series of modules (modules/mod.rs): `SERVICES` picks what this process serves.
    let mut app: Router<AppState> = Router::new();
    if settings.services.api() {
        app = app.merge(api_router(&settings));
    }
    if settings.services.multiplayer() {
        app = app.merge(multiplayer_router());
    }
    let app = app.fallback(not_found);

    let app = if settings.is_local() && settings.services.api() {
        // Published game builds on <name>.play.localhost (engine dev services).
        app.layer(axum::middleware::from_fn_with_state(
            state.clone(),
            modules::api::engine::serve_play,
        ))
    } else {
        app
    };

    app.layer(cors_layer(&settings))
        .layer(access_log_layer())
        .with_state(state)
}

/// One INFO line per request: method, path, status, latency (like uvicorn's access log).
/// Quiet it with `RUST_LOG=info,tower_http=warn`.
fn access_log_layer(
) -> TraceLayer<tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>>
{
    TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
        .on_response(
            DefaultOnResponse::new()
                .level(Level::INFO)
                .latency_unit(LatencyUnit::Millis),
        )
}
