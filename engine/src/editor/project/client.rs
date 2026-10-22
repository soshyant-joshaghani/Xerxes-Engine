//! The web editor's calls to the local backend's engine services (`/api/v1/engine/...`):
//! the catalog, project files and New Project. A page cannot touch the disk, so this is the
//! web adapter of `store`; the native editor uses the disk directly and needs no backend.
//! The page reaches the API same-origin (proxied to :8000).

use serde::Serialize;
use serde::de::DeserializeOwned;

/// Shown when nothing answers on the API.
pub const BACKEND_DOWN: &str = "The backend is not reachable.";

/// What a non-2xx answer means for the editor.
pub fn status_error(code: u16, body: &str) -> String {
    match code {
        // The dev proxy answers 5xx when nothing listens on :8000.
        500 | 502 | 503 | 504 => BACKEND_DOWN.to_string(),
        404 if body.is_empty() => {
            "The backend has no engine services (they run with ENVIRONMENT=local).".to_string()
        }
        _ => {
            // The API answers `{"detail": "..."}`.
            let detail = serde_json::from_str::<serde_json::Value>(body)
                .ok()
                .and_then(|v| v["detail"].as_str().map(str::to_string));
            detail.unwrap_or_else(|| format!("HTTP {code}"))
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Method {
    Get,
    Put,
    Post,
    Delete,
}

/// One request to `/api/v1/engine/<path>`; returns the response body.
async fn request(
    method: Method,
    path: &str,
    body: Option<(Vec<u8>, &'static str)>,
) -> Result<Vec<u8>, String> {
    imp::request(method, &format!("/api/v1/engine/{path}"), body).await
}

pub async fn get_json<T: DeserializeOwned>(path: &str) -> Result<T, String> {
    let bytes = request(Method::Get, path, None).await?;
    serde_json::from_slice(&bytes).map_err(|e| format!("unexpected answer from {path}: {e}"))
}

pub async fn get_bytes(path: &str) -> Result<Vec<u8>, String> {
    request(Method::Get, path, None).await
}

pub async fn get_text(path: &str) -> Result<String, String> {
    String::from_utf8(get_bytes(path).await?).map_err(|_| format!("{path} is not UTF-8 text"))
}

pub async fn put_bytes(path: &str, bytes: Vec<u8>) -> Result<(), String> {
    request(Method::Put, path, Some((bytes, "application/octet-stream")))
        .await
        .map(|_| ())
}

pub async fn post_json<B: Serialize, T: DeserializeOwned>(
    path: &str,
    body: &B,
) -> Result<T, String> {
    let body = serde_json::to_vec(body).map_err(|e| e.to_string())?;
    let bytes = request(Method::Post, path, Some((body, "application/json"))).await?;
    if bytes.is_empty() {
        return serde_json::from_slice(b"null").map_err(|e| e.to_string());
    }
    serde_json::from_slice(&bytes).map_err(|e| format!("unexpected answer from {path}: {e}"))
}

pub async fn delete(path: &str) -> Result<(), String> {
    request(Method::Delete, path, None).await.map(|_| ())
}

/// Waits `ms` milliseconds without blocking the UI (polling a job).
pub async fn sleep(ms: u32) {
    imp::sleep(ms).await
}

mod imp {
    pub async fn sleep(ms: u32) {
        gloo_timers::future::TimeoutFuture::new(ms).await;
    }

    use super::{BACKEND_DOWN, Method, status_error};

    pub async fn request(
        method: Method,
        path: &str,
        body: Option<(Vec<u8>, &'static str)>,
    ) -> Result<Vec<u8>, String> {
        let origin = web_sys::window()
            .and_then(|w| w.location().origin().ok())
            .unwrap_or_default();
        let url = format!("{origin}{path}");
        let client = reqwest::Client::new();
        let mut builder = match method {
            Method::Get => client.get(&url),
            Method::Put => client.put(&url),
            Method::Post => client.post(&url),
            Method::Delete => client.delete(&url),
        };
        if let Some((bytes, kind)) = body {
            builder = builder.header("content-type", kind).body(bytes);
        }
        let response = builder.send().await.map_err(|_| BACKEND_DOWN.to_string())?;
        let code = response.status().as_u16();
        let bytes = response.bytes().await.map_err(|e| e.to_string())?.to_vec();
        if (200..300).contains(&code) {
            Ok(bytes)
        } else {
            Err(status_error(code, &String::from_utf8_lossy(&bytes)))
        }
    }
}
