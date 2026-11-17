//! Generated from omnicui/contracts/badge.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.badge`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct BadgeProps {
    pub tone: String,
    pub shape: String,
    pub size: String,
    pub icon: bool,
    pub children: String,
    pub title: String,
    pub srLabel: String,
    pub visible: bool,
}

impl Default for BadgeProps {
    fn default() -> Self {
        Self {
            tone: "outline".to_owned(),
            shape: "pill".to_owned(),
            size: "md".to_owned(),
            icon: false,
            children: "".to_owned(),
            title: "".to_owned(),
            srLabel: "".to_owned(),
            visible: true,
        }
    }
}

/// The events `ui.badge` reports.
#[derive(Debug, Clone)]
pub enum BadgeEvent {
}

impl BadgeEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match *self {
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const BADGE: &str = "ui.badge.badge";
    pub const ICON: &str = "ui.badge.icon";
}
