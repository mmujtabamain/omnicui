//! Generated from omnicui/contracts/trashwindow.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `app.trashwindow`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct TrashwindowProps {
    pub dataset: String,
    pub openRequests: i64,
}

impl Default for TrashwindowProps {
    fn default() -> Self {
        Self {
            dataset: "standard".to_owned(),
            openRequests: 1,
        }
    }
}

/// The events `app.trashwindow` reports.
#[derive(Debug, Clone)]
pub enum TrashwindowEvent {
    Opened { created: bool },
    Closed { reason: String },
    Restored { title: String },
    Purged { title: String },
    EmptyAsked { count: i64 },
}

impl TrashwindowEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::Opened { created } => ("opened", serde_json::json!({ "created": created })),
            Self::Closed { reason } => ("closed", serde_json::json!({ "reason": reason })),
            Self::Restored { title } => ("restored", serde_json::json!({ "title": title })),
            Self::Purged { title } => ("purged", serde_json::json!({ "title": title })),
            Self::EmptyAsked { count } => ("emptyAsked", serde_json::json!({ "count": count })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const TITLE: &str = "app.trashwindow.title";
    pub const TITLEBAR: &str = "app.trashwindow.titleBar";
    pub const ACTIONSTRIP: &str = "app.trashwindow.actionStrip";
    pub const LISTBODY: &str = "app.trashwindow.listBody";
    pub const RESTOREALL: &str = "app.trashwindow.restoreAll";
    pub const EMPTYTRASH: &str = "app.trashwindow.emptyTrash";
    pub const EMPTYSTATE: &str = "app.trashwindow.emptyState";
    pub const ROW: &str = "app.trashwindow.row";
    pub const RESTOREROW: &str = "app.trashwindow.restoreRow";
    pub const PURGEROW: &str = "app.trashwindow.purgeRow";
    pub const SECONDROW: &str = "app.trashwindow.secondRow";
    pub const SECONDRESTOREROW: &str = "app.trashwindow.secondRestoreRow";
    pub const SECONDPURGEROW: &str = "app.trashwindow.secondPurgeRow";
}
