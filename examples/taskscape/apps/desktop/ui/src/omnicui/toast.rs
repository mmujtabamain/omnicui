//! Generated from omnicui/contracts/toast.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.toast`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct ToastProps {
    pub awaitTimeout: bool,
    pub open: bool,
    pub message: String,
    pub actionLabel: String,
    pub showAction: bool,
    pub failed: bool,
    pub errorMessage: String,
    pub duration: f64,
    pub focusHotkey: String,
    pub reducedMotion: bool,
    pub swipeDirection: String,
    pub position: String,
    pub tone: String,
}

impl Default for ToastProps {
    fn default() -> Self {
        Self {
            awaitTimeout: false,
            open: false,
            message: "".to_owned(),
            actionLabel: "Undo".to_owned(),
            showAction: true,
            failed: false,
            errorMessage: "Could not undo".to_owned(),
            duration: 5000.0,
            focusHotkey: "F8".to_owned(),
            reducedMotion: false,
            swipeDirection: "down".to_owned(),
            position: "bottom-centre".to_owned(),
            tone: "neutral".to_owned(),
        }
    }
}

/// The events `ui.toast` reports.
#[derive(Debug, Clone)]
pub enum ToastEvent {
    OpenChanged { open: bool },
    Dismissed { reason: String },
    Actioned { label: String },
}

impl ToastEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::OpenChanged { open } => ("openChanged", serde_json::json!({ "open": open })),
            Self::Dismissed { reason } => ("dismissed", serde_json::json!({ "reason": reason })),
            Self::Actioned { label } => ("actioned", serde_json::json!({ "label": label })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const VIEWPORT: &str = "ui.toast.viewport";
    pub const TOAST: &str = "ui.toast.toast";
    pub const MESSAGE: &str = "ui.toast.message";
    pub const ACTION: &str = "ui.toast.action";
    pub const CLOSE: &str = "ui.toast.close";
}
