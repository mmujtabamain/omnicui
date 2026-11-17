//! Generated from omnicui/contracts/richtexteditor.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `ui.richtexteditor`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct RichtexteditorProps {
    pub value: String,
    pub toolbar: String,
    pub size: String,
    pub placeholder: String,
    pub autoFocus: bool,
    pub maxLength: i64,
    pub submitOnModEnter: bool,
    pub escapeCancels: bool,
    pub collapseWhenEmpty: bool,
    pub mentions: String,
    pub disabled: bool,
    pub ariaLabel: String,
}

impl Default for RichtexteditorProps {
    fn default() -> Self {
        Self {
            value: "".to_owned(),
            toolbar: "floating".to_owned(),
            size: "sm".to_owned(),
            placeholder: "".to_owned(),
            autoFocus: false,
            maxLength: 2000,
            submitOnModEnter: false,
            escapeCancels: false,
            collapseWhenEmpty: false,
            mentions: "".to_owned(),
            disabled: false,
            ariaLabel: String::new(),
        }
    }
}

/// The events `ui.richtexteditor` reports.
#[derive(Debug, Clone)]
pub enum RichtexteditorEvent {
    Changed { value: String, paragraphs: i64 },
    Blurred { value: String },
    Submitted {  },
    Cancelled {  },
    LinkRequested {  },
    AttachmentDropped { file: String },
    MentionOpened { id: String },
}

impl RichtexteditorEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::Changed { value, paragraphs } => ("changed", serde_json::json!({ "value": value, "paragraphs": paragraphs })),
            Self::Blurred { value } => ("blurred", serde_json::json!({ "value": value })),
            Self::Submitted {  } => ("submitted", serde_json::json!({  })),
            Self::Cancelled {  } => ("cancelled", serde_json::json!({  })),
            Self::LinkRequested {  } => ("linkRequested", serde_json::json!({  })),
            Self::AttachmentDropped { file } => ("attachmentDropped", serde_json::json!({ "file": file })),
            Self::MentionOpened { id } => ("mentionOpened", serde_json::json!({ "id": id })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const EDITOR: &str = "ui.richtexteditor.editor";
    pub const PLACEHOLDER: &str = "ui.richtexteditor.placeholder";
    pub const FORMATTING: &str = "ui.richtexteditor.formatting";
    pub const BOLD: &str = "ui.richtexteditor.bold";
    pub const ITALIC: &str = "ui.richtexteditor.italic";
    pub const UNDERLINE: &str = "ui.richtexteditor.underline";
    pub const STRIKETHROUGH: &str = "ui.richtexteditor.strikethrough";
    pub const BULLETLIST: &str = "ui.richtexteditor.bulletList";
    pub const ORDEREDLIST: &str = "ui.richtexteditor.orderedList";
    pub const LINK: &str = "ui.richtexteditor.link";
    pub const MENTIONLIST: &str = "ui.richtexteditor.mentionList";
    pub const MENTIONOPTION: &str = "ui.richtexteditor.mentionOption";
    pub const MENTIONCHIP: &str = "ui.richtexteditor.mentionChip";
}
