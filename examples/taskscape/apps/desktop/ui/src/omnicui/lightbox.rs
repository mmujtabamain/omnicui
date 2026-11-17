//! Generated from omnicui/contracts/lightbox.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.lightbox`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct LightboxProps {
    pub presentation: String,
    pub open: bool,
}

impl Default for LightboxProps {
    fn default() -> Self {
        Self {
            presentation: "overlay".to_owned(),
            open: false,
        }
    }
}

/// The events `ui.lightbox` reports.
#[derive(Debug, Clone)]
pub enum LightboxEvent {
    Close { reason: String },
}

impl LightboxEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::Close { reason } => ("close", serde_json::json!({ "reason": reason })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const LIGHTBOX: &str = "ui.lightbox.lightbox";
    pub const STAGE: &str = "ui.lightbox.stage";
    pub const MEDIA: &str = "ui.lightbox.media";
}
