//! Generated from omnicui/contracts/panefooter.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.panefooter`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct PanefooterProps {
}

impl Default for PanefooterProps {
    fn default() -> Self {
        Self {
        }
    }
}

/// The events `ui.panefooter` reports.
#[derive(Debug, Clone)]
pub enum PanefooterEvent {
}

impl PanefooterEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match *self {
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const FOOTER: &str = "ui.panefooter.footer";
}
