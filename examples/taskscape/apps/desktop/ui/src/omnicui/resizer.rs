//! Generated from omnicui/contracts/resizer.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.resizer`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct ResizerProps {
    pub orientation: String,
    pub value: f64,
    pub min: f64,
    pub max: f64,
    pub reset: String,
    pub open: bool,
    pub label: String,
}

impl Default for ResizerProps {
    fn default() -> Self {
        Self {
            orientation: "vertical".to_owned(),
            value: 0.5,
            min: 0.25,
            max: 0.75,
            reset: "none".to_owned(),
            open: true,
            label: "Resize columns".to_owned(),
        }
    }
}

/// The events `ui.resizer` reports.
#[derive(Debug, Clone)]
pub enum ResizerEvent {
    Resized { value: f64 },
    Reset {  },
}

impl ResizerEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::Resized { value } => ("resized", serde_json::json!({ "value": value })),
            Self::Reset {  } => ("reset", serde_json::json!({  })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const RESIZER: &str = "ui.resizer.resizer";
    pub const HAIRLINE: &str = "ui.resizer.hairline";
}
