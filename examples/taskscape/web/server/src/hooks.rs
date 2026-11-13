//! What the omnicui target needs from the server, built only with the `test-hooks` feature and
//! switched on with `--test-hooks <folder>`:
//!
//! - `POST /__test/fixture {seed}`: a fresh data folder under `<folder>` with that fixture,
//!   written by the example's fixture loader through `Store`; the previous one is closed and
//!   removed. Answers `{keys}`, the fixture's keys mapped to the store's ids.
//! - `GET /__test/events`: the domain events since the last call (`[{name, fields}]`).
//!
//! The events are what users changed in the store, one per change, in order, named as the
//! contracts name them and with entities named by fixture key (an id the fixture didn't make is
//! passed as it is). They are read off the API's own requests and answers, as a layer over the
//! API router, so the routes themselves carry no test code. The mapping mirrors the tauri
//! target's (`omnicui/targets/tauri/server/src/main.rs`), command for route.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use axum::{
    body::{to_bytes, Body},
    extract::{Request, State},
    http::{Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Extension, Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use taskscape_common::{storage::strip_html, Store};

use crate::live::{Live, Server};

struct Hooks {
    data: PathBuf,
    log: Arc<Mutex<Vec<Value>>>,
    /// The current fixture's data folder, removed when the next one is loaded.
    home: tokio::sync::Mutex<Option<PathBuf>>,
}

pub fn routes(data: PathBuf) -> Router<Arc<Server>> {
    let hooks = Arc::new(Hooks { data, log: Arc::default(), home: tokio::sync::Mutex::new(None) });
    Router::new()
        .route("/__test/fixture", post(fixture))
        .route("/__test/events", get(events))
        .layer(Extension(hooks))
}

#[derive(Deserialize)]
struct FixtureReq {
    seed: String,
}

async fn fixture(
    State(server): State<Arc<Server>>,
    Extension(hooks): Extension<Arc<Hooks>>,
    Json(req): Json<FixtureReq>,
) -> Response {
    // One at a time: the loader points HOME at the new folder.
    let mut home = hooks.home.lock().await;
    let loaded = match taskscape_fixtures::load(&req.seed, &hooks.data).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[taskscape-web] fixture {}: {e:#}", req.seed);
            return (StatusCode::BAD_REQUEST, format!("{e:#}")).into_response();
        }
    };
    let store = Arc::new(loaded.store);
    let rec = Recorder {
        store: store.clone(),
        names: Arc::new(loaded.keys.iter().map(|(k, id)| (id.clone(), k.clone())).collect()),
        log: hooks.log.clone(),
        gate: Arc::default(),
    };
    let old = server
        .install(Live::new(store, Some(Box::new(move |api| api.layer(middleware::from_fn_with_state(rec, record))))))
        .await;
    drop(old);
    hooks.log.lock().unwrap().clear();
    if let Some(prev) = home.replace(loaded.home) {
        let _ = std::fs::remove_dir_all(prev);
    }
    Json(json!({ "keys": loaded.keys })).into_response()
}

async fn events(Extension(hooks): Extension<Arc<Hooks>>) -> Json<Vec<Value>> {
    Json(std::mem::take(&mut *hooks.log.lock().unwrap()))
}

#[derive(Clone)]
struct Recorder {
    store: Arc<Store>,
    /// Store id → fixture key.
    names: Arc<HashMap<String, String>>,
    log: Arc<Mutex<Vec<Value>>>,
    /// Writes one at a time, in the order they arrive, as the desktop app's commands run.
    gate: Arc<tokio::sync::Mutex<()>>,
}

impl Recorder {
    fn name(&self, id: &str) -> String {
        self.names.get(id).cloned().unwrap_or_else(|| id.to_string())
    }
    fn name_of(&self, v: &Value) -> String {
        self.name(v.as_str().unwrap_or(""))
    }
}

/// What a request needs from the store before it runs (what a delete removes).
#[derive(Default)]
struct Before {
    task: Option<String>,
    name: Option<String>,
    setting: Option<String>,
}

async fn record(State(rec): State<Recorder>, req: Request<Body>, next: Next) -> Response {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let (parts, body) = req.into_parts();
    let bytes = to_bytes(body, usize::MAX).await.unwrap_or_default();
    let input: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    let segs: Vec<String> = path.trim_matches('/').split('/').map(String::from).collect();
    let segs: Vec<&str> = segs.iter().map(String::as_str).collect();
    let _turn = if method == Method::GET { None } else { Some(rec.gate.clone().lock_owned().await) };
    let before = before(&rec, &method, &segs).await;

    let res = next.run(Request::from_parts(parts, Body::from(bytes))).await;
    if !res.status().is_success() || method == Method::GET {
        return res;
    }
    let (parts, body) = res.into_parts();
    let bytes = to_bytes(body, usize::MAX).await.unwrap_or_default();
    let output: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    let events = derive(&rec, &method, &segs, &input, &output, before);
    rec.log.lock().unwrap().extend(events.into_iter().map(|(n, f)| json!({ "name": n, "fields": f })));
    Response::from_parts(parts, Body::from(bytes))
}

async fn before(rec: &Recorder, method: &Method, segs: &[&str]) -> Before {
    let s = &rec.store;
    match (method.as_str(), segs) {
        ("DELETE", ["notes", id]) => {
            let task = s
                .all_tasks()
                .await
                .ok()
                .and_then(|ts| ts.into_iter().find(|t| t.note_items.iter().any(|n| n.id == *id)).map(|t| t.id));
            Before { task, ..Default::default() }
        }
        ("DELETE", ["attachments", id]) => match s.get_attachment(id).await {
            Ok(a) => Before { task: Some(a.task_id), name: Some(a.name), ..Default::default() },
            Err(_) => Before::default(),
        },
        ("PUT", ["settings", key]) => Before { setting: s.get_setting(key).await.ok().flatten(), ..Default::default() },
        _ => Before::default(),
    }
}

fn derive(rec: &Recorder, method: &Method, segs: &[&str], input: &Value, out: &Value, before: Before) -> Vec<(&'static str, Value)> {
    let n = |id: &str| rec.name(id);
    let nv = |v: &Value| rec.name_of(v);
    let each = |name: &'static str| -> Vec<(&'static str, Value)> {
        input["ids"].as_array().into_iter().flatten().map(|id| (name, json!({ "id": nv(id) }))).collect()
    };
    let task_created = |t: &Value| {
        let parent = if t["parent_id"].is_string() { nv(&t["parent_id"]) } else { String::new() };
        ("taskCreated", json!({ "id": nv(&t["id"]), "title": t["title"], "listId": nv(&t["list_id"]), "parentId": parent }))
    };
    match (method.as_str(), segs) {
        ("POST", ["projects"]) => vec![("projectCreated", json!({ "id": nv(&out["id"]), "name": out["name"] }))],
        ("PATCH", ["projects", id]) => vec![("projectRenamed", json!({ "id": n(id), "name": input["name"] }))],
        ("DELETE", ["projects", id]) => vec![("projectDeleted", json!({ "id": n(id) }))],
        ("POST", ["projects", _, "import"]) => vec![("listImported", json!({ "id": nv(&out["id"]), "name": out["name"] }))],
        ("POST", ["lists"]) => vec![(
            "listCreated",
            json!({ "id": nv(&out["id"]), "name": out["name"], "projectId": nv(&out["project_id"]) }),
        )],
        ("PATCH", ["lists", id]) => {
            let mut v = Vec::new();
            if input["name"].is_string() {
                v.push(("listRenamed", json!({ "id": n(id), "name": input["name"] })));
            }
            if input["sort_order"].is_number() {
                v.push(("listReordered", json!({ "id": n(id) })));
            }
            v
        }
        ("DELETE", ["lists", id]) => vec![("listDeleted", json!({ "id": n(id) }))],
        ("POST", ["tasks"]) => vec![task_created(out)],
        ("PATCH", ["tasks", id]) => {
            let mut v = Vec::new();
            if input["title"].is_string() {
                v.push(("taskRenamed", json!({ "id": n(id), "title": input["title"] })));
            }
            if let Some(done) = input["done"].as_bool() {
                v.push(("taskCompleted", json!({ "id": n(id), "done": done })));
            }
            v
        }
        ("POST", ["tasks", "trash"]) => each("taskTrashed"),
        ("POST", ["tasks", "done"]) => {
            let done = input["done"].as_bool().unwrap_or(false);
            input["ids"].as_array().into_iter().flatten().map(|id| ("taskCompleted", json!({ "id": nv(id), "done": done }))).collect()
        }
        ("POST", ["tasks", "copy"]) => vec![("copied", json!({ "text": out }))],
        ("POST", ["tasks", _, "move"]) => {
            let parent = if out["parent_id"].is_string() { nv(&out["parent_id"]) } else { String::new() };
            vec![("taskMoved", json!({ "id": nv(&out["id"]), "listId": nv(&out["list_id"]), "parentId": parent }))]
        }
        ("POST", ["tasks", id, "reorder"]) => vec![("taskReordered", json!({ "id": n(id) }))],
        ("POST", ["trash", "restore"]) => each("taskRestored"),
        ("POST", ["trash", "purge"]) => each("taskPurged"),
        ("DELETE", ["trash"]) => vec![("trashEmptied", json!({}))],
        ("POST", ["tasks", id, "notes"]) => {
            vec![("noteAdded", json!({ "taskId": n(id), "text": strip_html(input["content"].as_str().unwrap_or("")) }))]
        }
        ("PATCH", ["notes", _]) => vec![(
            "noteChanged",
            json!({ "taskId": nv(&out["task_id"]), "text": strip_html(input["content"].as_str().unwrap_or("")) }),
        )],
        ("DELETE", ["notes", _]) => before.task.map(|t| ("noteDeleted", json!({ "taskId": n(&t) }))).into_iter().collect(),
        ("POST", ["tasks", _, "attachments"] | ["tasks", _, "attachments", "upload"]) => {
            vec![("attachmentAdded", json!({ "taskId": nv(&out["task_id"]), "name": out["name"] }))]
        }
        ("PATCH", ["attachments", _]) => {
            vec![("attachmentRenamed", json!({ "taskId": nv(&out["task_id"]), "name": out["name"] }))]
        }
        ("DELETE", ["attachments", _]) => match (before.task, before.name) {
            (Some(t), Some(name)) => vec![("attachmentRemoved", json!({ "taskId": n(&t), "name": name }))],
            _ => vec![],
        },
        ("PUT", ["settings", key]) => {
            let value = input["value"].as_str().unwrap_or("");
            let selected = match *key {
                "last_active_list" => Some("listSelected"),
                "last_active_project" => Some("projectSelected"),
                _ => None,
            };
            match selected {
                // The app records the focused list on every pane click; only a change is news.
                Some(event) if before.setting.as_deref() != Some(value) => vec![(event, json!({ "id": n(value) }))],
                Some(_) => vec![],
                None => vec![("settingChanged", json!({ "key": key, "value": value }))],
            }
        }
        ("PUT", ["hotkeys", id]) => vec![("hotkeyChanged", json!({ "command": id, "accel": input["accel"] }))],
        ("DELETE", ["hotkeys", id]) => vec![("hotkeyChanged", json!({ "command": id, "accel": out["accel"] }))],
        _ => vec![],
    }
}
