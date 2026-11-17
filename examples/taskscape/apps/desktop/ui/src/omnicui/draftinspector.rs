//! Generated from omnicui/contracts/draftinspector.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `app.draftinspector`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct DraftinspectorProps {
    pub dataset: String,
    pub list: String,
}

impl Default for DraftinspectorProps {
    fn default() -> Self {
        Self {
            dataset: "standard".to_owned(),
            list: "Errands".to_owned(),
        }
    }
}

/// The events `app.draftinspector` reports.
#[derive(Debug, Clone)]
pub enum DraftinspectorEvent {
    DraftAction { action: String, title: String },
}

impl DraftinspectorEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::DraftAction { action, title } => ("draftAction", serde_json::json!({ "action": action, "title": title })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const TITLE: &str = "app.draftinspector.title";
    pub const BREADCRUMB: &str = "app.draftinspector.breadcrumb";
    pub const DISCARDDRAFT: &str = "app.draftinspector.discardDraft";
    pub const STARTWITH: &str = "app.draftinspector.startWith";
    pub const NOTEBUTTON: &str = "app.draftinspector.noteButton";
    pub const IMAGEBUTTON: &str = "app.draftinspector.imageButton";
    pub const HINT: &str = "app.draftinspector.hint";
}
