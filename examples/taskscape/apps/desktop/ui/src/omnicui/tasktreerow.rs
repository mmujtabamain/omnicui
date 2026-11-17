//! Generated from omnicui/contracts/tasktreerow.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.tasktreerow`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct TasktreerowProps {
    pub title: String,
    pub depth: i64,
    pub done: bool,
    pub selected: bool,
    pub picked: bool,
    pub dragging: bool,
    pub dropIndicator: String,
    pub checkAction: String,
    pub draggable: bool,
    pub hasChildren: bool,
    pub expanded: bool,
}

impl Default for TasktreerowProps {
    fn default() -> Self {
        Self {
            title: "".to_owned(),
            depth: 0,
            done: false,
            selected: false,
            picked: false,
            dragging: false,
            dropIndicator: "none".to_owned(),
            checkAction: "select".to_owned(),
            draggable: true,
            hasChildren: false,
            expanded: false,
        }
    }
}

/// The events `ui.tasktreerow` reports.
#[derive(Debug, Clone)]
pub enum TasktreerowEvent {
    Activate { platformModifier: bool, shift: bool },
    Check { action: String },
    ToggleDone {  },
    ContextMenu {  },
    Drop { position: String },
    ToggleExpanded { expanded: bool },
}

impl TasktreerowEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::Activate { platformModifier, shift } => ("activate", serde_json::json!({ "platformModifier": platformModifier, "shift": shift })),
            Self::Check { action } => ("check", serde_json::json!({ "action": action })),
            Self::ToggleDone {  } => ("toggleDone", serde_json::json!({  })),
            Self::ContextMenu {  } => ("contextMenu", serde_json::json!({  })),
            Self::Drop { position } => ("drop", serde_json::json!({ "position": position })),
            Self::ToggleExpanded { expanded } => ("toggleExpanded", serde_json::json!({ "expanded": expanded })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const ROW: &str = "ui.tasktreerow.row";
    pub const CHECK: &str = "ui.tasktreerow.check";
    pub const TITLE: &str = "ui.tasktreerow.title";
}
