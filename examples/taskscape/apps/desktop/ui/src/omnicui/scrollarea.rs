//! Generated from omnicui/contracts/scrollarea.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.scrollarea`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct ScrollareaProps {
    pub orientation: String,
    pub scrollbar: String,
    pub tone: String,
    pub label: String,
    pub overscroll: String,
    pub dropTarget: bool,
}

impl Default for ScrollareaProps {
    fn default() -> Self {
        Self {
            orientation: "vertical".to_owned(),
            scrollbar: "native".to_owned(),
            tone: "base".to_owned(),
            label: "".to_owned(),
            overscroll: "contain".to_owned(),
            dropTarget: false,
        }
    }
}

/// The events `ui.scrollarea` reports.
#[derive(Debug, Clone)]
pub enum ScrollareaEvent {
    ScrollingChanged { scrolling: bool },
    BottomChanged { atBottom: bool },
    DropOverChanged { over: bool },
    RootDropped { taskId: String },
    PressBlank { clearedSelection: bool },
}

impl ScrollareaEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::ScrollingChanged { scrolling } => ("scrollingChanged", serde_json::json!({ "scrolling": scrolling })),
            Self::BottomChanged { atBottom } => ("bottomChanged", serde_json::json!({ "atBottom": atBottom })),
            Self::DropOverChanged { over } => ("dropOverChanged", serde_json::json!({ "over": over })),
            Self::RootDropped { taskId } => ("rootDropped", serde_json::json!({ "taskId": taskId })),
            Self::PressBlank { clearedSelection } => ("pressBlank", serde_json::json!({ "clearedSelection": clearedSelection })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const REGION: &str = "ui.scrollarea.region";
    pub const FIRSTROW: &str = "ui.scrollarea.firstRow";
    pub const LASTROW: &str = "ui.scrollarea.lastRow";
    pub const THUMB: &str = "ui.scrollarea.thumb";
}
