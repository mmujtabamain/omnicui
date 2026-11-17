//! Generated from omnicui/contracts/icon.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.icon`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct IconProps {
    pub name: String,
    pub label: String,
    pub size: String,
    pub weight: i64,
    pub tone: String,
    pub tint: String,
    pub reveal: String,
    pub rotate: String,
}

impl Default for IconProps {
    fn default() -> Self {
        Self {
            name: "".to_owned(),
            label: "".to_owned(),
            size: "md".to_owned(),
            weight: 300,
            tone: "muted".to_owned(),
            tint: "none".to_owned(),
            reveal: "always".to_owned(),
            rotate: "none".to_owned(),
        }
    }
}

/// The events `ui.icon` reports.
#[derive(Debug, Clone)]
pub enum IconEvent {
}

impl IconEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match *self {
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const GLYPH: &str = "ui.icon.glyph";
    pub const TILE: &str = "ui.icon.tile";
    pub const BADGE: &str = "ui.icon.badge";
}
