//! Generated from omnicui/contracts/scrim.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.scrim`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct ScrimProps {
    pub dim: String,
    pub open: bool,
}

impl Default for ScrimProps {
    fn default() -> Self {
        Self {
            dim: "soft".to_owned(),
            open: true,
        }
    }
}

/// The events `ui.scrim` reports.
#[derive(Debug, Clone)]
pub enum ScrimEvent {
    Dismissed {  },
}

impl ScrimEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::Dismissed {  } => ("dismissed", serde_json::json!({  })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const SCRIM: &str = "ui.scrim.scrim";
}
