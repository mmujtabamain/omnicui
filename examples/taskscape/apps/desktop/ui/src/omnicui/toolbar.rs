//! Generated from omnicui/contracts/toolbar.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.toolbar`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct ToolbarProps {
    pub placement: String,
    pub ariaLabel: String,
    pub count: i64,
}

impl Default for ToolbarProps {
    fn default() -> Self {
        Self {
            placement: "inline".to_owned(),
            ariaLabel: "Toolbar".to_owned(),
            count: 1,
        }
    }
}

/// The events `ui.toolbar` reports.
#[derive(Debug, Clone)]
pub enum ToolbarEvent {
}

impl ToolbarEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match *self {
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const TOOLBAR: &str = "ui.toolbar.toolbar";
    pub const FIRST: &str = "ui.toolbar.first";
    pub const SECOND: &str = "ui.toolbar.second";
    pub const THIRD: &str = "ui.toolbar.third";
}
