//! The built frontend. A path that names a file in the build folder is that file; any other GET
//! is `index.html`, so a deep link (`/lists/<id>`, `/lists/<id>/tasks/<id>`) or a reload opens the
//! app, which reads the URL. Vite's hashed bundles (`/assets/…`) never change under one name, so
//! the browser keeps them; `index.html` is always asked again, so a new build is picked up.

use std::path::Path;
use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::{header, Method, StatusCode, Uri},
    response::{IntoResponse, Response},
};

use crate::live::Server;

pub async fn serve(State(server): State<Arc<Server>>, method: Method, uri: Uri) -> Response {
    if method != Method::GET && method != Method::HEAD {
        return StatusCode::METHOD_NOT_ALLOWED.into_response();
    }
    let rel = uri.path().trim_start_matches('/');
    if rel.split('/').any(|c| c == ".." || c.starts_with('.')) {
        return StatusCode::NOT_FOUND.into_response();
    }
    let file = server.dist.join(rel);
    if !rel.is_empty() && file.is_file() {
        let cache = if rel.starts_with("assets/") { "public, max-age=31536000, immutable" } else { "no-cache" };
        return send(&file, cache).await;
    }
    // A missing asset is a 404, not the app.
    if rel.starts_with("assets/") || Path::new(rel).extension().is_some() {
        return StatusCode::NOT_FOUND.into_response();
    }
    send(&server.dist.join("index.html"), "no-cache").await
}

async fn send(path: &Path, cache: &'static str) -> Response {
    match tokio::fs::read(path).await {
        Ok(bytes) => (
            [(header::CONTENT_TYPE, content_type(path)), (header::CACHE_CONTROL, cache)],
            Body::from(bytes),
        )
            .into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

pub fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()).map(str::to_ascii_lowercase).as_deref() {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("json") => "application/json",
        Some("woff2") => "font/woff2",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("pdf") => "application/pdf",
        Some("txt" | "md" | "log" | "csv") => "text/plain; charset=utf-8",
        Some("mp4") => "video/mp4",
        Some("webm") => "video/webm",
        Some("mp3") => "audio/mpeg",
        Some("ico") => "image/x-icon",
        _ => "application/octet-stream",
    }
}
