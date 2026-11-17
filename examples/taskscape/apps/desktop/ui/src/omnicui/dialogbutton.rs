//! Generated from omnicui/contracts/dialogbutton.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.dialogbutton`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct DialogbuttonProps {
    pub role: String,
    pub insteadId: String,
    pub variant: String,
    pub label: String,
    pub disabled: bool,
}

impl Default for DialogbuttonProps {
    fn default() -> Self {
        Self {
            role: "".to_owned(),
            insteadId: "".to_owned(),
            variant: "".to_owned(),
            label: "".to_owned(),
            disabled: false,
        }
    }
}

/// The events `ui.dialogbutton` reports.
#[derive(Debug, Clone)]
pub enum DialogbuttonEvent {
    Activate { role: String },
    Dismiss { role: String },
    Resolve { insteadId: String },
}

impl DialogbuttonEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::Activate { role } => ("activate", serde_json::json!({ "role": role })),
            Self::Dismiss { role } => ("dismiss", serde_json::json!({ "role": role })),
            Self::Resolve { insteadId } => ("resolve", serde_json::json!({ "insteadId": insteadId })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const BUTTON: &str = "ui.dialogbutton.button";
}
