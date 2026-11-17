//! Generated from omnicui/contracts/taskrow.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.taskrow`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct TaskrowProps {
    pub title: String,
    pub done: bool,
}

impl Default for TaskrowProps {
    fn default() -> Self {
        Self {
            title: "".to_owned(),
            done: false,
        }
    }
}

/// The events `ui.taskrow` reports.
#[derive(Debug, Clone)]
pub enum TaskrowEvent {
    Toggled { done: bool },
    Removed { title: String },
}

impl TaskrowEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::Toggled { done } => ("toggled", serde_json::json!({ "done": done })),
            Self::Removed { title } => ("removed", serde_json::json!({ "title": title })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const ROW: &str = "ui.taskrow.row";
    pub const TITLE: &str = "ui.taskrow.title";
    pub const DONE: &str = "ui.taskrow.done";
    pub const STATUS: &str = "ui.taskrow.status";
    pub const REMOVE: &str = "ui.taskrow.remove";
}
