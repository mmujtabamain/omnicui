//! Generated from omnicui/contracts/text.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.text`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct TextProps {
    pub content: String,
}

impl Default for TextProps {
    fn default() -> Self {
        Self {
            content: "".to_owned(),
        }
    }
}

/// The events `ui.text` reports.
#[derive(Debug, Clone)]
pub enum TextEvent {
}

impl TextEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match *self {
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const TEXT: &str = "ui.text.text";
}
