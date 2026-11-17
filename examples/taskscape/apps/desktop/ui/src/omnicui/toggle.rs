//! Generated from omnicui/contracts/toggle.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.toggle`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct ToggleProps {
    pub pressed: bool,
    pub defaultPressed: bool,
    pub size: String,
    pub shape: String,
    pub icon: String,
    pub pressedIcon: String,
    pub label: String,
    pub ariaLabel: String,
    pub shortcut: String,
    pub retainFocus: bool,
    pub disabled: bool,
}

impl Default for ToggleProps {
    fn default() -> Self {
        Self {
            pressed: false,
            defaultPressed: false,
            size: "sm".to_owned(),
            shape: "icon".to_owned(),
            icon: "".to_owned(),
            pressedIcon: "".to_owned(),
            label: "".to_owned(),
            ariaLabel: "".to_owned(),
            shortcut: "".to_owned(),
            retainFocus: false,
            disabled: false,
        }
    }
}

/// The events `ui.toggle` reports.
#[derive(Debug, Clone)]
pub enum ToggleEvent {
    PressedChange { pressed: bool },
}

impl ToggleEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::PressedChange { pressed } => ("pressedChange", serde_json::json!({ "pressed": pressed })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const TOGGLE: &str = "ui.toggle.toggle";
    pub const TOOLTIP: &str = "ui.toggle.tooltip";
}
