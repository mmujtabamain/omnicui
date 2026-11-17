//! Generated from omnicui/contracts/hotkeyhint.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.hotkeyhint`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct HotkeyhintProps {
    pub accel: String,
    pub size: String,
    pub tone: String,
    pub platform: String,
}

impl Default for HotkeyhintProps {
    fn default() -> Self {
        Self {
            accel: "".to_owned(),
            size: "sm".to_owned(),
            tone: "inherit".to_owned(),
            platform: "".to_owned(),
        }
    }
}

/// The events `ui.hotkeyhint` reports.
#[derive(Debug, Clone)]
pub enum HotkeyhintEvent {
}

impl HotkeyhintEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match *self {
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const HINT: &str = "ui.hotkeyhint.hint";
}
