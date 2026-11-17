//! Generated from omnicui/contracts/separator.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.separator`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct SeparatorProps {
    pub orientation: String,
    pub length: String,
    pub inset: String,
    pub emphasis: String,
    pub depth: i64,
    pub decorative: bool,
    pub className: String,
}

impl Default for SeparatorProps {
    fn default() -> Self {
        Self {
            orientation: "horizontal".to_owned(),
            length: "sm".to_owned(),
            inset: "none".to_owned(),
            emphasis: "default".to_owned(),
            depth: 0,
            decorative: true,
            className: "".to_owned(),
        }
    }
}

/// The events `ui.separator` reports.
#[derive(Debug, Clone)]
pub enum SeparatorEvent {
}

impl SeparatorEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match *self {
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const RULE: &str = "ui.separator.rule";
}
