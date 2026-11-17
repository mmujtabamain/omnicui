//! Generated from omnicui/contracts/numberfield.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.numberfield`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct NumberfieldProps {
    pub value: i64,
    pub min: i64,
    pub max: i64,
    pub ariaLabel: String,
}

impl Default for NumberfieldProps {
    fn default() -> Self {
        Self {
            value: 1,
            min: 1,
            max: 365,
            ariaLabel: "".to_owned(),
        }
    }
}

/// The events `ui.numberfield` reports.
#[derive(Debug, Clone)]
pub enum NumberfieldEvent {
    Changed { value: i64 },
}

impl NumberfieldEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::Changed { value } => ("changed", serde_json::json!({ "value": value })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const FIELD: &str = "ui.numberfield.field";
}
