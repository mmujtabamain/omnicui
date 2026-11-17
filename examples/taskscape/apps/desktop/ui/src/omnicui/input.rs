//! Generated from omnicui/contracts/input.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.input`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct InputProps {
    pub value: String,
    pub placeholder: String,
    pub ariaLabel: String,
    pub size: String,
    pub mode: String,
    pub multiline: bool,
    pub autoFocus: bool,
    pub selectOnFocus: bool,
    pub selectOnOpen: bool,
    pub stopKeyPropagation: bool,
    pub spellCheck: bool,
    pub disabled: bool,
    pub editRequest: i64,
    pub className: String,
}

impl Default for InputProps {
    fn default() -> Self {
        Self {
            value: "".to_owned(),
            placeholder: "".to_owned(),
            ariaLabel: "".to_owned(),
            size: "md".to_owned(),
            mode: "inline".to_owned(),
            multiline: false,
            autoFocus: false,
            selectOnFocus: false,
            selectOnOpen: true,
            stopKeyPropagation: false,
            spellCheck: true,
            disabled: false,
            editRequest: 0,
            className: "".to_owned(),
        }
    }
}

/// The events `ui.input` reports.
#[derive(Debug, Clone)]
pub enum InputEvent {
    ValueChange { value: String },
    Submit { value: String },
    Commit { value: String },
    Cancel {  },
    Propagated { key: String },
}

impl InputEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::ValueChange { value } => ("valueChange", serde_json::json!({ "value": value })),
            Self::Submit { value } => ("submit", serde_json::json!({ "value": value })),
            Self::Commit { value } => ("commit", serde_json::json!({ "value": value })),
            Self::Cancel {  } => ("cancel", serde_json::json!({  })),
            Self::Propagated { key } => ("propagated", serde_json::json!({ "key": key })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const FIELD: &str = "ui.input.field";
}
