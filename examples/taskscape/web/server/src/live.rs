//! The store being served, and the API router built over it. Normally both are made once at
//! start; the test hooks replace them with a fresh fixture for every mount, so the API is reached
//! through `forward`, which always asks the current one.

use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Router,
};
use taskscape_common::{server::data_router, Store};
use tokio::sync::RwLock;
use tower::ServiceExt;

pub struct Live {
    pub store: Arc<Store>,
    router: Router,
}

impl Live {
    /// The common crate's `data_router` and this server's own routes, under `/api`. `wrap` adds
    /// layers over the whole API (the test hooks' event log).
    pub fn new(store: Arc<Store>, wrap: Option<Box<dyn FnOnce(Router) -> Router + Send>>) -> Self {
        let api = data_router(store.clone()).merge(crate::api::routes(store.clone()));
        let api = crate::alerts::layer(api);
        let api = match wrap {
            Some(w) => w(api),
            None => api,
        };
        Live { store, router: Router::new().nest("/api", api) }
    }
}

pub struct Server {
    pub dist: PathBuf,
    live: RwLock<Option<Live>>,
}

impl Server {
    pub fn new(dist: PathBuf) -> Self {
        Server { dist, live: RwLock::new(None) }
    }

    /// Serve `live` from now on; the one it replaces is dropped (its store closed).
    pub async fn install(&self, live: Live) -> Option<Live> {
        self.live.write().await.replace(live)
    }

    #[allow(dead_code)]
    pub async fn store(&self) -> Option<Arc<Store>> {
        self.live.read().await.as_ref().map(|l| l.store.clone())
    }
}

/// Every `/api/…` request, answered by the current API router.
pub async fn forward(State(server): State<Arc<Server>>, req: Request<Body>) -> Response {
    let router = match server.live.read().await.as_ref() {
        Some(live) => live.router.clone(),
        None => return (StatusCode::SERVICE_UNAVAILABLE, "no data loaded").into_response(),
    };
    // A fresh request with the same method, address, headers and body: the outer
    // route's own path parameters (its `{*rest}`) must not reach the API's routes.
    let (parts, body) = req.into_parts();
    let mut fresh = Request::new(body);
    *fresh.method_mut() = parts.method;
    *fresh.uri_mut() = parts.uri;
    *fresh.version_mut() = parts.version;
    *fresh.headers_mut() = parts.headers;
    match router.oneshot(fresh).await {
        Ok(res) => res,
        Err(never) => match never {},
    }
}
