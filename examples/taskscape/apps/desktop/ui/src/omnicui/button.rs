//! Generated from omnicui/contracts/button.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.button`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct ButtonProps {
    pub variant: String,
    pub size: String,
    pub tone: String,
    pub shape: String,
    pub layout: String,
    pub surface: String,
    pub reveal: String,
    pub width: String,
    pub hover: String,
    pub icon: String,
    pub trailingIcon: String,
    pub label: String,
    pub ariaLabel: String,
    pub shortcut: String,
    pub busy: bool,
    pub feedbackLabel: String,
    pub feedbackIcon: String,
    pub errorText: String,
    pub active: bool,
    pub tabbable: bool,
    pub disabled: bool,
    pub r#type: String,
    pub asChild: bool,
    pub iconSize: i64,
    pub iconWeight: i64,
    pub radius: f64,
    pub keepFocusOnPress: bool,
    pub retainFocus: bool,
    pub truncate: bool,
    pub mono: bool,
    pub block: bool,
    pub emptyLabel: String,
    pub description: String,
}

impl Default for ButtonProps {
    fn default() -> Self {
        Self {
            variant: "ghost".to_owned(),
            size: "md".to_owned(),
            tone: "default".to_owned(),
            shape: "text".to_owned(),
            layout: "inline".to_owned(),
            surface: "default".to_owned(),
            reveal: "always".to_owned(),
            width: "full".to_owned(),
            hover: "text".to_owned(),
            icon: "".to_owned(),
            trailingIcon: "".to_owned(),
            label: "".to_owned(),
            ariaLabel: "".to_owned(),
            shortcut: "".to_owned(),
            busy: false,
            feedbackLabel: "".to_owned(),
            feedbackIcon: "".to_owned(),
            errorText: "".to_owned(),
            active: false,
            tabbable: true,
            disabled: false,
            r#type: "button".to_owned(),
            asChild: false,
            iconSize: 0,
            iconWeight: 300,
            radius: 0.0,
            keepFocusOnPress: false,
            retainFocus: false,
            truncate: true,
            mono: false,
            block: false,
            emptyLabel: "".to_owned(),
            description: "".to_owned(),
        }
    }
}

/// The events `ui.button` reports.
#[derive(Debug, Clone)]
pub enum ButtonEvent {
    Activated { count: i64 },
}

impl ButtonEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::Activated { count } => ("activated", serde_json::json!({ "count": count })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const BUTTON: &str = "ui.button.button";
    pub const LABEL: &str = "ui.button.label";
    pub const ICON: &str = "ui.button.icon";
    pub const TRAILINGICON: &str = "ui.button.trailingIcon";
    pub const SPINNER: &str = "ui.button.spinner";
    pub const TOOLTIP: &str = "ui.button.tooltip";
    pub const FEEDBACK: &str = "ui.button.feedback";
    pub const CHILD: &str = "ui.button.child";
}
