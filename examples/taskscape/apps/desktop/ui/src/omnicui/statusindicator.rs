//! Generated from omnicui/contracts/statusindicator.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.statusindicator`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct StatusindicatorProps {
    pub icon: String,
    pub label: String,
    pub tooltip: String,
    pub active: bool,
}

impl Default for StatusindicatorProps {
    fn default() -> Self {
        Self {
            icon: "motion_photos_off".to_owned(),
            label: "Reduced motion".to_owned(),
            tooltip: "Reduced motion is on (Settings > General)".to_owned(),
            active: true,
        }
    }
}

/// The events `ui.statusindicator` reports.
#[derive(Debug, Clone)]
pub enum StatusindicatorEvent {
}

impl StatusindicatorEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match *self {
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const CELL: &str = "ui.statusindicator.cell";
    pub const GLYPH: &str = "ui.statusindicator.glyph";
    pub const TOOLTIP: &str = "ui.statusindicator.tooltip";
}
