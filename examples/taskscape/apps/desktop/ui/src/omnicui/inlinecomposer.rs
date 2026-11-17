//! Generated from omnicui/contracts/inlinecomposer.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.inlinecomposer`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct InlinecomposerProps {
    pub kind: String,
    pub depth: f64,
    pub autoFocus: bool,
}

impl Default for InlinecomposerProps {
    fn default() -> Self {
        Self {
            kind: "child".to_owned(),
            depth: 0.0,
            autoFocus: true,
        }
    }
}

/// The events `ui.inlinecomposer` reports.
#[derive(Debug, Clone)]
pub enum InlinecomposerEvent {
    Submitted { title: String },
    Dismissed {  },
}

impl InlinecomposerEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::Submitted { title } => ("submitted", serde_json::json!({ "title": title })),
            Self::Dismissed {  } => ("dismissed", serde_json::json!({  })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const FIELD: &str = "ui.inlinecomposer.field";
    pub const ICON: &str = "ui.inlinecomposer.icon";
}
