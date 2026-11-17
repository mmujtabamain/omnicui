//! Generated from omnicui/contracts/confirmwindows.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `app.confirmwindows`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct ConfirmwindowsProps {
    pub dataset: String,
    pub open: String,
    pub count: i64,
    pub target: String,
    pub initialName: String,
}

impl Default for ConfirmwindowsProps {
    fn default() -> Self {
        Self {
            dataset: "standard".to_owned(),
            open: "newList".to_owned(),
            count: 1,
            target: "".to_owned(),
            initialName: "".to_owned(),
        }
    }
}

/// The events `app.confirmwindows` reports.
#[derive(Debug, Clone)]
pub enum ConfirmwindowsEvent {
    Answered { action: String, text: String, id: String },
    Changed { kind: String, name: String },
}

impl ConfirmwindowsEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::Answered { action, text, id } => ("answered", serde_json::json!({ "action": action, "text": text, "id": id })),
            Self::Changed { kind, name } => ("changed", serde_json::json!({ "kind": kind, "name": name })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const BADGE: &str = "app.confirmwindows.badge";
    pub const CARD: &str = "app.confirmwindows.card";
    pub const HEADLINE: &str = "app.confirmwindows.headline";
    pub const DETAIL: &str = "app.confirmwindows.detail";
    pub const FIELD: &str = "app.confirmwindows.field";
    pub const ROLL: &str = "app.confirmwindows.roll";
    pub const INSTEAD: &str = "app.confirmwindows.instead";
    pub const FOOTER: &str = "app.confirmwindows.footer";
    pub const CANCEL: &str = "app.confirmwindows.cancel";
    pub const ACCEPT: &str = "app.confirmwindows.accept";
}
