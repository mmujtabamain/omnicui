//! Generated from omnicui/contracts/attachmenttile.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.attachmenttile`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct AttachmenttileProps {
    pub kind: String,
    pub name: String,
    pub thumbnailSrc: String,
    pub draggable: bool,
}

impl Default for AttachmenttileProps {
    fn default() -> Self {
        Self {
            kind: "image".to_owned(),
            name: "".to_owned(),
            thumbnailSrc: "".to_owned(),
            draggable: true,
        }
    }
}

/// The events `ui.attachmenttile` reports.
#[derive(Debug, Clone)]
pub enum AttachmenttileEvent {
    Opened { kind: String },
    ContextMenuOpened { kind: String },
    DragStarted { kind: String },
}

impl AttachmenttileEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::Opened { kind } => ("opened", serde_json::json!({ "kind": kind })),
            Self::ContextMenuOpened { kind } => ("contextMenuOpened", serde_json::json!({ "kind": kind })),
            Self::DragStarted { kind } => ("dragStarted", serde_json::json!({ "kind": kind })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const TILE: &str = "ui.attachmenttile.tile";
    pub const THUMB: &str = "ui.attachmenttile.thumb";
    pub const GLYPH: &str = "ui.attachmenttile.glyph";
    pub const OVERLAY: &str = "ui.attachmenttile.overlay";
}
