//! Generated from omnicui/contracts/spinner.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.spinner`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct SpinnerProps {
    pub size: String,
    pub label: String,
    pub className: String,
}

impl Default for SpinnerProps {
    fn default() -> Self {
        Self {
            size: "sm".to_owned(),
            label: "Loading".to_owned(),
            className: "".to_owned(),
        }
    }
}

/// The events `ui.spinner` reports.
#[derive(Debug, Clone)]
pub enum SpinnerEvent {
}

impl SpinnerEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match *self {
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const INDICATOR: &str = "ui.spinner.indicator";
}
