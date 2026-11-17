//! Generated from omnicui/contracts/tabpanel.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.tabpanel`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct TabpanelProps {
    pub value: String,
    pub scroll: bool,
    pub tone: String,
}

impl Default for TabpanelProps {
    fn default() -> Self {
        Self {
            value: "".to_owned(),
            scroll: true,
            tone: "surface-2".to_owned(),
        }
    }
}

/// The events `ui.tabpanel` reports.
#[derive(Debug, Clone)]
pub enum TabpanelEvent {
    Scrolled { atBottom: bool },
}

impl TabpanelEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::Scrolled { atBottom } => ("scrolled", serde_json::json!({ "atBottom": atBottom })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const PANEL: &str = "ui.tabpanel.panel";
}
