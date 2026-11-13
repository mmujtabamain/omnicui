//! The main window's state and what the user can do to it: one entity that owns the data read
//! from the store, the panes, the rail, and what is open over the window (a sheet, the palette, a
//! menu, the toast, Settings). Each surface draws itself from it (pane.rs, inspector.rs, …), so
//! a surface can also be shown on its own, which is how the contracts mount them.

use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::time::Duration;

use gpui::{
    div, point, prelude::*, px, size, AnyElement, App, Bounds, Context, Entity, FocusHandle, Focusable, Pixels,
    Point, SharedString, Subscription, Task as GpuiTask, UniformListScrollHandle, Window,
};
use serde_json::json;
use taskscape_common::{hotkeys::ResolvedBinding, util::now_millis, List, Project, Task};

use crate::db::{Db, Snapshot};
use crate::keys::*;
use crate::model::{self, SearchFields, Tree, View};
use crate::motion::Anims;
use crate::parts;
use crate::report;
use crate::text_field::{FieldEvent, TextField};
use crate::theme::Theme;
use crate::ui::Handler;

/// What a window shows: the whole main window, or one surface on its own.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Surface {
    Window,
    Pane,
    Inspector,
    TitleBar,
    Sheet,
    Palette,
    Settings,
    Filter,
    Trash,
    Toast,
}

/// What the rail (the details drawer) holds.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Rail {
    Inspector,
    Filter,
    Trash,
}

pub struct PaneState {
    pub list: Option<String>,
    pub collapsed: HashSet<String>,
    pub selection: Vec<String>,
    pub anchor: Option<String>,
    pub scroll: UniformListScrollHandle,
    pub search: Entity<TextField>,
    pub fields: SearchFields,
    pub composer: Option<String>,
    pub focus: FocusHandle,
    pub match_ix: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SheetKind {
    DeleteTasks(Vec<String>),
    DeleteList(String),
    NewList,
    RenameList(String),
    NewProject,
    RenameProject(String),
    DeleteProject(String),
    DeleteNote(String),
    EmptyTrash,
    LinkAttachment(String),
}

pub struct Sheet {
    pub kind: SheetKind,
    pub headline: String,
    pub detail: Option<String>,
    pub accept: String,
    pub instead: Option<String>,
    pub danger: bool,
    pub icon: &'static str,
    pub field: Option<Entity<TextField>>,
    pub serial: u64,
}

pub struct Menu {
    pub task: String,
    pub at: Point<Pixels>,
    pub submenu: bool,
}

pub struct Toast {
    pub message: String,
    pub restore: Vec<String>,
    pub serial: u64,
    _timer: Option<GpuiTask<()>>,
}

#[derive(Clone, Debug)]
pub enum UndoStep {
    Done(Vec<(String, bool)>),
    Trash(Vec<String>, Vec<String>),
    Create(String),
    Rename(String, String, String),
    Move { id: String, from: (Option<String>, String, f64), to: (Option<String>, String, f64) },
}

pub struct Command {
    pub id: String,
    pub group: &'static str,
    pub label: String,
    pub icon: &'static str,
    pub accel: String,
}

pub struct PaletteState {
    pub input: Entity<TextField>,
    pub highlight: usize,
    pub serial: u64,
}

/// The note being edited in the inspector: an existing note, or a new one.
pub struct NoteEdit {
    pub note: Option<String>,
    pub task: String,
    pub field: Entity<TextField>,
    pub saved: String,
    pub save_timer: Option<GpuiTask<()>>,
}

pub struct Workspace {
    pub db: Rc<Db>,
    pub surface: Surface,
    pub system_dark: bool,
    pub data: Snapshot,
    pub hotkeys: Vec<ResolvedBinding>,
    pub active_project: Option<String>,
    pub panes: Vec<PaneState>,
    pub focused_pane: usize,
    pub preview: bool,
    pub rail: Rail,
    pub focused_task: Option<String>,
    pub draft: Option<String>,
    pub draft_title: Entity<TextField>,
    pub show_completed: bool,
    pub reduced_motion: bool,
    pub theme_setting: String,
    pub screenshot_mode: String,
    pub sheet: Option<Sheet>,
    pub palette: Option<PaletteState>,
    pub toast: Option<Toast>,
    pub menu: Option<Menu>,
    pub project_menu: bool,
    pub settings: Option<Entity<crate::settings::SettingsView>>,
    pub renaming: Option<String>,
    pub title_field: Entity<TextField>,
    pub subtask_field: Entity<TextField>,
    pub note_edit: Option<NoteEdit>,
    pub undo: Vec<UndoStep>,
    pub redo: Vec<UndoStep>,
    pub views: HashMap<String, View>,
    pub focus: HashMap<String, FocusHandle>,
    pub root_focus: FocusHandle,
    pub anims: Anims,
    pub serial: u64,
    pub hover_note: Option<usize>,
    pub lists: HashMap<usize, crate::pane::ListCache>,
    _subs: Vec<Subscription>,
}

impl Workspace {
    pub fn new(db: Rc<Db>, surface: Surface, system_dark: bool, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let data = db.snapshot();
        let hotkeys = db.hotkeys();
        let setting = |k: &str| db.setting(k);
        let show_completed = setting("show_completed").as_deref() != Some("0");
        let reduced_motion = setting("reduced_motion").as_deref() == Some("1");
        crate::motion::set_reduced(reduced_motion);
        let theme_setting = setting("theme").unwrap_or_else(|| "system".into());
        let screenshot_mode = setting("screenshot_mode").unwrap_or_else(|| "fullscreen".into());
        let preview = setting("preview_open").as_deref() != Some("0");

        let draft_title = cx.new(|cx| TextField::new("Untitled", window, cx));
        let title_field = cx.new(|cx| TextField::new("Untitled", window, cx));
        let subtask_field = cx.new(|cx| TextField::new("Subtask title — Enter to add", window, cx));
        let mut subs = vec![
            cx.subscribe_in(&draft_title, window, Self::on_draft_field),
            cx.subscribe_in(&title_field, window, Self::on_title_field),
            cx.subscribe_in(&subtask_field, window, Self::on_subtask_field),
        ];
        let pane = Self::new_pane(window, cx, &mut subs, 0);
        let mut ws = Workspace {
            db,
            surface,
            system_dark,
            data,
            hotkeys,
            active_project: None,
            panes: vec![pane],
            focused_pane: 0,
            preview,
            rail: Rail::Inspector,
            focused_task: None,
            draft: None,
            draft_title,
            show_completed,
            reduced_motion,
            theme_setting,
            screenshot_mode,
            sheet: None,
            palette: None,
            toast: None,
            menu: None,
            project_menu: false,
            settings: None,
            renaming: None,
            title_field,
            subtask_field,
            note_edit: None,
            undo: vec![],
            redo: vec![],
            views: HashMap::new(),
            focus: HashMap::new(),
            root_focus: cx.focus_handle(),
            anims: Anims::default(),
            serial: 0,
            hover_note: None,
            lists: HashMap::new(),
            _subs: subs,
        };
        // It opens on the last project and list (SPEC-PRODUCT.md, the window).
        let last_list = ws.db.setting("last_active_list").filter(|id| ws.list(id).is_some());
        let project = last_list
            .as_deref()
            .and_then(|l| ws.list(l).map(|l| l.project_id.clone()))
            .or_else(|| ws.db.setting("last_active_project").filter(|p| ws.project(p).is_some()))
            .or_else(|| ws.data.projects.first().map(|p| p.id.clone()));
        ws.active_project = project;
        let list = last_list.or_else(|| ws.project_lists().first().map(|l| l.id.clone()));
        ws.panes[0].list = list;
        ws.sync_placeholders(window, cx);
        ws
    }

    fn new_pane(window: &mut Window, cx: &mut Context<Self>, subs: &mut Vec<Subscription>, ix: usize) -> PaneState {
        let search = cx.new(|cx| TextField::new("Search", window, cx));
        subs.push(cx.subscribe_in(&search, window, move |ws, _, e, w, cx| ws.on_search_field(ix, e, w, cx)));
        PaneState {
            list: None,
            collapsed: HashSet::new(),
            selection: vec![],
            anchor: None,
            scroll: UniformListScrollHandle::new(),
            search,
            fields: SearchFields::Both,
            composer: None,
            focus: cx.focus_handle(),
            match_ix: 0,
        }
    }

    // ---- reading -------------------------------------------------------------------------------

    pub fn theme(&self) -> Theme {
        let dark = match self.theme_setting.as_str() {
            "dark" => true,
            "light" => false,
            _ => self.system_dark,
        };
        Theme::of(dark)
    }

    pub fn reload(&mut self) {
        self.data = self.db.snapshot();
        if let Some(t) = &self.focused_task {
            if self.task(t).is_none() {
                self.focused_task = None;
            }
        }
        for p in &mut self.panes {
            let live: HashSet<&str> = self.data.tasks.iter().map(|t| t.id.as_str()).collect();
            p.selection.retain(|s| live.contains(s.as_str()));
            if let Some(l) = &p.list {
                if !self.data.lists.iter().any(|x| &x.id == l) {
                    p.list = None;
                }
            }
        }
    }

    pub fn task(&self, id: &str) -> Option<&Task> {
        self.data.tasks.iter().find(|t| t.id == id)
    }
    pub fn list(&self, id: &str) -> Option<&List> {
        self.data.lists.iter().find(|l| l.id == id)
    }
    pub fn project(&self, id: &str) -> Option<&Project> {
        self.data.projects.iter().find(|p| p.id == id)
    }
    pub fn project_lists(&self) -> Vec<&List> {
        let mut v: Vec<&List> = self.data.lists.iter().filter(|l| Some(&l.project_id) == self.active_project.as_ref()).collect();
        v.sort_by(|a, b| a.sort_order.partial_cmp(&b.sort_order).unwrap_or(std::cmp::Ordering::Equal).then(a.created_at.cmp(&b.created_at)));
        v
    }
    pub fn list_tasks(&self, list: &str) -> Vec<&Task> {
        self.data.tasks.iter().filter(|t| t.list_id == list).collect()
    }
    pub fn tree(&self, list: &str) -> Tree<'_> {
        Tree::new(self.data.tasks.iter().filter(|t| t.list_id == list))
    }
    /// "Atlas / Launch plan"
    pub fn where_of(&self, list: &str) -> String {
        let Some(l) = self.list(list) else { return String::new() };
        let p = self.project(&l.project_id).map(|p| p.name.clone()).unwrap_or_default();
        format!("{p} / {}", l.name)
    }
    pub fn pane_list(&self) -> Option<String> {
        self.panes.get(self.focused_pane).and_then(|p| p.list.clone())
    }
    pub fn glyphs(&self, id: &str) -> String {
        crate::keys::glyphs(&self.hotkeys, id)
    }
    pub fn view_of(&self, list: &str) -> View {
        self.views.get(list).cloned().unwrap_or_default()
    }
    pub fn query(&self, pane: usize, cx: &App) -> String {
        self.panes[pane].search.read(cx).text().to_string()
    }
    /// The rows a pane shows.
    pub fn rows(&self, pane: usize, cx: &App) -> Vec<model::Row> {
        let p = &self.panes[pane];
        let Some(list) = &p.list else { return vec![] };
        let tree = self.tree(list);
        let q = self.query(pane, cx);
        model::rows(&tree, &p.collapsed, self.show_completed, Some((q.as_str(), p.fields)), &self.view_of(list), now_millis())
    }
    /// The selection of the focused pane, when it is a multi-selection.
    pub fn multi(&self) -> Vec<String> {
        let s = &self.panes[self.focused_pane].selection;
        if s.len() >= 2 {
            s.clone()
        } else {
            vec![]
        }
    }

    pub fn fh(&mut self, name: &str, cx: &mut Context<Self>) -> FocusHandle {
        if let Some(f) = self.focus.get(name) {
            return f.clone();
        }
        let f = cx.focus_handle();
        self.focus.insert(name.to_string(), f.clone());
        f
    }

    /// A handler that runs `f` on this workspace.
    pub fn on(&self, cx: &Context<Self>, f: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static) -> Handler {
        let this = cx.entity().downgrade();
        Rc::new(move |window, cx| {
            this.update(cx, |ws, cx| {
                f(ws, window, cx);
                cx.notify();
            })
            .ok();
        })
    }

    fn sync_placeholders(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        for i in 0..self.panes.len() {
            let name = self.panes[i].list.as_deref().and_then(|l| self.list(l)).map(|l| l.name.clone()).unwrap_or_default();
            self.panes[i].search.update(cx, |f, cx| {
                f.set_placeholder(format!("Search {name}"));
                cx.notify()
            });
        }
    }

    fn next_serial(&mut self) -> u64 {
        self.serial += 1;
        self.serial
    }

    // ---- state the surfaces start in (props) ----------------------------------------------

    /// Shows a list in the focused pane (or focuses the pane already showing it).
    pub fn show_list(&mut self, list: &str, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(i) = self.panes.iter().position(|p| p.list.as_deref() == Some(list)) {
            self.focused_pane = i;
        } else {
            let p = &mut self.panes[self.focused_pane];
            p.list = Some(list.to_string());
            p.selection.clear();
            p.composer = None;
            p.search.update(cx, |f, cx| f.set_text("", cx));
            if self.focused_pane == 0 {
                report(cx, "paneShows", json!({ "listId": list }));
            }
        }
        if let Some(l) = self.list(list) {
            if self.active_project.as_deref() != Some(&l.project_id) {
                self.active_project = Some(l.project_id.clone());
            }
        }
        self.db.set_setting("last_active_list", list);
        report(cx, "listSelected", json!({ "id": list }));
        self.sync_placeholders(window, cx);
        cx.notify();
    }

    pub fn set_split(&mut self, list: Option<&str>, window: &mut Window, cx: &mut Context<Self>) {
        match list {
            Some(l) => {
                if self.panes.len() < 2 {
                    let mut subs = std::mem::take(&mut self._subs);
                    let p = Self::new_pane(window, cx, &mut subs, 1);
                    self._subs = subs;
                    self.panes.push(p);
                }
                self.panes[1].list = Some(l.to_string());
                report(cx, "splitChanged", json!({ "listId": l }));
            }
            None => {
                if self.panes.len() > 1 {
                    self.panes.truncate(1);
                    self.focused_pane = 0;
                    report(cx, "splitChanged", json!({ "listId": "" }));
                }
            }
        }
        self.sync_placeholders(window, cx);
        cx.notify();
    }

    pub fn set_preview(&mut self, open: bool, cx: &mut Context<Self>) {
        if self.preview != open {
            self.preview = open;
            self.db.set_setting("preview_open", if open { "1" } else { "0" });
            report(cx, "previewToggled", json!({ "open": open }));
        }
        cx.notify();
    }

    /// Focuses a task: the inspector shows it.
    pub fn focus_task(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.close_note_editor(window, cx);
        self.renaming = None;
        self.draft = None;
        if let Some(list) = self.task(id).map(|t| t.list_id.clone()) {
            if let Some(i) = self.panes.iter().position(|p| p.list.as_deref() == Some(&list)) {
                self.focused_pane = i;
            }
        }
        let pane = &mut self.panes[self.focused_pane];
        pane.selection = vec![id.to_string()];
        pane.anchor = Some(id.to_string());
        if self.focused_task.as_deref() != Some(id) {
            self.focused_task = Some(id.to_string());
            report(cx, "taskFocused", json!({ "id": id }));
        }
        if self.rail != Rail::Inspector {
            self.rail = Rail::Inspector;
        }
        let rows = self.rows(self.focused_pane, cx);
        if let Some(ix) = rows.iter().position(|r| r.id == id) {
            self.panes[self.focused_pane].scroll.scroll_to_item(ix, gpui::ScrollStrategy::Top);
        }
        cx.notify();
    }

    pub fn open_draft(&mut self, list: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.close_note_editor(window, cx);
        self.draft = Some(list.to_string());
        self.focused_task = None;
        self.rail = Rail::Inspector;
        for p in &mut self.panes {
            p.selection.clear();
        }
        self.draft_title.update(cx, |f, cx| f.set_text("", cx));
        let f = self.draft_title.read(cx).focus.clone();
        window.focus(&f);
        report(cx, "draftOpened", json!({ "listId": list }));
        cx.notify();
    }

    pub fn select(&mut self, ids: Vec<String>, cx: &mut Context<Self>) {
        if let Some(list) = ids.first().and_then(|i| self.task(i)).map(|t| t.list_id.clone()) {
            if let Some(i) = self.panes.iter().position(|p| p.list.as_deref() == Some(&list)) {
                self.focused_pane = i;
            }
        }
        self.panes[self.focused_pane].selection = ids;
        cx.notify();
    }

    pub fn clear_selection(&mut self, cx: &mut Context<Self>) {
        for p in &mut self.panes {
            p.selection.clear();
        }
        self.focused_task = None;
        cx.notify();
    }

    // ---- what changes the store ----------------------------------------------------------------

    /// Marks a task done or not, with the completion rules (model::Tree::completion).
    pub fn set_done(&mut self, id: &str, done: bool, cx: &mut Context<Self>) {
        let Some(list) = self.task(id).map(|t| t.list_id.clone()) else { return };
        let changes = self.tree(&list).completion(id, done);
        let before: Vec<(String, bool)> = changes.iter().map(|(i, d)| (i.clone(), !d)).collect();
        self.apply_done(&changes, cx);
        self.push_undo(UndoStep::Done(before));
    }

    fn apply_done(&mut self, changes: &[(String, bool)], cx: &mut Context<Self>) {
        for (tid, d) in changes {
            if self.db.run(self.db.store.update_task(tid, None, None, Some(*d))).is_ok() {
                report(cx, "taskCompleted", json!({ "id": tid, "done": d }));
            }
        }
        self.reload();
        cx.notify();
    }

    /// The bulk Done and Undone: every selected task, as chosen.
    pub fn set_many_done(&mut self, ids: &[String], done: bool, cx: &mut Context<Self>) {
        let before: Vec<(String, bool)> = ids.iter().filter_map(|i| self.task(i).map(|t| (i.clone(), t.done))).collect();
        let changes: Vec<(String, bool)> = ids.iter().map(|i| (i.clone(), done)).collect();
        self.apply_done(&changes, cx);
        self.push_undo(UndoStep::Done(before));
    }

    pub fn create_task(&mut self, list: &str, title: &str, parent: Option<&str>, cx: &mut Context<Self>) -> Option<String> {
        let t = self.db.run(self.db.store.create_task(list, title, None, parent)).ok()?;
        report(cx, "taskCreated", json!({ "id": t.id, "title": t.title, "listId": t.list_id, "parentId": t.parent_id.clone().unwrap_or_default() }));
        self.reload();
        self.push_undo(UndoStep::Create(t.id.clone()));
        cx.notify();
        Some(t.id)
    }

    pub fn rename_task(&mut self, id: &str, title: &str, cx: &mut Context<Self>) {
        let Some(old) = self.task(id).map(|t| t.title.clone()) else { return };
        let title = title.trim();
        if title.is_empty() || title == old {
            return;
        }
        if self.db.run(self.db.store.update_task(id, Some(title), None, None)).is_ok() {
            report(cx, "taskRenamed", json!({ "id": id, "title": title }));
            self.push_undo(UndoStep::Rename(id.to_string(), old, title.to_string()));
        }
        self.reload();
        cx.notify();
    }

    pub fn trash(&mut self, ids: &[String], cx: &mut Context<Self>) {
        let Ok(stamped) = self.db.run(self.db.store.soft_delete_tasks(ids)) else { return };
        for id in ids {
            report(cx, "taskTrashed", json!({ "id": id }));
        }
        self.reload();
        let message = if ids.len() == 1 { "Moved to Trash".to_string() } else { format!("{} moved to Trash", ids.len()) };
        self.show_toast(message, stamped.clone(), cx);
        self.push_undo(UndoStep::Trash(ids.to_vec(), stamped));
        cx.notify();
    }

    pub fn restore(&mut self, report_ids: &[String], stamped: &[String], cx: &mut Context<Self>) {
        if self.db.run(self.db.store.restore_tasks(stamped)).is_ok() {
            for id in report_ids {
                report(cx, "taskRestored", json!({ "id": id }));
            }
        }
        self.reload();
        cx.notify();
    }

    pub fn purge(&mut self, ids: &[String], cx: &mut Context<Self>) {
        if self.db.run(self.db.store.purge_tasks(ids)).is_ok() {
            for id in ids {
                report(cx, "taskPurged", json!({ "id": id }));
            }
        }
        self.reload();
        cx.notify();
    }

    pub fn move_task(&mut self, id: &str, parent: Option<&str>, list: Option<&str>, sort: Option<f64>, cx: &mut Context<Self>) {
        let Some(t) = self.task(id).cloned() else { return };
        if let Ok(m) = self.db.run(self.db.store.move_task(id, parent, list, sort)) {
            report(cx, "taskMoved", json!({ "id": m.id, "listId": m.list_id, "parentId": m.parent_id.clone().unwrap_or_default() }));
            self.push_undo(UndoStep::Move {
                id: id.to_string(),
                from: (t.parent_id.clone(), t.list_id.clone(), t.sort_order),
                to: (m.parent_id.clone(), m.list_id.clone(), m.sort_order),
            });
        }
        self.reload();
        cx.notify();
    }

    pub fn copy_tasks(&mut self, ids: &[String], cx: &mut Context<Self>) {
        let text = model::copy_markdown(&self.data.tasks, ids);
        cx.write_to_clipboard(gpui::ClipboardItem::new_string(text.clone()));
        report(cx, "copied", json!({ "text": text }));
    }

    pub fn set_setting(&mut self, key: &str, value: &str, cx: &mut Context<Self>) {
        self.db.set_setting(key, value);
        report(cx, "settingChanged", json!({ "key": key, "value": value }));
        match key {
            "show_completed" => self.show_completed = value != "0",
            "reduced_motion" => {
                self.reduced_motion = value == "1";
                crate::motion::set_reduced(self.reduced_motion);
            }
            "theme" => self.theme_setting = value.to_string(),
            "screenshot_mode" => self.screenshot_mode = value.to_string(),
            _ => {}
        }
        cx.notify();
    }

    fn push_undo(&mut self, u: UndoStep) {
        self.undo.push(u);
        self.redo.clear();
    }

    pub fn undo(&mut self, cx: &mut Context<Self>) {
        let Some(u) = self.undo.pop() else { return };
        self.apply_undo(&u, true, cx);
        self.redo.push(u);
    }

    pub fn redo(&mut self, cx: &mut Context<Self>) {
        let Some(u) = self.redo.pop() else { return };
        self.apply_undo(&u, false, cx);
        self.undo.push(u);
    }

    fn apply_undo(&mut self, u: &UndoStep, back: bool, cx: &mut Context<Self>) {
        match u {
            UndoStep::Done(before) => {
                let changes: Vec<(String, bool)> = before.iter().map(|(i, d)| (i.clone(), if back { *d } else { !d })).collect();
                self.apply_done(&changes, cx);
            }
            UndoStep::Trash(ids, stamped) => {
                if back {
                    self.restore(ids, stamped, cx);
                    self.toast = None;
                } else if self.db.run(self.db.store.soft_delete_tasks(ids)).is_ok() {
                    for id in ids {
                        report(cx, "taskTrashed", json!({ "id": id }));
                    }
                }
            }
            UndoStep::Create(id) => {
                let ids = vec![id.clone()];
                if back {
                    if self.db.run(self.db.store.soft_delete_tasks(&ids)).is_ok() {
                        report(cx, "taskTrashed", json!({ "id": id }));
                    }
                } else {
                    self.restore(&ids, &ids, cx);
                }
            }
            UndoStep::Rename(id, old, new) => {
                let title = if back { old } else { new };
                if self.db.run(self.db.store.update_task(id, Some(title), None, None)).is_ok() {
                    report(cx, "taskRenamed", json!({ "id": id, "title": title }));
                }
            }
            UndoStep::Move { id, from, to } => {
                let (p, l, s) = if back { from } else { to };
                if let Ok(m) = self.db.run(self.db.store.move_task(id, p.as_deref(), Some(l), Some(*s))) {
                    report(cx, "taskMoved", json!({ "id": m.id, "listId": m.list_id, "parentId": m.parent_id.clone().unwrap_or_default() }));
                }
            }
        }
        self.reload();
        cx.notify();
    }

    pub fn show_toast(&mut self, message: String, restore: Vec<String>, cx: &mut Context<Self>) {
        let serial = self.next_serial();
        let timer = cx.spawn(async move |this, cx| {
            cx.background_executor().timer(Duration::from_secs(6)).await;
            this.update(cx, |ws, cx| {
                if ws.toast.as_ref().is_some_and(|t| t.serial == serial) {
                    ws.toast = None;
                    cx.notify();
                }
            })
            .ok();
        });
        self.toast = Some(Toast { message, restore, serial, _timer: Some(timer) });
        cx.notify();
    }

    pub fn toast_undo(&mut self, cx: &mut Context<Self>) {
        // The toast's Undo is the history's (the delete is the last change).
        if matches!(self.undo.last(), Some(UndoStep::Trash(..))) {
            self.undo(cx);
        } else if let Some(t) = self.toast.take() {
            let ids = t.restore.clone();
            self.restore(&ids, &ids, cx);
        }
        self.toast = None;
        cx.notify();
    }

    // ---- questions (sheets) ----------------------------------------------------------------------

    pub fn ask_delete(&mut self, ids: Vec<String>, window: &mut Window, cx: &mut Context<Self>) {
        if !ids.is_empty() {
            self.open_sheet(SheetKind::DeleteTasks(ids), window, cx);
        }
    }

    pub fn open_sheet(&mut self, kind: SheetKind, window: &mut Window, cx: &mut Context<Self>) {
        self.menu = None;
        self.project_menu = false;
        let suggest = || taskscape_common::names::suggest_list_name();
        let (headline, detail, accept, instead, danger, icon, field): (String, Option<String>, &str, Option<&str>, bool, &str, Option<(String, String)>) = match &kind {
            SheetKind::DeleteTasks(ids) if ids.len() == 1 => {
                let title = self.task(&ids[0]).map(|t| t.title.clone()).unwrap_or_default();
                let list = self.task(&ids[0]).map(|t| t.list_id.clone()).unwrap_or_default();
                let n = self.tree(&list).descendants(&ids[0]).len();
                let detail = if n == 0 {
                    format!("“{title}” — ⌘Z puts it back.")
                } else {
                    format!("“{title}” and its {n} subtask{} — ⌘Z puts it back.", if n == 1 { "" } else { "s" })
                };
                ("Move to Trash?".into(), Some(detail), "Move", None, false, "delete", None)
            }
            SheetKind::DeleteTasks(ids) => (
                format!("Move {} tasks to Trash?", ids.len()),
                Some(format!("{} tasks including subtasks — ⌘Z puts them back.", ids.len())),
                "Move",
                None,
                false,
                "delete",
                None,
            ),
            SheetKind::DeleteList(l) => {
                let name = self.list(l).map(|l| l.name.clone()).unwrap_or_default();
                let n = self.list_tasks(l).len();
                let detail = if n == 0 { "The list is empty. No undo.".to_string() } else { format!("Its {n} task{} go with it. No undo.", if n == 1 { "" } else { "s" }) };
                (format!("Delete “{name}”?"), Some(detail), "Delete list", None, true, "delete_forever", None)
            }
            SheetKind::NewList => ("New list".into(), None, "Create", Some("Import JSON…"), false, "add", Some(("Name this list".into(), suggest()))),
            SheetKind::RenameList(l) => {
                let name = self.list(l).map(|l| l.name.clone()).unwrap_or_default();
                ("Rename list".into(), None, "Rename", None, false, "edit", Some(("Name this list".into(), name)))
            }
            SheetKind::NewProject => ("New project".into(), None, "Create", None, false, "add", Some(("Name this project".into(), suggest()))),
            SheetKind::RenameProject(p) => {
                let name = self.project(p).map(|p| p.name.clone()).unwrap_or_default();
                ("Rename project".into(), None, "Rename", None, false, "edit", Some(("Name this project".into(), name)))
            }
            SheetKind::DeleteProject(p) => {
                let name = self.project(p).map(|p| p.name.clone()).unwrap_or_default();
                let lists: Vec<String> = self.data.lists.iter().filter(|l| &l.project_id == p).map(|l| l.id.clone()).collect();
                let tasks = self.data.tasks.iter().filter(|t| lists.contains(&t.list_id)).count();
                let detail = format!(
                    "Its {} list{} and {} task{} go with it. No undo.",
                    lists.len(),
                    if lists.len() == 1 { "" } else { "s" },
                    tasks,
                    if tasks == 1 { "" } else { "s" }
                );
                (format!("Delete “{name}”?"), Some(detail), "Delete project", None, true, "delete_forever", None)
            }
            SheetKind::DeleteNote(_) => ("Delete note?".into(), Some("Notes skip the Trash — this one is gone for good.".into()), "Delete", None, true, "delete", None),
            SheetKind::EmptyTrash => {
                let n = self.data.trashed.iter().filter(|t| t.parent_id.as_deref().map_or(true, |p| !self.data.trashed.iter().any(|x| x.id == p))).count();
                ("Empty Trash?".into(), Some(format!("{n} task{} deleted for good. No undo.", if n == 1 { "" } else { "s" })), "Empty Trash", None, true, "delete_forever", None)
            }
            SheetKind::LinkAttachment(_) => ("Link a file or page".into(), None, "Link", None, false, "add_link", Some(("https://… or a file path".into(), String::new()))),
        };
        let field = field.map(|(placeholder, text)| {
            let f = cx.new(|cx| {
                let mut f = TextField::new(placeholder, window, cx);
                f.set_text(text, cx);
                f.select_all_text(cx);
                f
            });
            self._subs.push(cx.subscribe_in(&f, window, |ws, _, e, window, cx| match e {
                FieldEvent::Submit => ws.answer_sheet("accept", window, cx),
                FieldEvent::Cancel => ws.answer_sheet("dismiss", window, cx),
                _ => cx.notify(),
            }));
            window.focus(&f.read(cx).focus.clone());
            f
        });
        report(cx, "confirmAsked", json!({ "headline": headline }));
        let serial = self.next_serial();
        if field.is_none() {
            let accept = self.fh("sheetAccept", cx);
            window.focus(&accept);
        }
        self.sheet = Some(Sheet { kind, headline, detail, accept: accept.into(), instead: instead.map(Into::into), danger, icon, field, serial });
        cx.notify();
    }

    pub fn sheet_text(&self, cx: &App) -> String {
        self.sheet.as_ref().and_then(|s| s.field.as_ref()).map(|f| f.read(cx).text().trim().to_string()).unwrap_or_default()
    }

    /// Accept, instead or dismiss: what the sheet's question was for happens on accept.
    pub fn answer_sheet(&mut self, action: &str, window: &mut Window, cx: &mut Context<Self>) {
        let Some(sheet) = self.sheet.as_ref() else { return };
        let text = self.sheet_text(cx);
        if action == "accept" && sheet.field.is_some() && text.is_empty() {
            return;
        }
        let sheet = self.sheet.take().unwrap();
        report(cx, "answered", json!({ "action": action, "text": if action == "accept" { text.clone() } else { String::new() } }));
        if action == "accept" {
            match sheet.kind {
                SheetKind::DeleteTasks(ids) => {
                    self.trash(&ids, cx);
                    for p in &mut self.panes {
                        p.selection.retain(|s| !ids.contains(s));
                    }
                }
                SheetKind::DeleteList(l) => {
                    if self.db.run(self.db.store.delete_list(&l)).is_ok() {
                        report(cx, "listDeleted", json!({ "id": l }));
                    }
                    self.reload();
                    if self.panes[0].list.is_none() {
                        self.panes[0].list = self.project_lists().first().map(|l| l.id.clone());
                    }
                }
                SheetKind::NewList => {
                    if let Some(p) = self.active_project.clone() {
                        if let Ok(l) = self.db.run(self.db.store.create_list(&p, &text)) {
                            report(cx, "listCreated", json!({ "id": l.id, "name": l.name, "projectId": l.project_id }));
                            self.reload();
                            self.show_list(&l.id, window, cx);
                        }
                    }
                }
                SheetKind::RenameList(l) => {
                    if self.db.run(self.db.store.rename_list(&l, &text)).is_ok() {
                        report(cx, "listRenamed", json!({ "id": l, "name": text }));
                    }
                    self.reload();
                }
                SheetKind::NewProject => {
                    if let Ok(p) = self.db.run(self.db.store.create_project(&text)) {
                        report(cx, "projectCreated", json!({ "id": p.id, "name": p.name }));
                        self.reload();
                        self.choose_project(&p.id, window, cx);
                    }
                }
                SheetKind::RenameProject(p) => {
                    if self.db.run(self.db.store.rename_project(&p, &text)).is_ok() {
                        report(cx, "projectRenamed", json!({ "id": p, "name": text }));
                    }
                    self.reload();
                }
                SheetKind::DeleteProject(p) => {
                    if self.db.run(self.db.store.delete_project(&p)).is_ok() {
                        report(cx, "projectDeleted", json!({ "id": p }));
                    }
                    self.reload();
                    if self.active_project.as_deref() == Some(&p) {
                        if let Some(first) = self.data.projects.first().map(|p| p.id.clone()) {
                            self.choose_project(&first, window, cx);
                        }
                    }
                }
                SheetKind::DeleteNote(n) => {
                    let task = self.data.tasks.iter().find(|t| t.note_items.iter().any(|x| x.id == n)).map(|t| t.id.clone());
                    if self.db.run(self.db.store.delete_note(&n)).is_ok() {
                        if let Some(t) = task {
                            report(cx, "noteDeleted", json!({ "taskId": t }));
                        }
                    }
                    self.reload();
                }
                SheetKind::EmptyTrash => {
                    let ids: Vec<String> = self.data.trashed.iter().map(|t| t.id.clone()).collect();
                    if self.db.run(self.db.store.empty_trash()).is_ok() {
                        for id in ids {
                            report(cx, "taskPurged", json!({ "id": id }));
                        }
                    }
                    self.reload();
                }
                SheetKind::LinkAttachment(task) => {
                    let name = text.rsplit('/').find(|s| !s.is_empty()).unwrap_or(&text).to_string();
                    if let Ok(a) = self.db.run(taskscape_common::attachments::attach_reference(&self.db.store, &task, &name, &text)) {
                        report(cx, "attachmentAdded", json!({ "taskId": a.task_id, "name": a.name }));
                    }
                    self.reload();
                }
            }
        } else if action == "instead" {
            // "Import JSON…" picks a file with the system's dialog.
            let paths = cx.prompt_for_paths(gpui::PathPromptOptions { files: true, directories: false, multiple: false, prompt: None });
            drop(paths);
        }
        window.focus(&self.root_focus);
        cx.notify();
    }

    // ---- projects and lists ------------------------------------------------------------------

    pub fn choose_project(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.project_menu = false;
        if self.active_project.as_deref() != Some(id) {
            self.active_project = Some(id.to_string());
            self.db.set_setting("last_active_project", id);
            report(cx, "projectSelected", json!({ "id": id }));
            if self.panes.len() > 1 {
                self.panes.truncate(1);
                self.focused_pane = 0;
            }
            let first = self.project_lists().first().map(|l| l.id.clone());
            self.panes[0].list = None;
            if let Some(l) = first {
                self.show_list(&l, window, cx);
            }
        }
        self.sync_placeholders(window, cx);
        cx.notify();
    }

    pub fn tab_step(&mut self, by: isize, window: &mut Window, cx: &mut Context<Self>) {
        let lists: Vec<String> = self.project_lists().iter().map(|l| l.id.clone()).collect();
        if lists.is_empty() {
            return;
        }
        let cur = self.pane_list().and_then(|l| lists.iter().position(|x| *x == l)).unwrap_or(0) as isize;
        let next = (cur + by).rem_euclid(lists.len() as isize) as usize;
        let l = lists[next].clone();
        self.show_list(&l, window, cx);
    }

    pub fn switch_to(&mut self, n: usize, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(l) = self.project_lists().get(n).map(|l| l.id.clone()) {
            self.show_list(&l, window, cx);
        }
    }

    pub fn toggle_split(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.panes.len() > 1 {
            self.set_split(None, window, cx);
        } else {
            let lists: Vec<String> = self.project_lists().iter().map(|l| l.id.clone()).collect();
            let cur = self.panes[0].list.clone();
            let next = cur
                .as_ref()
                .and_then(|c| lists.iter().position(|l| l == c))
                .map(|i| lists[(i + 1) % lists.len()].clone())
                .or_else(|| lists.first().cloned());
            if let Some(n) = next.filter(|n| Some(n) != cur.as_ref()) {
                self.set_split(Some(&n), window, cx);
            }
        }
    }

    pub fn swap_panes(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.panes.len() > 1 {
            let (a, b) = (self.panes[0].list.clone(), self.panes[1].list.clone());
            self.panes[0].list = b.clone();
            self.panes[1].list = a.clone();
            self.focused_pane = 1 - self.focused_pane;
            if let Some(b) = b {
                report(cx, "paneShows", json!({ "listId": b }));
            }
            if let Some(a) = a {
                report(cx, "splitChanged", json!({ "listId": a }));
            }
            self.sync_placeholders(window, cx);
        }
        cx.notify();
    }

    pub fn open_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        report(cx, "settingsOpened", json!({}));
        if self.surface == Surface::Window {
            let ws = cx.entity();
            let v = cx.new(|cx| crate::settings::SettingsView::new(self.db.clone(), Some(ws), self.system_dark, window, cx));
            let f = v.read(cx).focus.clone();
            window.focus(&f);
            self.settings = Some(v);
        }
        cx.notify();
    }

    pub fn open_palette(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let input = cx.new(|cx| TextField::new("Run a command or jump to…", window, cx));
        self._subs.push(cx.subscribe_in(&input, window, |ws, _, e, window, cx| match e {
            FieldEvent::Submit => ws.run_highlighted(window, cx),
            FieldEvent::Cancel => ws.close_palette(window, cx),
            FieldEvent::Changed => {
                if let Some(p) = &mut ws.palette {
                    p.highlight = 0;
                }
                cx.notify()
            }
            _ => {}
        }));
        window.focus(&input.read(cx).focus.clone());
        let serial = self.next_serial();
        self.palette = Some(PaletteState { input, highlight: 0, serial });
        report(cx, "paletteOpened", json!({}));
        cx.notify();
    }

    pub fn close_palette(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.palette.take().is_some() {
            report(cx, "closed", json!({}));
            window.focus(&self.root_focus);
        }
        cx.notify();
    }

    pub fn commands(&self) -> Vec<Command> {
        let a = |id: &str| self.glyphs(id);
        let c = |id: &str, group: &'static str, label: &str, icon: &'static str, accel: String| Command { id: id.into(), group, label: label.into(), icon, accel };
        let mut v = vec![
            c("toggle_preview", "View", "Toggle preview", "right_panel_open", a("toggle_preview")),
            c("toggle_split", "View", "Toggle split view", "vertical_split", a("toggle_split")),
            c("swap_panes", "View", "Swap panes", "swap_horiz", String::new()),
            c("toggle_completed", "View", "Show / hide completed", "task_alt", a("toggle_completed")),
            c("collapse_all", "View", "Collapse all", "remove", a("collapse_all")),
            c("expand_all", "View", "Expand all", "unfold_more", a("expand_all")),
            c("new_task", "Task", "New task", "add", a("new_task")),
            c("search", "Task", "Search this list", "search", a("search")),
            c("undo", "Task", "Undo", "undo", a("undo")),
            c("redo", "Task", "Redo", "redo", a("redo")),
            c("new_list", "Manage", "New list", "add", a("new_list")),
            c("new_project", "Manage", "New project", "folder", a("new_project")),
            c("trash", "Manage", "Open Trash", "delete", String::new()),
            c("open_settings", "Manage", "Settings", "settings", a("open_settings")),
        ];
        if let Some(t) = self.focused_task.as_deref().and_then(|t| self.task(t)) {
            v.push(c("delete_sel", "Task", &format!("Delete “{}”", t.title), "delete", String::new()));
        }
        for l in self.project_lists() {
            v.push(c(&format!("go-list-{}", l.id), "Go to list", &l.name, "notes", String::new()));
        }
        for p in &self.data.projects {
            v.push(c(&format!("go-proj-{}", p.id), "Go to project", &p.name, "folder_open", String::new()));
        }
        v
    }

    /// The commands the palette shows for its text, grouped, earliest match first.
    pub fn palette_commands(&self, cx: &App) -> Vec<Command> {
        let q = self.palette.as_ref().map(|p| p.input.read(cx).text().trim().to_lowercase()).unwrap_or_default();
        let mut v: Vec<(usize, usize, Command)> = self
            .commands()
            .into_iter()
            .enumerate()
            .filter_map(|(i, c)| {
                if q.is_empty() {
                    return Some((0, i, c));
                }
                c.label.to_lowercase().find(&q).map(|at| (at, i, c))
            })
            .collect();
        v.sort_by_key(|(at, i, _)| (*at, *i));
        // Grouped in the order the groups first appear.
        let mut groups: Vec<&'static str> = vec![];
        for (_, _, c) in &v {
            if !groups.contains(&c.group) {
                groups.push(c.group);
            }
        }
        let mut out = vec![];
        for g in groups {
            for (_, _, c) in v.iter_mut().filter(|(_, _, c)| c.group == g) {
                out.push(Command { id: std::mem::take(&mut c.id), group: c.group, label: std::mem::take(&mut c.label), icon: c.icon, accel: std::mem::take(&mut c.accel) });
            }
        }
        out
    }

    pub fn run_highlighted(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let cmds = self.palette_commands(cx);
        let ix = self.palette.as_ref().map(|p| p.highlight).unwrap_or(0);
        if let Some(c) = cmds.get(ix) {
            let id = c.id.clone();
            self.run_command(&id, window, cx);
        }
    }

    pub fn run_command(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        report(cx, "commandRun", json!({ "command": id }));
        self.close_palette(window, cx);
        match id {
            "toggle_preview" => self.set_preview(!self.preview, cx),
            "toggle_split" => self.toggle_split(window, cx),
            "swap_panes" => self.swap_panes(window, cx),
            "toggle_completed" => self.toggle_completed(cx),
            "collapse_all" => self.collapse_all(true, cx),
            "expand_all" => self.collapse_all(false, cx),
            "new_task" => {
                if let Some(l) = self.pane_list() {
                    self.open_draft(&l, window, cx)
                }
            }
            "search" => {
                let f = self.panes[self.focused_pane].search.read(cx).focus.clone();
                window.focus(&f);
            }
            "undo" => self.undo(cx),
            "redo" => self.redo(cx),
            "new_list" => self.open_sheet(SheetKind::NewList, window, cx),
            "new_project" => self.open_sheet(SheetKind::NewProject, window, cx),
            "trash" => self.open_trash(cx),
            "open_settings" => self.open_settings(window, cx),
            "delete_sel" => {
                if let Some(t) = self.focused_task.clone() {
                    self.ask_delete(vec![t], window, cx)
                }
            }
            other => {
                if let Some(l) = other.strip_prefix("go-list-") {
                    let l = l.to_string();
                    self.show_list(&l, window, cx);
                } else if let Some(p) = other.strip_prefix("go-proj-") {
                    let p = p.to_string();
                    self.choose_project(&p, window, cx);
                }
            }
        }
        cx.notify();
    }

    pub fn open_trash(&mut self, cx: &mut Context<Self>) {
        self.rail = Rail::Trash;
        self.preview = true;
        report(cx, "trashToggled", json!({ "open": true }));
        cx.notify();
    }

    pub fn close_rail_panel(&mut self, cx: &mut Context<Self>) {
        match self.rail {
            Rail::Filter => report(cx, "filterToggled", json!({ "open": false })),
            Rail::Trash => report(cx, "trashToggled", json!({ "open": false })),
            Rail::Inspector => {}
        }
        report(cx, "closed", json!({}));
        self.rail = Rail::Inspector;
        cx.notify();
    }

    pub fn toggle_filter(&mut self, cx: &mut Context<Self>) {
        if self.rail == Rail::Filter {
            self.close_rail_panel(cx);
        } else {
            self.rail = Rail::Filter;
            self.preview = true;
            report(cx, "filterToggled", json!({ "open": true }));
        }
        cx.notify();
    }

    pub fn set_view(&mut self, list: &str, view: View, cx: &mut Context<Self>) {
        if self.view_of(list) == view {
            return;
        }
        report(
            cx,
            "viewChanged",
            json!({ "listId": list, "sort": view.sort.id(), "dir": if view.asc { "asc" } else { "desc" }, "filter": view.show.id() }),
        );
        if view.is_default() {
            self.views.remove(list);
        } else {
            self.views.insert(list.to_string(), view);
        }
        cx.notify();
    }

    pub fn toggle_completed(&mut self, cx: &mut Context<Self>) {
        let v = if self.show_completed { "0" } else { "1" };
        self.set_setting("show_completed", v, cx);
    }

    pub fn collapse_all(&mut self, collapse: bool, cx: &mut Context<Self>) {
        let pane = self.focused_pane;
        let Some(list) = self.panes[pane].list.clone() else { return };
        let parents: Vec<String> = {
            let tree = self.tree(&list);
            tree.flat().into_iter().filter(|t| !tree.kids(&t.id).is_empty()).map(|t| t.id.clone()).collect()
        };
        for id in parents {
            let was = self.panes[pane].collapsed.contains(&id);
            if collapse && !was {
                self.panes[pane].collapsed.insert(id.clone());
                report(cx, "taskExpanded", json!({ "id": id, "expanded": false }));
            } else if !collapse && was {
                self.panes[pane].collapsed.remove(&id);
                report(cx, "taskExpanded", json!({ "id": id, "expanded": true }));
            }
        }
        cx.notify();
    }

    // ---- fields ---------------------------------------------------------------------------------

    fn on_search_field(&mut self, pane: usize, e: &FieldEvent, window: &mut Window, cx: &mut Context<Self>) {
        match e {
            FieldEvent::Changed => {
                if let Some(p) = self.panes.get_mut(pane) {
                    p.match_ix = 0;
                }
                cx.notify()
            }
            FieldEvent::Submit => {
                // Enter steps through the matches.
                let rows = self.rows(pane, cx);
                let hits: Vec<String> = rows.iter().filter(|r| r.matched).map(|r| r.id.clone()).collect();
                if !hits.is_empty() {
                    let ix = self.panes[pane].match_ix % hits.len();
                    self.panes[pane].match_ix = ix + 1;
                    let id = hits[ix].clone();
                    self.focused_pane = pane;
                    self.focus_task(&id, window, cx);
                    let f = self.panes[pane].search.read(cx).focus.clone();
                    window.focus(&f);
                }
            }
            FieldEvent::Cancel => {
                self.panes[pane].search.update(cx, |f, cx| f.set_text("", cx));
                let f = self.panes[pane].focus.clone();
                window.focus(&f);
                cx.notify();
            }
            _ => {}
        }
    }

    fn on_draft_field(&mut self, _: &Entity<TextField>, e: &FieldEvent, window: &mut Window, cx: &mut Context<Self>) {
        match e {
            FieldEvent::Submit => self.commit_draft(window, cx),
            FieldEvent::Blur => {
                if self.draft.is_some() && !self.draft_title.read(cx).text().trim().is_empty() {
                    self.commit_draft(window, cx)
                }
            }
            FieldEvent::Cancel => {
                self.draft = None;
                window.focus(&self.root_focus);
                cx.notify()
            }
            _ => cx.notify(),
        }
    }

    pub fn commit_draft(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(list) = self.draft.take() else { return };
        let title = self.draft_title.read(cx).text().trim().to_string();
        if title.is_empty() {
            window.focus(&self.root_focus);
            cx.notify();
            return;
        }
        if let Some(id) = self.create_task(&list, &title, None, cx) {
            self.focus_task(&id, window, cx);
            let f = self.panes[self.focused_pane].focus.clone();
            window.focus(&f);
        }
    }

    /// "Note": an Untitled task with its note editor open.
    pub fn draft_with_note(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(list) = self.draft.take() else { return };
        let title = self.draft_title.read(cx).text().trim().to_string();
        let title = if title.is_empty() { "Untitled".to_string() } else { title };
        if let Some(id) = self.create_task(&list, &title, None, cx) {
            self.focus_task(&id, window, cx);
            self.open_note_editor(&id, None, window, cx);
        }
    }

    pub fn start_rename(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(id) = self.focused_task.clone() else { return };
        let title = self.task(&id).map(|t| t.title.clone()).unwrap_or_default();
        self.renaming = Some(id);
        self.title_field.update(cx, |f, cx| {
            f.set_text(title, cx);
            f.select_all_text(cx);
        });
        let f = self.title_field.read(cx).focus.clone();
        window.focus(&f);
        cx.notify();
    }

    fn on_title_field(&mut self, _: &Entity<TextField>, e: &FieldEvent, window: &mut Window, cx: &mut Context<Self>) {
        match e {
            FieldEvent::Submit | FieldEvent::Blur => {
                if let Some(id) = self.renaming.take() {
                    let t = self.title_field.read(cx).text().to_string();
                    self.rename_task(&id, &t, cx);
                    if matches!(e, FieldEvent::Submit) {
                        window.focus(&self.root_focus);
                    }
                }
                cx.notify()
            }
            FieldEvent::Cancel => {
                self.renaming = None;
                window.focus(&self.root_focus);
                cx.notify()
            }
            _ => cx.notify(),
        }
    }

    pub fn open_composer(&mut self, parent: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.menu = None;
        let pane = self.focused_pane;
        self.panes[pane].collapsed.remove(parent);
        self.panes[pane].composer = Some(parent.to_string());
        self.subtask_field.update(cx, |f, cx| f.set_text("", cx));
        let f = self.subtask_field.read(cx).focus.clone();
        window.focus(&f);
        report(cx, "subtaskComposerOpened", json!({ "parentId": parent }));
        cx.notify();
    }

    fn on_subtask_field(&mut self, _: &Entity<TextField>, e: &FieldEvent, window: &mut Window, cx: &mut Context<Self>) {
        let pane = self.focused_pane;
        match e {
            FieldEvent::Submit => {
                let title = self.subtask_field.read(cx).text().trim().to_string();
                let Some(parent) = self.panes[pane].composer.clone() else { return };
                if title.is_empty() {
                    self.panes[pane].composer = None;
                    window.focus(&self.panes[pane].focus.clone());
                } else if let Some(list) = self.task(&parent).map(|t| t.list_id.clone()) {
                    self.create_task(&list, &title, Some(&parent), cx);
                    self.subtask_field.update(cx, |f, cx| f.set_text("", cx));
                }
                cx.notify()
            }
            FieldEvent::Cancel => {
                self.panes[pane].composer = None;
                window.focus(&self.panes[pane].focus.clone());
                cx.notify()
            }
            _ => cx.notify(),
        }
    }

    // ---- notes ------------------------------------------------------------------------------------

    pub fn open_note_editor(&mut self, task: &str, note: Option<&str>, window: &mut Window, cx: &mut Context<Self>) {
        self.close_note_editor(window, cx);
        let html = note.and_then(|n| self.task(task).and_then(|t| t.note_items.iter().find(|x| x.id == n)).map(|x| x.content.clone())).unwrap_or_default();
        let source = model::note_source(&html);
        let field = cx.new(|cx| {
            let mut f = TextField::new("Write a note ... (drop an attachment to @mention it)", window, cx);
            f.multiline = true;
            f.set_text(source.clone(), cx);
            f
        });
        self._subs.push(cx.subscribe_in(&field, window, |ws, _, e, window, cx| ws.on_note_field(e, window, cx)));
        window.focus(&field.read(cx).focus.clone());
        self.note_edit = Some(NoteEdit { note: note.map(String::from), task: task.to_string(), field, saved: source, save_timer: None });
        cx.notify();
    }

    fn on_note_field(&mut self, e: &FieldEvent, window: &mut Window, cx: &mut Context<Self>) {
        match e {
            FieldEvent::Changed => {
                // A note saves 500 ms after typing stops.
                let timer = cx.spawn(async move |this, cx| {
                    cx.background_executor().timer(Duration::from_millis(500)).await;
                    this.update(cx, |ws, cx| ws.save_note(cx)).ok();
                });
                if let Some(ne) = &mut self.note_edit {
                    ne.save_timer = Some(timer);
                }
                cx.notify()
            }
            FieldEvent::Blur | FieldEvent::Tab | FieldEvent::Cancel => {
                self.close_note_editor(window, cx);
                if !matches!(e, FieldEvent::Blur) {
                    window.focus(&self.root_focus);
                }
            }
            _ => {}
        }
    }

    /// Writes the note being edited, if it changed: created on its first save.
    pub fn save_note(&mut self, cx: &mut Context<Self>) {
        let Some(ne) = &mut self.note_edit else { return };
        ne.save_timer = None;
        let source = ne.field.read(cx).text().to_string();
        if source == ne.saved {
            return;
        }
        let html = model::note_html(source.trim_end());
        let text = taskscape_common::storage::strip_html(&html);
        let task = ne.task.clone();
        match ne.note.clone() {
            None if text.is_empty() => return,
            None => {
                if let Ok(n) = self.db.run(self.db.store.create_note(&task, &html)) {
                    report(cx, "noteAdded", json!({ "taskId": task, "text": text }));
                    if let Some(ne) = &mut self.note_edit {
                        ne.note = Some(n.id);
                    }
                }
            }
            Some(id) => {
                if self.db.run(self.db.store.update_note(&id, &html)).is_ok() {
                    report(cx, "noteChanged", json!({ "taskId": task, "text": text }));
                }
            }
        }
        if let Some(ne) = &mut self.note_edit {
            ne.saved = source;
        }
        self.reload();
        cx.notify();
    }

    pub fn close_note_editor(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.note_edit.is_none() {
            return;
        }
        self.save_note(cx);
        if let Some(ne) = self.note_edit.take() {
            // A note emptied and left is deleted.
            let empty = ne.field.read(cx).text().trim().is_empty();
            if let (true, Some(id)) = (empty, ne.note) {
                if self.db.run(self.db.store.delete_note(&id)).is_ok() {
                    report(cx, "noteDeleted", json!({ "taskId": ne.task }));
                }
                self.reload();
            }
        }
        cx.notify();
    }

    // ---- the task list's keys ------------------------------------------------------------------

    pub fn press_row(&mut self, pane: usize, id: &str, shift: bool, cmd: bool, window: &mut Window, cx: &mut Context<Self>) {
        self.menu = None;
        self.focused_pane = pane;
        window.focus(&self.panes[pane].focus.clone());
        if cmd {
            let sel = &mut self.panes[pane].selection;
            if let Some(i) = sel.iter().position(|s| s == id) {
                sel.remove(i);
            } else {
                sel.push(id.to_string());
            }
            cx.notify();
            return;
        }
        if shift {
            let rows = self.rows(pane, cx);
            let anchor = self.panes[pane].anchor.clone().unwrap_or_else(|| id.to_string());
            let a = rows.iter().position(|r| r.id == anchor).unwrap_or(0);
            let b = rows.iter().position(|r| r.id == id).unwrap_or(0);
            let (lo, hi) = (a.min(b), a.max(b));
            self.panes[pane].selection = rows[lo..=hi].iter().map(|r| r.id.clone()).collect();
            cx.notify();
            return;
        }
        self.focus_task(id, window, cx);
    }

    pub fn step_focus(&mut self, by: isize, extend: bool, window: &mut Window, cx: &mut Context<Self>) {
        let pane = self.focused_pane;
        let rows = self.rows(pane, cx);
        if rows.is_empty() {
            return;
        }
        let cur = self.focused_task.as_ref().and_then(|f| rows.iter().position(|r| &r.id == f));
        let next = match cur {
            None => 0,
            Some(c) => (c as isize + by).clamp(0, rows.len() as isize - 1) as usize,
        };
        let id = rows[next].id.clone();
        if extend {
            let sel = &mut self.panes[pane].selection;
            if !sel.contains(&id) {
                sel.push(id.clone());
            }
            self.focused_task = Some(id.clone());
            report(cx, "taskFocused", json!({ "id": id }));
            cx.notify();
        } else {
            self.focus_task(&id, window, cx);
        }
    }

    pub fn collapse_or_parent(&mut self, expand: bool, window: &mut Window, cx: &mut Context<Self>) {
        let Some(id) = self.focused_task.clone() else { return };
        let pane = self.focused_pane;
        let Some(list) = self.task(&id).map(|t| t.list_id.clone()) else { return };
        let (has_kids, parent, first_kid) = {
            let tree = self.tree(&list);
            let kids = tree.kids(&id);
            (!kids.is_empty(), self.task(&id).and_then(|t| t.parent_id.clone()), kids.first().map(|k| k.id.clone()))
        };
        let collapsed = self.panes[pane].collapsed.contains(&id);
        if expand {
            if has_kids && collapsed {
                self.set_expanded(pane, &id, true, cx);
            } else if let Some(k) = first_kid {
                self.focus_task(&k, window, cx);
            }
        } else if has_kids && !collapsed {
            self.set_expanded(pane, &id, false, cx);
        } else if let Some(p) = parent {
            self.focus_task(&p, window, cx);
        }
    }

    pub fn set_expanded(&mut self, pane: usize, id: &str, expanded: bool, cx: &mut Context<Self>) {
        let c = &mut self.panes[pane].collapsed;
        let changed = if expanded { c.remove(id) } else { c.insert(id.to_string()) };
        if changed {
            report(cx, "taskExpanded", json!({ "id": id, "expanded": expanded }));
        }
        cx.notify();
    }

    /// Tab: under the previous sibling, at its end. ⇧Tab: after the parent.
    pub fn indent(&mut self, out: bool, cx: &mut Context<Self>) {
        let Some(id) = self.focused_task.clone() else { return };
        let Some(t) = self.task(&id).cloned() else { return };
        if out {
            let Some(parent) = t.parent_id.clone() else { return };
            let gp = self.task(&parent).and_then(|p| p.parent_id.clone());
            let sort = self.task(&parent).map(|p| p.sort_order + 0.5);
            self.move_task(&id, gp.as_deref(), Some(&t.list_id), sort, cx);
        } else {
            let prev = {
                let tree = self.tree(&t.list_id);
                let sibs = tree.siblings(&t);
                let i = sibs.iter().position(|s| s.id == id).unwrap_or(0);
                (i > 0).then(|| sibs[i - 1].id.clone())
            };
            if let Some(p) = prev {
                self.move_task(&id, Some(&p), Some(&t.list_id), None, cx);
            }
        }
    }

    /// The move-up and move-down inputs: past the next sibling.
    pub fn move_by(&mut self, down: bool, cx: &mut Context<Self>) {
        let Some(id) = self.focused_task.clone() else { return };
        let Some(t) = self.task(&id).cloned() else { return };
        let sort = {
            let tree = self.tree(&t.list_id);
            let sibs = tree.siblings(&t);
            let i = sibs.iter().position(|s| s.id == id).unwrap_or(0);
            if down {
                match (sibs.get(i + 1), sibs.get(i + 2)) {
                    (Some(a), Some(b)) => Some((a.sort_order + b.sort_order) / 2.0),
                    (Some(a), None) => Some(a.sort_order + 1.0),
                    _ => None,
                }
            } else {
                match (i.checked_sub(1).and_then(|j| sibs.get(j)), i.checked_sub(2).and_then(|j| sibs.get(j))) {
                    (Some(a), Some(b)) => Some((a.sort_order + b.sort_order) / 2.0),
                    (Some(a), None) => Some(a.sort_order - 1.0),
                    _ => None,
                }
            }
        };
        if let Some(s) = sort {
            self.move_task(&id, t.parent_id.as_deref(), Some(&t.list_id), Some(s), cx);
        }
    }

    pub fn escape(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.menu.take().is_some() || std::mem::take(&mut self.project_menu) {
            cx.notify();
            return;
        }
        if self.rail != Rail::Inspector {
            self.close_rail_panel(cx);
            return;
        }
        let pane = self.focused_pane;
        if self.panes[pane].selection.len() > 1 {
            self.panes[pane].selection.clear();
        } else if self.focused_task.is_some() {
            self.clear_selection(cx);
        } else {
            window.blur();
        }
        cx.notify();
    }

    pub fn open_menu(&mut self, task: &str, at: Point<Pixels>, cx: &mut Context<Self>) {
        self.menu = Some(Menu { task: task.to_string(), at, submenu: false });
        cx.notify();
    }

    // ---- drawing ----------------------------------------------------------------------------------

    fn render_window(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let t = self.theme();
        let has_lists = !self.project_lists().is_empty();
        let mut body = div().flex().flex_row().flex_1().min_h_0().w_full();
        if !has_lists || self.panes[0].list.is_none() {
            let (title, sub) = if has_lists { ("No list selected", "Pick a list tab to view its tasks") } else { ("No lists yet", "Create a list to start adding tasks") };
            body = body.child(
                div().flex_1().h_full().bg(t.surface_2).flex().flex_col().items_center().justify_center().gap(px(6.)).child(crate::ui::icon("notes", 30., t.content_3)).child(
                    parts::part("windowEmptyTitle", "heading").text(title).wrap(div().text_size(px(15.)).font_weight(gpui::FontWeight::MEDIUM).text_color(t.content_2).child(title)),
                ).child(div().text_size(px(12.5)).text_color(t.content_3).child(sub)),
            );
        } else {
            for i in 0..self.panes.len() {
                if i > 0 {
                    body = body.child(div().w(px(1.)).h_full().bg(t.edge_2));
                }
                body = body.child(div().flex_1().min_w_0().h_full().child(self.render_pane(i, window, cx)));
            }
        }
        if self.preview {
            body = body.child(div().w(px(320.)).flex_none().h_full().border_l_1().border_color(t.edge_2).child(self.render_rail(window, cx)));
        }
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(t.surface_1)
            .child(self.render_titlebar(window, cx))
            .child(body)
            .into_any_element()
    }

    pub fn render_rail(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        match self.rail {
            Rail::Inspector => self.render_inspector(window, cx),
            Rail::Filter => self.render_filter(window, cx),
            Rail::Trash => self.render_trash(window, cx),
        }
    }

    fn overlays(&mut self, window: &mut Window, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let mut out = vec![];
        if self.menu.is_some() {
            out.push(self.render_menu(window, cx));
        }
        if self.project_menu {
            out.push(self.render_project_menu(window, cx));
        }
        if self.toast.is_some() {
            out.push(self.render_toast(window, cx));
        }
        if self.sheet.is_some() {
            out.push(self.render_sheet(window, cx));
        }
        if self.palette.is_some() {
            out.push(self.render_palette(window, cx));
        }
        if let Some(s) = &self.settings {
            out.push(crate::settings::modal(s.clone(), &self.theme(), window, cx));
        }
        out
    }

    pub fn close_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.settings = None;
        self.hotkeys = self.db.hotkeys();
        let s = |k: &str| self.db.setting(k);
        self.show_completed = s("show_completed").as_deref() != Some("0");
        self.reduced_motion = s("reduced_motion").as_deref() == Some("1");
        self.theme_setting = s("theme").unwrap_or_else(|| "system".into());
        window.focus(&self.root_focus);
        cx.notify();
    }
}

impl Focusable for Workspace {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.root_focus.clone()
    }
}

impl Render for Workspace {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.theme();
        let content = match self.surface {
            Surface::Window => self.render_window(window, cx),
            Surface::Pane => div().size_full().bg(t.surface_1).flex().child(self.render_pane(0, window, cx)).into_any_element(),
            Surface::Inspector => self.render_inspector(window, cx),
            Surface::Filter => self.render_filter(window, cx),
            Surface::Trash => self.render_trash(window, cx),
            Surface::TitleBar => div().size_full().bg(t.surface_1).flex().flex_col().child(self.render_titlebar(window, cx)).into_any_element(),
            Surface::Sheet | Surface::Palette | Surface::Toast | Surface::Settings => div().size_full().bg(t.surface_1).into_any_element(),
        };
        let overlays = self.overlays(window, cx);
        let root = div()
            .id("workspace")
            .key_context("Workspace")
            .track_focus(&self.root_focus)
            .relative()
            .size_full()
            .font_family(crate::system_font())
            .text_size(px(14.))
            .line_height(px(19.))
            .text_color(t.content_1)
            .bg(t.surface_1)
            .on_action(cx.listener(|ws, _: &NewTask, w, cx| {
                if let Some(l) = ws.pane_list() {
                    ws.open_draft(&l, w, cx)
                }
            }))
            .on_action(cx.listener(|ws, _: &Search, w, cx| {
                let f = ws.panes[ws.focused_pane].search.read(cx).focus.clone();
                w.focus(&f);
            }))
            .on_action(cx.listener(|ws, _: &TogglePreview, _, cx| ws.set_preview(!ws.preview, cx)))
            .on_action(cx.listener(|ws, _: &OpenSettings, w, cx| ws.open_settings(w, cx)))
            .on_action(cx.listener(|ws, _: &SwitchList1, w, cx| ws.switch_to(0, w, cx)))
            .on_action(cx.listener(|ws, _: &SwitchList2, w, cx| ws.switch_to(1, w, cx)))
            .on_action(cx.listener(|ws, _: &SwitchList3, w, cx| ws.switch_to(2, w, cx)))
            .on_action(cx.listener(|ws, _: &SwitchList4, w, cx| ws.switch_to(3, w, cx)))
            .on_action(cx.listener(|ws, _: &SwitchList5, w, cx| ws.switch_to(4, w, cx)))
            .on_action(cx.listener(|ws, _: &SwitchList6, w, cx| ws.switch_to(5, w, cx)))
            .on_action(cx.listener(|ws, _: &SwitchList7, w, cx| ws.switch_to(6, w, cx)))
            .on_action(cx.listener(|ws, _: &SwitchList8, w, cx| ws.switch_to(7, w, cx)))
            .on_action(cx.listener(|ws, _: &SwitchList9, w, cx| ws.switch_to(8, w, cx)))
            .on_action(cx.listener(|ws, _: &PrevTab, w, cx| ws.tab_step(-1, w, cx)))
            .on_action(cx.listener(|ws, _: &NextTab, w, cx| ws.tab_step(1, w, cx)))
            .on_action(cx.listener(|ws, _: &PrevProject, w, cx| ws.project_step(-1, w, cx)))
            .on_action(cx.listener(|ws, _: &NextProject, w, cx| ws.project_step(1, w, cx)))
            .on_action(cx.listener(|ws, _: &CommandPalette, w, cx| ws.open_palette(w, cx)))
            .on_action(cx.listener(|ws, _: &ToggleSplit, w, cx| ws.toggle_split(w, cx)))
            .on_action(cx.listener(|ws, _: &FocusOtherPane, w, _| {
                if ws.panes.len() > 1 {
                    ws.focused_pane = 1 - ws.focused_pane;
                    w.focus(&ws.panes[ws.focused_pane].focus.clone());
                }
            }))
            .on_action(cx.listener(|ws, _: &NewList, w, cx| ws.open_sheet(SheetKind::NewList, w, cx)))
            .on_action(cx.listener(|ws, _: &NewProject, w, cx| ws.open_sheet(SheetKind::NewProject, w, cx)))
            .on_action(cx.listener(|ws, _: &ToggleCompleted, _, cx| ws.toggle_completed(cx)))
            .on_action(cx.listener(|ws, _: &CollapseAll, _, cx| ws.collapse_all(true, cx)))
            .on_action(cx.listener(|ws, _: &ExpandAll, _, cx| ws.collapse_all(false, cx)))
            .on_action(cx.listener(|ws, _: &Undo, _, cx| ws.undo(cx)))
            .on_action(cx.listener(|ws, _: &Redo, _, cx| ws.redo(cx)))
            .on_action(cx.listener(|ws, _: &Escape, w, cx| ws.escape(w, cx)))
            .on_mouse_down(gpui::MouseButton::Left, cx.listener(|ws, _, _, cx| {
                if ws.menu.take().is_some() || std::mem::take(&mut ws.project_menu) {
                    cx.notify();
                }
            }))
            .child(content)
            .children(overlays);
        parts::frame(root)
    }
}

impl Workspace {
    pub fn project_step(&mut self, by: isize, window: &mut Window, cx: &mut Context<Self>) {
        let ids: Vec<String> = self.data.projects.iter().map(|p| p.id.clone()).collect();
        if ids.is_empty() {
            return;
        }
        let cur = self.active_project.as_ref().and_then(|a| ids.iter().position(|p| p == a)).unwrap_or(0) as isize;
        let next = ids[(cur + by).rem_euclid(ids.len() as isize) as usize].clone();
        self.choose_project(&next, window, cx);
    }

    /// The window's bounds for a surface mounted on its own.
    pub fn default_bounds(w: f32, h: f32) -> Bounds<Pixels> {
        Bounds { origin: point(px(0.), px(0.)), size: size(px(w), px(h)) }
    }

    pub fn key_of(&self, _id: &SharedString) {}
}
