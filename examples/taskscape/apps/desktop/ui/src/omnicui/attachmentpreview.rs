//! Generated from omnicui/contracts/attachmentpreview.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.attachmentpreview`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct AttachmentpreviewProps {
    pub kind: String,
    pub src: String,
    pub name: String,
    pub autoPlay: bool,
    pub text: String,
}

impl Default for AttachmentpreviewProps {
    fn default() -> Self {
        Self {
            kind: "image".to_owned(),
            src: String::new(),
            name: String::new(),
            autoPlay: true,
            text: String::new(),
        }
    }
}

/// The events `ui.attachmentpreview` reports.
#[derive(Debug, Clone)]
pub enum AttachmentpreviewEvent {
    Loaded { kind: String },
    Playing { kind: String },
    Error { kind: String },
}

impl AttachmentpreviewEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::Loaded { kind } => ("loaded", serde_json::json!({ "kind": kind })),
            Self::Playing { kind } => ("playing", serde_json::json!({ "kind": kind })),
            Self::Error { kind } => ("error", serde_json::json!({ "kind": kind })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const LOADING: &str = "ui.attachmentpreview.loading";
    pub const IMAGE: &str = "ui.attachmentpreview.image";
    pub const VIDEO: &str = "ui.attachmentpreview.video";
    pub const AUDIO: &str = "ui.attachmentpreview.audio";
    pub const GLYPH: &str = "ui.attachmentpreview.glyph";
    pub const PDF: &str = "ui.attachmentpreview.pdf";
    pub const TEXT: &str = "ui.attachmentpreview.text";
    pub const FALLBACK: &str = "ui.attachmentpreview.fallback";
}
