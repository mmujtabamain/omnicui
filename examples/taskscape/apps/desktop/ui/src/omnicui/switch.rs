//! Generated from omnicui/contracts/switch.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.switch`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct SwitchProps {
    pub checked: bool,
    pub disabled: bool,
    pub reason: String,
    pub ariaLabel: String,
}

impl Default for SwitchProps {
    fn default() -> Self {
        Self {
            checked: false,
            disabled: false,
            reason: "".to_owned(),
            ariaLabel: "".to_owned(),
        }
    }
}

/// The events `ui.switch` reports.
#[derive(Debug, Clone)]
pub enum SwitchEvent {
    Changed { checked: bool },
}

impl SwitchEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::Changed { checked } => ("changed", serde_json::json!({ "checked": checked })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const SWITCH: &str = "ui.switch.switch";
    pub const THUMB: &str = "ui.switch.thumb";
    pub const REASON: &str = "ui.switch.reason";
}
