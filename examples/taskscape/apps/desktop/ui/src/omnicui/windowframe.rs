//! Generated from omnicui/contracts/windowframe.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.windowframe`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct WindowframeProps {
    pub chrome: String,
}

impl Default for WindowframeProps {
    fn default() -> Self {
        Self {
            chrome: "titled".to_owned(),
        }
    }
}

/// The events `ui.windowframe` reports.
#[derive(Debug, Clone)]
pub enum WindowframeEvent {
    Dismissed { reason: String },
    Submitted { via: String },
}

impl WindowframeEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::Dismissed { reason } => ("dismissed", serde_json::json!({ "reason": reason })),
            Self::Submitted { via } => ("submitted", serde_json::json!({ "via": via })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const FRAME: &str = "ui.windowframe.frame";
    pub const TITLEBAR: &str = "ui.windowframe.titleBar";
    pub const BODY: &str = "ui.windowframe.body";
    pub const OVERLAYS: &str = "ui.windowframe.overlays";
    pub const CAPTURETITLE: &str = "ui.windowframe.captureTitle";
    pub const CAPTURENOTES: &str = "ui.windowframe.captureNotes";
}
