//! The tauri target's backend. The Taskscape frontend runs in Chromium, where there is no Tauri
//! runtime; its `invoke(…)` calls are routed by Tauri's `mockIPC` (in the harness page) to
//! `POST /invoke` here, and each one is answered the way the app's own command does it: with
//! `taskscape_common`'s `Store` on a database the fixture loader wrote (`POST /fixture`). No
//! command is faked in memory. What a command does to the user's data is also reported as the
//! contracts' domain events (`taskCreated`, `noteChanged`, …), in the fixture's names.
//!
//! The commands are those of `main/src-tauri/src/lib.rs` and the tray's `src-tauri/src/*.rs`,
//! minus the window plumbing (presenting, revealing, theming the native window), which the
//! harness answers itself. Commands are applied one at a time, in arrival order, as the app's
//! note-write queue applies autosaves.
//!
//! `taskscape-tauri-target-server <dist folder> <data folder>`: serves the built harness from
//! `<dist folder>`, loads fixtures under `<data folder>`, and prints `listening <port>`.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{anyhow, bail, Result};
use axum::{
    body::Body,
    extract::{Query, State},
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use taskscape_common::{attachments, hotkeys, storage::strip_html, Store, Task};
use tokio::sync::Mutex;

struct Session {
    store: Store,
    /// Store id → fixture key, for events.
    names: HashMap<String, String>,
    home: PathBuf,
}

struct App {
    dist: PathBuf,
    data: PathBuf,
    session: Mutex<Option<Session>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let (Some(dist), Some(data)) = (args.next(), args.next()) else {
        bail!("usage: taskscape-tauri-target-server <dist folder> <data folder>");
    };
    let app = Arc::new(App { dist: dist.into(), data: data.into(), session: Mutex::new(None) });
    let router = Router::new()
        .route("/fixture", post(fixture))
        .route("/invoke", post(invoke))
        .route("/asset", get(asset))
        .fallback(get(static_file))
        .with_state(app);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    // The adapter reads this one line; everything else this process says goes to stderr.
    println!("listening {}", listener.local_addr()?.port());
    axum::serve(listener, router).await?;
    Ok(())
}

#[derive(Deserialize)]
struct FixtureReq {
    seed: String,
}

/// A fresh data folder with one fixture in it; the previous one is closed and removed.
async fn fixture(State(app): State<Arc<App>>, Json(req): Json<FixtureReq>) -> Response {
    let mut session = app.session.lock().await;
    if let Some(old) = session.take() {
        let home = old.home.clone();
        drop(old);
        let _ = std::fs::remove_dir_all(home);
    }
    match taskscape_fixtures::load(&req.seed, &app.data).await {
        Ok(loaded) => {
            let names = loaded.keys.iter().map(|(k, id)| (id.clone(), k.clone())).collect();
            let keys = json!(loaded.keys);
            *session = Some(Session { store: loaded.store, names, home: loaded.home });
            Json(json!({ "keys": keys })).into_response()
        }
        Err(e) => {
            eprintln!("[server] fixture {}: {e:#}", req.seed);
            (StatusCode::BAD_REQUEST, format!("{e:#}")).into_response()
        }
    }
}

#[derive(Deserialize)]
struct InvokeReq {
    cmd: String,
    #[serde(default)]
    args: Value,
}

async fn invoke(State(app): State<Arc<App>>, Json(req): Json<InvokeReq>) -> Response {
    let session = app.session.lock().await;
    let Some(s) = session.as_ref() else {
        return (StatusCode::CONFLICT, "no fixture loaded").into_response();
    };
    let mut events = Vec::new();
    let result = dispatch(s, &req.cmd, &req.args, &mut events).await;
    match result {
        Ok(value) => Json(json!({ "ok": value, "events": events })).into_response(),
        Err(e) => {
            eprintln!("[server] {} failed: {e:#}", req.cmd);
            // A command's error is a plain string in Tauri (`Result<T, String>`), and the frontend
            // gets it as the rejection.
            Json(json!({ "error": format!("{e:#}"), "events": events })).into_response()
        }
    }
}

/// A file the app stored (a copied attachment), for `convertFileSrc`: only under the data folder.
#[derive(Deserialize)]
struct AssetReq {
    path: String,
}

async fn asset(State(app): State<Arc<App>>, Query(q): Query<AssetReq>) -> Response {
    let session = app.session.lock().await;
    let Some(s) = session.as_ref() else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let path = PathBuf::from(&q.path);
    let inside = path.canonicalize().ok().zip(s.home.canonicalize().ok()).is_some_and(|(p, h)| p.starts_with(h));
    if !inside {
        return StatusCode::FORBIDDEN.into_response();
    }
    file_response(&path)
}

async fn static_file(State(app): State<Arc<App>>, uri: Uri) -> Response {
    let rel = uri.path().trim_start_matches('/');
    let rel = if rel.is_empty() { "harness.html" } else { rel };
    if rel.split('/').any(|c| c == "..") {
        return StatusCode::FORBIDDEN.into_response();
    }
    file_response(&app.dist.join(rel))
}

fn file_response(path: &std::path::Path) -> Response {
    let Ok(bytes) = std::fs::read(path) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let kind = match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        "ttf" => "font/ttf",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "svg" => "image/svg+xml",
        "json" => "application/json",
        _ => "application/octet-stream",
    };
    ([(header::CONTENT_TYPE, kind), (header::CACHE_CONTROL, "no-store")], Body::from(bytes)).into_response()
}

// ---- the commands --------------------------------------------------------------------------

struct Ev<'a> {
    s: &'a Session,
    out: &'a mut Vec<Value>,
}

impl Ev<'_> {
    /// The fixture's key for a store id, or the id itself (something made during the test).
    fn name(&self, id: &str) -> String {
        self.s.names.get(id).cloned().unwrap_or_else(|| id.to_string())
    }
    fn push(&mut self, name: &str, fields: Value) {
        self.out.push(json!({ "name": name, "fields": fields }));
    }
    fn task(&mut self, name: &str, id: &str) {
        let id = self.name(id);
        self.push(name, json!({ "id": id }));
    }
}

fn arg<T: serde::de::DeserializeOwned>(args: &Value, key: &str) -> Result<T> {
    let v = args.get(key).cloned().unwrap_or(Value::Null);
    serde_json::from_value(v).map_err(|e| anyhow!("argument `{key}`: {e}"))
}

fn opt<T: serde::de::DeserializeOwned>(args: &Value, key: &str) -> Result<Option<T>> {
    match args.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => serde_json::from_value(v.clone()).map(Some).map_err(|e| anyhow!("argument `{key}`: {e}")),
    }
}

fn to<T: serde::Serialize>(v: T) -> Result<Value> {
    Ok(serde_json::to_value(v)?)
}

async fn dispatch(s: &Session, cmd: &str, args: &Value, out: &mut Vec<Value>) -> Result<Value> {
    let store = &s.store;
    let mut ev = Ev { s, out };
    match cmd {
        // projects
        "list_projects" => to(store.list_projects().await?),
        "create_project" => {
            let p = store.create_project(&arg::<String>(args, "name")?).await?;
            let id = ev.name(&p.id);
            ev.push("projectCreated", json!({ "id": id, "name": p.name }));
            to(p)
        }
        "rename_project" => {
            let (id, name): (String, String) = (arg(args, "id")?, arg(args, "name")?);
            store.rename_project(&id, &name).await?;
            let id = ev.name(&id);
            ev.push("projectRenamed", json!({ "id": id, "name": name }));
            Ok(Value::Null)
        }
        "delete_project" => {
            let id: String = arg(args, "id")?;
            store.delete_project(&id).await?;
            ev.task("projectDeleted", &id);
            Ok(Value::Null)
        }
        "default_project" => to(store.default_project().await?),

        // lists
        "list_lists" => to(store.list_lists().await?),
        "create_list" => {
            let (project, name): (String, String) = (arg(args, "projectId")?, arg(args, "name")?);
            let l = store.create_list(&project, &name).await?;
            let (id, project) = (ev.name(&l.id), ev.name(&project));
            ev.push("listCreated", json!({ "id": id, "name": l.name, "projectId": project }));
            to(l)
        }
        "rename_list" => {
            let (id, name): (String, String) = (arg(args, "id")?, arg(args, "name")?);
            store.rename_list(&id, &name).await?;
            let id = ev.name(&id);
            ev.push("listRenamed", json!({ "id": id, "name": name }));
            Ok(Value::Null)
        }
        "delete_list" => {
            let id: String = arg(args, "id")?;
            store.delete_list(&id).await?;
            ev.task("listDeleted", &id);
            Ok(Value::Null)
        }
        "reorder_list" => {
            let id: String = arg(args, "id")?;
            store.reorder_list(&id, arg(args, "sortOrder")?).await?;
            ev.task("listReordered", &id);
            Ok(Value::Null)
        }

        // tasks
        "list_tasks" => to(store.list_tasks(&arg::<String>(args, "listId")?).await?),
        "all_tasks" => to(store.all_tasks().await?),
        "create_task" => {
            let list: String = arg(args, "listId")?;
            let title: String = arg(args, "title")?;
            let notes: Option<String> = opt(args, "notes")?;
            let parent: Option<String> = opt(args, "parentId")?;
            let t = store.create_task(&list, &title, notes.as_deref(), parent.as_deref()).await?;
            created(&mut ev, &t);
            to(t)
        }
        "update_task" => {
            let id: String = arg(args, "id")?;
            let title: Option<String> = opt(args, "title")?;
            let notes: Option<String> = opt(args, "notes")?;
            let done: Option<bool> = opt(args, "done")?;
            let t = store.update_task(&id, title.as_deref(), notes.as_deref(), done).await?;
            let name = ev.name(&id);
            if let Some(title) = title {
                ev.push("taskRenamed", json!({ "id": name, "title": title }));
            }
            if let Some(done) = done {
                ev.push("taskCompleted", json!({ "id": name, "done": done }));
            }
            to(t)
        }
        "delete_task" => {
            let id: String = arg(args, "id")?;
            let stamped = store.soft_delete_tasks(std::slice::from_ref(&id)).await?;
            ev.task("taskTrashed", &id);
            to(stamped)
        }
        "delete_tasks" => {
            let ids: Vec<String> = arg(args, "ids")?;
            let stamped = store.soft_delete_tasks(&ids).await?;
            for id in &ids {
                ev.task("taskTrashed", id);
            }
            to(stamped)
        }
        "move_task" => {
            let id: String = arg(args, "id")?;
            let parent: Option<String> = opt(args, "parentId")?;
            let list: Option<String> = opt(args, "listId")?;
            let t = store.move_task(&id, parent.as_deref(), list.as_deref(), opt(args, "sortOrder")?).await?;
            let (name, list, parent) =
                (ev.name(&t.id), ev.name(&t.list_id), t.parent_id.as_deref().map(|p| ev.name(p)).unwrap_or_default());
            ev.push("taskMoved", json!({ "id": name, "listId": list, "parentId": parent }));
            to(t)
        }
        "reorder_task" => {
            let id: String = arg(args, "id")?;
            let t = store.reorder_task(&id, arg(args, "sortOrder")?).await?;
            ev.task("taskReordered", &id);
            to(t)
        }
        "restore_tasks" => {
            let ids: Vec<String> = arg(args, "ids")?;
            store.restore_tasks(&ids).await?;
            for id in &ids {
                ev.task("taskRestored", id);
            }
            Ok(Value::Null)
        }
        "purge_tasks" => {
            let ids: Vec<String> = arg(args, "ids")?;
            store.purge_tasks(&ids).await?;
            for id in &ids {
                ev.task("taskPurged", id);
            }
            Ok(Value::Null)
        }
        "list_trashed" => to(store.list_trashed().await?),
        "empty_trash" => {
            store.empty_trash().await?;
            ev.push("trashEmptied", json!({}));
            Ok(Value::Null)
        }
        "set_tasks_done" => {
            let ids: Vec<String> = arg(args, "ids")?;
            let done: bool = arg(args, "done")?;
            for id in &ids {
                store.update_task(id, None, None, Some(done)).await?;
                let name = ev.name(id);
                ev.push("taskCompleted", json!({ "id": name, "done": done }));
            }
            Ok(Value::Null)
        }
        "copy_tasks" => {
            let ids: Vec<String> = arg(args, "ids")?;
            let text = copy_tasks(&store.all_tasks().await?, &ids);
            ev.push("copied", json!({ "text": text }));
            to(text)
        }
        "export_list" | "import_list" => {
            bail!("`{cmd}` reads or writes a file the user picks; this target's file dialogs always cancel")
        }

        // notes
        "list_notes" => to(store.list_notes(&arg::<String>(args, "taskId")?).await?),
        "create_note" => {
            let task: String = arg(args, "taskId")?;
            let content: String = arg(args, "content")?;
            let n = store.create_note(&task, &content).await?;
            let task = ev.name(&task);
            ev.push("noteAdded", json!({ "taskId": task, "text": strip_html(&content) }));
            to(n)
        }
        "update_note" => {
            let id: String = arg(args, "id")?;
            let content: String = arg(args, "content")?;
            // The app queues these and returns at once; its worker logs and drops a failed one.
            match store.update_note(&id, &content).await {
                Ok(n) => {
                    let task = ev.name(&n.task_id);
                    ev.push("noteChanged", json!({ "taskId": task, "text": strip_html(&content) }));
                }
                Err(e) => eprintln!("[server] queued note update failed ({id}): {e}"),
            }
            Ok(Value::Null)
        }
        "delete_note" => {
            let id: String = arg(args, "id")?;
            let task = note_task(store, &id).await?;
            store.delete_note(&id).await?;
            if let Some(task) = task {
                let task = ev.name(&task);
                ev.push("noteDeleted", json!({ "taskId": task }));
            }
            Ok(Value::Null)
        }

        // attachments
        "list_attachments" => to(store.list_attachments(&arg::<String>(args, "taskId")?).await?),
        "add_reference" => {
            let (task, name, location): (String, String, String) =
                (arg(args, "taskId")?, arg(args, "name")?, arg(args, "location")?);
            let a = attachments::attach_reference(store, &task, &name, &location).await?;
            attachment(&mut ev, "attachmentAdded", &a.task_id, &a.name);
            to(a)
        }
        "add_copy" => {
            let task: String = arg(args, "taskId")?;
            let source: String = arg(args, "sourcePath")?;
            let name: Option<String> = opt(args, "name")?;
            let a = attachments::attach_copy(store, &task, &source, name.as_deref()).await?;
            attachment(&mut ev, "attachmentAdded", &a.task_id, &a.name);
            to(a)
        }
        "delete_attachment" => {
            let id: String = arg(args, "id")?;
            let a = store.get_attachment(&id).await?;
            store.delete_attachment(&id).await?;
            attachment(&mut ev, "attachmentRemoved", &a.task_id, &a.name);
            Ok(Value::Null)
        }
        "rename_attachment" => {
            let (id, name): (String, String) = (arg(args, "id")?, arg(args, "name")?);
            let a = attachments::rename_attachment(store, &id, &name).await?;
            attachment(&mut ev, "attachmentRenamed", &a.task_id, &a.name);
            to(a)
        }
        // macOS's `screencapture`: on any other OS the app's own capture says it can't.
        "capture_screenshot" => {
            to(taskscape_common::screenshot::capture()?.map(|p| p.to_string_lossy().into_owned()))
        }
        "attach_screenshot" => {
            let task: String = arg(args, "taskId")?;
            let Some(path) = taskscape_common::screenshot::capture()? else {
                return Ok(Value::Null);
            };
            let a = attachments::attach_copy(store, &task, &path.to_string_lossy(), Some("screenshot.png")).await?;
            attachment(&mut ev, "attachmentAdded", &a.task_id, &a.name);
            to(a)
        }
        // These hand a file or URL to the OS; here that is reported, not done.
        "open_attachment" | "reveal_attachment" => {
            let location: String = arg(args, "location")?;
            let name = if cmd == "open_attachment" { "attachmentOpened" } else { "attachmentRevealed" };
            ev.push(name, json!({ "location": location }));
            Ok(Value::Null)
        }
        "open_path" => {
            ev.push("pathOpened", json!({ "path": arg::<String>(args, "path")? }));
            Ok(Value::Null)
        }
        "data_paths" => Ok(json!({
            "data_dir": taskscape_common::paths::root_dir(),
            "screenshots_dir": taskscape_common::paths::screenshots_dir(),
        })),

        // app state and settings
        "set_active_list" | "set_active_project" => {
            let id: String = arg(args, "id")?;
            let (key, event) = if cmd == "set_active_list" {
                ("last_active_list", "listSelected")
            } else {
                ("last_active_project", "projectSelected")
            };
            let before = store.get_setting(key).await?;
            store.set_setting(key, &id).await?;
            // The app re-persists the focused list on every pane click; only a change is news.
            if before.as_deref() != Some(id.as_str()) {
                ev.task(event, &id);
            }
            Ok(Value::Null)
        }
        "get_setting" => to(store.get_setting(&arg::<String>(args, "key")?).await?),
        "set_setting" => {
            let (key, value): (String, String) = (arg(args, "key")?, arg(args, "value")?);
            store.set_setting(&key, &value).await?;
            ev.push("settingChanged", json!({ "key": key, "value": value }));
            Ok(Value::Null)
        }
        "list_hotkeys" => to(hotkeys::resolve(store).await?),
        "set_hotkey" => {
            let (id, accel): (String, String) = (arg(args, "id")?, arg(args, "accel")?);
            hotkeys::set_binding(store, &id, &accel).await?;
            ev.push("hotkeyChanged", json!({ "command": id, "accel": accel }));
            Ok(Value::Null)
        }
        "reset_hotkey" => {
            let id: String = arg(args, "id")?;
            hotkeys::reset_binding(store, &id).await?;
            let accel = hotkeys::resolve(store)
                .await?
                .into_iter()
                .find(|b| b.id == id)
                .and_then(|b| b.accel)
                .map(|a| a.to_string())
                .unwrap_or_default();
            ev.push("hotkeyChanged", json!({ "command": id, "accel": accel }));
            Ok(Value::Null)
        }

        // the tray's capture bar (tray/src-tauri/src/task.rs, capture.rs, hotkeys.rs)
        "capture_target" => {
            let list = target_list(store).await?;
            let project = store
                .list_projects()
                .await?
                .into_iter()
                .find(|p| p.id == list.project_id)
                .map(|p| p.name)
                .unwrap_or_default();
            Ok(json!({ "project": project, "list": list.name }))
        }
        "submit_capture" => {
            let title: String = arg(args, "title")?;
            let notes: Option<String> = opt(args, "notes")?;
            let shots: Vec<String> = opt(args, "screenshotPaths")?.unwrap_or_default();
            let list = target_list(store).await?;
            let t = store.create_task(&list.id, &title, None, None).await?;
            created(&mut ev, &t);
            if let Some(html) = notes.as_deref().filter(|s| !s.trim().is_empty()) {
                store.create_note(&t.id, html).await?;
                let task = ev.name(&t.id);
                ev.push("noteAdded", json!({ "taskId": task, "text": strip_html(html) }));
            }
            let multiple = shots.len() > 1;
            for (i, path) in shots.iter().enumerate() {
                let name = if multiple { format!("screenshot-{}.png", i + 1) } else { "screenshot.png".into() };
                let a = attachments::attach_copy(store, &t.id, path, Some(&name)).await?;
                attachment(&mut ev, "attachmentAdded", &a.task_id, &a.name);
            }
            Ok(Value::Null)
        }
        // The tray's get_dark: the saved theme, else the system appearance, which only the page
        // knows here (null: the harness answers from prefers-color-scheme).
        "get_dark" => Ok(match store.get_setting("theme").await?.as_deref() {
            Some("dark") => json!(true),
            Some("light") => json!(false),
            _ => Value::Null,
        }),

        other => bail!("command {other} not found"),
    }
}

fn created(ev: &mut Ev, t: &Task) {
    let (id, list) = (ev.name(&t.id), ev.name(&t.list_id));
    let parent = t.parent_id.as_deref().map(|p| ev.name(p)).unwrap_or_default();
    ev.push("taskCreated", json!({ "id": id, "title": t.title, "listId": list, "parentId": parent }));
}

fn attachment(ev: &mut Ev, name: &str, task: &str, attachment: &str) {
    let task = ev.name(task);
    ev.push(name, json!({ "taskId": task, "name": attachment }));
}

async fn note_task(store: &Store, note: &str) -> Result<Option<String>> {
    for t in store.all_tasks().await? {
        if t.note_items.iter().any(|n| n.id == note) {
            return Ok(Some(t.id));
        }
    }
    Ok(None)
}

/// The tray's `target_list`: the last active list if it still exists, else the first list, else
/// a new list in the default project.
async fn target_list(store: &Store) -> Result<taskscape_common::List> {
    let lists = store.list_lists().await?;
    if let Some(id) = store.get_setting("last_active_list").await? {
        if let Some(list) = lists.iter().find(|l| l.id == id) {
            return Ok(list.clone());
        }
    }
    if let Some(first) = lists.into_iter().next() {
        return Ok(first);
    }
    let project = store.default_project().await?;
    store.create_list(&project.id, &taskscape_common::names::suggest_list_name()).await
}

/// `copy_tasks` from `main/src-tauri/src/lib.rs`: the selected tasks as a Markdown checklist in
/// tree order, a task indented by how many of its ancestors are also selected, its notes as
/// quoted lines under it.
fn copy_tasks(tasks: &[Task], ids: &[String]) -> String {
    let selected: HashSet<&str> = ids.iter().map(String::as_str).collect();
    let mut roots = Vec::new();
    let mut children: HashMap<&str, Vec<&Task>> = HashMap::new();
    for t in tasks {
        match &t.parent_id {
            Some(p) => children.entry(p.as_str()).or_default().push(t),
            None => roots.push(t),
        }
    }
    fn render<'a>(
        node: &'a Task,
        depth: usize,
        children: &HashMap<&'a str, Vec<&'a Task>>,
        selected: &HashSet<&str>,
        out: &mut String,
    ) {
        let picked = selected.contains(node.id.as_str());
        if picked {
            out.push_str(&"  ".repeat(depth));
            out.push_str(if node.done { "- [x] " } else { "- [ ] " });
            out.push_str(&node.title);
            out.push('\n');
            for note in &node.note_items {
                let text = strip_html(&note.content);
                if text.is_empty() {
                    continue;
                }
                out.push_str(&"  ".repeat(depth + 1));
                out.push_str("> ");
                out.push_str(&text);
                out.push('\n');
            }
        }
        let child_depth = depth + usize::from(picked);
        if let Some(kids) = children.get(node.id.as_str()) {
            for k in kids {
                render(k, child_depth, children, selected, out);
            }
        }
    }
    let mut out = String::new();
    for r in &roots {
        render(r, 0, &children, &selected, &mut out);
    }
    out.trim_end().to_string()
}
