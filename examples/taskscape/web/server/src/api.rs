//! The routes the app needs beyond `data_router` (which has lists and tasks: `GET/POST /lists`,
//! `GET /lists/{id}/tasks`, `GET/POST /tasks`, `PATCH /tasks/{id}`, `GET/POST
//! /tasks/{id}/attachments`). One route per Tauri command of the desktop app
//! (`tauri/main/src-tauri/src/lib.rs`), each a thin wrapper over `Store` as the command is, minus
//! what a browser does itself: windows, the system's file dialogs, screenshots and opening files
//! with the OS. Files come in by upload instead of by path, and go out by URL.

use std::path::Path as FsPath;
use std::sync::Arc;

use axum::{
    body::{Body, Bytes},
    extract::{DefaultBodyLimit, Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, patch, post, put},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use taskscape_common::{attachments, paths, LinkType, Store};

type S = State<Arc<Store>>;
type ApiResult<T> = Result<T, ApiError>;

/// Any failure: a 500 with the message as plain text (as `data_router`'s own `AppError`), which
/// the frontend rejects with, as Tauri rejects with a command's error string.
pub struct ApiError(anyhow::Error);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{:#}", self.0)).into_response()
    }
}

impl<E: Into<anyhow::Error>> From<E> for ApiError {
    fn from(e: E) -> Self {
        ApiError(e.into())
    }
}

pub fn routes(store: Arc<Store>) -> Router {
    Router::new()
        // projects
        .route("/projects", get(list_projects).post(create_project))
        .route("/projects/default", get(default_project))
        .route("/projects/{id}", patch(rename_project).delete(delete_project))
        .route("/projects/{id}/import", post(import_list))
        // lists (data_router has GET/POST /lists)
        .route("/lists/{id}", patch(update_list).delete(delete_list))
        .route("/lists/{id}/export", get(export_list))
        // tasks (data_router has GET/POST /tasks and PATCH /tasks/{id})
        .route("/tasks/{id}/move", post(move_task))
        .route("/tasks/{id}/reorder", post(reorder_task))
        .route("/tasks/done", post(set_tasks_done))
        .route("/tasks/copy", post(copy_tasks))
        .route("/tasks/trash", post(trash_tasks))
        // the Trash
        .route("/trash", get(list_trashed).delete(empty_trash))
        .route("/trash/restore", post(restore_tasks))
        .route("/trash/purge", post(purge_tasks))
        // notes
        .route("/tasks/{id}/notes", get(list_notes).post(create_note))
        .route("/notes/{id}", patch(update_note).delete(delete_note))
        // attachments (data_router has GET/POST /tasks/{id}/attachments, by reference)
        .route(
            "/tasks/{id}/attachments/upload",
            post(upload_attachment).layer(DefaultBodyLimit::max(64 * 1024 * 1024)),
        )
        .route("/attachments/{id}", patch(rename_attachment).delete(delete_attachment))
        .route("/attachments/{id}/file", get(attachment_file))
        // settings, shortcuts, where the data is
        .route("/settings/{key}", put(set_setting).get(get_setting))
        .route("/hotkeys", get(list_hotkeys))
        .route("/hotkeys/{id}", put(set_hotkey).delete(reset_hotkey))
        .route("/paths", get(data_paths))
        .with_state(store)
}

#[derive(Deserialize)]
struct Name {
    name: String,
}

#[derive(Deserialize)]
struct Ids {
    ids: Vec<String>,
}

// ---- projects ------------------------------------------------------------------------------

async fn list_projects(State(s): S) -> ApiResult<impl IntoResponse> {
    Ok(Json(s.list_projects().await?))
}

async fn create_project(State(s): S, Json(b): Json<Name>) -> ApiResult<impl IntoResponse> {
    Ok(Json(s.create_project(&b.name).await?))
}

async fn default_project(State(s): S) -> ApiResult<impl IntoResponse> {
    Ok(Json(s.default_project().await?))
}

async fn rename_project(State(s): S, Path(id): Path<String>, Json(b): Json<Name>) -> ApiResult<StatusCode> {
    s.rename_project(&id, &b.name).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_project(State(s): S, Path(id): Path<String>) -> ApiResult<StatusCode> {
    s.delete_project(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct ImportReq {
    #[serde(default)]
    name: Option<String>,
    /// The exported document itself (the file the user picked, read by the browser).
    data: Value,
}

async fn import_list(State(s): S, Path(project): Path<String>, Json(b): Json<ImportReq>) -> ApiResult<impl IntoResponse> {
    Ok(Json(crate::lists::import(&s, &project, b.data, b.name).await?))
}

// ---- lists ---------------------------------------------------------------------------------

#[derive(Deserialize)]
struct ListPatch {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    sort_order: Option<f64>,
}

async fn update_list(State(s): S, Path(id): Path<String>, Json(b): Json<ListPatch>) -> ApiResult<StatusCode> {
    if let Some(name) = &b.name {
        s.rename_list(&id, name).await?;
    }
    if let Some(order) = b.sort_order {
        s.reorder_list(&id, order).await?;
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_list(State(s): S, Path(id): Path<String>) -> ApiResult<StatusCode> {
    s.delete_list(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// The list as the desktop app's JSON export, as a download named after the list.
async fn export_list(State(s): S, Path(id): Path<String>) -> ApiResult<Response> {
    let (name, doc) = crate::lists::export(&s, &id).await?;
    let file = format!("{}.json", name.replace(['"', '/', '\\'], "_"));
    Ok((
        [
            (header::CONTENT_TYPE, "application/json".to_string()),
            (header::CONTENT_DISPOSITION, format!("attachment; filename=\"{file}\"")),
        ],
        serde_json::to_string_pretty(&doc)?,
    )
        .into_response())
}

// ---- tasks ---------------------------------------------------------------------------------

#[derive(Deserialize)]
struct MoveReq {
    #[serde(default)]
    parent_id: Option<String>,
    #[serde(default)]
    list_id: Option<String>,
    #[serde(default)]
    sort_order: Option<f64>,
}

async fn move_task(State(s): S, Path(id): Path<String>, Json(b): Json<MoveReq>) -> ApiResult<impl IntoResponse> {
    Ok(Json(s.move_task(&id, b.parent_id.as_deref(), b.list_id.as_deref(), b.sort_order).await?))
}

#[derive(Deserialize)]
struct Order {
    sort_order: f64,
}

async fn reorder_task(State(s): S, Path(id): Path<String>, Json(b): Json<Order>) -> ApiResult<impl IntoResponse> {
    Ok(Json(s.reorder_task(&id, b.sort_order).await?))
}

#[derive(Deserialize)]
struct DoneReq {
    ids: Vec<String>,
    done: bool,
}

async fn set_tasks_done(State(s): S, Json(b): Json<DoneReq>) -> ApiResult<StatusCode> {
    for id in &b.ids {
        s.update_task(id, None, None, Some(b.done)).await?;
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn copy_tasks(State(s): S, Json(b): Json<Ids>) -> ApiResult<impl IntoResponse> {
    Ok(Json(crate::lists::copy_markdown(&s.all_tasks().await?, &b.ids)))
}

/// Soft delete (to the Trash); the ids stamped, subtrees included, for undo. (`data_router`'s
/// `DELETE /tasks/{id}` deletes for good, which the app never does.)
async fn trash_tasks(State(s): S, Json(b): Json<Ids>) -> ApiResult<impl IntoResponse> {
    Ok(Json(s.soft_delete_tasks(&b.ids).await?))
}

async fn list_trashed(State(s): S) -> ApiResult<impl IntoResponse> {
    Ok(Json(s.list_trashed().await?))
}

async fn empty_trash(State(s): S) -> ApiResult<StatusCode> {
    s.empty_trash().await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn restore_tasks(State(s): S, Json(b): Json<Ids>) -> ApiResult<StatusCode> {
    s.restore_tasks(&b.ids).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn purge_tasks(State(s): S, Json(b): Json<Ids>) -> ApiResult<StatusCode> {
    s.purge_tasks(&b.ids).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ---- notes ---------------------------------------------------------------------------------

#[derive(Deserialize)]
struct Content {
    content: String,
}

async fn list_notes(State(s): S, Path(task): Path<String>) -> ApiResult<impl IntoResponse> {
    Ok(Json(s.list_notes(&task).await?))
}

async fn create_note(State(s): S, Path(task): Path<String>, Json(b): Json<Content>) -> ApiResult<impl IntoResponse> {
    Ok(Json(s.create_note(&task, &b.content).await?))
}

async fn update_note(State(s): S, Path(id): Path<String>, Json(b): Json<Content>) -> ApiResult<impl IntoResponse> {
    Ok(Json(s.update_note(&id, &b.content).await?))
}

async fn delete_note(State(s): S, Path(id): Path<String>) -> ApiResult<StatusCode> {
    s.delete_note(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ---- attachments ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct UploadQuery {
    name: String,
}

/// A file the user uploaded or dropped: the request body is the file. It is written to a
/// temporary file in the data folder and copied in as the desktop app copies a picked file
/// (`attach_copy`), so it is stored the same way.
async fn upload_attachment(
    State(s): S,
    Path(task): Path<String>,
    Query(q): Query<UploadQuery>,
    body: Bytes,
) -> ApiResult<impl IntoResponse> {
    let name = FsPath::new(&q.name).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    if name.trim().is_empty() {
        return Err(anyhow::anyhow!("an upload needs a file name").into());
    }
    paths::ensure_dirs()?;
    let tmp = paths::root_dir().join(format!(".upload-{}", taskscape_common::util::new_id()));
    std::fs::write(&tmp, &body)?;
    let result = attachments::attach_copy(&s, &task, &tmp.to_string_lossy(), Some(&name)).await;
    let _ = std::fs::remove_file(&tmp);
    Ok(Json(result?))
}

async fn rename_attachment(State(s): S, Path(id): Path<String>, Json(b): Json<Name>) -> ApiResult<impl IntoResponse> {
    Ok(Json(attachments::rename_attachment(&s, &id, &b.name).await?))
}

async fn delete_attachment(State(s): S, Path(id): Path<String>) -> ApiResult<StatusCode> {
    let a = s.get_attachment(&id).await?;
    s.delete_attachment(&id).await?;
    // A copied file goes with its attachment, as in the desktop app.
    if a.link_type == LinkType::Copy {
        let _ = std::fs::remove_file(paths::resolve(&a.location));
    }
    Ok(StatusCode::NO_CONTENT)
}

/// A copied attachment's file, for thumbnails, the lightbox and downloads. Only files the app
/// stored (`attachments/…` under the data folder) are served.
async fn attachment_file(State(s): S, Path(id): Path<String>) -> ApiResult<Response> {
    let a = s.get_attachment(&id).await?;
    if a.link_type != LinkType::Copy {
        return Ok(StatusCode::NOT_FOUND.into_response());
    }
    let path = paths::resolve(&a.location);
    let inside = path
        .canonicalize()
        .ok()
        .zip(paths::attachments_dir().canonicalize().ok())
        .is_some_and(|(p, root)| p.starts_with(root));
    if !inside {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }
    let bytes = tokio::fs::read(&path).await?;
    let kind = crate::static_files::content_type(&path);
    let disposition = format!("inline; filename=\"{}\"", a.name.replace(['"', '\\'], "_"));
    Ok(([(header::CONTENT_TYPE, kind.to_string()), (header::CONTENT_DISPOSITION, disposition)], Body::from(bytes))
        .into_response())
}

// ---- settings and shortcuts ----------------------------------------------------------------

#[derive(Deserialize)]
struct SettingValue {
    value: String,
}

async fn get_setting(State(s): S, Path(key): Path<String>) -> ApiResult<impl IntoResponse> {
    Ok(Json(s.get_setting(&key).await?))
}

/// Also how the app records the list and project last focused (`last_active_list`,
/// `last_active_project`), which the desktop's capture bar reads to decide where a capture goes.
async fn set_setting(State(s): S, Path(key): Path<String>, Json(b): Json<SettingValue>) -> ApiResult<StatusCode> {
    s.set_setting(&key, &b.value).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_hotkeys(State(s): S) -> ApiResult<impl IntoResponse> {
    Ok(Json(crate::hotkeys::resolve(&s).await?))
}

#[derive(Deserialize)]
struct AccelReq {
    accel: String,
}

/// A chord that is refused (taken, or with no modifier) is the user's to change: a 422 with the
/// reason, which Settings shows under the shortcut; not a server failure.
async fn set_hotkey(State(s): S, Path(id): Path<String>, Json(b): Json<AccelReq>) -> Response {
    match crate::hotkeys::set_binding(&s, &id, &b.accel).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::UNPROCESSABLE_ENTITY, format!("{e:#}")).into_response(),
    }
}

/// Back to the default; answers with the accelerator now in effect.
async fn reset_hotkey(State(s): S, Path(id): Path<String>) -> ApiResult<impl IntoResponse> {
    taskscape_common::hotkeys::reset_binding(&s, &id).await?;
    let accel = crate::hotkeys::resolve(&s)
        .await?
        .into_iter()
        .find(|b| b.id == id)
        .and_then(|b| b.accel)
        .map(|a| a.to_string())
        .unwrap_or_default();
    Ok(Json(json!({ "accel": accel })))
}

async fn data_paths() -> impl IntoResponse {
    Json(json!({
        "data_dir": paths::root_dir(),
        "screenshots_dir": paths::screenshots_dir(),
    }))
}
