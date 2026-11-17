//! Generated from omnicui/contracts/dialog.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.dialog`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct DialogProps {
    pub open: bool,
    pub kind: String,
    pub tone: String,
    pub presentation: String,
    pub size: String,
    pub badge: String,
    pub title: String,
    pub description: String,
    pub instead: String,
    pub acceptLabel: String,
    pub cancelLabel: String,
}

impl Default for DialogProps {
    fn default() -> Self {
        Self {
            open: false,
            kind: "name".to_owned(),
            tone: "".to_owned(),
            presentation: "overlay".to_owned(),
            size: "sheet".to_owned(),
            badge: "".to_owned(),
            title: "".to_owned(),
            description: "".to_owned(),
            instead: "".to_owned(),
            acceptLabel: "".to_owned(),
            cancelLabel: "Cancel".to_owned(),
        }
    }
}

/// The events `ui.dialog` reports.
#[derive(Debug, Clone)]
pub enum DialogEvent {
    OpenChanged { open: bool },
    Accepted {  },
}

impl DialogEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::OpenChanged { open } => ("openChanged", serde_json::json!({ "open": open })),
            Self::Accepted {  } => ("accepted", serde_json::json!({  })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const CARD: &str = "ui.dialog.card";
    pub const BADGE: &str = "ui.dialog.badge";
    pub const TITLE: &str = "ui.dialog.title";
    pub const DESCRIPTION: &str = "ui.dialog.description";
    pub const FIELD: &str = "ui.dialog.field";
    pub const INSTEAD: &str = "ui.dialog.instead";
    pub const FOOTER: &str = "ui.dialog.footer";
    pub const CANCEL: &str = "ui.dialog.cancel";
    pub const ACCEPT: &str = "ui.dialog.accept";
}
