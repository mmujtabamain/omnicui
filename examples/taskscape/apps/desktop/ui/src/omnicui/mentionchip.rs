//! Generated from omnicui/contracts/mentionchip.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.mentionchip`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct MentionchipProps {
    pub source: String,
    pub name: String,
}

impl Default for MentionchipProps {
    fn default() -> Self {
        Self {
            source: "local".to_owned(),
            name: "attachment".to_owned(),
        }
    }
}

/// The events `ui.mentionchip` reports.
#[derive(Debug, Clone)]
pub enum MentionchipEvent {
    Opened { source: String, name: String },
    Removed { name: String },
}

impl MentionchipEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::Opened { source, name } => ("opened", serde_json::json!({ "source": source, "name": name })),
            Self::Removed { name } => ("removed", serde_json::json!({ "name": name })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const CHIP: &str = "ui.mentionchip.chip";
}
