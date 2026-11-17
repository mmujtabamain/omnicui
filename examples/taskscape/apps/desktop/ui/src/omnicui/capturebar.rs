//! Generated from omnicui/contracts/capturebar.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `app.capturebar`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct CapturebarProps {
    pub dataset: String,
    pub draft: String,
    pub captureResult: String,
}

impl Default for CapturebarProps {
    fn default() -> Self {
        Self {
            dataset: "standard".to_owned(),
            draft: "empty".to_owned(),
            captureResult: "captured".to_owned(),
        }
    }
}

/// The events `app.capturebar` reports.
#[derive(Debug, Clone)]
pub enum CapturebarEvent {
    DraftChanged { title: String, notes: String, shots: i64 },
    TaskSaved { title: String, notes: String, shots: i64, project: String, list: String },
    BarHidden { title: String, notes: String },
    MainRequested { project: String, list: String },
    CaptureStarted { pending: i64 },
}

impl CapturebarEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::DraftChanged { title, notes, shots } => ("draftChanged", serde_json::json!({ "title": title, "notes": notes, "shots": shots })),
            Self::TaskSaved { title, notes, shots, project, list } => ("taskSaved", serde_json::json!({ "title": title, "notes": notes, "shots": shots, "project": project, "list": list })),
            Self::BarHidden { title, notes } => ("barHidden", serde_json::json!({ "title": title, "notes": notes })),
            Self::MainRequested { project, list } => ("mainRequested", serde_json::json!({ "project": project, "list": list })),
            Self::CaptureStarted { pending } => ("captureStarted", serde_json::json!({ "pending": pending })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const CARD: &str = "app.capturebar.card";
    pub const TITLEROW: &str = "app.capturebar.titleRow";
    pub const TITLE: &str = "app.capturebar.title";
    pub const CLEAR: &str = "app.capturebar.clear";
    pub const NOTESHINT: &str = "app.capturebar.notesHint";
    pub const NOTES: &str = "app.capturebar.notes";
    pub const TOOLBAR: &str = "app.capturebar.toolbar";
    pub const BOLD: &str = "app.capturebar.bold";
    pub const ITALIC: &str = "app.capturebar.italic";
    pub const UNDERLINE: &str = "app.capturebar.underline";
    pub const STRIKETHROUGH: &str = "app.capturebar.strikethrough";
    pub const BULLETLIST: &str = "app.capturebar.bulletList";
    pub const NUMBEREDLIST: &str = "app.capturebar.numberedList";
    pub const COUNTER: &str = "app.capturebar.counter";
    pub const FOOTER: &str = "app.capturebar.footer";
    pub const TARGET: &str = "app.capturebar.target";
    pub const SCREENSHOT: &str = "app.capturebar.screenshot";
    pub const SPINNER: &str = "app.capturebar.spinner";
}
