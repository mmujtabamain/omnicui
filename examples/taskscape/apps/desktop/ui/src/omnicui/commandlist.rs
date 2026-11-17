//! Generated from omnicui/contracts/commandlist.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.commandlist`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct CommandlistProps {
    pub content: String,
    pub activeId: String,
    pub emptyMessage: String,
    pub rowCount: i64,
}

impl Default for CommandlistProps {
    fn default() -> Self {
        Self {
            content: "rows".to_owned(),
            activeId: "".to_owned(),
            emptyMessage: "No results".to_owned(),
            rowCount: 6,
        }
    }
}

/// The events `ui.commandlist` reports.
#[derive(Debug, Clone)]
pub enum CommandlistEvent {
    Picked { id: String },
}

impl CommandlistEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::Picked { id } => ("picked", serde_json::json!({ "id": id })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const LIST: &str = "ui.commandlist.list";
    pub const FIRSTGROUP: &str = "ui.commandlist.firstGroup";
    pub const FIRSTGROUPHEADING: &str = "ui.commandlist.firstGroupHeading";
    pub const FIRSTROW: &str = "ui.commandlist.firstRow";
    pub const LASTROW: &str = "ui.commandlist.lastRow";
    pub const EMPTY: &str = "ui.commandlist.empty";
}
