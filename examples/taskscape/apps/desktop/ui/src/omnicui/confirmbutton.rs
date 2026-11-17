//! Generated from omnicui/contracts/confirmbutton.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.confirmbutton`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct ConfirmbuttonProps {
    pub tone: String,
    pub label: String,
    pub confirmLabel: String,
}

impl Default for ConfirmbuttonProps {
    fn default() -> Self {
        Self {
            tone: "default".to_owned(),
            label: "Reset all settings…".to_owned(),
            confirmLabel: "Confirm reset".to_owned(),
        }
    }
}

/// The events `ui.confirmbutton` reports.
#[derive(Debug, Clone)]
pub enum ConfirmbuttonEvent {
    Confirmed {  },
}

impl ConfirmbuttonEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::Confirmed {  } => ("confirmed", serde_json::json!({  })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const BUTTON: &str = "ui.confirmbutton.button";
}
