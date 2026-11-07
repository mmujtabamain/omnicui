//! Generated from omnicui/contracts/switch.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.switch`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct SwitchProps {
    pub on: bool,
    pub label: String,
}

impl Default for SwitchProps {
    fn default() -> Self {
        Self {
            on: false,
            label: "Dark mode".to_owned(),
        }
    }
}

/// The events `ui.switch` reports.
#[derive(Debug, Clone)]
pub enum SwitchEvent {
    Toggled { on: bool },
}

impl SwitchEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::Toggled { on } => ("toggled", serde_json::json!({ "on": on })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const SWITCH: &str = "ui.switch.switch";
    pub const THUMB: &str = "ui.switch.thumb";
}
