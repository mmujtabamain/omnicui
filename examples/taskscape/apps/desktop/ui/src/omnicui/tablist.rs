//! Generated from omnicui/contracts/tablist.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.tablist`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct TablistProps {
    pub value: String,
    pub r#loop: bool,
    pub label: String,
    pub orientation: String,
    pub overflow: String,
    pub tabCount: i64,
}

impl Default for TablistProps {
    fn default() -> Self {
        Self {
            value: "1".to_owned(),
            r#loop: true,
            label: "Tabs".to_owned(),
            orientation: "horizontal".to_owned(),
            overflow: "scroll".to_owned(),
            tabCount: 3,
        }
    }
}

/// The events `ui.tablist` reports.
#[derive(Debug, Clone)]
pub enum TablistEvent {
    ValueChanged { value: String },
}

impl TablistEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::ValueChanged { value } => ("valueChanged", serde_json::json!({ "value": value })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const TABLIST: &str = "ui.tablist.tablist";
    pub const FIRSTTAB: &str = "ui.tablist.firstTab";
    pub const SECONDTAB: &str = "ui.tablist.secondTab";
    pub const LASTTAB: &str = "ui.tablist.lastTab";
}
