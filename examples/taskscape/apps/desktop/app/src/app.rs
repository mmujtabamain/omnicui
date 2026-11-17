//! The app: its data and what the user is looking at, plus the things every screen shares (the toast,
//! the windows that are open). Screens take `&mut App`.
use crate::data::{Data, Id};
use crate::state::UiState;

/// The toast at the bottom of the main window (after Move to Trash, or a refused save).
#[derive(Clone, Debug)]
pub struct Toast {
    pub message: String,
    /// Undo puts these back from the Trash.
    pub undo: Vec<Id>,
    /// The save that failed, shown instead of the message, with Try again.
    pub error: Option<String>,
}

/// A confirmation or prompt window (decision: real windows on the desktop, no backdrop).
#[derive(Clone, Debug, PartialEq)]
pub enum Sheet {
    NewList,
    RenameList(Id),
    DeleteList(Id),
    NewProject,
    RenameProject(Id),
    DeleteProject(Id),
    /// Move to Trash, asked first (decision 10).
    DeleteTask(Id),
    EmptyTrash,
    PurgeTasks(Vec<Id>),
}

pub struct App {
    pub data: Data,
    pub ui: UiState,
    pub toast: Option<Toast>,
    pub sheet: Option<Sheet>,
    /// The Trash and Settings windows (decisions 8, 19), and which Settings page is shown.
    pub trash_window: bool,
    pub settings_window: Option<String>,
    /// The capture bar window and its pending draft.
    pub capture_open: bool,
    /// The error a save returned, for the toast.
    pub last_error: Option<String>,
}

impl App {
    pub fn new(data: Data) -> Self {
        let mut app = Self { data, ui: UiState::default(), toast: None, sheet: None, trash_window: false, settings_window: None, capture_open: false, last_error: None };
        app.open_last();
        app
    }

    /// The demo data set, opened on its last list, as every screen test starts.
    pub fn demo(dataset: &str) -> anyhow::Result<Self> {
        Ok(Self::new(Data::demo(dataset)?))
    }

    /// Opens the project and list the app had open last (remembered across launches).
    pub fn open_last(&mut self) {
        let project = self.data.get_setting("last_active_project").filter(|p| self.data.project(p).is_some()).or_else(|| self.data.projects.first().map(|p| p.id.clone()));
        self.ui.set_project(project.clone());
        let list = self.data.get_setting("last_active_list").filter(|l| self.data.list(l).is_some()).or_else(|| project.as_deref().and_then(|p| self.data.lists_of(p).first().map(|l| l.id.clone())));
        self.ui.open_list(list, Some(0));
    }

    /// Opens a project by name and, if given, one of its lists by name: how a test's props say where
    /// a screen starts. Unknown names leave things as they are.
    pub fn open_named(&mut self, project: Option<&str>, list: Option<&str>) {
        if let Some(p) = project.and_then(|n| self.data.project_named(n)).map(|p| p.id.clone()) {
            self.ui.set_project(Some(p.clone()));
            let first = self.data.lists_of(&p).first().map(|l| l.id.clone());
            self.ui.open_list(first, Some(0));
        }
        if let Some(l) = list.and_then(|n| self.data.list_named(self.ui.project.as_deref(), n)).map(|l| l.id.clone()) {
            self.ui.open_list(Some(l), Some(0));
        }
    }

    /// The list of the active pane.
    pub fn active_list(&self) -> Option<Id> {
        self.ui.panes.get(self.ui.active_pane).cloned().flatten()
    }

    /// Remembers what is open, as the reference does.
    pub fn remember(&mut self) {
        if let Some(p) = self.ui.project.clone() {
            let _ = self.data.set_setting("last_active_project", &p);
        }
        if let Some(l) = self.active_list() {
            let _ = self.data.set_setting("last_active_list", &l);
        }
    }

    /// Moves tasks to the Trash and raises the toast with Undo.
    pub fn trash(&mut self, ids: &[Id]) {
        match self.data.trash_tasks(ids) {
            Ok(gone) => {
                let gone = if gone.is_empty() { ids.to_vec() } else { gone };
                self.ui.selection.retain(|s| !gone.contains(s));
                if self.ui.preview.as_ref().is_some_and(|p| gone.contains(p)) {
                    self.ui.preview = None;
                }
                // One task (with its subtasks) reads "Moved to Trash"; several picked roots read "<n> moved to Trash".
                let message = if ids.len() > 1 { format!("{} moved to Trash", ids.len()) } else { "Moved to Trash".into() };
                self.toast = Some(Toast { message, undo: gone, error: None });
            }
            Err(e) => self.refused(e),
        }
    }

    /// Undo of the last Move to Trash.
    pub fn undo(&mut self) {
        if let Some(t) = self.toast.take() {
            if !t.undo.is_empty() {
                if let Err(e) = self.data.restore_tasks(&t.undo) {
                    self.refused(e);
                }
            }
        }
    }

    /// A save the database refused: the data is already read back; the toast says so.
    pub fn refused(&mut self, e: anyhow::Error) {
        self.last_error = Some(e.to_string());
        self.toast = Some(Toast { message: String::new(), undo: vec![], error: Some("Couldn't save that change".into()) });
    }
}
