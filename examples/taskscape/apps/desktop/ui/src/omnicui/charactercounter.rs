//! Generated from omnicui/contracts/charactercounter.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.charactercounter`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct CharactercounterProps {
    pub count: i64,
    pub limit: i64,
    pub nearRatio: f64,
}

impl Default for CharactercounterProps {
    fn default() -> Self {
        Self {
            count: 0,
            limit: 2000,
            nearRatio: 0.9,
        }
    }
}

/// The events `ui.charactercounter` reports.
#[derive(Debug, Clone)]
pub enum CharactercounterEvent {
}

impl CharactercounterEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match *self {
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const READOUT: &str = "ui.charactercounter.readout";
    pub const ANNOUNCEMENT: &str = "ui.charactercounter.announcement";
}
