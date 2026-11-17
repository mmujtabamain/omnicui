//! Generated from omnicui/contracts/overlay.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.overlay`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct OverlayProps {
    pub open: bool,
    pub placement: String,
    pub scrim: String,
    pub dismissOn: String,
    pub captureEscape: bool,
}

impl Default for OverlayProps {
    fn default() -> Self {
        Self {
            open: false,
            placement: "center".to_owned(),
            scrim: "dim".to_owned(),
            dismissOn: "outside".to_owned(),
            captureEscape: true,
        }
    }
}

/// The events `ui.overlay` reports.
#[derive(Debug, Clone)]
pub enum OverlayEvent {
    OpenChanged { open: bool },
    Dismissed {  },
}

impl OverlayEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::OpenChanged { open } => ("openChanged", serde_json::json!({ "open": open })),
            Self::Dismissed {  } => ("dismissed", serde_json::json!({  })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const LAYER: &str = "ui.overlay.layer";
    pub const SCRIM: &str = "ui.overlay.scrim";
    pub const TITLE: &str = "ui.overlay.title";
    pub const FIELD: &str = "ui.overlay.field";
    pub const ACTION: &str = "ui.overlay.action";
    pub const OUTSIDE: &str = "ui.overlay.outside";
    pub const OPENER: &str = "ui.overlay.opener";
}
