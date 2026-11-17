//! Generated from omnicui/contracts/commandpalette.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.commandpalette`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct CommandpaletteProps {
    pub presentation: String,
    pub open: bool,
    pub query: String,
    pub activeIndex: i64,
    pub ariaLabel: String,
    pub commandSet: String,
}

impl Default for CommandpaletteProps {
    fn default() -> Self {
        Self {
            presentation: "overlay".to_owned(),
            open: true,
            query: "".to_owned(),
            activeIndex: 0,
            ariaLabel: "Command palette".to_owned(),
            commandSet: "all".to_owned(),
        }
    }
}

/// The events `ui.commandpalette` reports.
#[derive(Debug, Clone)]
pub enum CommandpaletteEvent {
    ActiveChange { index: i64 },
    Select { index: i64 },
    Close { reason: String },
}

impl CommandpaletteEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::ActiveChange { index } => ("activeChange", serde_json::json!({ "index": index })),
            Self::Select { index } => ("select", serde_json::json!({ "index": index })),
            Self::Close { reason } => ("close", serde_json::json!({ "reason": reason })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const PANEL: &str = "ui.commandpalette.panel";
    pub const SEARCH: &str = "ui.commandpalette.search";
    pub const LIST: &str = "ui.commandpalette.list";
    pub const FIRSTROW: &str = "ui.commandpalette.firstRow";
    pub const SECONDROW: &str = "ui.commandpalette.secondRow";
    pub const THIRDROW: &str = "ui.commandpalette.thirdRow";
    pub const LASTROW: &str = "ui.commandpalette.lastRow";
    pub const EMPTYMESSAGE: &str = "ui.commandpalette.emptyMessage";
}
