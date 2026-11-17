//! Generated from omnicui/contracts/searchfield.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.searchfield`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct SearchfieldProps {
    pub value: String,
    pub placeholder: String,
    pub size: String,
    pub navigation: String,
    pub clearable: bool,
    pub autoFocus: bool,
    pub disabled: bool,
    pub ariaLabel: String,
}

impl Default for SearchfieldProps {
    fn default() -> Self {
        Self {
            value: "".to_owned(),
            placeholder: "Search".to_owned(),
            size: "md".to_owned(),
            navigation: "filter".to_owned(),
            clearable: false,
            autoFocus: false,
            disabled: false,
            ariaLabel: "".to_owned(),
        }
    }
}

/// The events `ui.searchfield` reports.
#[derive(Debug, Clone)]
pub enum SearchfieldEvent {
    Changed { value: String },
    StepMatch { direction: String },
    Cleared {  },
}

impl SearchfieldEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::Changed { value } => ("changed", serde_json::json!({ "value": value })),
            Self::StepMatch { direction } => ("stepMatch", serde_json::json!({ "direction": direction })),
            Self::Cleared {  } => ("cleared", serde_json::json!({  })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const WELL: &str = "ui.searchfield.well";
    pub const ICON: &str = "ui.searchfield.icon";
    pub const FIELD: &str = "ui.searchfield.field";
    pub const CLEARBUTTON: &str = "ui.searchfield.clearButton";
}
