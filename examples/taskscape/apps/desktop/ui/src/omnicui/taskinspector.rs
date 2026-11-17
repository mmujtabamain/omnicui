//! Generated from omnicui/contracts/taskinspector.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `app.taskinspector`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct TaskinspectorProps {
    pub dataset: String,
    pub task: String,
}

impl Default for TaskinspectorProps {
    fn default() -> Self {
        Self {
            dataset: "standard".to_owned(),
            task: "Book bike service".to_owned(),
        }
    }
}

/// The events `app.taskinspector` reports.
#[derive(Debug, Clone)]
pub enum TaskinspectorEvent {
    TitleChanged { title: String },
    DoneChanged { title: String, done: bool },
    PreviewOpenChanged { open: bool },
    DeleteRequested { target: String, title: String },
    TaskSelected { title: String },
    NotesChanged { count: i64 },
    NoteSaved { text: String },
    AttachRequested { kind: String },
    LinkRequested { r#where: String },
    AttachmentAction { action: String, name: String },
}

impl TaskinspectorEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::TitleChanged { title } => ("titleChanged", serde_json::json!({ "title": title })),
            Self::DoneChanged { title, done } => ("doneChanged", serde_json::json!({ "title": title, "done": done })),
            Self::PreviewOpenChanged { open } => ("previewOpenChanged", serde_json::json!({ "open": open })),
            Self::DeleteRequested { target, title } => ("deleteRequested", serde_json::json!({ "target": target, "title": title })),
            Self::TaskSelected { title } => ("taskSelected", serde_json::json!({ "title": title })),
            Self::NotesChanged { count } => ("notesChanged", serde_json::json!({ "count": count })),
            Self::NoteSaved { text } => ("noteSaved", serde_json::json!({ "text": text })),
            Self::AttachRequested { kind } => ("attachRequested", serde_json::json!({ "kind": kind })),
            Self::LinkRequested { r#where } => ("linkRequested", serde_json::json!({ "where": r#where })),
            Self::AttachmentAction { action, name } => ("attachmentAction", serde_json::json!({ "action": action, "name": name })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const ACCENTTICK: &str = "app.taskinspector.accentTick";
    pub const DONETOGGLE: &str = "app.taskinspector.doneToggle";
    pub const TITLE: &str = "app.taskinspector.title";
    pub const BREADCRUMB: &str = "app.taskinspector.breadcrumb";
    pub const CREATED: &str = "app.taskinspector.created";
    pub const UPDATED: &str = "app.taskinspector.updated";
    pub const CLOSEPANEL: &str = "app.taskinspector.closePanel";
    pub const RENAME: &str = "app.taskinspector.rename";
    pub const COPYTASK: &str = "app.taskinspector.copyTask";
    pub const DELETETASK: &str = "app.taskinspector.deleteTask";
    pub const NOTESHEADING: &str = "app.taskinspector.notesHeading";
    pub const NOTESCOUNT: &str = "app.taskinspector.notesCount";
    pub const ADDNOTE: &str = "app.taskinspector.addNote";
    pub const NOTEBODY: &str = "app.taskinspector.noteBody";
    pub const NOTEBODYSECOND: &str = "app.taskinspector.noteBodySecond";
    pub const NOTEDELETE: &str = "app.taskinspector.noteDelete";
    pub const NEWNOTEEDITOR: &str = "app.taskinspector.newNoteEditor";
    pub const NOTEBOLD: &str = "app.taskinspector.noteBold";
    pub const NOTEITALIC: &str = "app.taskinspector.noteItalic";
    pub const NOTEUNDERLINE: &str = "app.taskinspector.noteUnderline";
    pub const NOTESTRIKE: &str = "app.taskinspector.noteStrike";
    pub const NOTEBULLET: &str = "app.taskinspector.noteBullet";
    pub const NOTENUMBERED: &str = "app.taskinspector.noteNumbered";
    pub const NOTELINK: &str = "app.taskinspector.noteLink";
    pub const NOTECHARS: &str = "app.taskinspector.noteChars";
    pub const NOTECHARSSECOND: &str = "app.taskinspector.noteCharsSecond";
    pub const MENTIONPOPUP: &str = "app.taskinspector.mentionPopup";
    pub const MENTIONITEM: &str = "app.taskinspector.mentionItem";
    pub const MENTIONCHIP: &str = "app.taskinspector.mentionChip";
    pub const SUBTASKSHEADING: &str = "app.taskinspector.subtasksHeading";
    pub const SUBTASKSCOUNT: &str = "app.taskinspector.subtasksCount";
    pub const SUBTASKDONE: &str = "app.taskinspector.subtaskDone";
    pub const SUBTASKTITLE: &str = "app.taskinspector.subtaskTitle";
    pub const SUBTASKSOURDOUGH: &str = "app.taskinspector.subtaskSourdough";
    pub const ATTACHMENTSHEADING: &str = "app.taskinspector.attachmentsHeading";
    pub const ATTACHFILE: &str = "app.taskinspector.attachFile";
    pub const ATTACHLINK: &str = "app.taskinspector.attachLink";
    pub const ATTACHSHOT: &str = "app.taskinspector.attachShot";
    pub const EMPTYCOPYFILE: &str = "app.taskinspector.emptyCopyFile";
    pub const EMPTYADDLINK: &str = "app.taskinspector.emptyAddLink";
    pub const ATTACHMENTTILELINK: &str = "app.taskinspector.attachmentTileLink";
    pub const ATTACHMENTTILEIMAGE: &str = "app.taskinspector.attachmentTileImage";
    pub const LIGHTBOX: &str = "app.taskinspector.lightbox";
    pub const LIGHTBOXFILENAME: &str = "app.taskinspector.lightboxFileName";
}
