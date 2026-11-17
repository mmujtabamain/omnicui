//! Generated from omnicui/contracts/trash.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `app.trash`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct TrashProps {
    pub dataset: String,
    pub staleTitle: String,
}

impl Default for TrashProps {
    fn default() -> Self {
        Self {
            dataset: "standard".to_owned(),
            staleTitle: "".to_owned(),
        }
    }
}

/// The events `app.trash` reports.
#[derive(Debug, Clone)]
pub enum TrashEvent {
    Restored { count: i64 },
    Purged { count: i64 },
    Closed { pane: String },
}

impl TrashEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::Restored { count } => ("restored", serde_json::json!({ "count": count })),
            Self::Purged { count } => ("purged", serde_json::json!({ "count": count })),
            Self::Closed { pane } => ("closed", serde_json::json!({ "pane": pane })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const PANE: &str = "app.trash.pane";
    pub const TITLE: &str = "app.trash.title";
    pub const CLOSECONTROL: &str = "app.trash.closeControl";
    pub const MINIMIZELIGHT: &str = "app.trash.minimizeLight";
    pub const FULLSCREENLIGHT: &str = "app.trash.fullscreenLight";
    pub const LIST: &str = "app.trash.list";
    pub const ACTIONSTRIP: &str = "app.trash.actionStrip";
    pub const RESTOREALL: &str = "app.trash.restoreAll";
    pub const EMPTYTRASH: &str = "app.trash.emptyTrash";
    pub const EMPTYSTATE: &str = "app.trash.emptyState";
    pub const NOTICE: &str = "app.trash.notice";
    pub const ROW: &str = "app.trash.row";
    pub const RESTOREITEM: &str = "app.trash.restoreItem";
    pub const PURGEITEM: &str = "app.trash.purgeItem";
    pub const ROWPRINTER: &str = "app.trash.rowPrinter";
    pub const METAPRINTER: &str = "app.trash.metaPrinter";
    pub const RESTOREPRINTER: &str = "app.trash.restorePrinter";
    pub const PURGEPRINTER: &str = "app.trash.purgePrinter";
    pub const ROWFLYER: &str = "app.trash.rowFlyer";
    pub const METAFLYER: &str = "app.trash.metaFlyer";
    pub const RESTOREFLYER: &str = "app.trash.restoreFlyer";
    pub const PURGEFLYER: &str = "app.trash.purgeFlyer";
    pub const EMPTYSHEET: &str = "app.trash.emptySheet";
    pub const EMPTYSHEETTITLE: &str = "app.trash.emptySheetTitle";
    pub const EMPTYSHEETDETAIL: &str = "app.trash.emptySheetDetail";
    pub const CANCELEMPTY: &str = "app.trash.cancelEmpty";
    pub const CONFIRMEMPTY: &str = "app.trash.confirmEmpty";
}
