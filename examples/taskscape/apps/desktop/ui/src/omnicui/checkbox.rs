//! Generated from omnicui/contracts/checkbox.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.checkbox`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct CheckboxProps {
    pub checked: bool,
    pub ariaLabel: String,
    pub size: String,
    pub shape: String,
    pub intent: String,
    pub disabled: bool,
}

impl Default for CheckboxProps {
    fn default() -> Self {
        Self {
            checked: false,
            ariaLabel: String::new(),
            size: "sm".to_owned(),
            shape: "square".to_owned(),
            intent: "done".to_owned(),
            disabled: false,
        }
    }
}

/// The events `ui.checkbox` reports.
#[derive(Debug, Clone)]
pub enum CheckboxEvent {
    CheckedChange { checked: bool },
}

impl CheckboxEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::CheckedChange { checked } => ("checkedChange", serde_json::json!({ "checked": checked })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const CONTROL: &str = "ui.checkbox.control";
    pub const INDICATOR: &str = "ui.checkbox.indicator";
    pub const ROW: &str = "ui.checkbox.row";
}
