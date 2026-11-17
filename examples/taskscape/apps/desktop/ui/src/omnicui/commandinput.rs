//! Generated from omnicui/contracts/commandinput.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.commandinput`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct CommandinputProps {
    pub value: String,
    pub placeholder: String,
    pub leading: String,
    pub autoFocus: bool,
    pub ariaLabel: String,
    pub ariaExpanded: bool,
    pub ariaControls: String,
    pub activeDescendantId: String,
}

impl Default for CommandinputProps {
    fn default() -> Self {
        Self {
            value: "".to_owned(),
            placeholder: "".to_owned(),
            leading: "icon".to_owned(),
            autoFocus: true,
            ariaLabel: "".to_owned(),
            ariaExpanded: false,
            ariaControls: "".to_owned(),
            activeDescendantId: "".to_owned(),
        }
    }
}

/// The events `ui.commandinput` reports.
#[derive(Debug, Clone)]
pub enum CommandinputEvent {
    QueryChanged { value: String },
    MoveActive { direction: String },
    Submit { value: String },
}

impl CommandinputEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::QueryChanged { value } => ("queryChanged", serde_json::json!({ "value": value })),
            Self::MoveActive { direction } => ("moveActive", serde_json::json!({ "direction": direction })),
            Self::Submit { value } => ("submit", serde_json::json!({ "value": value })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const INPUT: &str = "ui.commandinput.input";
    pub const LEADING: &str = "ui.commandinput.leading";
    pub const ROW: &str = "ui.commandinput.row";
}
