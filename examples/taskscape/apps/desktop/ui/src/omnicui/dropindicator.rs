//! Generated from omnicui/contracts/dropindicator.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.dropindicator`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct DropindicatorProps {
    pub target: String,
    pub edge: String,
    pub visible: bool,
}

impl Default for DropindicatorProps {
    fn default() -> Self {
        Self {
            target: "row".to_owned(),
            edge: "before".to_owned(),
            visible: false,
        }
    }
}

/// The events `ui.dropindicator` reports.
#[derive(Debug, Clone)]
pub enum DropindicatorEvent {
}

impl DropindicatorEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match *self {
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const INDICATOR: &str = "ui.dropindicator.indicator";
    pub const DOT: &str = "ui.dropindicator.dot";
}
