//! What the user is looking at, as opposed to the data: the web's state/ui.ts, for the desktop.
use crate::data::Id;
use std::collections::HashMap;

/// What the main window's right-hand panel shows. On the desktop the Trash and Settings are
/// windows of their own (decisions 8, 19), so the aside is the preview or Filter & Sort.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Aside {
    Preview,
    Filter,
    None,
}

/// Filter & Sort for one list (the web's filter-panel model).
#[derive(Clone, Debug, PartialEq)]
pub struct View {
    /// "manual", "title", "created", "updated"
    pub sort: String,
    pub descending: bool,
    /// "all", "open", "done"
    pub show: String,
    /// Refine: only tasks with notes, with attachments, with subtasks; or without them.
    pub refine: Vec<String>,
    /// "any", "today", "week", "month"
    pub date: String,
}

impl Default for View {
    fn default() -> Self {
        Self { sort: "manual".into(), descending: false, show: "all".into(), refine: vec![], date: "any".into() }
    }
}

impl View {
    /// Whether it differs from the default (the footer then reads "Filtered").
    pub fn active(&self) -> bool {
        *self != View::default()
    }
}

pub const PREVIEW_MIN: f32 = 280.0;
pub const PREVIEW_MAX: f32 = 420.0;
pub const SPLIT_MIN: f32 = 0.25;
pub const SPLIT_MAX: f32 = 0.75;

#[derive(Clone, Debug)]
pub struct UiState {
    pub project: Option<Id>,
    /// The active list of each pane; two panes in split view.
    pub panes: Vec<Option<Id>>,
    /// The pane last clicked: keys and the tab highlight follow it.
    pub active_pane: usize,
    /// Picked task ids (the bulk bar shows when there are any).
    pub selection: Vec<Id>,
    /// The task shown in the inspector.
    pub preview: Option<Id>,
    /// A task whose inspector opens with a new note ready to type.
    pub note_for: Option<Id>,
    /// Filter & Sort per list.
    pub views: HashMap<Id, View>,
    pub aside: Aside,
    pub palette_open: bool,
    /// The list a new-task draft is started in, if one is open.
    pub draft_list: Option<Id>,
    pub preview_width: f32,
    pub split_ratio: f32,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            project: None,
            panes: vec![None],
            active_pane: 0,
            selection: vec![],
            preview: None,
            note_for: None,
            views: HashMap::new(),
            aside: Aside::Preview,
            palette_open: false,
            draft_list: None,
            preview_width: 322.0,
            split_ratio: 0.5,
        }
    }
}

impl UiState {
    pub fn set_project(&mut self, id: Option<Id>) {
        self.project = id;
        self.panes = vec![None];
        self.active_pane = 0;
        self.selection.clear();
        self.preview = None;
        self.draft_list = None;
    }
    /// Opens a list in a pane (the active one when None).
    pub fn open_list(&mut self, id: Option<Id>, pane: Option<usize>) {
        let i = pane.unwrap_or(self.active_pane).min(self.panes.len().saturating_sub(1));
        self.panes[i] = id;
        self.active_pane = i;
        self.selection.clear();
    }
    pub fn split(&mut self, on: bool) {
        if on {
            let left = self.panes[0].clone();
            let right = self.panes.get(1).cloned().flatten().or(left.clone());
            self.panes = vec![left, right];
        } else {
            let keep = self.panes.get(self.active_pane).cloned().flatten();
            self.panes = vec![keep];
            self.active_pane = 0;
        }
    }
    pub fn split_on(&self) -> bool {
        self.panes.len() > 1
    }
    pub fn view(&self, list: &str) -> View {
        self.views.get(list).cloned().unwrap_or_default()
    }
    pub fn set_preview_width(&mut self, px: f32) {
        self.preview_width = px.clamp(PREVIEW_MIN, PREVIEW_MAX);
    }
    pub fn set_split_ratio(&mut self, r: f32) {
        self.split_ratio = r.clamp(SPLIT_MIN, SPLIT_MAX);
    }
}
