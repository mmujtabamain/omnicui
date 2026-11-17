//! Generated from omnicui/contracts/menubutton.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.menubutton`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct MenubuttonProps {
    pub variant: String,
    pub size: String,
    pub leadingIcon: String,
    pub trailingIcon: String,
    pub label: String,
    pub placeholder: String,
    pub disabled: bool,
    pub ariaLabel: String,
    pub open: bool,
}

impl Default for MenubuttonProps {
    fn default() -> Self {
        Self {
            variant: "ghost".to_owned(),
            size: "sm".to_owned(),
            leadingIcon: "".to_owned(),
            trailingIcon: "".to_owned(),
            label: "".to_owned(),
            placeholder: "".to_owned(),
            disabled: false,
            ariaLabel: "".to_owned(),
            open: false,
        }
    }
}

/// The events `ui.menubutton` reports.
#[derive(Debug, Clone)]
pub enum MenubuttonEvent {
    OpenChanged { open: bool },
}

impl MenubuttonEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::OpenChanged { open } => ("openChanged", serde_json::json!({ "open": open })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const TRIGGER: &str = "ui.menubutton.trigger";
    pub const LABEL: &str = "ui.menubutton.label";
    pub const LEADINGICON: &str = "ui.menubutton.leadingIcon";
    pub const TRAILINGICON: &str = "ui.menubutton.trailingIcon";
    pub const MENU: &str = "ui.menubutton.menu";
}
