//! Generated from omnicui/contracts/select.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.select`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct SelectProps {
    pub value: String,
    pub optionSet: String,
    pub leadingIcon: String,
    pub fullWidth: bool,
    pub disabled: bool,
    pub ariaLabel: String,
}

impl Default for SelectProps {
    fn default() -> Self {
        Self {
            value: String::new(),
            optionSet: "sort".to_owned(),
            leadingIcon: "".to_owned(),
            fullWidth: true,
            disabled: false,
            ariaLabel: String::new(),
        }
    }
}

/// The events `ui.select` reports.
#[derive(Debug, Clone)]
pub enum SelectEvent {
    ValueChanged { value: String },
}

impl SelectEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::ValueChanged { value } => ("valueChanged", serde_json::json!({ "value": value })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const TRIGGER: &str = "ui.select.trigger";
    pub const CURRENT: &str = "ui.select.current";
    pub const ICON: &str = "ui.select.icon";
    pub const LISTBOX: &str = "ui.select.listbox";
    pub const OPTION1: &str = "ui.select.option1";
    pub const OPTION2: &str = "ui.select.option2";
    pub const OPTION3: &str = "ui.select.option3";
    pub const OPTION4: &str = "ui.select.option4";
    pub const OPTION5: &str = "ui.select.option5";
}
