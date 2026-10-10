//! The OpenAPI document, generated from the routes (see `core::api`: never written by hand),
//! plus Swagger UI (`/docs`) and Scalar (`/sdoc`) over it.

use std::sync::Arc;

use aide::axum::ApiRouter;
use aide::openapi::{Info, OAuth2Flow, OAuth2Flows, OpenApi, SecurityScheme, Server};
use axum::response::Html;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;

use crate::core::api::BEARER;
use crate::core::config::Settings;
use crate::core::state::AppState;

/// Turns the documented API routes into a plain router, and the document they describe.
/// Paths in the document are relative to the API prefix (the `servers` entry).
pub fn finish(api: ApiRouter<AppState>, settings: &Settings) -> (Router<AppState>, Arc<OpenApi>) {
    let mut doc = OpenApi {
        info: Info {
            title: settings.project_name.clone(),
            version: env!("CARGO_PKG_VERSION").into(),
            description: Some(
                "Generated from the routes. Errors are always `{\"detail\": \"...\"}`. \
                 Engine services (`[ENGINE]`, `[EDITOR]`) exist only in local \
                 development. Published web builds are served on `<name>.play.localhost` \
                 (by host, not under this prefix)."
                    .into(),
            ),
            ..Info::default()
        },
        servers: vec![Server {
            url: settings.api_v1_str.clone(),
            ..Server::default()
        }],
        ..OpenApi::default()
    };
    let token_url = format!("{}/base/login/access-token", settings.api_v1_str);
    let router = api.finish_api_with(&mut doc, |api| {
        api.security_scheme(
            BEARER,
            SecurityScheme::OAuth2 {
                flows: OAuth2Flows {
                    password: Some(OAuth2Flow::Password {
                        refresh_url: None,
                        token_url,
                        scopes: Default::default(),
                    }),
                    ..OAuth2Flows::default()
                },
                description: None,
                extensions: Default::default(),
            },
        )
    });
    // axum's catch-all `{*path}` is an ordinary `{path}` parameter in OpenAPI.
    if let Some(paths) = doc.paths.as_mut() {
        paths.paths = std::mem::take(&mut paths.paths)
            .into_iter()
            .map(|(path, item)| (path.replace("{*", "{"), item))
            .collect();
    }
    (router, Arc::new(doc))
}

fn swagger_html(title: &str, spec_url: &str) -> String {
    format!(
        r##"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<title>{title} - Swagger UI</title>
<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui.css">
</head>
<body>
<div id="swagger-ui"></div>
<script src="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
<script>
window.ui = SwaggerUIBundle({{ url: "{spec_url}", dom_id: "#swagger-ui", persistAuthorization: true }});
</script>
</body>
</html>"##,
        title = title,
        spec_url = spec_url
    )
}

fn scalar_html(title: &str, spec_url: &str) -> String {
    let configuration = json!({
        "theme": "elysiajs",
        "layout": "modern",
        "persistAuth": true,
        "authentication": { "preferredSecurityScheme": BEARER }
    })
    .to_string();
    format!(
        r##"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<title>{title} - Scalar</title>
<meta name="viewport" content="width=device-width, initial-scale=1">
</head>
<body>
<script id="api-reference" data-url="{spec_url}" data-configuration='{configuration}'></script>
<script src="https://cdn.jsdelivr.net/npm/@scalar/api-reference"></script>
</body>
</html>"##,
        title = title,
        spec_url = spec_url,
        configuration = configuration
    )
}

/// `/docs`, `/sdoc` and `<api prefix>/openapi.json`.
pub fn router<S>(settings: &Settings, doc: Arc<OpenApi>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    let spec_url = format!("{}/openapi.json", settings.api_v1_str);
    let swagger = swagger_html(&settings.project_name, &spec_url);
    let scalar = scalar_html(&settings.project_name, &spec_url);

    Router::new()
        .route(
            "/docs",
            get(move || {
                let page = swagger.clone();
                async move { Html(page) }
            }),
        )
        .route(
            "/sdoc",
            get(move || {
                let page = scalar.clone();
                async move { Html(page) }
            }),
        )
        .route(
            &spec_url,
            get(move || {
                let doc = doc.clone();
                async move { Json(doc) }
            }),
        )
}
