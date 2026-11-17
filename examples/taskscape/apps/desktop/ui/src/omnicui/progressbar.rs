//! Generated from omnicui/contracts/progressbar.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.progressbar`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct ProgressbarProps {
    pub value: f64,
    pub max: f64,
    pub ariaLabel: String,
    pub tone: String,
    pub className: String,
}

impl Default for ProgressbarProps {
    fn default() -> Self {
        Self {
            value: 0.0,
            max: 1.0,
            ariaLabel: "".to_owned(),
            tone: "done".to_owned(),
            className: "".to_owned(),
        }
    }
}

/// The events `ui.progressbar` reports.
#[derive(Debug, Clone)]
pub enum ProgressbarEvent {
}

impl ProgressbarEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match *self {
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const BAR: &str = "ui.progressbar.bar";
    pub const FILL: &str = "ui.progressbar.fill";
}
