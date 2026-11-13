//! Monitoring: every API request that fails is logged (method, path, status, time, the error),
//! and when `TASKSCAPE_ALERT_URL` is set, a server error is also posted there as JSON
//! (`{"text", "level", "source"}`, with `Authorization: Bearer $TASKSCAPE_ALERT_TOKEN` if that is
//! set), at most one every ten seconds, so whoever runs the server hears about it. The token is
//! read from the environment only; it never reaches the browser.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use axum::{
    body::{to_bytes, Body},
    extract::Request,
    middleware::{self, Next},
    response::Response,
    Router,
};

static LAST_ALERT: Mutex<Option<Instant>> = Mutex::new(None);

pub fn layer(api: Router) -> Router {
    api.layer(middleware::from_fn(watch))
}

async fn watch(req: Request<Body>, next: Next) -> Response {
    let (method, path) = (req.method().clone(), req.uri().path().to_string());
    let started = Instant::now();
    let res = next.run(req).await;
    let status = res.status();
    if status.is_success() {
        return res;
    }
    let ms = started.elapsed().as_millis();
    let (parts, body) = res.into_parts();
    // Errors are short plain-text messages (AppError); keep them for the log and the alert.
    let bytes = to_bytes(body, 64 * 1024).await.unwrap_or_default();
    let message = String::from_utf8_lossy(&bytes).into_owned();
    eprintln!("[taskscape-web] {method} /api{path} -> {} in {ms} ms: {message}", status.as_u16());
    if status.is_server_error() {
        alert(format!("{method} /api{path} failed ({}): {message}", status.as_u16()));
    }
    Response::from_parts(parts, Body::from(bytes))
}

fn alert(text: String) {
    let Ok(url) = std::env::var("TASKSCAPE_ALERT_URL") else { return };
    {
        let mut last = LAST_ALERT.lock().unwrap();
        if last.is_some_and(|t| t.elapsed() < Duration::from_secs(10)) {
            return;
        }
        *last = Some(Instant::now());
    }
    tokio::spawn(async move {
        let mut req = reqwest::Client::new()
            .post(&url)
            .timeout(Duration::from_secs(5))
            .json(&serde_json::json!({ "text": text, "level": "error", "source": "taskscape-web" }));
        if let Ok(token) = std::env::var("TASKSCAPE_ALERT_TOKEN") {
            req = req.bearer_auth(token);
        }
        if let Err(e) = req.send().await.and_then(|r| r.error_for_status()) {
            eprintln!("[taskscape-web] the alert could not be sent: {e}");
        }
    });
}
