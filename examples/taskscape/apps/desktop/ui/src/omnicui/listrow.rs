//! Generated from omnicui/contracts/listrow.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.listrow`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct ListrowProps {
    pub title: String,
    pub meta: String,
    pub host: String,
    pub indicator: String,
    pub actions: String,
    pub selected: bool,
}

impl Default for ListrowProps {
    fn default() -> Self {
        Self {
            title: String::new(),
            meta: "".to_owned(),
            host: "list".to_owned(),
            indicator: "none".to_owned(),
            actions: "none".to_owned(),
            selected: false,
        }
    }
}

/// The events `ui.listrow` reports.
#[derive(Debug, Clone)]
pub enum ListrowEvent {
    Activated { title: String },
    ActionRequested { action: String },
}

impl ListrowEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::Activated { title } => ("activated", serde_json::json!({ "title": title })),
            Self::ActionRequested { action } => ("actionRequested", serde_json::json!({ "action": action })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const LISTROW: &str = "ui.listrow.listRow";
    pub const BODY: &str = "ui.listrow.body";
    pub const TITLE: &str = "ui.listrow.title";
    pub const META: &str = "ui.listrow.meta";
    pub const CHECK: &str = "ui.listrow.check";
    pub const RESTORE: &str = "ui.listrow.restore";
    pub const EDIT: &str = "ui.listrow.edit";
    pub const DELETEACTION: &str = "ui.listrow.deleteAction";
}
