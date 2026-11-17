//! Generated from omnicui/contracts/sidebartab.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.sidebartab`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct SidebartabProps {
    pub icon: String,
    pub label: String,
    pub selected: bool,
    pub scoped: bool,
    pub dimmed: bool,
    pub badge: i64,
}

impl Default for SidebartabProps {
    fn default() -> Self {
        Self {
            icon: "".to_owned(),
            label: "".to_owned(),
            selected: false,
            scoped: true,
            dimmed: false,
            badge: 0,
        }
    }
}

/// The events `ui.sidebartab` reports.
#[derive(Debug, Clone)]
pub enum SidebartabEvent {
    Select { pane: String },
}

impl SidebartabEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::Select { pane } => ("select", serde_json::json!({ "pane": pane })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const TAB: &str = "ui.sidebartab.tab";
    pub const ICON: &str = "ui.sidebartab.icon";
    pub const LABEL: &str = "ui.sidebartab.label";
    pub const BADGE: &str = "ui.sidebartab.badge";
}
