//! Generated from omnicui/contracts/subtaskrow.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.subtaskrow`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct SubtaskrowProps {
    pub title: String,
    pub done: bool,
    pub depth: f64,
}

impl Default for SubtaskrowProps {
    fn default() -> Self {
        Self {
            title: "".to_owned(),
            done: false,
            depth: 0.0,
        }
    }
}

/// The events `ui.subtaskrow` reports.
#[derive(Debug, Clone)]
pub enum SubtaskrowEvent {
    Toggled { done: bool },
    Selected {  },
}

impl SubtaskrowEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::Toggled { done } => ("toggled", serde_json::json!({ "done": done })),
            Self::Selected {  } => ("selected", serde_json::json!({  })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const ROW: &str = "ui.subtaskrow.row";
    pub const CHECKBOX: &str = "ui.subtaskrow.checkbox";
    pub const TITLE: &str = "ui.subtaskrow.title";
    pub const CHILDREN: &str = "ui.subtaskrow.children";
}
