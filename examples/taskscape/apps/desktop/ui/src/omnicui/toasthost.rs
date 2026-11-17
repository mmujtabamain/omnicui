//! Generated from omnicui/contracts/toasthost.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `app.toasthost`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct ToasthostProps {
    pub dataset: String,
    pub bulkTrash: i64,
    pub trash: String,
    pub trashAgain: String,
    pub undoFails: bool,
}

impl Default for ToasthostProps {
    fn default() -> Self {
        Self {
            dataset: "standard".to_owned(),
            bulkTrash: 0,
            trash: "".to_owned(),
            trashAgain: "".to_owned(),
            undoFails: false,
        }
    }
}

/// The events `app.toasthost` reports.
#[derive(Debug, Clone)]
pub enum ToasthostEvent {
    Undone { count: i64, title: String },
    Dismissed { via: String },
}

impl ToasthostEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::Undone { count, title } => ("undone", serde_json::json!({ "count": count, "title": title })),
            Self::Dismissed { via } => ("dismissed", serde_json::json!({ "via": via })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const NOTICE: &str = "app.toasthost.notice";
    pub const MESSAGE: &str = "app.toasthost.message";
    pub const UNDO: &str = "app.toasthost.undo";
    pub const DISMISS: &str = "app.toasthost.dismiss";
}
