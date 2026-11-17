//! Generated from omnicui/contracts/attachmentlightbox.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `app.attachmentlightbox`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct AttachmentlightboxProps {
    pub dataset: String,
    pub task: String,
    pub attachment: String,
}

impl Default for AttachmentlightboxProps {
    fn default() -> Self {
        Self {
            dataset: "standard".to_owned(),
            task: "Rewrite the hero section".to_owned(),
            attachment: "hero-sketch.png".to_owned(),
        }
    }
}

/// The events `app.attachmentlightbox` reports.
#[derive(Debug, Clone)]
pub enum AttachmentlightboxEvent {
    Closed { via: String, name: String },
    OpenedExternally { name: String },
    Renamed { name: String },
    Removed { name: String },
    Stepped { name: String },
}

impl AttachmentlightboxEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::Closed { via, name } => ("closed", serde_json::json!({ "via": via, "name": name })),
            Self::OpenedExternally { name } => ("openedExternally", serde_json::json!({ "name": name })),
            Self::Renamed { name } => ("renamed", serde_json::json!({ "name": name })),
            Self::Removed { name } => ("removed", serde_json::json!({ "name": name })),
            Self::Stepped { name } => ("stepped", serde_json::json!({ "name": name })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const OPENEXTERNALLY: &str = "app.attachmentlightbox.openExternally";
    pub const RENAME: &str = "app.attachmentlightbox.rename";
    pub const REMOVE: &str = "app.attachmentlightbox.remove";
    pub const CLOSE: &str = "app.attachmentlightbox.close";
    pub const PREVIOUS: &str = "app.attachmentlightbox.previous";
    pub const NEXT: &str = "app.attachmentlightbox.next";
    pub const COUNTER: &str = "app.attachmentlightbox.counter";
    pub const FILENAME: &str = "app.attachmentlightbox.fileName";
    pub const KINDICON: &str = "app.attachmentlightbox.kindIcon";
    pub const IMAGE: &str = "app.attachmentlightbox.image";
    pub const FALLBACKNAME: &str = "app.attachmentlightbox.fallbackName";
    pub const FALLBACKSTATUS: &str = "app.attachmentlightbox.fallbackStatus";
    pub const FALLBACKOPEN: &str = "app.attachmentlightbox.fallbackOpen";
    pub const RENAMEFIELD: &str = "app.attachmentlightbox.renameField";
    pub const SHEETTAIL: &str = "app.attachmentlightbox.sheetTail";
    pub const SHEETHEADING: &str = "app.attachmentlightbox.sheetHeading";
    pub const SHEETDETAIL: &str = "app.attachmentlightbox.sheetDetail";
    pub const SHEETCANCEL: &str = "app.attachmentlightbox.sheetCancel";
    pub const SHEETACCEPT: &str = "app.attachmentlightbox.sheetAccept";
}
